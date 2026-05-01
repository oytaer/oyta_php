//! 静态方法模块
//!
//! 实现模型的静态方法功能
//! 对应 ThinkPHP 的模型静态方法
//!
//! 主要功能：
//! - create 创建并保存
//! - update 静态更新
//! - destroy 静态删除
//! - findOrFail 查找或抛异常
//! - firstOrNew 查找或新建
//! - firstOrCreate 查找或创建
//! - updateOrCreate 更新或创建

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

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

/// 静态方法执行器
///
/// 提供模型的静态方法
pub struct StaticMethods;

impl StaticMethods {
    /// 创建并保存模型
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `data`: 模型数据
    ///
    /// # 返回值
    /// 创建的模型实例
    ///
    /// # 示例
    /// ```php
    /// User::create(['name' => 'thinkphp', 'email' => 'test@test.com']);
    /// ```
    pub async fn create(config: ModelConfig, data: HashMap<String, Value>) -> anyhow::Result<ModelInstance> {
        // 创建模型实例
        let mut model = ModelInstance::new(config);

        // 设置属性
        for (key, value) in data {
            model.set_attr(&key, value);
        }

        // 保存
        model.save().await?;

        Ok(model)
    }

    /// 静态更新方法
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `data`: 更新数据
    /// - `where_conditions`: WHERE 条件
    ///
    /// # 返回值
    /// 更新的行数
    ///
    /// # 示例
    /// ```php
    /// User::update(['status' => 1], ['id' => 1]);
    /// ```
    pub async fn update(
        config: &ModelConfig,
        data: &HashMap<String, Value>,
        where_conditions: &HashMap<String, Value>,
    ) -> anyhow::Result<i64> {
        // 构建更新 SQL
        let table = config.full_table();
        let set_parts: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();
        let where_parts: Vec<String> = where_conditions.keys().map(|k| format!("{} = ?", k)).collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table,
            set_parts.join(", "),
            where_parts.join(" AND ")
        );

        // 构建参数
        let mut params: Vec<Value> = data.values().cloned().collect();
        params.extend(where_conditions.values().cloned());

        // 执行更新
        let result = executor::query_with_params(&sql, &params).await?;

        // 返回影响的行数（将 u64 转换为 i64）
        Ok(result.affected_rows as i64)
    }

    /// 静态删除方法
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `ids`: 要删除的 ID 列表
    ///
    /// # 返回值
    /// 删除的行数
    ///
    /// # 示例
    /// ```php
    /// User::destroy(1);
    /// User::destroy([1, 2, 3]);
    /// ```
    pub async fn destroy(config: &ModelConfig, ids: &[Value]) -> anyhow::Result<i64> {
        if ids.is_empty() {
            return Ok(0);
        }

        // 检查是否启用软删除
        if config.soft_delete {
            // 软删除
            let table = config.full_table();
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();

            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let sql = format!(
                "UPDATE {} SET {} = ? WHERE {} IN ({})",
                table, config.delete_time_field, config.pk, placeholders.join(", ")
            );

            // 构建参数
            let mut params = vec![Value::String(now)];
            params.extend(ids.iter().cloned());

            // 执行更新
            let result = executor::query_with_params(&sql, &params).await?;

            // 返回影响的行数（将 u64 转换为 i64）
            Ok(result.affected_rows as i64)
        } else {
            // 物理删除
            let table = config.full_table();
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();

            let sql = format!(
                "DELETE FROM {} WHERE {} IN ({})",
                table, config.pk, placeholders.join(", ")
            );

            // 执行删除
            let result = executor::query_with_params(&sql, ids).await?;

            // 返回影响的行数（将 u64 转换为 i64）
            Ok(result.affected_rows as i64)
        }
    }

    /// 查找或抛出异常
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回错误
    ///
    /// # 示例
    /// ```php
    /// User::findOrFail(1);
    /// ```
    pub async fn find_or_fail(config: ModelConfig, id: Value) -> anyhow::Result<ModelInstance> {
        let result = ModelInstance::find(&config, id).await?;

        match result {
            Some(model) => Ok(model),
            None => Err(anyhow::anyhow!("Model not found")),
        }
    }

    /// 查找或新建实例
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `where_conditions`: 查找条件
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回新实例（未保存）
    ///
    /// # 示例
    /// ```php
    /// User::firstOrNew(['email' => 'test@test.com']);
    /// ```
    pub async fn first_or_new(
        config: ModelConfig,
        where_conditions: &HashMap<String, Value>,
    ) -> anyhow::Result<ModelInstance> {
        // 构建查询
        let table = config.full_table();
        let where_parts: Vec<String> = where_conditions.keys().map(|k| format!("{} = ?", k)).collect();

        let sql = format!(
            "SELECT * FROM {} WHERE {} LIMIT 1",
            table,
            where_parts.join(" AND ")
        );

        let params: Vec<Value> = where_conditions.values().cloned().collect();

        // 执行查询
        let result = executor::query_with_params(&sql, &params).await?;

        // 返回结果
        if let Some(row) = result.first() {
            Ok(ModelInstance::from_row(config, row.clone()))
        } else {
            // 创建新实例
            let mut model = ModelInstance::new(config);
            for (key, value) in where_conditions {
                model.attributes.insert(key.clone(), value.clone());
            }
            Ok(model)
        }
    }

    /// 查找或创建
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `where_conditions`: 查找条件
    /// - `create_data`: 创建时的额外数据
    ///
    /// # 返回值
    /// 找到返回模型实例，否则创建并返回
    ///
    /// # 示例
    /// ```php
    /// User::firstOrCreate(['email' => 'test@test.com'], ['name' => 'test']);
    /// ```
    pub async fn first_or_create(
        config: ModelConfig,
        where_conditions: &HashMap<String, Value>,
        create_data: Option<&HashMap<String, Value>>,
    ) -> anyhow::Result<ModelInstance> {
        // 先尝试查找
        let existing = Self::first_or_new(config.clone(), where_conditions).await?;

        if existing.exists {
            return Ok(existing);
        }

        // 创建新记录
        let mut model = existing;

        // 添加额外数据
        if let Some(data) = create_data {
            for (key, value) in data {
                model.attributes.insert(key.clone(), value.clone());
            }
        }

        // 保存
        model.save().await?;

        Ok(model)
    }

    /// 更新或创建
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `where_conditions`: 查找条件
    /// - `update_data`: 更新/创建数据
    ///
    /// # 返回值
    /// 模型实例
    ///
    /// # 示例
    /// ```php
    /// User::updateOrCreate(['email' => 'test@test.com'], ['name' => 'new_name']);
    /// ```
    pub async fn update_or_create(
        config: ModelConfig,
        where_conditions: &HashMap<String, Value>,
        update_data: &HashMap<String, Value>,
    ) -> anyhow::Result<ModelInstance> {
        // 先尝试查找
        let existing = Self::first_or_new(config.clone(), where_conditions).await?;

        if existing.exists {
            // 更新现有记录
            let mut model = existing;
            for (key, value) in update_data {
                model.set_attr(key, value.clone());
            }
            model.save().await?;
            Ok(model)
        } else {
            // 创建新记录
            let mut model = existing;

            // 添加更新数据
            for (key, value) in update_data {
                model.attributes.insert(key.clone(), value.clone());
            }

            // 保存
            model.save().await?;

            Ok(model)
        }
    }

    /// 获取单个字段值
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `where_conditions`: WHERE 条件
    ///
    /// # 返回值
    /// 字段值
    ///
    /// # 示例
    /// ```php
    /// User::where('id', 1)->value('name');
    /// ```
    pub async fn value(
        config: &ModelConfig,
        field: &str,
        where_conditions: Option<&HashMap<String, Value>>,
    ) -> anyhow::Result<Value> {
        // 构建查询
        let table = config.full_table();
        let mut sql = format!("SELECT {} FROM {}", safe_escape_id(field), safe_escape_id(&table));

        // 添加 WHERE 条件
        let params = if let Some(conditions) = where_conditions {
            let where_parts: Vec<String> = conditions.keys().map(|k| format!("{} = ?", safe_escape_id(k))).collect();
            sql.push_str(&format!(" WHERE {}", where_parts.join(" AND ")));
            conditions.values().cloned().collect()
        } else {
            vec![]
        };

        sql.push_str(" LIMIT 1");

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 返回结果
        Ok(result.scalar().cloned().unwrap_or(Value::Null))
    }

    /// 获取某列的值
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `key_field`: 作为键的字段（可选）
    /// - `where_conditions`: WHERE 条件
    ///
    /// # 返回值
    /// 字段值列表
    ///
    /// # 示例
    /// ```php
    /// User::column('name');
    /// User::column('name', 'id');
    /// ```
    pub async fn column(
        config: &ModelConfig,
        field: &str,
        key_field: Option<&str>,
        where_conditions: Option<&HashMap<String, Value>>,
    ) -> anyhow::Result<Value> {
        // 构建查询
        let table = config.full_table();
        let select_field = if let Some(key) = key_field {
            format!("{}, {}", safe_escape_id(key), safe_escape_id(field))
        } else {
            safe_escape_id(field)
        };

        let mut sql = format!("SELECT {} FROM {}", select_field, safe_escape_id(&table));

        // 添加 WHERE 条件
        let params = if let Some(conditions) = where_conditions {
            let where_parts: Vec<String> = conditions.keys().map(|k| format!("{} = ?", safe_escape_id(k))).collect();
            sql.push_str(&format!(" WHERE {}", where_parts.join(" AND ")));
            conditions.values().cloned().collect()
        } else {
            vec![]
        };

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 处理结果
        if let Some(key) = key_field {
            // 返回键值对
            let mut map = Vec::new();
            for row in result.rows {
                if let (Some(k), Some(v)) = (row.get(key), row.get(field)) {
                    map.push((k.to_string_value(), v.clone()));
                }
            }
            Ok(Value::AssociativeArray(map))
        } else {
            // 返回值列表
            let values: Vec<Value> = result.rows
                .iter()
                .filter_map(|row| row.get(field).cloned())
                .collect();
            Ok(Value::IndexedArray(values))
        }
    }

    /// 分块处理数据
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `size`: 每块大小
    /// - `callback`: 处理回调
    ///
    /// # 示例
    /// ```php
    /// User::chunk(100, function($users) {
    ///     foreach ($users as $user) {
    ///         // 处理
    ///     }
    /// });
    /// ```
    pub async fn chunk<F>(
        config: &ModelConfig,
        size: usize,
        mut callback: F,
    ) -> anyhow::Result<bool>
    where
        F: FnMut(&[ModelInstance]) -> bool,
    {
        let table = config.full_table();
        let mut offset = 0;

        loop {
            // 查询一块数据
            let sql = format!(
                "SELECT * FROM {} LIMIT {} OFFSET {}",
                table, size, offset
            );

            let result = executor::query(&sql).await?;

            // 检查是否为空
            if result.rows.is_empty() {
                break;
            }

            // 转换为模型实例
            let models: Vec<ModelInstance> = result.rows
                .into_iter()
                .map(|row| ModelInstance::from_row(config.clone(), row))
                .collect();

            // 调用回调
            let continue_processing = callback(&models);

            // 检查是否继续
            if !continue_processing || models.len() < size {
                break;
            }

            offset += size;
        }

        Ok(true)
    }

    /// 游标查询
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 模型实例迭代器（简化实现返回列表）
    ///
    /// # 示例
    /// ```php
    /// foreach (User::cursor() as $user) {
    ///     // 处理
    /// }
    /// ```
    pub async fn cursor(config: ModelConfig) -> anyhow::Result<Vec<ModelInstance>> {
        // 简化实现：返回所有数据
        ModelInstance::select(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ModelConfig {
        ModelConfig {
            table: "users".to_string(),
            pk: "id".to_string(),
            ..ModelConfig::default()
        }
    }

    #[test]
    fn test_static_methods_exist() {
        // 验证静态方法结构正确
        let config = create_test_config();

        // 这些测试验证方法存在，实际数据库操作需要集成测试
        assert!(!config.table.is_empty());
    }
}
