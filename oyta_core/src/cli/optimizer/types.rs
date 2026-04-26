//! 优化器类型定义模块
//!
//! 包含路由、数据库结构等相关的数据结构定义

/// 路由信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// HTTP 方法
    pub method: String,
    /// 路由路径
    pub path: String,
    /// 处理器
    pub handler: String,
}

/// 表结构信息
#[derive(Debug, Clone)]
pub struct TableSchema {
    /// 表名
    pub name: String,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 主键
    pub primary_key: Option<String>,
    /// 索引
    pub indexes: Vec<IndexInfo>,
}

/// 列信息
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可空
    pub nullable: bool,
    /// 默认值
    pub default: Option<String>,
    /// 注释
    pub comment: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// 索引名
    pub name: String,
    /// 列名列表
    pub columns: Vec<String>,
    /// 是否唯一
    pub unique: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试路由信息
    #[test]
    fn test_route_info() {
        let route = RouteInfo {
            method: "GET".to_string(),
            path: "/users".to_string(),
            handler: "UserController@index".to_string(),
        };

        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/users");
    }

    /// 测试列信息
    #[test]
    fn test_column_info() {
        let col = ColumnInfo {
            name: "id".to_string(),
            data_type: "bigint".to_string(),
            nullable: false,
            default: None,
            comment: Some("主键ID".to_string()),
        };

        assert_eq!(col.name, "id");
        assert!(!col.nullable);
    }

    /// 测试索引信息
    #[test]
    fn test_index_info() {
        let idx = IndexInfo {
            name: "users_email_unique".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
        };

        assert!(idx.unique);
        assert_eq!(idx.columns.len(), 1);
    }
}
