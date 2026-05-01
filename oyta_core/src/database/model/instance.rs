//! 模型实例模块
//!
//! 代表数据库中的一行记录，实现 Active Record 模式

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;

use super::helpers::{class_to_table, current_datetime, json_to_value, soft_delete_condition, value_to_json_string};
use super::paginate::PaginateResult;
use super::query_ext::ModelQueryBuilder;
use super::types::{ModelConfig, RelationType};

/// 安全转义SQL标识符
/// 使用反引号包裹，防止SQL注入和保留字冲突
#[inline]
fn escape_id(name: &str) -> String {
    if name.is_empty() {
        return "``".to_string();
    }
    format!("`{}`", name.replace('`', "``"))
}

/// 验证标识符是否安全
/// 只允许字母、数字、下划线和点号
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

/// 模型实例
/// 代表数据库中的一行记录
/// 实现 Active Record 模式
#[derive(Debug, Clone)]
pub struct ModelInstance {
    /// 模型配置
    pub config: ModelConfig,
    /// 原始数据（从数据库读取的值）
    pub original: HashMap<String, Value>,
    /// 当前属性数据（可能经过修改器处理）
    pub attributes: HashMap<String, Value>,
    /// 已修改的字段列表（用于脏检查）
    pub dirty: Vec<String>,
    /// 是否为新记录（未持久化到数据库）
    pub exists: bool,
}

impl ModelInstance {
    /// 创建新的模型实例
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 新的空模型实例
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            original: HashMap::new(),
            attributes: HashMap::new(),
            dirty: Vec::new(),
            exists: false,
        }
    }

    /// 从数据库行数据创建模型实例
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `row`: 数据库行数据（字段名 → 值）
    ///
    /// # 返回值
    /// 已填充数据的模型实例
    pub fn from_row(config: ModelConfig, row: HashMap<String, Value>) -> Self {
        // 复制原始数据
        let original = row.clone();
        Self {
            config,
            original,
            attributes: row,
            dirty: Vec::new(),
            exists: true,
        }
    }

    /// 获取属性值（经过获取器处理）
    ///
    /// # 参数
    /// - `key`: 属性名
    ///
    /// # 返回值
    /// 属性值，如果不存在返回 Null
    pub fn get_attr(&self, key: &str) -> Value {
        // 获取原始值
        let raw_value = self.attributes.get(key).cloned().unwrap_or(Value::Null);

        // 检查是否有获取器
        if let Some(getter_name) = self.config.getters.get(key) {
            // 获取器需要通过解释器执行，这里返回标记值
            // 实际执行在解释器的模型方法调用中处理
            return Value::String(format!(
                "__getter:{}:{}",
                getter_name,
                raw_value.to_string_value()
            ));
        }

        // 检查是否为 JSON 字段，自动反序列化
        if self.config.json_fields.contains(&key.to_string()) {
            if let Value::String(s) = &raw_value {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                    return json_to_value(&val);
                }
            }
        }

        // 检查字段类型转换
        if let Some(field_type) = self.config.field_types.get(key) {
            return super::helpers::apply_field_type(&raw_value, field_type);
        }

        raw_value
    }

    /// 设置属性值（经过修改器处理）
    ///
    /// # 参数
    /// - `key`: 属性名
    /// - `value`: 属性值
    pub fn set_attr(&mut self, key: &str, value: Value) {
        // 检查只读字段
        if self.config.readonly_fields.contains(&key.to_string()) && self.exists {
            return;
        }

        // 处理值
        let final_value = if let Some(_setter_name) = self.config.setters.get(key) {
            // 修改器需要通过解释器执行，这里存储标记
            value
        } else {
            // 检查是否为 JSON 字段，自动序列化
            if self.config.json_fields.contains(&key.to_string()) {
                value_to_json_string(&value)
            } else {
                value
            }
        };

        // 标记为脏字段
        if !self.dirty.contains(&key.to_string()) {
            self.dirty.push(key.to_string());
        }

        // 设置属性值
        self.attributes.insert(key.to_string(), final_value);
    }

    /// 获取主键值
    ///
    /// # 返回值
    /// 主键字段的值
    pub fn get_key(&self) -> Value {
        self.get_attr(&self.config.pk)
    }

    /// 判断是否为脏数据（有未保存的修改）
    ///
    /// # 返回值
    /// 如果有未保存的修改返回 true
    pub fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// 判断指定字段是否为脏数据
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 如果字段已修改返回 true
    pub fn is_field_dirty(&self, field: &str) -> bool {
        self.dirty.contains(&field.to_string())
    }

    /// 获取脏数据（已修改但未保存的字段）
    ///
    /// # 返回值
    /// 脏字段及其值的映射
    pub fn get_dirty(&self) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        for key in &self.dirty {
            if let Some(value) = self.attributes.get(key) {
                result.insert(key.clone(), value.clone());
            }
        }
        result
    }

    /// 同步原始数据（保存后调用）
    ///
    /// 将当前属性复制到原始数据，并清空脏字段列表
    pub fn sync_original(&mut self) {
        self.original = self.attributes.clone();
        self.dirty.clear();
    }

    /// 将模型数据转换为 Value（用于解释器）
    /// 应用隐藏字段和追加字段
    ///
    /// # 返回值
    /// 关联数组形式的 Value
    pub fn to_value(&self) -> Value {
        let mut data = Vec::new();

        // 遍历所有属性
        for (key, _value) in &self.attributes {
            // 跳过隐藏字段
            if self.config.hidden_fields.contains(key) {
                continue;
            }

            // 获取器处理
            let display_value = self.get_attr(key);
            data.push((key.clone(), display_value));
        }

        // 追加字段
        for append_key in &self.config.append_fields {
            let value = self.get_attr(append_key);
            data.push((append_key.clone(), value));
        }

        Value::AssociativeArray(data)
    }

    // ==================== Active Record 操作 ====================

    /// 保存模型到数据库
    /// 如果是新记录则 INSERT，否则 UPDATE 脏数据
    ///
    /// # 返回值
    /// 保存成功返回 true
    pub async fn save(&mut self) -> anyhow::Result<bool> {
        if self.exists {
            self.update().await
        } else {
            self.insert().await
        }
    }

    /// 插入新记录
    ///
    /// # 返回值
    /// 插入成功返回 true
    pub async fn insert(&mut self) -> anyhow::Result<bool> {
        // 自动写入创建时间
        if self.config.auto_timestamp {
            let now = current_datetime(&self.config.datetime_format);
            self.attributes
                .insert(self.config.create_time_field.clone(), Value::String(now.clone()));
            self.attributes
                .insert(self.config.update_time_field.clone(), Value::String(now));
        }

        // 获取完整表名
        let table = self.config.full_table();
        let qb = QueryBuilder::new(&table);

        // 构建 INSERT 语句
        let sql = qb.build_insert_sql(&self.attributes);

        // 执行插入
        let result = executor::query(&sql).await?;

        // 设置自增主键
        if let Some(id) = result.last_insert_id {
            self.attributes.insert(self.config.pk.clone(), Value::Int(id));
        }

        // 标记为已存在
        self.exists = true;
        self.sync_original();

        Ok(true)
    }

    /// 更新已有记录（仅更新脏数据）
    ///
    /// # 返回值
    /// 更新成功返回 true，无修改返回 false
    pub async fn update(&mut self) -> anyhow::Result<bool> {
        // 检查是否有修改
        if !self.is_dirty() {
            return Ok(false);
        }

        // 自动写入更新时间
        if self.config.auto_timestamp {
            let now = current_datetime(&self.config.datetime_format);
            self.attributes
                .insert(self.config.update_time_field.clone(), Value::String(now));
        }

        // 获取脏数据
        let table = self.config.full_table();
        let dirty = self.get_dirty();

        if dirty.is_empty() {
            return Ok(false);
        }

        // 构建 UPDATE 语句
        let mut qb = QueryBuilder::new(&table);
        let pk = self.config.pk.clone();
        let pk_value = self.get_key();

        // 设置 WHERE 条件
        qb = qb.where_clause(&pk, "=", pk_value);

        let sql = qb.build_update_sql(&dirty);

        // 执行更新
        executor::execute(&sql).await?;

        // 同步原始数据
        self.sync_original();

        Ok(true)
    }

    /// 删除记录
    /// 如果启用软删除，则设置删除时间而不是物理删除
    ///
    /// # 返回值
    /// 删除成功返回 true
    pub async fn delete(&mut self) -> anyhow::Result<bool> {
        if self.config.soft_delete {
            // 软删除：设置删除时间字段
            let now = current_datetime(&self.config.datetime_format);
            let table = self.config.full_table();
            let pk = self.config.pk.clone();
            let pk_value = self.get_key();

            // 构建更新语句
            let sql = format!(
                "UPDATE {} SET {} = ? WHERE {} = ?",
                safe_escape_id(&table), safe_escape_id(&self.config.delete_time_field), safe_escape_id(&pk)
            );

            // 执行软删除
            executor::execute_with_params(&sql, &[Value::String(now), pk_value]).await?;
        } else {
            // 物理删除
            let table = self.config.full_table();
            let pk = self.config.pk.clone();
            let pk_value = self.get_key();

            // 构建删除语句
            let sql = format!("DELETE FROM {} WHERE {} = ?", safe_escape_id(&table), safe_escape_id(&pk));

            // 执行物理删除
            executor::execute_with_params(&sql, &[pk_value]).await?;
        }

        // 标记为不存在
        self.exists = false;
        Ok(true)
    }

    // ==================== 静态查询方法 ====================

    /// 创建查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 查询构建器实例
    ///
    /// # 示例
    /// ```php
    /// User::where('id', 1)->find();
    /// ```
    pub fn query(config: ModelConfig) -> ModelQueryBuilder {
        // 创建并返回查询构建器
        ModelQueryBuilder::new(config)
    }

    /// 创建带 WHERE 条件的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `args`: 参数（可以是值，或操作符+值）
    ///
    /// # 返回值
    /// 查询构建器实例
    ///
    /// # 示例
    /// ```php
    /// User::where('id', 1)->find();
    /// User::where('name', 'like', '%think%')->select();
    /// ```
    pub fn where_builder(config: ModelConfig, field: &str, operator: &str, value: Value) -> ModelQueryBuilder {
        // 创建查询构建器并添加条件
        ModelQueryBuilder::new(config).where_condition(field, operator, value)
    }

    /// 创建带 WHERE IN 条件的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 查询构建器实例
    pub fn where_in_builder(config: ModelConfig, field: &str, values: Vec<Value>) -> ModelQueryBuilder {
        // 创建查询构建器并添加 IN 条件
        ModelQueryBuilder::new(config).where_in(field, values)
    }

    /// 创建带 ORDER BY 条件的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回值
    /// 查询构建器实例
    pub fn order_builder(config: ModelConfig, field: &str, direction: &str) -> ModelQueryBuilder {
        // 创建查询构建器并添加排序
        ModelQueryBuilder::new(config).order(field, direction)
    }

    /// 创建带 LIMIT 条件的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `limit`: 限制数量
    ///
    /// # 返回值
    /// 查询构建器实例
    pub fn limit_builder(config: ModelConfig, limit: usize) -> ModelQueryBuilder {
        // 创建查询构建器并添加限制
        ModelQueryBuilder::new(config).limit(limit)
    }

    /// 创建带预加载的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 查询构建器实例
    pub fn with_builder(config: ModelConfig, relations: Vec<&str>) -> ModelQueryBuilder {
        // 创建查询构建器并添加预加载
        ModelQueryBuilder::new(config).with(relations)
    }

    /// 创建带 JOIN 预加载的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 查询构建器实例
    pub fn with_join_builder(config: ModelConfig, relations: Vec<&str>) -> ModelQueryBuilder {
        // 创建查询构建器并添加 JOIN 预加载
        ModelQueryBuilder::new(config).with_join(relations)
    }

    /// 根据主键查找记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回 None
    pub async fn find(config: &ModelConfig, id: Value) -> anyhow::Result<Option<Self>> {
        // 构建查询语句
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} = ?", safe_escape_id(&table), safe_escape_id(&config.pk));

        // 添加软删除条件
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        // 执行查询
        let result = executor::query_with_params(&sql, &[id]).await?;

        // 返回结果
        if let Some(row) = result.first() {
            Ok(Some(Self::from_row(config.clone(), row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 查询所有记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 模型实例列表
    pub async fn select(config: &ModelConfig) -> anyhow::Result<Vec<Self>> {
        // 构建查询语句
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {}", safe_escape_id(&table));

        // 添加软删除条件
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} WHERE {}", sql, condition);
        }

        // 执行查询
        let result = executor::query(&sql).await?;

        // 转换为模型实例列表
        Ok(result
            .rows
            .into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect())
    }

    /// 根据条件查询单条记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回 None
    pub async fn where_first(
        config: &ModelConfig,
        field: &str,
        operator: &str,
        value: Value,
    ) -> anyhow::Result<Option<Self>> {
        // 构建查询语句
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} {} ?", safe_escape_id(&table), safe_escape_id(field), operator);

        // 添加软删除条件
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        // 限制返回一条
        sql = format!("{} LIMIT 1", sql);

        // 执行查询
        let result = executor::query_with_params(&sql, &[value]).await?;

        // 返回结果
        if let Some(row) = result.first() {
            Ok(Some(Self::from_row(config.clone(), row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 根据条件查询多条记录
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 模型实例列表
    pub async fn where_all(
        config: &ModelConfig,
        field: &str,
        operator: &str,
        value: Value,
    ) -> anyhow::Result<Vec<Self>> {
        // 构建查询语句
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} {} ?", safe_escape_id(&table), safe_escape_id(field), operator);

        // 添加软删除条件
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        // 执行查询
        let result = executor::query_with_params(&sql, &[value]).await?;

        // 转换为模型实例列表
        Ok(result
            .rows
            .into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect())
    }

    /// 统计记录数
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 记录总数
    pub async fn count(config: &ModelConfig) -> anyhow::Result<i64> {
        // 构建查询语句
        let table = config.full_table();
        let mut sql = format!("SELECT COUNT(*) as count FROM {}", safe_escape_id(&table));

        // 添加软删除条件
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} WHERE {}", sql, condition);
        }

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

    /// 分页查询
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `page`: 页码（从 1 开始）
    /// - `page_size`: 每页条数
    ///
    /// # 返回值
    /// 分页结果
    pub async fn paginate(
        config: &ModelConfig,
        page: i64,
        page_size: i64,
    ) -> anyhow::Result<PaginateResult> {
        let table = config.full_table();

        // 查询总数
        let mut count_sql = format!("SELECT COUNT(*) as count FROM {}", safe_escape_id(&table));
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            count_sql = format!("{} WHERE {}", count_sql, condition);
        }

        // 执行计数查询
        let total = executor::query(&count_sql)
            .await?
            .scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            })
            .unwrap_or(0);

        // 查询当前页数据
        let offset = (page - 1) * page_size;
        let mut data_sql = format!("SELECT * FROM {}", safe_escape_id(&table));
        if config.soft_delete {
            let condition =
                soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            data_sql = format!("{} WHERE {}", data_sql, condition);
        }
        data_sql = format!("{} LIMIT {} OFFSET {}", data_sql, page_size, offset);

        // 执行数据查询
        let result = executor::query(&data_sql).await?;
        let items: Vec<Self> = result
            .rows
            .into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect();

        // 计算总页数
        let total_pages = if total > 0 {
            (total + page_size - 1) / page_size
        } else {
            0
        };

        Ok(PaginateResult {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    // ==================== 关联关系 ====================

    /// 加载关联关系
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    ///
    /// # 返回值
    /// 关联的模型实例
    pub async fn load_relation(
        &self,
        relation_name: &str,
    ) -> anyhow::Result<Option<ModelInstance>> {
        // 获取关联定义
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(None),
        };

        match relation.relation_type {
            // 一对一关联：查询单条记录
            RelationType::HasOne | RelationType::BelongsTo => {
                let local_value = self.get_attr(&relation.local_key);
                let related_config = ModelConfig {
                    table: class_to_table(&relation.related_model),
                    ..ModelConfig::default()
                };

                ModelInstance::where_first(&related_config, &relation.foreign_key, "=", local_value)
                    .await
            }
            _ => Ok(None),
        }
    }

    /// 加载一对多关联
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    ///
    /// # 返回值
    /// 关联的模型实例列表
    pub async fn load_has_many(
        &self,
        relation_name: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        // 获取关联定义
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        // 构建关联配置
        let local_value = self.get_attr(&relation.local_key);
        let related_config = ModelConfig {
            table: class_to_table(&relation.related_model),
            ..ModelConfig::default()
        };

        // 查询关联记录
        ModelInstance::where_all(&related_config, &relation.foreign_key, "=", local_value).await
    }

    /// 加载多对多关联（通过中间表）
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    /// - `pivot_table`: 中间表名
    /// - `pivot_foreign_key`: 中间表外键
    /// - `pivot_related_key`: 中间表关联键
    ///
    /// # 返回值
    /// 关联的模型实例列表
    pub async fn load_belongs_to_many(
        &self,
        relation_name: &str,
        pivot_table: &str,
        pivot_foreign_key: &str,
        pivot_related_key: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        // 获取关联定义
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        // 构建查询参数
        let local_value = self.get_attr(&relation.local_key);
        let related_table = class_to_table(&relation.related_model);
        let related_pk = "id".to_string();

        // 构建联表查询语句
        // SELECT related_table.* FROM related_table
        // INNER JOIN pivot_table ON related_table.id = pivot_table.pivot_related_key
        // WHERE pivot_table.pivot_foreign_key = ?
        let sql = format!(
            "SELECT {}.* FROM {} INNER JOIN {} ON {}.{} = {}.{} WHERE {}.{} = ?",
            related_table,
            related_table,
            pivot_table,
            related_table,
            related_pk,
            pivot_table,
            pivot_related_key,
            pivot_table,
            pivot_foreign_key
        );

        // 执行查询
        let result = executor::query_with_params(&sql, &[local_value]).await?;

        // 构建关联配置
        let related_config = ModelConfig {
            table: related_table,
            ..ModelConfig::default()
        };

        // 转换为模型实例列表
        Ok(result
            .rows
            .into_iter()
            .map(|row| ModelInstance::from_row(related_config.clone(), row))
            .collect())
    }
}
