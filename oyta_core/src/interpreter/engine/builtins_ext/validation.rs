//! 验证函数模块
//!
//! 包含 filter_var、validate_email、validate_url 等验证函数

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;

/// filter_var — 使用特定的过滤器过滤变量
/// filter_var(mixed $value, int $filter = FILTER_DEFAULT, array|int $options = 0): mixed
pub fn builtin_filter_var(args: &[Value]) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Null);
    let filter = args.get(1).map(|v| v.to_int()).unwrap_or(0);

    // 过滤器常量
    const FILTER_VALIDATE_EMAIL: i64 = 274;
    const FILTER_VALIDATE_URL: i64 = 257;
    const FILTER_VALIDATE_IP: i64 = 275;
    const FILTER_VALIDATE_INT: i64 = 257;
    const FILTER_VALIDATE_BOOLEAN: i64 = 258;
    const FILTER_SANITIZE_STRING: i64 = 513;
    const FILTER_SANITIZE_EMAIL: i64 = 517;
    const FILTER_SANITIZE_URL: i64 = 518;

    match filter {
        FILTER_VALIDATE_EMAIL => {
            let email = value.to_string_value();
            let valid = email.contains('@') && email.contains('.');
            if valid { Ok(value) } else { Ok(Value::Bool(false)) }
        }
        FILTER_VALIDATE_URL => {
            let url = value.to_string_value();
            let valid = url.starts_with("http://") || url.starts_with("https://");
            if valid { Ok(value) } else { Ok(Value::Bool(false)) }
        }
        FILTER_VALIDATE_IP => {
            let ip = value.to_string_value();
            let valid = ip.parse::<std::net::IpAddr>().is_ok();
            if valid { Ok(value) } else { Ok(Value::Bool(false)) }
        }
        FILTER_VALIDATE_INT => {
            match &value {
                Value::Int(_) => Ok(value),
                Value::String(s) => {
                    if s.parse::<i64>().is_ok() { Ok(value) } else { Ok(Value::Bool(false)) }
                }
                _ => Ok(Value::Bool(false))
            }
        }
        FILTER_VALIDATE_BOOLEAN => {
            Ok(Value::Bool(value.is_truthy()))
        }
        FILTER_SANITIZE_STRING => {
            let s = value.to_string_value();
            let re = regex::Regex::new(r"<[^>]*>").unwrap();
            Ok(Value::String(re.replace_all(&s, "").to_string()))
        }
        FILTER_SANITIZE_EMAIL => {
            let s = value.to_string_value();
            Ok(Value::String(s.replace(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '-', "")))
        }
        FILTER_SANITIZE_URL => {
            let s = value.to_string_value();
            Ok(Value::String(urlencoding::encode(&s).to_string()))
        }
        _ => Ok(value)
    }
}

/// filter_input — 获取特定外部变量并通过过滤器过滤
/// filter_input(int $type, string $variable_name, int $filter = FILTER_DEFAULT, array|int $options = 0): mixed
pub fn builtin_filter_input(args: &[Value]) -> Result<Value> {
    let _input_type = args.first().map(|v| v.to_int()).unwrap_or(0);
    let _var_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let _filter = args.get(2).map(|v| v.to_int()).unwrap_or(0);

    // 简化实现：返回 null
    // 实际实现需要访问请求上下文
    Ok(Value::Null)
}

/// validate_email — 验证邮箱地址格式
/// validate_email(string $email): bool
pub fn builtin_validate_email(args: &[Value]) -> Result<Value> {
    let email = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // 简单的邮箱验证正则
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    Ok(Value::Bool(email_regex.is_match(&email)))
}

/// validate_url — 验证 URL 格式
/// validate_url(string $url): bool
pub fn builtin_validate_url(args: &[Value]) -> Result<Value> {
    let url = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // URL 验证正则
    let url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    Ok(Value::Bool(url_regex.is_match(&url)))
}

/// validate_ip — 验证 IP 地址格式
/// validate_ip(string $ip, string $version = "both"): bool
pub fn builtin_validate_ip(args: &[Value]) -> Result<Value> {
    let ip = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let version = args.get(1).map(|v| v.to_string_value()).unwrap_or_else(|| "both".to_string());

    match version.as_str() {
        "ipv4" => {
            // IPv4 验证
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() != 4 {
                return Ok(Value::Bool(false));
            }
            let valid = parts.iter().all(|p| {
                p.parse::<u8>().is_ok()
            });
            Ok(Value::Bool(valid))
        }
        "ipv6" => {
            // IPv6 验证（简化）
            Ok(Value::Bool(ip.contains(':')))
        }
        _ => {
            // 两者都接受
            Ok(Value::Bool(ip.parse::<std::net::IpAddr>().is_ok()))
        }
    }
}

/// validate_regex — 使用正则表达式验证字符串
/// validate_regex(string $value, string $pattern): bool
pub fn builtin_validate_regex(args: &[Value]) -> Result<Value> {
    let value = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let pattern = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();

    match regex::Regex::new(&pattern) {
        Ok(re) => Ok(Value::Bool(re.is_match(&value))),
        Err(_) => Ok(Value::Bool(false))
    }
}

/// validate_length — 验证字符串长度
/// validate_length(string $value, int $min, ?int $max = null): bool
pub fn builtin_validate_length(args: &[Value]) -> Result<Value> {
    let value = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let min = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
    let max = args.get(2).map(|v| v.to_int());

    let len = value.chars().count();

    match max {
        Some(m) if m >= 0 => Ok(Value::Bool(len >= min && len <= m as usize)),
        _ => Ok(Value::Bool(len >= min))
    }
}

/// validate_range — 验证数值范围
/// validate_range(int|float $value, int|float $min, int|float $max): bool
pub fn builtin_validate_range(args: &[Value]) -> Result<Value> {
    let value = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let min = args.get(1).map(|v| v.to_float()).unwrap_or(f64::MIN);
    let max = args.get(2).map(|v| v.to_float()).unwrap_or(f64::MAX);

    Ok(Value::Bool(value >= min && value <= max))
}

/// validate_required — 验证必填字段
/// validate_required(mixed $value): bool
pub fn builtin_validate_required(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Null) => Ok(Value::Bool(false)),
        Some(Value::String(s)) if s.is_empty() => Ok(Value::Bool(false)),
        Some(Value::IndexedArray(arr)) if arr.is_empty() => Ok(Value::Bool(false)),
        Some(Value::AssociativeArray(map)) if map.is_empty() => Ok(Value::Bool(false)),
        Some(_) => Ok(Value::Bool(true)),
        None => Ok(Value::Bool(false))
    }
}
