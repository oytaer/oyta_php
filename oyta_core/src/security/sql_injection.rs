//! SQL 注入防护模块
//!
//! 提供 SQL 注入防护功能
//! 包括参数化查询绑定、标识符引用、输入过滤
//! 兼容 ThinkPHP 8.0 的数据库安全接口

/// 检测 SQL 注入特征
///
/// 检查输入字符串是否包含常见的 SQL 注入模式
///
/// # 参数
/// - `input`: 待检测的字符串
///
/// # 返回
/// 是否检测到可疑的 SQL 注入特征
pub fn detect_injection(input: &str) -> bool {
    let lower = input.to_lowercase();
    let dangerous_patterns = [
        "' or ",
        "' and ",
        "\" or ",
        "\" and ",
        "1=1",
        "1=2",
        "' --",
        "\" --",
        "' #",
        "\" #",
        "union select",
        "union all select",
        "drop table",
        "drop database",
        "delete from",
        "insert into",
        "update ",
        "exec(",
        "execute(",
        "xp_",
        "sp_",
        "0x",
        "char(",
        "concat(",
        "group_concat(",
        "information_schema",
        "sleep(",
        "benchmark(",
        "load_file(",
        "into outfile",
        "into dumpfile",
    ];
    for pattern in &dangerous_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }
    false
}

/// 转义 SQL 字符串值
///
/// 对字符串值中的特殊字符进行转义
/// 注意：推荐使用参数化查询而非手动转义
///
/// # 参数
/// - `input`: 待转义的字符串
///
/// # 返回
/// 转义后的字符串
pub fn escape_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '\'' => result.push_str("''"),
            '\\' => result.push_str("\\\\"),
            '\0' => result.push_str("\\0"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\x1a' => result.push_str("\\Z"),
            _ => result.push(ch),
        }
    }
    result
}

/// 引用 SQL 标识符
///
/// 用反引号包裹标识符，防止保留字冲突
///
/// # 参数
/// - `identifier`: SQL 标识符（表名、字段名等）
///
/// # 返回
/// 反引号引用的标识符
pub fn quote_identifier(identifier: &str) -> String {
    let escaped = identifier.replace('`', "``");
    format!("`{}`", escaped)
}

/// 构建参数化查询的占位符
///
/// 生成指定数量的参数占位符
///
/// # 参数
/// - `count`: 参数数量
///
/// # 返回
/// 占位符字符串，如 "?, ?, ?"
pub fn placeholders(count: usize) -> String {
    (0..count)
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ")
}

/// 清理表名
///
/// 移除表名中的非法字符
/// 只允许字母、数字和下划线
pub fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// 清理字段名
///
/// 移除字段名中的非法字符
/// 只允许字母、数字、下划线和点号（表.字段 格式）
pub fn sanitize_column_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
        .collect()
}

/// 构建安全的 IN 子句
///
/// 使用参数化占位符构建 IN 子句
///
/// # 参数
/// - `count`: IN 子句中的参数数量
///
/// # 返回
/// 安全的 IN 子句，如 "(?, ?, ?)"
pub fn safe_in_clause(count: usize) -> String {
    if count == 0 {
        return "(NULL)".to_string();
    }
    format!("({})", placeholders(count))
}

/// 构建安全的 LIMIT 子句
///
/// 确保 LIMIT 值为非负整数
///
/// # 参数
/// - `limit`: 限制数量
/// - `offset`: 偏移量
///
/// # 返回
/// 安全的 LIMIT 子句
pub fn safe_limit(limit: i64, offset: Option<i64>) -> String {
    let safe_limit = limit.max(0);
    match offset {
        Some(off) => {
            let safe_offset = off.max(0);
            format!("LIMIT {}, {}", safe_offset, safe_limit)
        }
        None => format!("LIMIT {}", safe_limit),
    }
}

/// 构建安全的 ORDER BY 子句
///
/// 验证排序方向和字段名
///
/// # 参数
/// - `column`: 字段名
/// - `direction`: 排序方向（ASC 或 DESC）
///
/// # 返回
/// 安全的 ORDER BY 子句
pub fn safe_order_by(column: &str, direction: &str) -> String {
    let safe_column = sanitize_column_name(column);
    let safe_direction = match direction.to_uppercase().as_str() {
        "ASC" => "ASC",
        "DESC" => "DESC",
        _ => "ASC",
    };
    format!("ORDER BY {} {}", quote_identifier(&safe_column), safe_direction)
}

/// 过滤 LIKE 查询中的通配符
///
/// 转义 LIKE 查询中的 % 和 _ 通配符
///
/// # 参数
/// - `input`: 待转义的字符串
/// - `escape_char`: 转义字符，默认为 \
///
/// # 返回
/// 转义后的字符串
pub fn escape_like(input: &str, escape_char: char) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '%' => {
                result.push(escape_char);
                result.push('%');
            }
            '_' => {
                result.push(escape_char);
                result.push('_');
            }
            _ if ch == escape_char => {
                result.push(escape_char);
                result.push(escape_char);
            }
            _ => result.push(ch),
        }
    }
    result
}
