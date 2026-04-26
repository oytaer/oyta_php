//! 分页结果模块
//!
//! 包含分页查询结果的数据结构和转换方法

use crate::interpreter::value::Value;

use super::instance::ModelInstance;

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
    ///
    /// # 返回值
    /// 如果当前页不是最后一页返回 true
    pub fn has_more(&self) -> bool {
        self.page < self.total_pages
    }

    /// 转换为 Value（用于解释器返回）
    ///
    /// 将分页结果转换为关联数组形式的 Value
    ///
    /// # 返回值
    /// 包含分页信息的 Value::AssociativeArray
    pub fn to_value(&self) -> Value {
        let mut data = Vec::new();

        // 将模型列表转换为 Value 数组
        let items_values: Vec<Value> = self.items.iter().map(|m| m.to_value()).collect();

        // 添加分页信息
        data.push(("total".to_string(), Value::Int(self.total)));
        data.push(("page".to_string(), Value::Int(self.page)));
        data.push(("page_size".to_string(), Value::Int(self.page_size)));
        data.push(("total_pages".to_string(), Value::Int(self.total_pages)));
        data.push(("has_more".to_string(), Value::Bool(self.has_more())));
        data.push(("items".to_string(), Value::IndexedArray(items_values)));

        Value::AssociativeArray(data)
    }
}
