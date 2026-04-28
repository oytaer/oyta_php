//! 队列系统门面
//!
//! 提供静态方法访问队列管理器功能
//! 对应 ThinkPHP 8.0 的 Queue 类
//!
//! # 使用示例
//! ```php
//! // 推送任务到队列
//! Queue::push('SendEmailJob', ['email' => 'user@example.com'], 'emails');
//!
//! // 延迟任务
//! Queue::later(60, 'SendEmailJob', $data, 'emails');
//!
//! // 弹出任务
//! $job = Queue::pop('emails');
//! ```

use std::sync::Arc;
use parking_lot::RwLock;
use serde_json::Value as JsonValue;

use super::queue::{QueueManager, QueueJob};

/// 全局队列管理器实例
///
/// 使用 RwLock 实现线程安全的单例模式
static QUEUE_MANAGER: RwLock<Option<Arc<QueueManager>>> = RwLock::new(None);

/// Queue 门面结构体
///
/// 提供静态方法访问队列管理器
/// 所有方法都是线程安全的
pub struct Queue;

impl Queue {
    /// 初始化全局队列管理器
    ///
    /// 必须在使用其他方法前调用
    pub fn init() {
        // 获取写锁
        let mut guard = QUEUE_MANAGER.write();
        // 创建新的队列管理器实例
        let manager = QueueManager::new();
        // 存储到全局静态变量
        *guard = Some(Arc::new(manager));
    }

    /// 获取全局队列管理器实例
    ///
    /// 内部方法，用于获取管理器引用
    fn get_manager() -> Option<Arc<QueueManager>> {
        // 获取读锁
        let guard = QUEUE_MANAGER.read();
        // 克隆内部的 Arc 引用
        guard.clone()
    }

    /// 推送任务到队列
    ///
    /// 将任务添加到指定队列的末尾
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `job`: 任务名称/处理器类名
    /// - `data`: 任务数据（JSON 格式）
    /// - `queue`: 队列名称，默认为 "default"
    ///
    /// # 返回
    /// 任务 ID
    ///
    /// # 示例
    /// ```php
    /// $jobId = Queue::push('SendEmailJob', ['email' => 'user@example.com'], 'emails');
    /// ```
    pub fn push(job: &str, data: &JsonValue, queue: &str) -> String {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 将 JSON 数据转换为字符串
            let data_str = data.to_string();
            // 创建队列任务
            let queue_job = QueueJob::new(job, job, &data_str, queue);
            // 获取任务 ID
            let job_id = queue_job.id.clone();
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 spawn_blocking
                let manager_clone = manager.clone();
                let queue_job_clone = queue_job.clone();
                handle.spawn(async move {
                    manager_clone.push(queue_job_clone).await;
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.push(queue_job).await;
                });
            }
            // 返回任务 ID
            job_id
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法推送任务: {}", job);
            // 返回空 ID
            String::new()
        }
    }

    /// 推送延迟任务
    ///
    /// 将任务添加到队列，延迟指定秒数后执行
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `delay_secs`: 延迟秒数
    /// - `job`: 任务名称
    /// - `data`: 任务数据
    /// - `queue`: 队列名称
    ///
    /// # 返回
    /// 任务 ID
    ///
    /// # 示例
    /// ```php
    /// // 60 秒后执行
    /// $jobId = Queue::later(60, 'SendEmailJob', $data, 'emails');
    /// ```
    pub fn later(delay_secs: u64, job: &str, data: &JsonValue, queue: &str) -> String {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 将 JSON 数据转换为字符串
            let data_str = data.to_string();
            // 创建延迟任务
            let queue_job = QueueJob::new_delayed(job, job, &data_str, queue, delay_secs);
            // 获取任务 ID
            let job_id = queue_job.id.clone();
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 spawn_blocking
                let manager_clone = manager.clone();
                let queue_job_clone = queue_job.clone();
                handle.spawn(async move {
                    manager_clone.later(queue_job_clone, delay_secs).await;
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.later(queue_job, delay_secs).await;
                });
            }
            // 返回任务 ID
            job_id
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法推送延迟任务: {}", job);
            // 返回空 ID
            String::new()
        }
    }

    /// 从队列弹出任务
    ///
    /// 从队列头部取出一个任务
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    ///
    /// # 返回
    /// 队列任务，如果队列为空返回 None
    ///
    /// # 示例
    /// ```php
    /// $job = Queue::pop('emails');
    /// if ($job) {
    ///     // 处理任务
    /// }
    /// ```
    pub fn pop(queue: &str) -> Option<QueueJob> {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.pop(queue).await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.pop(queue).await
                })
            }
        } else {
            // 未初始化时返回 None
            None
        }
    }

    /// 获取队列大小
    ///
    /// 返回指定队列中的任务数量
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    ///
    /// # 返回
    /// 任务数量
    pub fn size(queue: &str) -> usize {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.len(queue).await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.len(queue).await
                })
            }
        } else {
            // 未初始化时返回 0
            0
        }
    }

    /// 检查队列是否为空
    ///
    /// 判断指定队列是否没有待处理任务
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    ///
    /// # 返回
    /// 如果队列为空返回 true
    pub fn is_empty(queue: &str) -> bool {
        // 获取队列大小并判断是否为 0
        Self::size(queue) == 0
    }

    /// 清空队列
    ///
    /// 删除指定队列中的所有任务
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    pub fn clear(queue: &str) {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.clear(queue).await;
                    })
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.clear(queue).await;
                });
            }
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法清空队列: {}", queue);
        }
    }

    /// 清空所有队列
    ///
    /// 删除所有队列中的所有任务
    /// 注意：此方法使用阻塞方式执行异步操作
    pub fn clear_all() {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.clear_all().await;
                    })
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.clear_all().await;
                });
            }
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法清空所有队列");
        }
    }

    /// 记录失败任务
    ///
    /// 将任务添加到失败任务列表
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `job`: 失败的任务
    pub fn record_failure(job: QueueJob) {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.record_failure(job).await;
                    })
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.record_failure(job).await;
                });
            }
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法记录失败任务");
        }
    }

    /// 重试失败任务
    ///
    /// 将失败任务重新放回队列
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    /// - `job_id`: 任务 ID
    ///
    /// # 返回
    /// 是否重试成功
    pub fn retry(queue: &str, job_id: &str) -> bool {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.retry(queue, job_id).await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.retry(queue, job_id).await
                })
            }
        } else {
            // 未初始化时返回 false
            false
        }
    }

    /// 获取失败任务列表
    ///
    /// 返回指定队列的失败任务
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    ///
    /// # 返回
    /// 失败任务列表
    pub fn get_failed(queue: &str) -> Vec<QueueJob> {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.failed_jobs(queue).await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.failed_jobs(queue).await
                })
            }
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 获取所有队列名称
    ///
    /// 返回所有已创建的队列名称列表
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 返回
    /// 队列名称列表
    pub fn queues() -> Vec<String> {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.queue_names().await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.queue_names().await
                })
            }
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 启动队列消费者工作进程
    ///
    /// 在后台持续消费指定队列中的任务
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 参数
    /// - `queue`: 队列名称
    /// - `worker_count`: 工作进程数量
    pub fn start_workers(queue: &str, worker_count: usize) {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 spawn
                let manager_clone = manager.clone();
                let queue_str = queue.to_string();
                handle.spawn(async move {
                    manager_clone.start_workers(&queue_str, worker_count).await;
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.start_workers(queue, worker_count).await;
                });
            }
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法启动工作进程");
        }
    }

    /// 停止所有工作进程
    ///
    /// 停止队列消费者工作进程
    /// 注意：此方法使用阻塞方式执行异步操作
    pub fn stop_workers() {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.stop_workers().await;
                    })
                });
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.stop_workers().await;
                });
            }
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Queue 门面未初始化，无法停止工作进程");
        }
    }

    /// 检查工作进程是否在运行
    ///
    /// 注意：此方法使用阻塞方式执行异步操作
    ///
    /// # 返回
    /// 如果工作进程正在运行返回 true
    pub fn is_running() -> bool {
        // 尝试获取管理器实例
        if let Some(manager) = Self::get_manager() {
            // 使用 tokio 运行时执行异步操作
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                // 已有运行时，使用 block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        manager.is_running().await
                    })
                })
            } else {
                // 没有运行时，创建一个临时运行时
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    manager.is_running().await
                })
            }
        } else {
            // 未初始化时返回 false
            false
        }
    }

    /// 检查队列门面是否已初始化
    ///
    /// 用于判断是否需要调用 init 方法
    ///
    /// # 返回
    /// 如果已初始化返回 true，否则返回 false
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = QUEUE_MANAGER.read();
        guard.is_some()
    }

    /// 重置队列门面
    ///
    /// 清除队列管理器实例
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = QUEUE_MANAGER.write();
        // 清空管理器实例
        *guard = None;
    }
}
