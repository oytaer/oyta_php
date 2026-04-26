//! 数据库关联关系扩展模块
//!
//! 实现高级关联关系：
//! - 远程一对一（hasOneThrough）
//! - 远程一对多（hasManyThrough）
//! - 多态一对一（morphOne）
//! - 多态一对多（morphMany）
//! - 多态多对多（morphToMany）
//!
//! 这些关联关系用于处理复杂的数据库关系

use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::model::{ModelConfig, ModelInstance, RelationDef};
use super::executor;

/// 远程关联配置
///
/// 用于定义通过中间表关联的远程关系
#[derive(Debug, Clone)]
pub struct ThroughRelation {
    /// 关联类型
    pub relation_type: ThroughRelationType,
    /// 目标模型类名
    pub target_model: String,
    /// 中间模型类名
    pub through_model: String,
    /// 本地键（当前模型）
    pub local_key: String,
    /// 第一外键（本地模型 → 中间表）
    pub first_foreign_key: String,
    /// 第二外键（中间表 → 目标表）
    pub second_foreign_key: String,
    /// 目标键（目标模型）
    pub target_key: String,
}

/// 远程关联类型
#[derive(Debug, Clone, Copy)]
pub enum ThroughRelationType {
    /// 远程一对一
    HasOneThrough,
    /// 远程一对多
    HasManyThrough,
}

impl ModelInstance {
    /// 远程一对一关联
    ///
    /// 通过中间表关联到一个远程模型
    ///
    /// # 示例
    /// User → Profile → History
    /// 用户通过档案关联到历史记录
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    /// - `through_config`: 中间表配置
    ///
    /// # SQL 示例
    /// ```sql
    /// SELECT history.*
    /// FROM history
    /// INNER JOIN profiles ON history.profile_id = profiles.id
    /// WHERE profiles.user_id = ?
    /// ```
    pub async fn load_has_one_through(
        &self,
        _relation_name: &str,
        target_table: &str,
        through_table: &str,
        local_key: &str,
        first_foreign_key: &str,
        second_foreign_key: &str,
        target_key: &str,
    ) -> anyhow::Result<Option<ModelInstance>> {
        // 获取本地键值
        let local_value = self.get_attr(local_key);

        // 构建远程查询 SQL
        let sql = format!(
            "SELECT t.* FROM {} t \
             INNER JOIN {} m ON t.{} = m.{} \
             WHERE m.{} = ? LIMIT 1",
            target_table, through_table,
            target_key, second_foreign_key,
            first_foreign_key
        );

        tracing::debug!("远程一对一查询: {}", sql);

        let result = executor::query_with_params(&sql, &[local_value]).await?;

        if let Some(row) = result.first() {
            // 创建目标模型配置
            let target_config = ModelConfig {
                table: target_table.to_string(),
                ..ModelConfig::default()
            };

            Ok(Some(ModelInstance::from_row(target_config, row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 远程一对多关联
    ///
    /// 通过中间表关联到多个远程模型
    ///
    /// # 示例
    /// User → Article → Comment
    /// 用户通过文章关联到评论
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    /// - `through_config`: 中间表配置
    pub async fn load_has_many_through(
        &self,
        _relation_name: &str,
        target_table: &str,
        through_table: &str,
        local_key: &str,
        first_foreign_key: &str,
        second_foreign_key: &str,
        target_key: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        // 获取本地键值
        let local_value = self.get_attr(local_key);

        // 构建远程查询 SQL
        let sql = format!(
            "SELECT t.* FROM {} t \
             INNER JOIN {} m ON t.{} = m.{} \
             WHERE m.{} = ?",
            target_table, through_table,
            target_key, second_foreign_key,
            first_foreign_key
        );

        tracing::debug!("远程一对多查询: {}", sql);

        let result = executor::query_with_params(&sql, &[local_value]).await?;

        // 创建目标模型配置
        let target_config = ModelConfig {
            table: target_table.to_string(),
            ..ModelConfig::default()
        };

        Ok(result.rows.into_iter()
            .map(|row| ModelInstance::from_row(target_config.clone(), row))
            .collect())
    }
}

/// 多态关联配置
///
/// 用于定义多态关联关系
/// 多态关联允许一个模型关联到多种不同类型的模型
#[derive(Debug, Clone)]
pub struct MorphRelation {
    /// 关联类型
    pub morph_type: MorphRelationType,
    /// 多态类型字段名
    pub morph_type_field: String,
    /// 多态 ID 字段名
    pub morph_id_field: String,
    /// 关联的目标模型类型列表
    pub morph_targets: Vec<MorphTarget>,
}

/// 多态关联类型
#[derive(Debug, Clone, Copy)]
pub enum MorphRelationType {
    /// 多态一对一
    MorphOne,
    /// 多态一对多
    MorphMany,
    /// 多态多对多
    MorphToMany,
}

/// 多态目标配置
#[derive(Debug, Clone)]
pub struct MorphTarget {
    /// 类型标识
    pub type_name: String,
    /// 对应的模型类名
    pub model_class: String,
    /// 对应的表名
    pub table_name: String,
}

impl MorphRelation {
    /// 创建新的多态关联
    pub fn new(morph_type: MorphRelationType, type_field: &str, id_field: &str) -> Self {
        Self {
            morph_type,
            morph_type_field: type_field.to_string(),
            morph_id_field: id_field.to_string(),
            morph_targets: Vec::new(),
        }
    }

    /// 添加多态目标
    pub fn add_target(mut self, type_name: &str, model_class: &str, table_name: &str) -> Self {
        self.morph_targets.push(MorphTarget {
            type_name: type_name.to_string(),
            model_class: model_class.to_string(),
            table_name: table_name.to_string(),
        });
        self
    }
}

impl ModelInstance {
    /// 多态一对一关联
    ///
    /// 一个模型可以关联到多种不同类型的模型中的一个
    ///
    /// # 示例
    /// Comment 可以关联到 Article 或 Video
    /// commentable_type = 'Article' AND commentable_id = article.id
    /// 或
    /// commentable_type = 'Video' AND commentable_id = video.id
    ///
    /// # 参数
    /// - `table`: 多态关联表名
    /// - `type_field`: 类型字段名
    /// - `id_field`: ID 字段名
    /// - `target_type`: 目标类型标识
    pub async fn load_morph_one(
        &self,
        table: &str,
        type_field: &str,
        id_field: &str,
        target_type: &str,
        local_key: &str,
    ) -> anyhow::Result<Option<ModelInstance>> {
        // 获取本地键值
        let local_value = self.get_attr(local_key);

        // 构建多态查询 SQL
        let sql = format!(
            "SELECT * FROM {} WHERE {} = ? AND {} = ? LIMIT 1",
            table, type_field, id_field
        );

        tracing::debug!("多态一对一查询: {}", sql);

        let result = executor::query_with_params(
            &sql,
            &[Value::String(target_type.to_string()), local_value],
        ).await?;

        if let Some(row) = result.first() {
            let config = ModelConfig {
                table: table.to_string(),
                ..ModelConfig::default()
            };

            Ok(Some(ModelInstance::from_row(config, row.clone())))
        } else {
            Ok(None)
        }
    }

    /// 多态一对多关联
    ///
    /// 一个模型可以关联到多种不同类型的模型中的多个
    ///
    /// # 示例
    /// Tag 可以关联到多个 Article 或 Video
    /// 一个标签可以同时标记文章和视频
    pub async fn load_morph_many(
        &self,
        table: &str,
        type_field: &str,
        id_field: &str,
        target_type: &str,
        local_key: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        // 获取本地键值
        let local_value = self.get_attr(local_key);

        // 构建多态查询 SQL
        let sql = format!(
            "SELECT * FROM {} WHERE {} = ? AND {} = ?",
            table, type_field, id_field
        );

        tracing::debug!("多态一对多查询: {}", sql);

        let result = executor::query_with_params(
            &sql,
            &[Value::String(target_type.to_string()), local_value],
        ).await?;

        let config = ModelConfig {
            table: table.to_string(),
            ..ModelConfig::default()
        };

        Ok(result.rows.into_iter()
            .map(|row| ModelInstance::from_row(config.clone(), row))
            .collect())
    }

    /// 多态多对多关联
    ///
    /// 多个模型可以关联到多种不同类型的模型
    ///
    /// # 示例
    /// Tag 可以通过中间表关联到 Article 或 Video
    /// 中间表: taggables (tag_id, taggable_type, taggable_id)
    ///
    /// # 参数
    /// - `pivot_table`: 中间表名
    /// - `target_table`: 目标表名
    /// - `pivot_foreign_key`: 中间表外键（指向当前模型）
    /// - `pivot_morph_type`: 中间表类型字段
    /// - `pivot_morph_id`: 中间表 ID 字段
    /// - `target_key`: 目标表主键
    /// - `target_type`: 目标类型标识
    pub async fn load_morph_to_many(
        &self,
        pivot_table: &str,
        target_table: &str,
        pivot_foreign_key: &str,
        pivot_morph_type: &str,
        pivot_morph_id: &str,
        target_key: &str,
        target_type: &str,
        local_key: &str,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        // 获取本地键值
        let local_value = self.get_attr(local_key);

        // 构建多态多对多查询 SQL
        let sql = format!(
            "SELECT t.* FROM {} t \
             INNER JOIN {} p ON t.{} = p.{} \
             WHERE p.{} = ? AND p.{} = ?",
            target_table, pivot_table,
            target_key, pivot_morph_id,
            pivot_foreign_key, pivot_morph_type
        );

        tracing::debug!("多态多对多查询: {}", sql);

        let result = executor::query_with_params(
            &sql,
            &[local_value, Value::String(target_type.to_string())],
        ).await?;

        let config = ModelConfig {
            table: target_table.to_string(),
            ..ModelConfig::default()
        };

        Ok(result.rows.into_iter()
            .map(|row| ModelInstance::from_row(config.clone(), row))
            .collect())
    }

    /// 保存多态关联
    ///
    /// # 参数
    /// - `table`: 多态关联表名
    /// - `type_field`: 类型字段名
    /// - `id_field`: ID 字段名
    /// - `target_type`: 目标类型
    /// - `target_id`: 目标 ID
    /// - `local_key`: 本地键
    pub async fn save_morph_relation(
        &self,
        table: &str,
        type_field: &str,
        id_field: &str,
        target_type: &str,
        target_id: Value,
        local_key: &str,
    ) -> anyhow::Result<bool> {
        let _local_value = self.get_attr(local_key);

        let sql = format!(
            "INSERT INTO {} ({}, {}) VALUES (?, ?)",
            table, type_field, id_field
        );

        executor::execute_with_params(
            &sql,
            &[Value::String(target_type.to_string()), target_id],
        ).await?;

        Ok(true)
    }

    /// 删除多态关联
    pub async fn delete_morph_relation(
        &self,
        table: &str,
        type_field: &str,
        id_field: &str,
        target_type: &str,
        local_key: &str,
    ) -> anyhow::Result<bool> {
        let local_value = self.get_attr(local_key);

        let sql = format!(
            "DELETE FROM {} WHERE {} = ? AND {} = ?",
            table, type_field, id_field
        );

        executor::execute_with_params(
            &sql,
            &[Value::String(target_type.to_string()), local_value],
        ).await?;

        Ok(true)
    }
}

/// 关联预加载器
///
/// 用于批量加载关联关系，避免 N+1 查询问题
pub struct RelationLoader {
    /// 预加载的关联列表
    relations: Vec<String>,
}

impl RelationLoader {
    /// 创建新的关联预加载器
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
        }
    }

    /// 添加要预加载的关联
    pub fn with(mut self, relation: &str) -> Self {
        self.relations.push(relation.to_string());
        self
    }

    /// 添加多个要预加载的关联
    pub fn with_many(mut self, relations: &[&str]) -> Self {
        for r in relations {
            self.relations.push(r.to_string());
        }
        self
    }

    /// 预加载关联
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    /// - `config`: 模型配置
    pub async fn load(
        &self,
        models: &[ModelInstance],
        config: &ModelConfig,
    ) -> anyhow::Result<HashMap<String, Vec<ModelInstance>>> {
        let mut loaded = HashMap::new();

        for relation_name in &self.relations {
            if let Some(relation) = config.relations.get(relation_name) {
                let related_models = self.load_relation_for_models(models, relation).await?;
                loaded.insert(relation_name.clone(), related_models);
            }
        }

        Ok(loaded)
    }

    /// 为多个模型加载关联
    async fn load_relation_for_models(
        &self,
        models: &[ModelInstance],
        relation: &RelationDef,
    ) -> anyhow::Result<Vec<ModelInstance>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }

        // 收集所有本地键值
        let local_keys: Vec<Value> = models
            .iter()
            .map(|m| m.get_attr(&relation.local_key))
            .collect();

        // 构建批量查询
        let related_table = class_to_table(&relation.related_model);
        let placeholders: Vec<String> = local_keys.iter().map(|_| "?".to_string()).collect();

        let sql = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            related_table,
            relation.foreign_key,
            placeholders.join(", ")
        );

        tracing::debug!("预加载关联查询: {}", sql);

        let result = executor::query_with_params(&sql, &local_keys).await?;

        let related_config = ModelConfig {
            table: related_table,
            ..ModelConfig::default()
        };

        Ok(result.rows.into_iter()
            .map(|row| ModelInstance::from_row(related_config.clone(), row))
            .collect())
    }
}

impl Default for RelationLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 将类名转换为表名
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
    fn test_morph_relation() {
        let relation = MorphRelation::new(
            MorphRelationType::MorphOne,
            "commentable_type",
            "commentable_id",
        )
        .add_target("Article", "app\\model\\Article", "articles")
        .add_target("Video", "app\\model\\Video", "videos");

        assert_eq!(relation.morph_targets.len(), 2);
        assert_eq!(relation.morph_targets[0].type_name, "Article");
    }

    #[test]
    fn test_relation_loader() {
        let loader = RelationLoader::new()
            .with("profile")
            .with("posts")
            .with_many(&["comments", "likes"]);

        assert_eq!(loader.relations.len(), 4);
    }
}
