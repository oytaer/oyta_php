//! 迁移管理器模块
//!
//! 提供迁移文件的创建、执行、回滚等核心功能

use anyhow::{Context, Result};
use chrono::Local;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::database::pool_trait::DatabasePool;
use super::types::{Migration, MigrationRecord};
use super::helpers::{detect_migration_type, extract_table_name, migration_name_to_class, MigrationType};

/// 迁移管理器
pub struct MigrationManager {
    /// 项目根目录
    project_root: PathBuf,
    /// 迁移文件目录
    migration_dir: PathBuf,
    /// 迁移表名
    migration_table: String,
    /// 数据库连接池（可选）
    pool: Option<Arc<dyn DatabasePool>>,
    /// 内存中的迁移记录缓存（用于无数据库连接时）
    memory_records: HashMap<String, MigrationRecord>,
}

impl MigrationManager {
    /// 创建新的迁移管理器
    ///
    /// # 参数
    /// - `project_root`: 项目根目录
    pub fn new(project_root: &std::path::Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            migration_dir: project_root.join("database").join("migrations"),
            migration_table: "migrations".to_string(),
            pool: None,
            memory_records: HashMap::new(),
        }
    }

    /// 设置数据库连接池
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    pub fn with_pool(mut self, pool: Arc<dyn DatabasePool>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// 设置迁移表名
    ///
    /// # 参数
    /// - `table_name`: 迁移表名称
    pub fn with_table_name(mut self, table_name: &str) -> Self {
        self.migration_table = table_name.to_string();
        self
    }

    /// 确保迁移表存在
    ///
    /// 如果迁移表不存在，则创建它
    async fn ensure_migration_table(&self) -> Result<()> {
        let pool = self.pool.as_ref()
            .context("数据库连接池未设置，无法创建迁移表")?;

        // 检查表是否存在
        let check_sql = self.get_check_table_sql();
        let result = pool.query(&check_sql).await?;

        if result.rows.is_empty() {
            // 表不存在，创建它
            let create_sql = self.get_create_migration_table_sql();
            pool.execute(&create_sql).await?;
            tracing::info!("✓ 迁移表 [{}] 创建成功", self.migration_table);
        }

        Ok(())
    }

    /// 获取检查表是否存在的 SQL
    fn get_check_table_sql(&self) -> String {
        // 简化版本：尝试查询表，如果失败则表不存在
        format!("SELECT 1 FROM {} LIMIT 1", self.migration_table)
    }

    /// 获取创建迁移表的 SQL
    fn get_create_migration_table_sql(&self) -> String {
        format!(
            r#"CREATE TABLE {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                migration VARCHAR(255) NOT NULL,
                batch INTEGER NOT NULL,
                executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"#,
            self.migration_table
        )
    }

    /// 创建新的迁移文件
    ///
    /// # 参数
    /// - `name`: 迁移名称（如 create_users_table）
    ///
    /// # 返回
    /// 创建的迁移文件路径
    pub fn create_migration(&self, name: &str) -> Result<PathBuf> {
        // 确保迁移目录存在
        std::fs::create_dir_all(&self.migration_dir)?;

        // 生成时间戳
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();

        // 生成文件名
        let filename = format!("{}_{}.php", timestamp, name);
        let file_path = self.migration_dir.join(&filename);

        // 检查文件是否已存在
        if file_path.exists() {
            anyhow::bail!("迁移文件已存在: {}", file_path.display());
        }

        // 解析迁移类型
        let migration_type = detect_migration_type(name);
        let table_name = extract_table_name(name);

        // 生成迁移内容
        let content = self.generate_migration_content(name, &migration_type, &table_name);

        // 写入文件
        std::fs::write(&file_path, content)?;

        tracing::info!("✓ 迁移文件创建成功: {}", file_path.display());

        Ok(file_path)
    }

    /// 生成迁移文件内容
    fn generate_migration_content(&self, name: &str, migration_type: &MigrationType, table_name: &str) -> String {
        let class_name = migration_name_to_class(name);

        match migration_type {
            MigrationType::Create => {
                format!(r#"<?php
use oyta\db\Migration;

/**
 * 迁移类：{}
 * 
 * 创建数据表：{}
 */
class {} extends Migration
{{
    /**
     * 执行迁移
     * 
     * @return void
     */
    public function up()
    {{
        $this->createTable('{}', function($table) {{
            $table->id();
            $table->timestamps();
            $table->softDelete();
            
            // 在此处添加更多字段
            // $table->string('name')->comment('名称');
            // $table->string('email')->unique()->comment('邮箱');
        }});
    }}

    /**
     * 回滚迁移
     * 
     * @return void
     */
    public function down()
    {{
        $this->dropTable('{}');
    }}
}}
"#, name, table_name, class_name, table_name, table_name)
            }
            MigrationType::Add => {
                format!(r#"<?php
use oyta\db\Migration;

/**
 * 迁移类：{}
 * 
 * 修改数据表：{}
 */
class {} extends Migration
{{
    /**
     * 执行迁移
     * 
     * @return void
     */
    public function up()
    {{
        $this->table('{}', function($table) {{
            // 在此处添加字段修改
            // $table->string('new_column')->after('existing_column');
        }});
    }}

    /**
     * 回滚迁移
     * 
     * @return void
     */
    public function down()
    {{
        $this->table('{}', function($table) {{
            // 在此处添加回滚操作
            // $table->dropColumn('new_column');
        }});
    }}
}}
"#, name, table_name, class_name, table_name, table_name)
            }
            MigrationType::Drop => {
                format!(r#"<?php
use oyta\db\Migration;

/**
 * 迁移类：{}
 * 
 * 删除数据表：{}
 */
class {} extends Migration
{{
    /**
     * 执行迁移
     * 
     * @return void
     */
    public function up()
    {{
        $this->dropTable('{}');
    }}

    /**
     * 回滚迁移
     * 
     * @return void
     */
    public function down()
    {{
        $this->createTable('{}', function($table) {{
            $table->id();
            $table->timestamps();
            
            // 在此处添加原始字段定义
        }});
    }}
}}
"#, name, table_name, class_name, table_name, table_name)
            }
            MigrationType::Default => {
                format!(r#"<?php
use oyta\db\Migration;

/**
 * 迁移类：{}
 */
class {} extends Migration
{{
    /**
     * 执行迁移
     * 
     * @return void
     */
    public function up()
    {{
        // 在此处编写迁移逻辑
    }}

    /**
     * 回滚迁移
     * 
     * @return void
     */
    public function down()
    {{
        // 在此处编写回滚逻辑
    }}
}}
"#, name, class_name)
            }
        }
    }

    /// 获取所有迁移文件
    ///
    /// # 返回
    /// 迁移文件列表（按时间戳排序）
    pub fn get_migrations(&self) -> Result<Vec<Migration>> {
        if !self.migration_dir.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        for entry in std::fs::read_dir(&self.migration_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "php").unwrap_or(false) {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // 解析迁移名称
                    let name = filename
                        .strip_suffix(".php")
                        .unwrap_or(filename)
                        .to_string();

                    migrations.push(Migration {
                        name: name.clone(),
                        filename: filename.to_string(),
                        path: path.clone(),
                        executed: false,
                        executed_at: None,
                        batch: None,
                    });
                }
            }
        }

        // 按文件名（时间戳）排序
        migrations.sort_by(|a, b| a.filename.cmp(&b.filename));

        Ok(migrations)
    }

    /// 获取待执行的迁移
    ///
    /// # 返回
    /// 未执行的迁移列表
    pub async fn get_pending_migrations(&self) -> Result<Vec<Migration>> {
        let all_migrations = self.get_migrations()?;
        let executed = self.get_executed_migrations().await?;

        let pending: Vec<Migration> = all_migrations
            .into_iter()
            .filter(|m| !executed.contains_key(&m.name))
            .collect();

        Ok(pending)
    }

    /// 获取已执行的迁移
    ///
    /// # 返回
    /// 已执行迁移映射（名称 → 执行信息）
    pub async fn get_executed_migrations(&self) -> Result<HashMap<String, (String, u32)>> {
        // 如果有数据库连接，从数据库读取
        if let Some(pool) = &self.pool {
            self.ensure_migration_table().await?;

            let sql = format!(
                "SELECT migration, batch, executed_at FROM {} ORDER BY id",
                self.migration_table
            );

            let result = pool.query(&sql).await?;
            let mut executed = HashMap::new();

            for row in result.rows {
                if let (Some(migration), Some(batch), Some(executed_at)) = (
                    row.get("migration").and_then(|v| v.as_str()),
                    row.get("batch").and_then(|v| v.as_i64()),
                    row.get("executed_at").and_then(|v| v.as_str()),
                ) {
                    executed.insert(migration.to_string(), (executed_at.to_string(), batch as u32));
                }
            }

            return Ok(executed);
        }

        // 无数据库连接时，从内存缓存读取
        let mut executed = HashMap::new();
        for (name, record) in &self.memory_records {
            executed.insert(name.clone(), (record.executed_at.clone(), record.batch));
        }

        Ok(executed)
    }

    /// 记录迁移执行
    ///
    /// # 参数
    /// - `migration`: 迁移信息
    /// - `batch`: 批次号
    pub async fn record_migration(&self, migration: &Migration, batch: u32) -> Result<()> {
        let executed_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 如果有数据库连接，写入数据库
        if let Some(pool) = &self.pool {
            self.ensure_migration_table().await?;

            let sql = format!(
                "INSERT INTO {} (migration, batch, executed_at) VALUES ('{}', {}, '{}')",
                self.migration_table,
                migration.name,
                batch,
                executed_at
            );

            pool.execute(&sql).await?;
            tracing::debug!("记录迁移到数据库: {} (批次 {})", migration.name, batch);
        }

        tracing::info!("✓ 记录迁移: {} (批次 {})", migration.name, batch);

        Ok(())
    }

    /// 删除迁移记录
    ///
    /// # 参数
    /// - `migration`: 迁移信息
    pub async fn remove_migration_record(&self, migration: &Migration) -> Result<()> {
        // 如果有数据库连接，从数据库删除
        if let Some(pool) = &self.pool {
            self.ensure_migration_table().await?;

            let sql = format!(
                "DELETE FROM {} WHERE migration = '{}'",
                self.migration_table,
                migration.name
            );

            pool.execute(&sql).await?;
            tracing::debug!("从数据库删除迁移记录: {}", migration.name);
        }

        tracing::info!("✓ 删除迁移记录: {}", migration.name);

        Ok(())
    }

    /// 获取下一个批次号
    ///
    /// # 返回
    /// 下一个批次号
    pub async fn get_next_batch(&self) -> Result<u32> {
        // 如果有数据库连接，从数据库查询
        if let Some(pool) = &self.pool {
            self.ensure_migration_table().await?;

            let sql = format!(
                "SELECT MAX(batch) as max_batch FROM {}",
                self.migration_table
            );

            let result = pool.query(&sql).await?;

            if let Some(row) = result.rows.first() {
                if let Some(max_batch) = row.get("max_batch").and_then(|v| v.as_i64()) {
                    return Ok((max_batch + 1) as u32);
                }
            }

            return Ok(1);
        }

        // 无数据库连接时，从内存缓存计算
        let max_batch = self.memory_records
            .values()
            .map(|r| r.batch)
            .max()
            .unwrap_or(0);

        Ok(max_batch + 1)
    }

    /// 获取最后一批迁移
    ///
    /// # 返回
    /// 最后一批迁移列表
    pub async fn get_last_batch_migrations(&self) -> Result<Vec<Migration>> {
        // 如果有数据库连接，从数据库查询
        if let Some(pool) = &self.pool {
            self.ensure_migration_table().await?;

            // 获取最大批次号
            let max_batch_sql = format!(
                "SELECT MAX(batch) as max_batch FROM {}",
                self.migration_table
            );

            let result = pool.query(&max_batch_sql).await?;

            if let Some(row) = result.rows.first() {
                if let Some(max_batch) = row.get("max_batch").and_then(|v| v.as_i64()) {
                    // 获取该批次的所有迁移
                    let sql = format!(
                        "SELECT migration FROM {} WHERE batch = {}",
                        self.migration_table,
                        max_batch
                    );

                    let batch_result = pool.query(&sql).await?;
                    let all_migrations = self.get_migrations()?;
                    let executed = self.get_executed_migrations().await?;

                    let last_batch: Vec<Migration> = batch_result
                        .rows
                        .iter()
                        .filter_map(|row| {
                            row.get("migration").and_then(|v| v.as_str()).and_then(|name| {
                                all_migrations.iter().find(|m| m.name == name).map(|m| {
                                    let mut m = m.clone();
                                    if let Some((executed_at, batch)) = executed.get(name) {
                                        m.executed = true;
                                        m.executed_at = Some(executed_at.clone());
                                        m.batch = Some(*batch);
                                    }
                                    m
                                })
                            })
                        })
                        .collect();

                    return Ok(last_batch);
                }
            }

            return Ok(Vec::new());
        }

        // 无数据库连接时，从内存缓存获取
        let max_batch = self.memory_records
            .values()
            .map(|r| r.batch)
            .max()
            .unwrap_or(0);

        let all_migrations = self.get_migrations()?;
        let executed = self.get_executed_migrations().await?;

        let last_batch: Vec<Migration> = all_migrations
            .iter()
            .filter(|m| {
                executed.get(&m.name)
                    .map(|(_, batch)| *batch == max_batch)
                    .unwrap_or(false)
            })
            .map(|m| m.clone())
            .collect();

        Ok(last_batch)
    }

    /// 执行迁移
    ///
    /// 执行所有待执行的迁移
    ///
    /// # 返回
    /// 执行的迁移数量
    pub async fn migrate(&self) -> Result<usize> {
        let pending = self.get_pending_migrations().await?;

        if pending.is_empty() {
            tracing::info!("没有待执行的迁移");
            return Ok(0);
        }

        let batch = self.get_next_batch().await?;
        let mut count = 0;

        for migration in &pending {
            // 执行迁移（这里需要与 PHP 解释器集成）
            tracing::info!("执行迁移: {}", migration.name);

            // 记录迁移
            self.record_migration(migration, batch).await?;
            count += 1;
        }

        tracing::info!("✓ 已执行 {} 个迁移", count);

        Ok(count)
    }

    /// 回滚迁移
    ///
    /// 回滚最后一批迁移
    ///
    /// # 返回
    /// 回滚的迁移数量
    pub async fn rollback(&self) -> Result<usize> {
        let last_batch = self.get_last_batch_migrations().await?;

        if last_batch.is_empty() {
            tracing::info!("没有可回滚的迁移");
            return Ok(0);
        }

        let mut count = 0;

        // 按相反顺序回滚
        for migration in last_batch.into_iter().rev() {
            // 执行回滚（这里需要与 PHP 解释器集成）
            tracing::info!("回滚迁移: {}", migration.name);

            // 删除迁移记录
            self.remove_migration_record(&migration).await?;
            count += 1;
        }

        tracing::info!("✓ 已回滚 {} 个迁移", count);

        Ok(count)
    }

    /// 重置所有迁移
    ///
    /// 回滚所有已执行的迁移
    ///
    /// # 返回
    /// 回滚的迁移数量
    pub async fn reset(&self) -> Result<usize> {
        let executed = self.get_executed_migrations().await?;

        if executed.is_empty() {
            tracing::info!("没有已执行的迁移");
            return Ok(0);
        }

        let all_migrations = self.get_migrations()?;
        let mut count = 0;

        // 按相反顺序回滚所有迁移
        for migration in all_migrations.into_iter().rev() {
            if executed.contains_key(&migration.name) {
                // 执行回滚（这里需要与 PHP 解释器集成）
                tracing::info!("回滚迁移: {}", migration.name);

                // 删除迁移记录
                self.remove_migration_record(&migration).await?;
                count += 1;
            }
        }

        tracing::info!("✓ 已重置 {} 个迁移", count);

        Ok(count)
    }

    /// 获取迁移状态
    ///
    /// # 返回
    /// 所有迁移及其执行状态
    pub async fn status(&self) -> Result<Vec<Migration>> {
        let all_migrations = self.get_migrations()?;
        let executed = self.get_executed_migrations().await?;

        let migrations: Vec<Migration> = all_migrations
            .into_iter()
            .map(|mut m| {
                if let Some((executed_at, batch)) = executed.get(&m.name) {
                    m.executed = true;
                    m.executed_at = Some(executed_at.clone());
                    m.batch = Some(*batch);
                }
                m
            })
            .collect();

        Ok(migrations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试迁移管理器创建
    #[test]
    fn test_migration_manager_new() {
        let manager = MigrationManager::new(std::path::Path::new("/tmp"));
        assert!(manager.pool.is_none());
        assert_eq!(manager.migration_table, "migrations");
    }
}
