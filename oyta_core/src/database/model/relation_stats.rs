//! 关联统计模块
//!
//! 实现模型的关联统计功能
//! 对应 ThinkPHP 的关联统计功能
//!
//! 主要功能：
//! - withCount 关联计数
//! - withSum 关联求和
//! - withMax 关联最大值
//! - withMin 关联最小值
//! - withAvg 关联平均值

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::{ModelConfig, RelationType};

/// 关联统计类型
#[derive(Debug, Clone, Copy)]
pub enum AggregateType {
    /// 计数
    Count,
    /// 求和
    Sum,
    /// 最大值
    Max,
    /// 最小值
    Min,
    /// 平均值
    Avg,
}

impl AggregateType {
    /// 获取 SQL 函数名
    pub fn sql_function(&self) -> &'static str {
        match self {
            AggregateType::Count => "COUNT",
            AggregateType::Sum => "SUM",
            AggregateType::Max => "MAX",
            AggregateType::Min => "MIN",
            AggregateType::Avg => "AVG",
        }
    }
}

/// 关联统计定义
#[derive(Debug, Clone)]
pub struct AggregateDefinition {
    /// 关联名称
    pub relation: String,
    /// 统计类型
    pub aggregate_type: AggregateType,
    /// 统计字段（Count 时可为空）
    pub field: Option<String>,
    /// 结果别名
    pub alias: String,
    /// 额外条件
    pub conditions: HashMap<String, Value>,
}

impl AggregateDefinition {
    /// 创建计数定义
    pub fn count(relation: &str, alias: Option<&str>) -> Self {
        Self {
            relation: relation.to_string(),
            aggregate_type: AggregateType::Count,
            field: None,
            alias: alias.map(|s| s.to_string()).unwrap_or_else(|| format!("{}_count", relation)),
            conditions: HashMap::new(),
        }
    }

    /// 创建求和定义
    pub fn sum(relation: &str, field: &str, alias: Option<&str>) -> Self {
        Self {
            relation: relation.to_string(),
            aggregate_type: AggregateType::Sum,
            field: Some(field.to_string()),
            alias: alias.map(|s| s.to_string()).unwrap_or_else(|| format!("{}_sum", relation)),
            conditions: HashMap::new(),
        }
    }

    /// 创建最大值定义
    pub fn max(relation: &str, field: &str, alias: Option<&str>) -> Self {
        Self {
            relation: relation.to_string(),
            aggregate_type: AggregateType::Max,
            field: Some(field.to_string()),
            alias: alias.map(|s| s.to_string()).unwrap_or_else(|| format!("{}_max", relation)),
            conditions: HashMap::new(),
        }
    }

    /// 创建最小值定义
    pub fn min(relation: &str, field: &str, alias: Option<&str>) -> Self {
        Self {
            relation: relation.to_string(),
            aggregate_type: AggregateType::Min,
            field: Some(field.to_string()),
            alias: alias.map(|s| s.to_string()).unwrap_or_else(|| format!("{}_min", relation)),
            conditions: HashMap::new(),
        }
    }

    /// 创建平均值定义
    pub fn avg(relation: &str, field: &str, alias: Option<&str>) -> Self {
        Self {
            relation: relation.to_string(),
            aggregate_type: AggregateType::Avg,
            field: Some(field.to_string()),
            alias: alias.map(|s| s.to_string()).unwrap_or_else(|| format!("{}_avg", relation)),
            conditions: HashMap::new(),
        }
    }

    /// 添加条件
    pub fn with_condition(mut self, field: &str, value: Value) -> Self {
        self.conditions.insert(field.to_string(), value);
        self
    }
}

/// 关联统计查询构建器
pub struct AggregateQueryBuilder {
    /// 主表名
    pub main_table: String,
    /// 主键名
    pub primary_key: String,
    /// 统计定义列表
    pub aggregates: Vec<AggregateDefinition>,
}

impl AggregateQueryBuilder {
    /// 创建新的构建器
    pub fn new(table: &str, primary_key: &str) -> Self {
        Self {
            main_table: table.to_string(),
            primary_key: primary_key.to_string(),
            aggregates: Vec::new(),
        }
    }

    /// 添加计数统计
    pub fn with_count(&mut self, relation: &str, alias: Option<&str>) -> &mut Self {
        self.aggregates.push(AggregateDefinition::count(relation, alias));
        self
    }

    /// 添加求和统计
    pub fn with_sum(&mut self, relation: &str, field: &str, alias: Option<&str>) -> &mut Self {
        self.aggregates.push(AggregateDefinition::sum(relation, field, alias));
        self
    }

    /// 添加最大值统计
    pub fn with_max(&mut self, relation: &str, field: &str, alias: Option<&str>) -> &mut Self {
        self.aggregates.push(AggregateDefinition::max(relation, field, alias));
        self
    }

    /// 添加最小值统计
    pub fn with_min(&mut self, relation: &str, field: &str, alias: Option<&str>) -> &mut Self {
        self.aggregates.push(AggregateDefinition::min(relation, field, alias));
        self
    }

    /// 添加平均值统计
    pub fn with_avg(&mut self, relation: &str, field: &str, alias: Option<&str>) -> &mut Self {
        self.aggregates.push(AggregateDefinition::avg(relation, field, alias));
        self
    }

    /// 构建 SELECT SQL（带子查询统计）
    pub fn build_select_with_aggregates(&self, relations: &HashMap<String, super::types::RelationDef>) -> String {
        let mut select_parts: Vec<String> = vec![format!("{}.*", self.main_table)];

        // 为每个统计添加子查询
        for agg in &self.aggregates {
            if let Some(relation) = relations.get(&agg.relation) {
                let subquery = self.build_aggregate_subquery(agg, relation);
                select_parts.push(format!("({}) as {}", subquery, agg.alias));
            }
        }

        format!(
            "SELECT {} FROM {}",
            select_parts.join(", "),
            self.main_table
        )
    }

    /// 构建统计子查询
    fn build_aggregate_subquery(
        &self,
        agg: &AggregateDefinition,
        relation: &super::types::RelationDef,
    ) -> String {
        let related_table = class_to_table(&relation.related_model);
        let func = agg.aggregate_type.sql_function();

        let field = match &agg.field {
            Some(f) => f.clone(),
            None => "*".to_string(),
        };

        // 构建条件
        let mut conditions = vec![format!(
            "{}.{} = {}.{}",
            related_table, relation.foreign_key,
            self.main_table, relation.local_key
        )];

        // 添加额外条件
        for (cond_field, cond_value) in &agg.conditions {
            let cond = match cond_value {
                Value::Int(i) => format!("{} = {}", cond_field, i),
                Value::String(s) => format!("{} = '{}'", cond_field, s),
                _ => format!("{} = ?", cond_field),
            };
            conditions.push(cond);
        }

        format!(
            "SELECT {}({}) FROM {} WHERE {}",
            func, field, related_table,
            conditions.join(" AND ")
        )
    }
}

/// 关联统计执行器
pub struct AggregateExecutor;

impl AggregateExecutor {
    /// 执行关联计数
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relation_name`: 关联名称
    /// - `model_id`: 模型 ID
    ///
    /// # 返回值
    /// 计数结果
    pub async fn count(
        config: &ModelConfig,
        relation_name: &str,
        model_id: Value,
    ) -> anyhow::Result<i64> {
        let relation = config.relations.get(relation_name)
            .ok_or_else(|| anyhow::anyhow!("Relation {} not found", relation_name))?;

        let related_table = class_to_table(&relation.related_model);

        let sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE {} = ?",
            related_table, relation.foreign_key
        );

        let result = executor::query_with_params(&sql, &[model_id]).await?;

        Ok(result
            .scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            })
            .unwrap_or(0))
    }

    /// 执行关联求和
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `relation_name`: 关联名称
    /// - `field`: 统计字段
    /// - `model_id`: 模型 ID
    ///
    /// # 返回值
    /// 求和结果
    pub async fn sum(
        config: &ModelConfig,
        relation_name: &str,
        field: &str,
        model_id: Value,
    ) -> anyhow::Result<f64> {
        let relation = config.relations.get(relation_name)
            .ok_or_else(|| anyhow::anyhow!("Relation {} not found", relation_name))?;

        let related_table = class_to_table(&relation.related_model);

        let sql = format!(
            "SELECT SUM({}) as sum FROM {} WHERE {} = ?",
            field, related_table, relation.foreign_key
        );

        let result = executor::query_with_params(&sql, &[model_id]).await?;

        // 解析结果并转换为 f64
        Ok(result
            .scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64), // 解引用后转换为 f64
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }

    /// 执行关联最大值
    pub async fn max(
        config: &ModelConfig,
        relation_name: &str,
        field: &str,
        model_id: Value,
    ) -> anyhow::Result<Option<Value>> {
        let relation = config.relations.get(relation_name)
            .ok_or_else(|| anyhow::anyhow!("Relation {} not found", relation_name))?;

        let related_table = class_to_table(&relation.related_model);

        let sql = format!(
            "SELECT MAX({}) as max FROM {} WHERE {} = ?",
            field, related_table, relation.foreign_key
        );

        let result = executor::query_with_params(&sql, &[model_id]).await?;

        Ok(result.scalar().cloned())
    }

    /// 执行关联最小值
    pub async fn min(
        config: &ModelConfig,
        relation_name: &str,
        field: &str,
        model_id: Value,
    ) -> anyhow::Result<Option<Value>> {
        let relation = config.relations.get(relation_name)
            .ok_or_else(|| anyhow::anyhow!("Relation {} not found", relation_name))?;

        let related_table = class_to_table(&relation.related_model);

        let sql = format!(
            "SELECT MIN({}) as min FROM {} WHERE {} = ?",
            field, related_table, relation.foreign_key
        );

        let result = executor::query_with_params(&sql, &[model_id]).await?;

        Ok(result.scalar().cloned())
    }

    /// 执行关联平均值
    pub async fn avg(
        config: &ModelConfig,
        relation_name: &str,
        field: &str,
        model_id: Value,
    ) -> anyhow::Result<f64> {
        let relation = config.relations.get(relation_name)
            .ok_or_else(|| anyhow::anyhow!("Relation {} not found", relation_name))?;

        let related_table = class_to_table(&relation.related_model);

        let sql = format!(
            "SELECT AVG({}) as avg FROM {} WHERE {} = ?",
            field, related_table, relation.foreign_key
        );

        let result = executor::query_with_params(&sql, &[model_id]).await?;

        // 解析结果并转换为 f64
        Ok(result
            .scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64), // 解引用后转换为 f64
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }
}

/// 为 ModelInstance 添加关联统计方法
impl ModelInstance {
    /// 获取关联计数
    pub async fn get_relation_count(&self, relation_name: &str) -> anyhow::Result<i64> {
        AggregateExecutor::count(&self.config, relation_name, self.get_key()).await
    }

    /// 获取关联求和
    pub async fn get_relation_sum(&self, relation_name: &str, field: &str) -> anyhow::Result<f64> {
        AggregateExecutor::sum(&self.config, relation_name, field, self.get_key()).await
    }

    /// 获取关联最大值
    pub async fn get_relation_max(&self, relation_name: &str, field: &str) -> anyhow::Result<Option<Value>> {
        AggregateExecutor::max(&self.config, relation_name, field, self.get_key()).await
    }

    /// 获取关联最小值
    pub async fn get_relation_min(&self, relation_name: &str, field: &str) -> anyhow::Result<Option<Value>> {
        AggregateExecutor::min(&self.config, relation_name, field, self.get_key()).await
    }

    /// 获取关联平均值
    pub async fn get_relation_avg(&self, relation_name: &str, field: &str) -> anyhow::Result<f64> {
        AggregateExecutor::avg(&self.config, relation_name, field, self.get_key()).await
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
    fn test_aggregate_definition() {
        let count_def = AggregateDefinition::count("comments", Some("comment_count"));
        assert_eq!(count_def.relation, "comments");
        assert_eq!(count_def.alias, "comment_count");
        assert!(matches!(count_def.aggregate_type, AggregateType::Count));

        let sum_def = AggregateDefinition::sum("orders", "total", None);
        assert_eq!(sum_def.field, Some("total".to_string()));
        assert_eq!(sum_def.alias, "orders_sum");
    }

    #[test]
    fn test_aggregate_query_builder() {
        let mut builder = AggregateQueryBuilder::new("users", "id");

        builder
            .with_count("posts", None)
            .with_sum("orders", "total", Some("total_amount"));

        assert_eq!(builder.aggregates.len(), 2);
    }

    #[test]
    fn test_aggregate_type() {
        assert_eq!(AggregateType::Count.sql_function(), "COUNT");
        assert_eq!(AggregateType::Sum.sql_function(), "SUM");
        assert_eq!(AggregateType::Max.sql_function(), "MAX");
        assert_eq!(AggregateType::Min.sql_function(), "MIN");
        assert_eq!(AggregateType::Avg.sql_function(), "AVG");
    }
}
