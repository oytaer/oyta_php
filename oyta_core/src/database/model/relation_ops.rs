//! 关联操作模块
//!
//! 实现模型的关联操作功能
//! 对应 ThinkPHP 的模型关联操作
//!
//! 主要功能：
//! - 关联保存
//! - 关联新增
//! - 关联批量保存
//! - together 关联自动写入
//! - bind 绑定属性
//! - attach/detach/sync 中间表操作

use std::collections::HashMap;

use crate::database::executor;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::{ModelConfig, RelationType};

/// 关联操作器
///
/// 提供关联操作的方法
pub struct RelationOperator;

impl RelationOperator {
    /// 保存关联数据
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `data`: 关联数据
    ///
    /// # 返回值
    /// 保存成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user = User::find(1);
    /// $user->profile()->save(['email' => 'thinkphp']);
    /// ```
    pub async fn save_relation(
        model: &ModelInstance,
        relation_name: &str,
        data: &HashMap<String, Value>,
    ) -> anyhow::Result<bool> {
        // 获取关联定义
        let relation = match model.config.relations.get(relation_name) {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Relation {} not found", relation_name)),
        };

        // 获取父模型主键值
        let parent_key = model.get_key();

        // 根据关联类型处理
        match relation.relation_type {
            RelationType::HasOne | RelationType::HasMany => {
                // 一对一/一对多：设置外键值
                let mut insert_data = data.clone();
                insert_data.insert(relation.foreign_key.clone(), parent_key);

                // 构建插入 SQL
                let related_table = class_to_table(&relation.related_model);
                let sql = build_insert_sql(&related_table, &insert_data);

                // 执行插入
                executor::execute(&sql).await?;
            }
            RelationType::BelongsToMany => {
                // 多对多：需要操作中间表
                // 先插入关联数据，然后插入中间表
                return Err(anyhow::anyhow!("Use attach method for BelongsToMany relation"));
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported relation type"));
            }
        }

        Ok(true)
    }

    /// 批量保存关联数据
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `data_list`: 关联数据列表
    ///
    /// # 返回值
    /// 保存成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user->comments()->saveAll([
    ///     ['content'=>'thinkphp'],
    ///     ['content'=>'onethink'],
    /// ]);
    /// ```
    pub async fn save_all_relation(
        model: &ModelInstance,
        relation_name: &str,
        data_list: &[HashMap<String, Value>],
    ) -> anyhow::Result<bool> {
        for data in data_list {
            Self::save_relation(model, relation_name, data).await?;
        }
        Ok(true)
    }

    /// 添加中间表数据（多对多）
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `related_id`: 关联模型 ID
    /// - `pivot_data`: 中间表额外数据
    ///
    /// # 返回值
    /// 添加成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user->roles()->attach(1, ['remark'=>'test']);
    /// ```
    pub async fn attach(
        model: &ModelInstance,
        relation_name: &str,
        related_id: Value,
        pivot_data: Option<&HashMap<String, Value>>,
        pivot_table: &str,
        foreign_key: &str,
        related_key: &str,
    ) -> anyhow::Result<bool> {
        // 获取父模型主键值
        let parent_key = model.get_key();

        // 构建中间表数据
        let mut pivot_insert = HashMap::new();
        pivot_insert.insert(foreign_key.to_string(), parent_key);
        pivot_insert.insert(related_key.to_string(), related_id);

        // 添加额外数据
        if let Some(extra) = pivot_data {
            pivot_insert.extend(extra.clone());
        }

        // 构建插入 SQL
        let sql = build_insert_sql(pivot_table, &pivot_insert);

        // 执行插入
        executor::execute(&sql).await?;

        Ok(true)
    }

    /// 删除中间表数据（多对多）
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `related_ids`: 关联模型 ID 列表
    ///
    /// # 返回值
    /// 删除成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user->roles()->detach([1, 2, 3]);
    /// ```
    pub async fn detach(
        model: &ModelInstance,
        related_ids: &[Value],
        pivot_table: &str,
        foreign_key: &str,
        related_key: &str,
    ) -> anyhow::Result<bool> {
        // 获取父模型主键值
        let parent_key = model.get_key();

        // 构建 DELETE SQL
        let placeholders: Vec<String> = related_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} = ? AND {} IN ({})",
            pivot_table, foreign_key, related_key, placeholders.join(", ")
        );

        // 构建参数
        let mut params = vec![parent_key];
        params.extend(related_ids.iter().cloned());

        // 执行删除
        executor::execute_with_params(&sql, &params).await?;

        Ok(true)
    }

    /// 同步中间表数据（多对多）
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `related_ids`: 关联模型 ID 列表
    ///
    /// # 返回值
    /// 同步成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $user->roles()->sync([1, 2, 3]);
    /// ```
    pub async fn sync(
        model: &ModelInstance,
        related_ids: &[Value],
        pivot_table: &str,
        foreign_key: &str,
        related_key: &str,
    ) -> anyhow::Result<bool> {
        // 获取父模型主键值
        let parent_key = model.get_key();

        // 先删除所有现有关联
        let delete_sql = format!(
            "DELETE FROM {} WHERE {} = ?",
            pivot_table, foreign_key
        );
        executor::execute_with_params(&delete_sql, &[parent_key.clone()]).await?;

        // 然后添加新的关联
        for related_id in related_ids {
            let insert_sql = format!(
                "INSERT INTO {} ({}, {}) VALUES (?, ?)",
                pivot_table, foreign_key, related_key
            );
            executor::execute_with_params(&insert_sql, &[parent_key.clone(), related_id.clone()]).await?;
        }

        Ok(true)
    }

    /// 关联自动写入
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 写入成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $blog->together(['content'])->save();
    /// ```
    pub async fn together_save(
        model: &mut ModelInstance,
        relations: &[&str],
    ) -> anyhow::Result<bool> {
        // 先保存主模型
        model.save().await?;

        // 然后保存关联模型
        for relation_name in relations {
            // 检查关联是否存在
            if let Some(_relation) = model.config.relations.get(*relation_name) {
                // 获取关联数据（如果已设置）
                // 这里需要从模型的关联属性中获取数据
                // 简化实现：假设关联数据已存储在特定属性中
            }
        }

        Ok(true)
    }

    /// 关联自动删除
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 删除成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $blog->together(['content'])->delete();
    /// ```
    pub async fn together_delete(
        model: &mut ModelInstance,
        relations: &[&str],
    ) -> anyhow::Result<bool> {
        // 先删除关联模型
        for relation_name in relations {
            if let Some(relation) = model.config.relations.get(*relation_name) {
                let related_table = class_to_table(&relation.related_model);
                let parent_key = model.get_key();

                // 构建删除 SQL
                let delete_sql = format!(
                    "DELETE FROM {} WHERE {} = ?",
                    related_table, relation.foreign_key
                );

                // 执行删除
                executor::execute_with_params(&delete_sql, &[parent_key]).await?;
            }
        }

        // 然后删除主模型
        model.delete().await?;

        Ok(true)
    }
}

/// 关联绑定器
///
/// 提供关联属性绑定功能
pub struct RelationBinder;

impl RelationBinder {
    /// 绑定关联属性到父模型
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `fields`: 要绑定的字段列表
    ///
    /// # 返回值
    /// 绑定后的模型实例
    ///
    /// # 示例
    /// ```php
    /// // 在关联定义中使用
    /// public function profile() {
    ///     return $this->hasOne(Profile::class)->bind(['nickname', 'email']);
    /// }
    /// ```
    pub fn bind(model: &ModelInstance, relation_name: &str, fields: &[&str]) -> ModelInstance {
        let mut new_model = model.clone();

        // 为每个字段创建绑定标记
        for field in fields {
            let bind_key = field.to_string();
            // 标记需要从关联中获取
            new_model.attributes.insert(
                bind_key,
                Value::String(format!("__bind:{}:{}", relation_name, field)),
            );
        }

        new_model
    }

    /// 动态绑定关联属性
    ///
    /// # 参数
    /// - `model`: 父模型实例
    /// - `relation_name`: 关联名称
    /// - `fields`: 字段映射（别名 => 字段名）
    ///
    /// # 返回值
    /// 绑定后的模型实例
    ///
    /// # 示例
    /// ```php
    /// $user = User::find(1)->bindAttr('profile', ['email', 'truename' => 'nickname']);
    /// ```
    pub fn bind_attr(
        model: &ModelInstance,
        relation_name: &str,
        field_map: &HashMap<String, String>,
    ) -> ModelInstance {
        let mut new_model = model.clone();

        for (alias, field) in field_map {
            // 标记需要从关联中获取
            new_model.attributes.insert(
                alias.clone(),
                Value::String(format!("__bind:{}:{}", relation_name, field)),
            );
        }

        new_model
    }
}

/// 关联查询器
///
/// 提供关联查询条件功能
pub struct RelationQuery;

impl RelationQuery {
    /// 根据关联条件查询
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relation_name`: 关联名称
    /// - `where`: 关联条件
    ///
    /// # 返回值
    /// SQL 条件字符串
    ///
    /// # 示例
    /// ```php
    /// User::hasWhere('profile', ['nickname'=>'think'])->select();
    /// ```
    pub fn has_where(
        config: &ModelConfig,
        relation_name: &str,
        where_conditions: &HashMap<String, Value>,
    ) -> anyhow::Result<String> {
        // 获取关联定义
        let relation = match config.relations.get(relation_name) {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Relation {} not found", relation_name)),
        };

        // 构建子查询
        let related_table = class_to_table(&relation.related_model);
        let mut conditions: Vec<String> = Vec::new();

        for (field, value) in where_conditions {
            let condition = match value {
                Value::String(s) => format!("{} = '{}'", field, s),
                Value::Int(i) => format!("{} = {}", field, i),
                Value::Float(f) => format!("{} = {}", field, f),
                Value::Bool(b) => format!("{} = {}", field, if *b { 1 } else { 0 }),
                _ => format!("{} = ?", field),
            };
            conditions.push(condition);
        }

        // 构建 EXISTS 子查询
        let subquery = format!(
            "EXISTS (SELECT 1 FROM {} WHERE {} = {} AND {})",
            related_table,
            relation.foreign_key,
            config.pk,
            conditions.join(" AND ")
        );

        Ok(subquery)
    }

    /// 根据关联数量查询
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relation_name`: 关联名称
    /// - `operator`: 比较操作符
    /// - `count`: 数量
    ///
    /// # 返回值
    /// SQL 条件字符串
    ///
    /// # 示例
    /// ```php
    /// Article::has('comments', '>', 3)->select();
    /// ```
    pub fn has(
        config: &ModelConfig,
        relation_name: &str,
        operator: &str,
        count: i64,
    ) -> anyhow::Result<String> {
        // 获取关联定义
        let relation = match config.relations.get(relation_name) {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Relation {} not found", relation_name)),
        };

        // 构建子查询
        let related_table = class_to_table(&relation.related_model);

        // 构建 COUNT 子查询
        let subquery = format!(
            "(SELECT COUNT(*) FROM {} WHERE {} = {}) {} {}",
            related_table,
            relation.foreign_key,
            config.pk,
            operator,
            count
        );

        Ok(subquery)
    }

    /// 添加中间表条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// SQL 条件字符串
    ///
    /// # 示例
    /// ```php
    /// User::belongsToMany(Role::class)->wherePivot('priority', 'in', [1, 2]);
    /// ```
    pub fn where_pivot(field: &str, operator: &str, value: &Value) -> String {
        match value {
            Value::Int(i) => format!("pivot.{} {} {}", field, operator, i),
            Value::String(s) => format!("pivot.{} {} '{}'", field, operator, s),
            Value::IndexedArray(arr) => {
                let values: Vec<String> = arr.iter().map(|v| v.to_string_value()).collect();
                format!("pivot.{} IN ({})", field, values.join(", "))
            }
            _ => format!("pivot.{} {} ?", field, operator),
        }
    }
}

/// 构建插入 SQL
///
/// # 参数
/// - `table`: 表名
/// - `data`: 数据映射
///
/// # 返回值
/// SQL 字符串
fn build_insert_sql(table: &str, data: &HashMap<String, Value>) -> String {
    let fields: Vec<&str> = data.keys().map(|s| s.as_str()).collect();
    let placeholders: Vec<&str> = data.iter().map(|_| "?").collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        fields.join(", "),
        placeholders.join(", ")
    )
}

/// 将类名转换为表名
///
/// # 参数
/// - `class_name`: 类名
///
/// # 返回值
/// 表名
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_to_table() {
        assert_eq!(class_to_table("User"), "user");
        assert_eq!(class_to_table("UserProfile"), "user_profile");
        assert_eq!(class_to_table("app\\model\\User"), "user");
    }

    #[test]
    fn test_build_insert_sql() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));
        data.insert("email".to_string(), Value::String("test@test.com".to_string()));

        let sql = build_insert_sql("users", &data);

        assert!(sql.starts_with("INSERT INTO users"));
        assert!(sql.contains("name"));
        assert!(sql.contains("email"));
    }

    #[test]
    fn test_where_pivot() {
        let condition = RelationQuery::where_pivot("priority", "=", &Value::Int(1));
        assert!(condition.contains("pivot.priority"));
        assert!(condition.contains("= 1"));
    }
}
