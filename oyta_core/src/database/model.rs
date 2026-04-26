//! Model 基类模块
//!
//! 实现 ThinkPHP 8.0 风格的 ORM 模型
//! 支持 Active Record 模式、关联关系、获取器/修改器、软删除、自动时间戳等
//!
//! # 使用方式
//! 在 PHP 代码中定义模型类继承 Model：
//! ```php
//! class User extends Model {
//!     protected $table = 'users';
//!     protected $pk = 'id';
//! }
//! ```
//!
//! Rust 端通过 ModelInstance 在解释器中实现模型操作

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;

/// 模型配置
/// 定义模型的元数据配置
/// 对应 PHP 模型类中的 protected 属性
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// 表名（不含前缀）
    /// 对应 PHP: protected $table = 'users';
    pub table: String,
    /// 主键字段名
    /// 对应 PHP: protected $pk = 'id';
    /// 默认为 "id"
    pub pk: String,
    /// 是否自动写入时间戳
    /// 对应 PHP: protected $autoWriteTimestamp = true;
    pub auto_timestamp: bool,
    /// 创建时间字段名
    /// 对应 PHP: protected $createTime = 'create_time';
    pub create_time_field: String,
    /// 更新时间字段名
    /// 对应 PHP: protected $updateTime = 'update_time';
    pub update_time_field: String,
    /// 时间字段格式
    /// "datetime" 或 "timestamp"
    pub datetime_format: String,
    /// 是否使用软删除
    /// 对应 PHP: protected $softDelete = true;
    pub soft_delete: bool,
    /// 软删除字段名
    /// 对应 PHP: protected $deleteTime = 'delete_time';
    pub delete_time_field: String,
    /// 软删除默认值（未删除时的字段值）
    pub soft_delete_default: SoftDeleteDefault,
    /// 表前缀
    /// 从数据库配置中获取
    pub prefix: String,
    /// 是否严格字段检查
    /// 对应 PHP: protected $strict = true;
    pub strict: bool,
    /// 获取器定义
    /// 键：字段名，值：获取器方法名
    /// 对应 PHP: protected $getAttr = ['name' => 'getNameAttr'];
    pub getters: HashMap<String, String>,
    /// 修改器定义
    /// 键：字段名，值：修改器方法名
    /// 对应 PHP: protected $setAttr = ['name' => 'setNameAttr'];
    pub setters: HashMap<String, String>,
    /// 搜索器定义
    /// 键：字段名，值：搜索器方法名
    /// 对应 PHP: protected $searchAttr = ['name' => 'searchNameAttr'];
    pub searchers: HashMap<String, String>,
    /// 字段类型定义
    /// 键：字段名，值：类型（int/string/float/json/datetime/array/serialize）
    /// 对应 PHP: protected $type = ['status' => 'int', 'config' => 'json'];
    pub field_types: HashMap<String, FieldType>,
    /// JSON 字段列表
    /// 对应 PHP: protected $json = ['config', 'extra'];
    pub json_fields: Vec<String>,
    /// 只读字段列表
    /// 对应 PHP: protected $readonly = ['id', 'create_time'];
    pub readonly_fields: Vec<String>,
    /// 隐藏字段列表（输出时排除）
    /// 对应 PHP: protected $hidden = ['password', 'salt'];
    pub hidden_fields: Vec<String>,
    /// 追加属性列表（输出时追加获取器计算的字段）
    /// 对应 PHP: protected $append = ['status_text'];
    pub append_fields: Vec<String>,
    /// 关联关系定义
    /// 键：关联名称，值：关联定义
    /// 对应 PHP 中的关联方法
    pub relations: HashMap<String, RelationDef>,
}

/// 软删除默认值类型
#[derive(Debug, Clone)]
pub enum SoftDeleteDefault {
    /// 字段值为 NULL 表示未删除
    Null,
    /// 字段值为 0 表示未删除
    Zero,
    /// 字段值为空字符串表示未删除
    Empty,
}

/// 字段类型枚举
/// 定义模型字段的自动类型转换
#[derive(Debug, Clone)]
pub enum FieldType {
    /// 整数类型
    Int,
    /// 浮点类型
    Float,
    /// 字符串类型
    String,
    /// 布尔类型
    Bool,
    /// JSON 类型（自动序列化/反序列化）
    Json,
    /// 日期时间类型
    Datetime,
    /// 数组类型（PHP serialize 格式）
    Array,
    /// 枚举类型
    Enum(Vec<String>),
}

/// 关联关系定义
#[derive(Debug, Clone)]
pub struct RelationDef {
    /// 关联类型
    pub relation_type: RelationType,
    /// 关联的模型类名
    pub related_model: String,
    /// 外键字段名
    pub foreign_key: String,
    /// 主键字段名（本表或关联表的主键）
    pub local_key: String,
}

/// 关联关系类型
#[derive(Debug, Clone, Copy)]
pub enum RelationType {
    /// 一对一：hasOne
    /// 如：User hasOne Profile
    HasOne,
    /// 一对多：hasMany
    /// 如：User hasMany Articles
    HasMany,
    /// 反向一对一：belongsTo
    /// 如：Profile belongsTo User
    BelongsTo,
    /// 多对多：belongsToMany
    /// 如：User belongsToMany Role（通过中间表）
    BelongsToMany,
    /// 远程一对一：hasOneThrough
    /// 如：User hasOneThrough History（通过 Profile）
    HasOneThrough,
    /// 远程一对多：hasManyThrough
    /// 如：User hasManyThrough Comment（通过 Article）
    HasManyThrough,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            table: String::new(),
            pk: "id".to_string(),
            auto_timestamp: false,
            create_time_field: "create_time".to_string(),
            update_time_field: "update_time".to_string(),
            datetime_format: "datetime".to_string(),
            soft_delete: false,
            delete_time_field: "delete_time".to_string(),
            soft_delete_default: SoftDeleteDefault::Null,
            prefix: String::new(),
            strict: true,
            getters: HashMap::new(),
            setters: HashMap::new(),
            searchers: HashMap::new(),
            field_types: HashMap::new(),
            json_fields: Vec::new(),
            readonly_fields: Vec::new(),
            hidden_fields: Vec::new(),
            append_fields: Vec::new(),
            relations: HashMap::new(),
        }
    }
}

impl ModelConfig {
    /// 获取完整表名（含前缀）
    pub fn full_table(&self) -> String {
        format!("{}{}", self.prefix, self.table)
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
    pub fn from_row(config: ModelConfig, row: HashMap<String, Value>) -> Self {
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
    pub fn get_attr(&self, key: &str) -> Value {
        let raw_value = self.attributes.get(key).cloned().unwrap_or(Value::Null);

        // 检查是否有获取器
        if let Some(getter_name) = self.config.getters.get(key) {
            // 获取器需要通过解释器执行，这里返回标记值
            // 实际执行在解释器的模型方法调用中处理
            return Value::String(format!("__getter:{}:{}", getter_name, raw_value.to_string_value()));
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
            return apply_field_type(&raw_value, field_type);
        }

        raw_value
    }

    /// 设置属性值（经过修改器处理）
    pub fn set_attr(&mut self, key: &str, value: Value) {
        // 检查只读字段
        if self.config.readonly_fields.contains(&key.to_string()) && self.exists {
            return;
        }

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

        self.attributes.insert(key.to_string(), final_value);
    }

    /// 获取主键值
    pub fn get_key(&self) -> Value {
        self.get_attr(&self.config.pk)
    }

    /// 判断是否为脏数据（有未保存的修改）
    pub fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// 判断指定字段是否为脏数据
    pub fn is_field_dirty(&self, field: &str) -> bool {
        self.dirty.contains(&field.to_string())
    }

    /// 获取脏数据（已修改但未保存的字段）
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
    pub fn sync_original(&mut self) {
        self.original = self.attributes.clone();
        self.dirty.clear();
    }

    /// 将模型数据转换为 Value（用于解释器）
    /// 应用隐藏字段和追加字段
    pub fn to_value(&self) -> Value {
        let mut data = Vec::new();

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
    pub async fn save(&mut self) -> anyhow::Result<bool> {
        if self.exists {
            self.update().await
        } else {
            self.insert().await
        }
    }

    /// 插入新记录
    pub async fn insert(&mut self) -> anyhow::Result<bool> {
        // 自动写入创建时间
        if self.config.auto_timestamp {
            let now = current_datetime(&self.config.datetime_format);
            self.attributes.insert(self.config.create_time_field.clone(), Value::String(now.clone()));
            self.attributes.insert(self.config.update_time_field.clone(), Value::String(now));
        }

        let table = self.config.full_table();
        let qb = QueryBuilder::new(&table);

        // 构建 INSERT 语句
        let sql = qb.build_insert_sql(&self.attributes);

        let result = executor::query(&sql).await?;

        if let Some(id) = result.last_insert_id {
            self.attributes.insert(self.config.pk.clone(), Value::Int(id));
        }

        self.exists = true;
        self.sync_original();

        Ok(true)
    }

    /// 更新已有记录（仅更新脏数据）
    pub async fn update(&mut self) -> anyhow::Result<bool> {
        if !self.is_dirty() {
            return Ok(false);
        }

        // 自动写入更新时间
        if self.config.auto_timestamp {
            let now = current_datetime(&self.config.datetime_format);
            self.attributes.insert(self.config.update_time_field.clone(), Value::String(now));
        }

        let table = self.config.full_table();
        let dirty = self.get_dirty();

        if dirty.is_empty() {
            return Ok(false);
        }

        let mut qb = QueryBuilder::new(&table);
        let pk = self.config.pk.clone();
        let pk_value = self.get_key();

        // 设置 WHERE 条件
        qb = qb.where_clause(&pk, "=", pk_value);

        let sql = qb.build_update_sql(&dirty);

        executor::execute(&sql).await?;

        self.sync_original();

        Ok(true)
    }

    /// 删除记录
    /// 如果启用软删除，则设置删除时间而不是物理删除
    pub async fn delete(&mut self) -> anyhow::Result<bool> {
        if self.config.soft_delete {
            // 软删除：设置删除时间字段
            let now = current_datetime(&self.config.datetime_format);
            let table = self.config.full_table();
            let pk = self.config.pk.clone();
            let pk_value = self.get_key();

            let sql = format!(
                "UPDATE {} SET {} = ? WHERE {} = ?",
                table, self.config.delete_time_field, pk
            );

            executor::execute_with_params(&sql, &[Value::String(now), pk_value]).await?;
        } else {
            // 物理删除
            let table = self.config.full_table();
            let pk = self.config.pk.clone();
            let pk_value = self.get_key();

            let sql = format!("DELETE FROM {} WHERE {} = ?", table, pk);
            executor::execute_with_params(&sql, &[pk_value]).await?;
        }

        self.exists = false;
        Ok(true)
    }

    // ==================== 静态查询方法 ====================

    /// 根据主键查找记录
    pub async fn find(config: &ModelConfig, id: Value) -> anyhow::Result<Option<Self>> {
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} = ?", table, config.pk);

        // 软删除条件
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        let result = executor::query_with_params(&sql, &[id]).await?;

        if let Some(row) = result.first() {
            Ok(Some(Self::from_row(config.clone(), row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 查询所有记录
    pub async fn select(config: &ModelConfig) -> anyhow::Result<Vec<Self>> {
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {}", table);

        // 软删除条件
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} WHERE {}", sql, condition);
        }

        let result = executor::query(&sql).await?;

        Ok(result.rows.into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect())
    }

    /// 根据条件查询单条记录
    pub async fn where_first(
        config: &ModelConfig,
        field: &str,
        operator: &str,
        value: Value,
    ) -> anyhow::Result<Option<Self>> {
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} {} ?", table, field, operator);

        // 软删除条件
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        sql = format!("{} LIMIT 1", sql);

        let result = executor::query_with_params(&sql, &[value]).await?;

        if let Some(row) = result.first() {
            Ok(Some(Self::from_row(config.clone(), row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 根据条件查询多条记录
    pub async fn where_all(
        config: &ModelConfig,
        field: &str,
        operator: &str,
        value: Value,
    ) -> anyhow::Result<Vec<Self>> {
        let table = config.full_table();
        let mut sql = format!("SELECT * FROM {} WHERE {} {} ?", table, field, operator);

        // 软删除条件
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} AND {}", sql, condition);
        }

        let result = executor::query_with_params(&sql, &[value]).await?;

        Ok(result.rows.into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect())
    }

    /// 统计记录数
    pub async fn count(config: &ModelConfig) -> anyhow::Result<i64> {
        let table = config.full_table();
        let mut sql = format!("SELECT COUNT(*) as count FROM {}", table);

        // 软删除条件
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            sql = format!("{} WHERE {}", sql, condition);
        }

        let result = executor::query(&sql).await?;
        Ok(result.scalar().and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        }).unwrap_or(0))
    }

    /// 分页查询
    pub async fn paginate(
        config: &ModelConfig,
        page: i64,
        page_size: i64,
    ) -> anyhow::Result<PaginateResult> {
        let table = config.full_table();

        // 查询总数
        let mut count_sql = format!("SELECT COUNT(*) as count FROM {}", table);
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            count_sql = format!("{} WHERE {}", count_sql, condition);
        }

        let total = executor::query(&count_sql).await?
            .scalar().and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            }).unwrap_or(0);

        // 查询当前页数据
        let offset = (page - 1) * page_size;
        let mut data_sql = format!("SELECT * FROM {}", table);
        if config.soft_delete {
            let condition = soft_delete_condition(&config.soft_delete_default, &config.delete_time_field);
            data_sql = format!("{} WHERE {}", data_sql, condition);
        }
        data_sql = format!("{} LIMIT {} OFFSET {}", data_sql, page_size, offset);

        let result = executor::query(&data_sql).await?;
        let items: Vec<Self> = result.rows.into_iter()
            .map(|row| Self::from_row(config.clone(), row))
            .collect();

        let total_pages = if total > 0 { (total + page_size - 1) / page_size } else { 0 };

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
    pub async fn load_relation(
        &self,
        relation_name: &str,
    ) -> anyhow::Result<Option<ModelInstance>> {
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(None),
        };

        match relation.relation_type {
            RelationType::HasOne | RelationType::BelongsTo => {
                // 一对一：查询单条记录
                let local_value = self.get_attr(&relation.local_key);
                let related_config = ModelConfig {
                    table: class_to_table(&relation.related_model),
                    ..ModelConfig::default()
                };

                ModelInstance::where_first(
                    &related_config,
                    &relation.foreign_key,
                    "=",
                    local_value,
                ).await
            }
            _ => Ok(None),
        }
    }

    /// 加载一对多关联
    pub async fn load_has_many(
        &self,
        relation_name: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let local_value = self.get_attr(&relation.local_key);
        let related_config = ModelConfig {
            table: class_to_table(&relation.related_model),
            ..ModelConfig::default()
        };

        ModelInstance::where_all(
            &related_config,
            &relation.foreign_key,
            "=",
            local_value,
        ).await
    }

    /// 加载多对多关联（通过中间表）
    pub async fn load_belongs_to_many(
        &self,
        relation_name: &str,
        pivot_table: &str,
        pivot_foreign_key: &str,
        pivot_related_key: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        let relation = match self.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let local_value = self.get_attr(&relation.local_key);
        let related_table = class_to_table(&relation.related_model);
        let related_pk = "id".to_string();

        // SELECT related_table.* FROM related_table
        // INNER JOIN pivot_table ON related_table.id = pivot_table.pivot_related_key
        // WHERE pivot_table.pivot_foreign_key = ?
        let sql = format!(
            "SELECT {}.* FROM {} INNER JOIN {} ON {}.{} = {}.{} WHERE {}.{} = ?",
            related_table, related_table,
            pivot_table,
            related_table, related_pk, pivot_table, pivot_related_key,
            pivot_table, pivot_foreign_key
        );

        let result = executor::query_with_params(&sql, &[local_value]).await?;

        let related_config = ModelConfig {
            table: related_table,
            ..ModelConfig::default()
        };

        Ok(result.rows.into_iter()
            .map(|row| ModelInstance::from_row(related_config.clone(), row))
            .collect())
    }
}

/// 分页查询结果
#[derive(Debug, Clone)]
pub struct PaginateResult {
    /// 当前页数据
    pub items: Vec<ModelInstance>,
    /// 总记录数
    pub total: i64,
    /// 当前页码
    pub page: i64,
    /// 每页条数
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

impl PaginateResult {
    /// 是否有下一页
    pub fn has_more(&self) -> bool {
        self.page < self.total_pages
    }

    /// 转换为 Value（用于解释器返回）
    pub fn to_value(&self) -> Value {
        let mut data = Vec::new();

        let items_values: Vec<Value> = self.items.iter()
            .map(|m| m.to_value())
            .collect();

        data.push(("total".to_string(), Value::Int(self.total)));
        data.push(("page".to_string(), Value::Int(self.page)));
        data.push(("page_size".to_string(), Value::Int(self.page_size)));
        data.push(("total_pages".to_string(), Value::Int(self.total_pages)));
        data.push(("has_more".to_string(), Value::Bool(self.has_more())));
        data.push(("items".to_string(), Value::IndexedArray(items_values)));

        Value::AssociativeArray(data)
    }
}

/// 生成软删除条件 SQL
fn soft_delete_condition(default: &SoftDeleteDefault, field: &str) -> String {
    match default {
        SoftDeleteDefault::Null => format!("{} IS NULL", field),
        SoftDeleteDefault::Zero => format!("{} = 0", field),
        SoftDeleteDefault::Empty => format!("{} = ''", field),
    }
}

/// 获取当前日期时间字符串
fn current_datetime(format: &str) -> String {
    let now = chrono::Local::now();
    match format {
        "timestamp" => now.timestamp().to_string(),
        _ => now.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// 将类名转换为表名
/// "User" → "user", "UserInfo" → "user_info"
fn class_to_table(class_name: &str) -> String {
    let short_name = if let Some(pos) = class_name.rfind('\\') {
        &class_name[pos + 1..]
    } else {
        class_name
    };

    let mut result = String::new();
    for (i, c) in short_name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// 应用字段类型转换
fn apply_field_type(value: &Value, field_type: &FieldType) -> Value {
    match field_type {
        FieldType::Int => Value::Int(value.to_int()),
        FieldType::Float => match value {
            Value::Float(f) => Value::Float(*f),
            _ => Value::Float(value.to_float()),
        },
        FieldType::String => Value::String(value.to_string_value()),
        FieldType::Bool => Value::Bool(value.is_truthy()),
        FieldType::Json => {
            if let Value::String(s) = value {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                    return json_to_value(&val);
                }
            }
            value.clone()
        }
        FieldType::Datetime => value.clone(),
        FieldType::Array => value.clone(),
        FieldType::Enum(variants) => {
            if let Value::String(s) = value {
                if variants.contains(&s.to_string()) {
                    Value::String(s.clone())
                } else {
                    Value::Null
                }
            } else {
                value.clone()
            }
        }
    }
}

/// 将 JSON 值转换为 Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::IndexedArray(values)
        }
        serde_json::Value::Object(obj) => {
            let pairs: Vec<(String, Value)> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::AssociativeArray(pairs)
        }
    }
}

/// 将 Value 转换为 JSON 字符串
fn value_to_json_string(value: &Value) -> Value {
    let json_val = value.to_json_value();
    match serde_json::to_string(&json_val) {
        Ok(s) => Value::String(s),
        Err(_) => value.clone(),
    }
}
