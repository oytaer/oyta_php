//! 迁移辅助函数模块
//!
//! 包含迁移类型检测、表名提取、类名转换等辅助函数

/// 迁移类型
#[derive(Debug, Clone)]
pub enum MigrationType {
    /// 创建表
    Create,
    /// 添加字段
    Add,
    /// 删除表
    Drop,
    /// 默认
    Default,
}

/// 检测迁移类型
///
/// 根据迁移名称判断迁移类型
///
/// # 参数
/// - `name`: 迁移名称
///
/// # 返回值
/// 迁移类型枚举
pub fn detect_migration_type(name: &str) -> MigrationType {
    let name_lower = name.to_lowercase();

    // 根据前缀判断类型
    if name_lower.starts_with("create_") {
        MigrationType::Create
    } else if name_lower.starts_with("add_") {
        MigrationType::Add
    } else if name_lower.starts_with("drop_") {
        MigrationType::Drop
    } else {
        MigrationType::Default
    }
}

/// 提取表名
///
/// 从迁移名称中提取数据表名称
///
/// # 参数
/// - `name`: 迁移名称
///
/// # 返回值
/// 数据表名称
pub fn extract_table_name(name: &str) -> String {
    let name_lower = name.to_lowercase();

    // 移除前缀
    let name = name_lower
        .strip_prefix("create_")
        .or_else(|| name_lower.strip_prefix("add_"))
        .or_else(|| name_lower.strip_prefix("drop_"))
        .unwrap_or(&name_lower);

    // 移除 _table 后缀
    name.strip_suffix("_table").unwrap_or(name).to_string()
}

/// 将迁移名称转换为类名
///
/// 将下划线命名转换为驼峰命名
///
/// # 参数
/// - `name`: 迁移名称
///
/// # 返回值
/// 类名
pub fn migration_name_to_class(name: &str) -> String {
    // 移除时间戳前缀（如 20240101120000_）
    let name = if let Some(pos) = name.find('_') {
        // 检查是否为14位数字时间戳
        if pos == 14 && name[..14].chars().all(|c| c.is_numeric()) {
            &name[pos + 1..]
        } else {
            name
        }
    } else {
        name
    };

    // 转换为驼峰命名
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试迁移类型检测
    #[test]
    fn test_detect_migration_type() {
        assert!(matches!(
            detect_migration_type("create_users_table"),
            MigrationType::Create
        ));
        assert!(matches!(
            detect_migration_type("add_email_to_users"),
            MigrationType::Add
        ));
        assert!(matches!(
            detect_migration_type("drop_users_table"),
            MigrationType::Drop
        ));
    }

    /// 测试表名提取
    #[test]
    fn test_extract_table_name() {
        assert_eq!(extract_table_name("create_users_table"), "users");
        assert_eq!(extract_table_name("add_email_to_users"), "email_to_users");
        assert_eq!(extract_table_name("drop_users_table"), "users");
    }

    /// 测试类名转换
    #[test]
    fn test_migration_name_to_class() {
        assert_eq!(
            migration_name_to_class("20240101120000_create_users_table"),
            "CreateUsersTable"
        );
        assert_eq!(migration_name_to_class("create_users_table"), "CreateUsersTable");
        assert_eq!(migration_name_to_class("add_email_to_users"), "AddEmailToUsers");
    }
}
