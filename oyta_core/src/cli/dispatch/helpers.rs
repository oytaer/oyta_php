//! 辅助函数模块
//!
//! 包含命令处理中使用的各种辅助函数

/// 解析名称和命名空间
///
/// 将输入的名称解析为命名空间和类名
/// 例如：admin/Index → (app\controller\admin, Index)
///
/// # 参数
/// - `name`: 输入的名称
/// - `base_namespace`: 基础命名空间
///
/// # 返回值
/// 返回元组 (命名空间, 类名)
pub fn parse_name_with_namespace(name: &str, base_namespace: &str) -> (String, String) {
    // 检查是否包含路径分隔符
    if let Some(pos) = name.rfind('/') {
        // 分割命名空间部分和类名部分
        let namespace_part = &name[..pos];
        let class_name = &name[pos + 1..];

        // 将路径分隔符转换为命名空间分隔符
        let namespace = format!("{}\\{}", base_namespace, namespace_part.replace('/', "\\"));
        (namespace, class_name.to_string())
    } else {
        // 没有路径分隔符，直接使用基础命名空间
        (base_namespace.to_string(), name.to_string())
    }
}

/// 将驼峰命名转换为下划线命名
///
/// 例如：UserController → user_controller
///
/// # 参数
/// - `s`: 驼峰命名的字符串
///
/// # 返回值
/// 下划线命名的字符串
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();

    // 遍历每个字符
    for (i, c) in s.chars().enumerate() {
        // 如果是大写字母
        if c.is_uppercase() {
            // 如果不是第一个字符，添加下划线
            if i > 0 {
                result.push('_');
            }
            // 转换为小写
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// 解析包规范
///
/// 解析包名和版本约束
/// 例如：vendor/package:^1.0 → (vendor/package, ^1.0)
///
/// # 参数
/// - `spec`: 包规范字符串
///
/// # 返回值
/// 返回元组 (包名, 版本约束)
pub fn parse_package_spec(spec: &str) -> (String, String) {
    // 查找版本分隔符的位置
    if let Some(pos) = spec.find(':') {
        // 分割包名和版本
        let package = &spec[..pos];
        let version = &spec[pos + 1..];
        (package.to_string(), version.to_string())
    } else {
        // 没有版本约束，使用默认值
        (spec.to_string(), "*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试命名空间解析
    #[test]
    fn test_parse_name_with_namespace() {
        // 测试带路径的名称
        let (ns, class) = parse_name_with_namespace("admin/Index", "app\\controller");
        assert_eq!(ns, "app\\controller\\admin");
        assert_eq!(class, "Index");

        // 测试不带路径的名称
        let (ns, class) = parse_name_with_namespace("User", "app\\model");
        assert_eq!(ns, "app\\model");
        assert_eq!(class, "User");

        // 测试多级路径
        let (ns, class) = parse_name_with_namespace("admin/api/User", "app\\controller");
        assert_eq!(ns, "app\\controller\\admin\\api");
        assert_eq!(class, "User");
    }

    /// 测试驼峰转下划线
    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("UserController"), "user_controller");
        assert_eq!(camel_to_snake("User"), "user");
        assert_eq!(camel_to_snake("APIController"), "a_p_i_controller");
        assert_eq!(camel_to_snake("userController"), "user_controller");
    }

    /// 测试包规范解析
    #[test]
    fn test_parse_package_spec() {
        // 测试带版本的包
        let (pkg, ver) = parse_package_spec("vendor/package:^1.0");
        assert_eq!(pkg, "vendor/package");
        assert_eq!(ver, "^1.0");

        // 测试不带版本的包
        let (pkg, ver) = parse_package_spec("vendor/package");
        assert_eq!(pkg, "vendor/package");
        assert_eq!(ver, "*");
    }
}
