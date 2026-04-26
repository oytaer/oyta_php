//! 数据库结构缓存模块
//!
//! 提供数据库表结构的缓存生成功能

use std::collections::HashMap;

use crate::symbol_table::types::ConfigValue;

use super::types::{ColumnInfo, IndexInfo, TableSchema};

/// 数据库结构缓存生成器
pub struct SchemaCacheGenerator;

impl SchemaCacheGenerator {
    /// 生成数据库结构缓存内容
    ///
    /// # 参数
    /// - `schema`: 表结构信息
    /// - `config_store`: 配置存储引用
    ///
    /// # 返回
    /// PHP 缓存文件内容
    pub fn generate_cache_content(
        schema: &HashMap<String, TableSchema>,
        config_store: &crate::config::store::ConfigStore,
    ) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 数据库字段缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 记录数据库表结构信息\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        // 获取默认连接
        let default_conn = config_store
            .get("database.default")
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "mysql".to_string());

        content.push_str(&format!(
            "    'default_connection' => '{}',\n\n",
            default_conn
        ));
        content.push_str("    'tables' => [\n");

        // 遍历每个表
        for (table_name, table_schema) in schema {
            content.push_str(&format!("        '{}' => [\n", table_name));
            content.push_str("            'columns' => [\n");

            // 输出列信息
            for col in &table_schema.columns {
                content.push_str(&format!("                '{}' => [\n", col.name));
                content.push_str(&format!("                    'type' => '{}',\n", col.data_type));
                content.push_str(&format!("                    'nullable' => {},\n", col.nullable));
                if let Some(default) = &col.default {
                    content.push_str(&format!("                    'default' => '{}',\n", default));
                }
                if let Some(comment) = &col.comment {
                    content.push_str(&format!("                    'comment' => '{}',\n", comment));
                }
                content.push_str("                ],\n");
            }

            content.push_str("            ],\n");

            // 输出主键
            if let Some(pk) = &table_schema.primary_key {
                content.push_str(&format!("            'primary_key' => '{}',\n", pk));
            }

            // 输出索引
            content.push_str("            'indexes' => [\n");
            for idx in &table_schema.indexes {
                content.push_str(&format!("                '{}' => [\n", idx.name));
                content.push_str(&format!(
                    "                    'columns' => ['{}'],\n",
                    idx.columns.join("', '")
                ));
                content.push_str(&format!("                    'unique' => {},\n", idx.unique));
                content.push_str("                ],\n");
            }
            content.push_str("            ],\n");

            content.push_str("        ],\n");
        }

        content.push_str("    ],\n");
        content.push_str("];\n");

        content
    }

    /// 获取模拟的数据库表结构
    ///
    /// 在实际使用时会通过数据库连接获取真实结构
    ///
    /// # 返回
    /// 表结构信息映射
    pub fn get_mock_schema() -> HashMap<String, TableSchema> {
        let mut schema = HashMap::new();

        // 用户表
        schema.insert(
            "users".to_string(),
            TableSchema {
                name: "users".to_string(),
                columns: vec![
                    ColumnInfo {
                        name: "id".to_string(),
                        data_type: "bigint".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("主键ID".to_string()),
                    },
                    ColumnInfo {
                        name: "name".to_string(),
                        data_type: "varchar(255)".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("用户名".to_string()),
                    },
                    ColumnInfo {
                        name: "email".to_string(),
                        data_type: "varchar(255)".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("邮箱".to_string()),
                    },
                    ColumnInfo {
                        name: "password".to_string(),
                        data_type: "varchar(255)".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("密码".to_string()),
                    },
                    ColumnInfo {
                        name: "created_at".to_string(),
                        data_type: "timestamp".to_string(),
                        nullable: true,
                        default: Some("NULL".to_string()),
                        comment: Some("创建时间".to_string()),
                    },
                    ColumnInfo {
                        name: "updated_at".to_string(),
                        data_type: "timestamp".to_string(),
                        nullable: true,
                        default: Some("NULL".to_string()),
                        comment: Some("更新时间".to_string()),
                    },
                ],
                primary_key: Some("id".to_string()),
                indexes: vec![IndexInfo {
                    name: "users_email_unique".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                }],
            },
        );

        // 迁移表
        schema.insert(
            "migrations".to_string(),
            TableSchema {
                name: "migrations".to_string(),
                columns: vec![
                    ColumnInfo {
                        name: "id".to_string(),
                        data_type: "int".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("迁移ID".to_string()),
                    },
                    ColumnInfo {
                        name: "migration".to_string(),
                        data_type: "varchar(255)".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("迁移名称".to_string()),
                    },
                    ColumnInfo {
                        name: "batch".to_string(),
                        data_type: "int".to_string(),
                        nullable: false,
                        default: None,
                        comment: Some("批次号".to_string()),
                    },
                ],
                primary_key: Some("id".to_string()),
                indexes: vec![],
            },
        );

        schema
    }
}
