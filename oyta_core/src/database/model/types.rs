//! 模型类型定义模块
//!
//! 包含模型相关的数据结构定义

use std::collections::HashMap;

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
    ///
    /// # 返回值
    /// 完整表名字符串
    pub fn full_table(&self) -> String {
        format!("{}{}", self.prefix, self.table)
    }
}
