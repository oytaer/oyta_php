//! 软删除扩展模块
//!
//! 实现模型的软删除扩展功能
//! 对应 ThinkPHP 的软删除（SoftDelete）功能
//!
//! 主要功能：
//! - withTrashed 包含软删除数据
//! - onlyTrashed 仅查询软删除数据
//! - restore 恢复软删除数据
//! - force 强制物理删除

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::{ModelConfig, SoftDeleteDefault};

/// 安全转义SQL标识符
#[inline]
fn escape_id(name: &str) -> String {
    if name.is_empty() {
        return "``".to_string();
    }
    format!("`{}`", name.replace('`', "``"))
}

/// 验证标识符是否安全
#[inline]
fn is_valid_id(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
}

/// 安全转义标识符（带验证）
fn safe_escape_id(name: &str) -> String {
    if is_valid_id(name) {
        escape_id(name)
    } else {
        "``".to_string()
    }
}

/// 软删除扩展 trait
///
/// 为模型提供软删除扩展方法
/// 使用 async_trait 宏支持异步方法
/// 使用 ?Send 允许非 Send 的 future
#[async_trait::async_trait(?Send)]
pub trait SoftDeleteExt {
    /// 包含软删除数据
    ///
    /// # 返回值
    /// 模型实例的可变引用
    ///
    /// # 示例
    /// ```php
    /// User::withTrashed()->find();
    /// User::withTrashed()->select();
    /// ```
    fn with_trashed(&mut self) -> &mut Self;

    /// 仅查询软删除数据
    ///
    /// # 返回值
    /// 模型实例的可变引用
    ///
    /// # 示例
    /// ```php
    /// User::onlyTrashed()->find();
    /// User::onlyTrashed()->select();
    /// ```
    fn only_trashed(&mut self) -> &mut Self;

    /// 恢复软删除数据
    ///
    /// # 返回值
    /// 恢复成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user = User::onlyTrashed()->find(1);
    /// $user->restore();
    /// ```
    async fn restore(&mut self) -> anyhow::Result<bool>;

    /// 强制物理删除
    ///
    /// # 返回值
    /// 删除成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user->force()->delete();
    /// ```
    async fn force_delete(&mut self) -> anyhow::Result<bool>;
}

/// 软删除查询状态
///
/// 用于控制查询是否包含软删除数据
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashedState {
    /// 默认状态：不包含软删除数据
    Default,
    /// 包含软删除数据
    WithTrashed,
    /// 仅查询软删除数据
    OnlyTrashed,
}

/// 软删除查询构建器
///
/// 用于构建软删除相关的查询
pub struct SoftDeleteQueryBuilder {
    /// 模型配置
    pub config: ModelConfig,
    /// 软删除状态
    pub trashed_state: TrashedState,
    /// 查询构建器
    pub query_builder: QueryBuilder,
}

impl SoftDeleteQueryBuilder {
    /// 创建新的软删除查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    pub fn new(config: ModelConfig) -> Self {
        let table = config.full_table();
        Self {
            config,
            trashed_state: TrashedState::Default,
            query_builder: QueryBuilder::new(&table),
        }
    }

    /// 设置包含软删除数据
    ///
    /// # 返回值
    /// 自身引用
    pub fn with_trashed(&mut self) -> &mut Self {
        self.trashed_state = TrashedState::WithTrashed;
        self
    }

    /// 设置仅查询软删除数据
    ///
    /// # 返回值
    /// 自身引用
    pub fn only_trashed(&mut self) -> &mut Self {
        self.trashed_state = TrashedState::OnlyTrashed;
        self
    }

    /// 构建软删除条件
    ///
    /// # 返回值
    /// SQL 条件字符串
    pub fn build_soft_delete_condition(&self) -> Option<String> {
        // 如果模型未启用软删除，返回 None
        if !self.config.soft_delete {
            return None;
        }

        let field = &self.config.delete_time_field;

        match self.trashed_state {
            // 默认：不包含软删除数据
            TrashedState::Default => {
                match &self.config.soft_delete_default {
                    SoftDeleteDefault::Null => Some(format!("{} IS NULL", field)),
                    SoftDeleteDefault::Zero => Some(format!("{} = 0", field)),
                    SoftDeleteDefault::Empty => Some(format!("{} = ''", field)),
                }
            }
            // 包含软删除数据：不添加条件
            TrashedState::WithTrashed => None,
            // 仅查询软删除数据
            TrashedState::OnlyTrashed => {
                match &self.config.soft_delete_default {
                    SoftDeleteDefault::Null => Some(format!("{} IS NOT NULL", field)),
                    SoftDeleteDefault::Zero => Some(format!("{} != 0", field)),
                    SoftDeleteDefault::Empty => Some(format!("{} != ''", field)),
                }
            }
        }
    }

    /// 构建 SELECT SQL
    ///
    /// # 返回值
    /// SQL 字符串
    pub fn build_select_sql(&self) -> String {
        let mut sql = self.query_builder.build_select_sql();

        // 添加软删除条件
        if let Some(condition) = self.build_soft_delete_condition() {
            if sql.contains("WHERE") {
                sql = sql.replace("WHERE", &format!("WHERE {} AND ", condition));
            } else {
                sql = sql.trim_end_matches(';').to_string();
                sql.push_str(&format!(" WHERE {}", condition));
            }
        }

        sql
    }

    /// 添加 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 自身引用
    pub fn where_clause(&mut self, field: &str, operator: &str, value: Value) -> &mut Self {
        // 使用可变引用版本的方法添加 WHERE 条件
        self.query_builder.where_clause_mut(field, operator, value);
        self
    }

    /// 根据主键查找
    ///
    /// # 参数
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 模型实例
    pub async fn find(&self, id: Value) -> anyhow::Result<Option<ModelInstance>> {
        // 构建查询 SQL
        let table = self.config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} = ?", table, self.config.pk);

        // 添加软删除条件
        if let Some(condition) = self.build_soft_delete_condition() {
            sql = format!("{} AND {}", sql, condition);
        }

        // 执行查询
        let result = executor::query_with_params(&sql, &[id]).await?;

        // 返回结果
        if let Some(row) = result.first() {
            Ok(Some(ModelInstance::from_row(self.config.clone(), row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 查询所有记录
    ///
    /// # 返回值
    /// 模型实例列表
    pub async fn select(&self) -> anyhow::Result<Vec<ModelInstance>> {
        // 构建查询 SQL
        let table = self.config.full_table();
        let mut sql = format!("SELECT * FROM {}", safe_escape_id(&table));

        // 添加软删除条件
        if let Some(condition) = self.build_soft_delete_condition() {
            sql = format!("{} WHERE {}", sql, condition);
        }

        // 执行查询
        let result = executor::query(&sql).await?;

        // 转换为模型实例列表
        Ok(result
            .rows
            .into_iter()
            .map(|row| ModelInstance::from_row(self.config.clone(), row))
            .collect())
    }
}

/// 软删除操作器
///
/// 提供软删除相关的操作方法
pub struct SoftDeleteOperator;

impl SoftDeleteOperator {
    /// 恢复软删除记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 恢复成功返回 true
    pub async fn restore(config: &ModelConfig, id: Value) -> anyhow::Result<bool> {
        // 检查是否启用软删除
        if !config.soft_delete {
            return Err(anyhow::anyhow!("Model does not support soft delete"));
        }

        // 构建恢复 SQL
        let table = config.full_table();
        let default_value = match &config.soft_delete_default {
            SoftDeleteDefault::Null => "NULL",
            SoftDeleteDefault::Zero => "0",
            SoftDeleteDefault::Empty => "''",
        };

        let sql = format!(
            "UPDATE {} SET {} = {} WHERE {} = ?",
            table, config.delete_time_field, default_value, config.pk
        );

        // 执行恢复
        executor::execute_with_params(&sql, &[id]).await?;

        Ok(true)
    }

    /// 批量恢复软删除记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `ids`: 主键值列表
    ///
    /// # 返回值
    /// 恢复成功返回 true
    pub async fn restore_batch(config: &ModelConfig, ids: &[Value]) -> anyhow::Result<bool> {
        // 检查是否启用软删除
        if !config.soft_delete {
            return Err(anyhow::anyhow!("Model does not support soft delete"));
        }

        if ids.is_empty() {
            return Ok(true);
        }

        // 构建批量恢复 SQL
        let table = config.full_table();
        let default_value = match &config.soft_delete_default {
            SoftDeleteDefault::Null => "NULL",
            SoftDeleteDefault::Zero => "0",
            SoftDeleteDefault::Empty => "''",
        };

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE {} SET {} = {} WHERE {} IN ({})",
            table, config.delete_time_field, default_value, config.pk, placeholders.join(", ")
        );

        // 执行恢复
        executor::execute_with_params(&sql, ids).await?;

        Ok(true)
    }

    /// 强制删除记录（物理删除）
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 删除成功返回 true
    pub async fn force_delete(config: &ModelConfig, id: Value) -> anyhow::Result<bool> {
        // 构建删除 SQL
        let table = config.full_table();
        let sql = format!("DELETE FROM {} WHERE {} = ?", safe_escape_id(&table), safe_escape_id(&config.pk));

        // 执行删除
        executor::execute_with_params(&sql, &[id]).await?;

        Ok(true)
    }

    /// 批量强制删除记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `ids`: 主键值列表
    ///
    /// # 返回值
    /// 删除成功返回 true
    pub async fn force_delete_batch(config: &ModelConfig, ids: &[Value]) -> anyhow::Result<bool> {
        if ids.is_empty() {
            return Ok(true);
        }

        // 构建批量删除 SQL
        let table = config.full_table();
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            table, config.pk, placeholders.join(", ")
        );

        // 执行删除
        executor::execute_with_params(&sql, ids).await?;

        Ok(true)
    }

    /// 清空软删除数据（永久删除所有软删除的记录）
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 删除成功返回 true
    pub async fn clear_trashed(config: &ModelConfig) -> anyhow::Result<bool> {
        // 检查是否启用软删除
        if !config.soft_delete {
            return Err(anyhow::anyhow!("Model does not support soft delete"));
        }

        // 构建清空 SQL
        let table = config.full_table();
        let condition = match &config.soft_delete_default {
            SoftDeleteDefault::Null => format!("{} IS NOT NULL", safe_escape_id(&config.delete_time_field)),
            SoftDeleteDefault::Zero => format!("{} != 0", safe_escape_id(&config.delete_time_field)),
            SoftDeleteDefault::Empty => format!("{} != ''", safe_escape_id(&config.delete_time_field)),
        };

        let sql = format!("DELETE FROM {} WHERE {}", safe_escape_id(&table), condition);

        // 执行删除
        executor::execute(&sql).await?;

        Ok(true)
    }

    /// 统计软删除记录数量
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 软删除记录数量
    pub async fn count_trashed(config: &ModelConfig) -> anyhow::Result<i64> {
        // 检查是否启用软删除
        if !config.soft_delete {
            return Ok(0);
        }

        // 构建统计 SQL
        let table = config.full_table();
        let condition = match &config.soft_delete_default {
            SoftDeleteDefault::Null => format!("{} IS NOT NULL", safe_escape_id(&config.delete_time_field)),
            SoftDeleteDefault::Zero => format!("{} != 0", safe_escape_id(&config.delete_time_field)),
            SoftDeleteDefault::Empty => format!("{} != ''", safe_escape_id(&config.delete_time_field)),
        };

        let sql = format!("SELECT COUNT(*) as count FROM {} WHERE {}", safe_escape_id(&table), condition);

        // 执行查询
        let result = executor::query(&sql).await?;

        // 提取计数值
        Ok(result
            .scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            })
            .unwrap_or(0))
    }
}

/// 为 ModelInstance 实现软删除扩展
/// 使用 ?Send 允许非 Send 的 future
#[async_trait::async_trait(?Send)]
impl SoftDeleteExt for ModelInstance {
    /// 包含软删除数据
    fn with_trashed(&mut self) -> &mut Self {
        // 设置查询状态（需要在查询时处理）
        // 这里只是标记，实际查询在 SoftDeleteQueryBuilder 中处理
        self
    }

    /// 仅查询软删除数据
    fn only_trashed(&mut self) -> &mut Self {
        // 设置查询状态
        self
    }

    /// 恢复软删除数据
    async fn restore(&mut self) -> anyhow::Result<bool> {
        // 检查是否启用软删除
        if !self.config.soft_delete {
            return Err(anyhow::anyhow!("Model does not support soft delete"));
        }

        // 获取主键值
        let pk_value = self.get_key();

        // 执行恢复
        SoftDeleteOperator::restore(&self.config, pk_value).await?;

        // 更新模型状态
        let default_value = match &self.config.soft_delete_default {
            SoftDeleteDefault::Null => Value::Null,
            SoftDeleteDefault::Zero => Value::Int(0),
            SoftDeleteDefault::Empty => Value::String(String::new()),
        };
        self.attributes.insert(self.config.delete_time_field.clone(), default_value);
        self.exists = true;

        Ok(true)
    }

    /// 强制物理删除
    async fn force_delete(&mut self) -> anyhow::Result<bool> {
        // 获取主键值
        let pk_value = self.get_key();

        // 执行物理删除
        SoftDeleteOperator::force_delete(&self.config, pk_value).await?;

        // 更新模型状态
        self.exists = false;

        Ok(true)
    }
}

/// 软删除模型扩展方法
impl ModelInstance {
    /// 检查是否已被软删除
    ///
    /// # 返回值
    /// 如果已被软删除返回 true
    pub fn is_trashed(&self) -> bool {
        if !self.config.soft_delete {
            return false;
        }

        let delete_time = self.get_attr(&self.config.delete_time_field);

        match &self.config.soft_delete_default {
            SoftDeleteDefault::Null => !matches!(delete_time, Value::Null),
            SoftDeleteDefault::Zero => {
                match delete_time {
                    Value::Int(i) => i != 0,
                    _ => true,
                }
            }
            SoftDeleteDefault::Empty => {
                match delete_time {
                    Value::String(s) => !s.is_empty(),
                    _ => true,
                }
            }
        }
    }

    /// 获取删除时间
    ///
    /// # 返回值
    /// 删除时间值
    pub fn get_deleted_at(&self) -> Value {
        self.get_attr(&self.config.delete_time_field)
    }

    /// 设置强制删除标记
    ///
    /// # 返回值
    /// 自身引用
    pub fn force(&mut self) -> &mut Self {
        // 设置强制删除标记
        self.config.soft_delete = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_soft_delete_config() -> ModelConfig {
        ModelConfig {
            table: "users".to_string(),
            pk: "id".to_string(),
            soft_delete: true,
            delete_time_field: "deleted_at".to_string(),
            soft_delete_default: SoftDeleteDefault::Null,
            ..ModelConfig::default()
        }
    }

    #[test]
    fn test_soft_delete_query_builder() {
        let config = create_soft_delete_config();
        let builder = SoftDeleteQueryBuilder::new(config);

        // 测试默认状态
        let condition = builder.build_soft_delete_condition();
        assert!(condition.is_some());
        assert!(condition.unwrap().contains("IS NULL"));
    }

    #[test]
    fn test_with_trashed() {
        let config = create_soft_delete_config();
        let mut builder = SoftDeleteQueryBuilder::new(config);

        builder.with_trashed();

        // 测试包含软删除状态
        let condition = builder.build_soft_delete_condition();
        assert!(condition.is_none());
    }

    #[test]
    fn test_only_trashed() {
        let config = create_soft_delete_config();
        let mut builder = SoftDeleteQueryBuilder::new(config);

        builder.only_trashed();

        // 测试仅查询软删除状态
        let condition = builder.build_soft_delete_condition();
        assert!(condition.is_some());
        assert!(condition.unwrap().contains("IS NOT NULL"));
    }

    #[test]
    fn test_is_trashed() {
        let config = create_soft_delete_config();
        let mut model = ModelInstance::new(config);

        // 未删除状态
        model.attributes.insert("deleted_at".to_string(), Value::Null);
        assert!(!model.is_trashed());

        // 已删除状态
        model.attributes.insert("deleted_at".to_string(), Value::String("2024-01-01".to_string()));
        assert!(model.is_trashed());
    }

    #[test]
    fn test_trashed_state() {
        assert_eq!(TrashedState::Default, TrashedState::Default);
        assert_ne!(TrashedState::Default, TrashedState::WithTrashed);
        assert_ne!(TrashedState::WithTrashed, TrashedState::OnlyTrashed);
    }
}
