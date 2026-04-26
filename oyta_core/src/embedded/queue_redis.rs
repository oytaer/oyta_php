//! Redis 队列驱动
//!
//! 使用 Redis 作为队列后端
//! 支持：BRPOP/LPUSH 原子操作、延迟队列、死信队列
//!
//! # 功能特性
//! - 分布式队列支持
//! - 持久化存储
//! - 高性能原子操作
//! - 支持延迟任务

use anyhow::{Context, Result};

use super::queue::QueueJob;

/// Redis 队列驱动
pub struct RedisQueueDriver {
    /// Redis 连接
    client: Option<redis::Client>,
    /// 队列前缀
    prefix: String,
}

impl RedisQueueDriver {
    /// 创建新的 Redis 队列驱动
    ///
    /// # 参数
    /// - `url`: Redis 连接 URL（如 redis://127.0.0.1:6379）
    /// - `prefix`: 键前缀
    pub fn new(url: &str, prefix: &str) -> Result<Self> {
        let client = redis::Client::open(url)
            .with_context(|| format!("无法连接 Redis: {}", url))?;

        Ok(Self {
            client: Some(client),
            prefix: prefix.to_string(),
        })
    }

    /// 创建未连接的驱动
    pub fn disconnected() -> Self {
        Self {
            client: None,
            prefix: "queue:".to_string(),
        }
    }

    /// 获取队列键名
    fn queue_key(&self, queue_name: &str) -> String {
        format!("{}{}", self.prefix, queue_name)
    }

    /// 获取延迟队列键名
    fn delayed_key(&self, queue_name: &str) -> String {
        format!("{}{}:delayed", self.prefix, queue_name)
    }

    /// 获取死信队列键名
    fn dead_letter_key(&self, queue_name: &str) -> String {
        format!("{}{}:dead", self.prefix, queue_name)
    }

    /// 推送任务到队列
    ///
    /// # 参数
    /// - `queue_name`: 队列名称
    /// - `job`: 任务
    pub async fn push(&self, queue_name: &str, job: &QueueJob) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.queue_key(queue_name);
        let job_json = serde_json::to_string(job)?;

        let mut conn = client.get_connection()?;

        redis::cmd("LPUSH")
            .arg(&key)
            .arg(&job_json)
            .query::<i32>(&mut conn)?;

        tracing::debug!("Redis 队列推送任务: {} → {}", queue_name, job.id);

        Ok(())
    }

    /// 推送延迟任务
    ///
    /// # 参数
    /// - `queue_name`: 队列名称
    /// - `job`: 任务
    /// - `delay_secs`: 延迟秒数
    pub async fn push_delayed(&self, queue_name: &str, job: &QueueJob, delay_secs: u64) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.delayed_key(queue_name);
        let execute_at = job.created_at + delay_secs;
        let job_json = serde_json::to_string(job)?;

        let mut conn = client.get_connection()?;

        // 使用有序集合存储延迟任务，score 为执行时间
        redis::cmd("ZADD")
            .arg(&key)
            .arg(execute_at)
            .arg(&job_json)
            .query::<i32>(&mut conn)?;

        tracing::debug!("Redis 延迟队列推送任务: {} → {} (延迟 {} 秒)", queue_name, job.id, delay_secs);

        Ok(())
    }

    /// 从队列弹出任务（阻塞）
    ///
    /// # 参数
    /// - `queue_name`: 队列名称
    /// - `timeout`: 超时时间（秒）
    ///
    /// # 返回
    /// 任务，如果超时返回 None
    pub async fn pop(&self, queue_name: &str, timeout: u64) -> Result<Option<QueueJob>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        // 先检查延迟队列
        self.move_ready_delayed_jobs(queue_name).await?;

        let key = self.queue_key(queue_name);

        let mut conn = client.get_connection()?;

        // 使用 BRPOP 阻塞弹出
        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&key)
            .arg(timeout)
            .query(&mut conn)?;

        if let Some((_, job_json)) = result {
            let job: QueueJob = serde_json::from_str(&job_json)?;
            return Ok(Some(job));
        }

        Ok(None)
    }

    /// 移动已到时间的延迟任务到主队列
    async fn move_ready_delayed_jobs(&self, queue_name: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let delayed_key = self.delayed_key(queue_name);
        let queue_key = self.queue_key(queue_name);

        let mut conn = client.get_connection()?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 获取所有已到时间的任务
        let ready_jobs: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&delayed_key)
            .arg(0)
            .arg(now)
            .query(&mut conn)?;

        if ready_jobs.is_empty() {
            return Ok(());
        }

        // 移动到主队列
        for job_json in ready_jobs {
            redis::cmd("LPUSH")
                .arg(&queue_key)
                .arg(&job_json)
                .query::<i32>(&mut conn)?;
        }

        // 从延迟队列删除
        redis::cmd("ZREMRANGEBYSCORE")
            .arg(&delayed_key)
            .arg(0)
            .arg(now)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 推送到死信队列
    ///
    /// # 参数
    /// - `queue_name`: 队列名称
    /// - `job`: 失败的任务
    pub async fn push_to_dead_letter(&self, queue_name: &str, job: &QueueJob) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.dead_letter_key(queue_name);
        let job_json = serde_json::to_string(job)?;

        let mut conn = client.get_connection()?;

        redis::cmd("LPUSH")
            .arg(&key)
            .arg(&job_json)
            .query::<i32>(&mut conn)?;

        tracing::warn!("任务已移至死信队列: {} → {}", queue_name, job.id);

        Ok(())
    }

    /// 获取死信队列任务
    pub async fn get_dead_letter_jobs(&self, queue_name: &str) -> Result<Vec<QueueJob>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.dead_letter_key(queue_name);

        let mut conn = client.get_connection()?;

        let jobs: Vec<String> = redis::cmd("LRANGE")
            .arg(&key)
            .arg(0)
            .arg(-1)
            .query(&mut conn)?;

        jobs.into_iter()
            .map(|json| serde_json::from_str(&json).with_context(|| "解析任务失败"))
            .collect()
    }

    /// 重试死信队列任务
    pub async fn retry_dead_letter(&self, queue_name: &str, job_id: &str) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let dead_key = self.dead_letter_key(queue_name);
        let queue_key = self.queue_key(queue_name);

        let mut conn = client.get_connection()?;

        // 获取所有死信任务
        let jobs: Vec<String> = redis::cmd("LRANGE")
            .arg(&dead_key)
            .arg(0)
            .arg(-1)
            .query(&mut conn)?;

        for job_json in jobs {
            if let Ok(mut job) = serde_json::from_str::<QueueJob>(&job_json) {
                if job.id == job_id {
                    // 重置重试次数
                    job.retries = 0;
                    job.delay_secs = 0;

                    let new_json = serde_json::to_string(&job)?;

                    // 推回主队列
                    redis::cmd("LPUSH")
                        .arg(&queue_key)
                        .arg(&new_json)
                        .query::<i32>(&mut conn)?;

                    // 从死信队列删除
                    redis::cmd("LREM")
                        .arg(&dead_key)
                        .arg(1)
                        .arg(&job_json)
                        .query::<i32>(&mut conn)?;

                    tracing::info!("重试死信任务: {} → {}", queue_name, job_id);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// 获取队列长度
    pub async fn len(&self, queue_name: &str) -> Result<usize> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.queue_key(queue_name);

        let mut conn = client.get_connection()?;

        let len: usize = redis::cmd("LLEN")
            .arg(&key)
            .query(&mut conn)?;

        Ok(len)
    }

    /// 获取延迟队列长度
    pub async fn delayed_len(&self, queue_name: &str) -> Result<usize> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.delayed_key(queue_name);

        let mut conn = client.get_connection()?;

        let len: usize = redis::cmd("ZCARD")
            .arg(&key)
            .query(&mut conn)?;

        Ok(len)
    }

    /// 清空队列
    pub async fn clear(&self, queue_name: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;

        redis::cmd("DEL")
            .arg(self.queue_key(queue_name))
            .arg(self.delayed_key(queue_name))
            .arg(self.dead_letter_key(queue_name))
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 获取队列统计信息
    pub async fn stats(&self, queue_name: &str) -> Result<QueueStats> {
        Ok(QueueStats {
            pending: self.len(queue_name).await?,
            delayed: self.delayed_len(queue_name).await?,
            dead_letter: self.get_dead_letter_jobs(queue_name).await?.len(),
        })
    }
}

/// 队列统计信息
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// 等待中的任务数
    pub pending: usize,
    /// 延迟任务数
    pub delayed: usize,
    /// 死信任务数
    pub dead_letter: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_driver_disconnected() {
        let driver = RedisQueueDriver::disconnected();
        assert!(driver.client.is_none());
    }
}
