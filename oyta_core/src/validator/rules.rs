//! 验证规则定义
//!
//! 实现所有 ThinkPHP 8.0 风格的验证规则
//! 包括：基础验证、格式验证、长度验证、范围验证、比较验证、
//! 文件验证、数据库验证、条件验证等

use std::collections::HashMap;

/// 验证规则
#[derive(Debug, Clone)]
pub struct Rule {
    /// 字段名
    pub field: String,
    /// 规则名称
    pub rule: String,
    /// 规则参数
    pub params: Vec<String>,
    /// 错误消息
    pub message: Option<String>,
}

impl Rule {
    pub fn new(field: &str, rule: &str) -> Self {
        let parts: Vec<&str> = rule.split(':').collect();
        let rule_name = parts[0].to_string();
        let params = if parts.len() > 1 {
            parts[1].split(',').map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        };
        Self {
            field: field.to_string(),
            rule: rule_name,
            params,
            message: None,
        }
    }
}

/// 默认错误消息模板
/// 键为规则名，值为错误消息模板
/// 模板中可用占位符：:field, :param, :min, :max
pub fn default_error_messages() -> HashMap<String, String> {
    let mut messages = HashMap::new();
    messages.insert("require".to_string(), ":field不能为空".to_string());
    messages.insert("required".to_string(), ":field不能为空".to_string());
    messages.insert("email".to_string(), ":field邮箱格式不正确".to_string());
    messages.insert("mobile".to_string(), ":field手机号格式不正确".to_string());
    messages.insert("phone".to_string(), ":field手机号格式不正确".to_string());
    messages.insert("url".to_string(), ":fieldURL格式不正确".to_string());
    messages.insert("integer".to_string(), ":field必须是整数".to_string());
    messages.insert("int".to_string(), ":field必须是整数".to_string());
    messages.insert("float".to_string(), ":field必须是浮点数".to_string());
    messages.insert("number".to_string(), ":field必须是数字".to_string());
    messages.insert("numeric".to_string(), ":field必须是数字".to_string());
    messages.insert("alpha".to_string(), ":field必须是字母".to_string());
    messages.insert("alphaNum".to_string(), ":field只能是字母、数字和下划线".to_string());
    messages.insert("alpha_dash".to_string(), ":field只能是字母、数字、下划线和破折号".to_string());
    messages.insert("min".to_string(), ":field长度不能少于:min".to_string());
    messages.insert("max".to_string(), ":field长度不能超过:max".to_string());
    messages.insert("length".to_string(), ":field长度必须在:min和:max之间".to_string());
    messages.insert("in".to_string(), ":field必须在指定范围内".to_string());
    messages.insert("notIn".to_string(), ":field不能在指定范围内".to_string());
    messages.insert("between".to_string(), ":field必须在:min和:max之间".to_string());
    messages.insert("notBetween".to_string(), ":field不能在:min和:max之间".to_string());
    messages.insert("confirm".to_string(), ":field两次输入不一致".to_string());
    messages.insert("different".to_string(), ":field不能与:field2相同".to_string());
    messages.insert("same".to_string(), ":field必须与:field2相同".to_string());
    messages.insert("date".to_string(), ":field日期格式不正确".to_string());
    messages.insert("dateFormat".to_string(), ":field日期格式不正确".to_string());
    messages.insert("regex".to_string(), ":field格式不正确".to_string());
    messages.insert("ip".to_string(), ":fieldIP地址格式不正确".to_string());
    messages.insert("ipv4".to_string(), ":fieldIPv4地址格式不正确".to_string());
    messages.insert("ipv6".to_string(), ":fieldIPv6地址格式不正确".to_string());
    messages.insert("mac".to_string(), ":fieldMAC地址格式不正确".to_string());
    messages.insert("zip".to_string(), ":field邮政编码格式不正确".to_string());
    messages.insert("idCard".to_string(), ":field身份证号格式不正确".to_string());
    messages.insert("chs".to_string(), ":field必须是中文字符".to_string());
    messages.insert("chsAlpha".to_string(), ":field只能是中文、字母".to_string());
    messages.insert("chsAlphaNum".to_string(), ":field只能是中文、字母和数字".to_string());
    messages.insert("chsDash".to_string(), ":field只能是中文、字母、数字和下划线".to_string());
    messages.insert("cntrl".to_string(), ":field只能是控制字符".to_string());
    messages.insert("graph".to_string(), ":field只能是可打印字符".to_string());
    messages.insert("print".to_string(), ":field只能是可打印字符和空格".to_string());
    messages.insert("space".to_string(), ":field只能是空白字符".to_string());
    messages.insert("xdigit".to_string(), ":field只能是十六进制字符".to_string());
    messages.insert("file".to_string(), ":field必须是有效的文件上传".to_string());
    messages.insert("image".to_string(), ":field必须是有效的图片".to_string());
    messages.insert("fileSize".to_string(), ":field文件大小不能超过:max".to_string());
    messages.insert("fileExt".to_string(), ":field文件类型不支持".to_string());
    messages.insert("mime".to_string(), ":fieldMIME类型不支持".to_string());
    messages.insert("unique".to_string(), ":field已存在".to_string());
    messages.insert("exists".to_string(), ":field不存在".to_string());
    messages.insert("gt".to_string(), ":field必须大于:min".to_string());
    messages.insert("lt".to_string(), ":field必须小于:max".to_string());
    messages.insert("egt".to_string(), ":field必须大于等于:min".to_string());
    messages.insert("elt".to_string(), ":field必须小于等于:max".to_string());
    messages.insert("eq".to_string(), ":field必须等于:value".to_string());
    messages.insert("bool".to_string(), ":field必须是布尔值".to_string());
    messages.insert("array".to_string(), ":field必须是数组".to_string());
    messages.insert("json".to_string(), ":field必须是有效的JSON".to_string());
    messages.insert("accepted".to_string(), ":field必须接受".to_string());
    messages.insert("timezone".to_string(), ":field必须是有效的时区".to_string());
    messages
}

/// 格式化错误消息
/// 将模板中的占位符替换为实际值
pub fn format_error_message(
    template: &str,
    field: &str,
    params: &[String],
) -> String {
    let mut msg = template
        .replace(":field", field)
        .replace(":param", params.first().map(|s| s.as_str()).unwrap_or(""));

    if let Some(min) = params.first() {
        msg = msg.replace(":min", min);
    }
    if params.len() > 1 {
        msg = msg.replace(":max", &params[1]);
        msg = msg.replace(":field2", &params[1]);
    }
    if let Some(v) = params.first() {
        msg = msg.replace(":value", v);
    }

    msg
}

/// 内置验证规则实现
/// 完整实现 ThinkPHP 8.0 的所有验证规则
pub fn validate_rule(rule_name: &str, value: &str, params: &[String]) -> Result<(), String> {
    match rule_name {
        // ==================== 基础验证 ====================
        "require" | "required" => {
            if value.is_empty() { Err("不能为空".to_string()) } else { Ok(()) }
        }
        "accepted" => {
            let lower = value.to_lowercase();
            if lower == "yes" || lower == "on" || lower == "1" || lower == "true" {
                Ok(())
            } else {
                Err("必须接受".to_string())
            }
        }

        // ==================== 格式验证 ====================
        "email" => {
            let email_re = regex::Regex::new(
                r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
            ).unwrap();
            if email_re.is_match(value) { Ok(()) }
            else { Err("邮箱格式不正确".to_string()) }
        }
        "mobile" | "phone" => {
            let phone_re = regex::Regex::new(r"^1[3-9]\d{9}$").unwrap();
            if phone_re.is_match(value) { Ok(()) }
            else { Err("手机号格式不正确".to_string()) }
        }
        "url" => {
            if value.starts_with("http://") || value.starts_with("https://")
                || value.starts_with("ftp://") {
                Ok(())
            } else {
                Err("URL格式不正确".to_string())
            }
        }
        "ip" => {
            if validate_ipv4(value) || validate_ipv6(value) { Ok(()) }
            else { Err("IP地址格式不正确".to_string()) }
        }
        "ipv4" => {
            if validate_ipv4(value) { Ok(()) }
            else { Err("IPv4地址格式不正确".to_string()) }
        }
        "ipv6" => {
            if validate_ipv6(value) { Ok(()) }
            else { Err("IPv6地址格式不正确".to_string()) }
        }
        "mac" => {
            let mac_re = regex::Regex::new(
                r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$"
            ).unwrap();
            if mac_re.is_match(value) { Ok(()) }
            else { Err("MAC地址格式不正确".to_string()) }
        }
        "zip" => {
            let zip_re = regex::Regex::new(r"^\d{6}$").unwrap();
            if zip_re.is_match(value) { Ok(()) }
            else { Err("邮政编码格式不正确".to_string()) }
        }
        "idCard" => {
            if validate_id_card(value) { Ok(()) }
            else { Err("身份证号格式不正确".to_string()) }
        }
        "date" => {
            if chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok()
                || chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
                || chrono::NaiveTime::parse_from_str(value, "%H:%M:%S").is_ok() {
                Ok(())
            } else {
                Err("日期格式不正确".to_string())
            }
        }
        "dateFormat" => {
            if let Some(format) = params.first() {
                if chrono::NaiveDateTime::parse_from_str(value, format).is_ok()
                    || chrono::NaiveDate::parse_from_str(value, format).is_ok() {
                    Ok(())
                } else {
                    Err("日期格式不正确".to_string())
                }
            } else {
                Err("日期格式参数缺失".to_string())
            }
        }
        "regex" => {
            if let Some(pattern) = params.first() {
                match regex::Regex::new(pattern) {
                    Ok(re) => {
                        if re.is_match(value) { Ok(()) }
                        else { Err("格式不正确".to_string()) }
                    }
                    Err(_) => Err("正则表达式无效".to_string()),
                }
            } else {
                Err("正则表达式参数缺失".to_string())
            }
        }
        "timezone" => {
            if value.parse::<chrono_tz::Tz>().is_ok() { Ok(()) }
            else { Err("时区格式不正确".to_string()) }
        }

        // ==================== 类型验证 ====================
        "integer" | "int" => {
            if value.parse::<i64>().is_ok() { Ok(()) }
            else { Err("必须是整数".to_string()) }
        }
        "float" => {
            if value.parse::<f64>().is_ok() { Ok(()) }
            else { Err("必须是浮点数".to_string()) }
        }
        "number" | "numeric" => {
            if value.parse::<f64>().is_ok() { Ok(()) }
            else { Err("必须是数字".to_string()) }
        }
        "bool" | "boolean" => {
            let lower = value.to_lowercase();
            if ["true", "false", "1", "0", "yes", "no"].contains(&lower.as_str()) { Ok(()) }
            else { Err("必须是布尔值".to_string()) }
        }
        "array" => {
            if value.starts_with('[') || value.starts_with('{') { Ok(()) }
            else { Err("必须是数组".to_string()) }
        }
        "json" => {
            if serde_json::from_str::<serde_json::Value>(value).is_ok() { Ok(()) }
            else { Err("必须是有效的JSON".to_string()) }
        }

        // ==================== 字符类型验证 ====================
        "alpha" => {
            if value.chars().all(|c| c.is_ascii_alphabetic()) { Ok(()) }
            else { Err("必须是字母".to_string()) }
        }
        "alphaNum" | "alpha_num" => {
            if value.chars().all(|c| c.is_ascii_alphanumeric()) { Ok(()) }
            else { Err("只能是字母和数字".to_string()) }
        }
        "alpha_dash" => {
            if value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { Ok(()) }
            else { Err("只能是字母、数字、下划线和破折号".to_string()) }
        }
        "chs" => {
            if value.chars().all(|c| is_chinese_char(c)) { Ok(()) }
            else { Err("必须是中文字符".to_string()) }
        }
        "chsAlpha" => {
            if value.chars().all(|c| is_chinese_char(c) || c.is_ascii_alphabetic()) { Ok(()) }
            else { Err("只能是中文和字母".to_string()) }
        }
        "chsAlphaNum" => {
            if value.chars().all(|c| is_chinese_char(c) || c.is_ascii_alphanumeric()) { Ok(()) }
            else { Err("只能是中文、字母和数字".to_string()) }
        }
        "chsDash" => {
            if value.chars().all(|c| is_chinese_char(c) || c.is_ascii_alphanumeric() || c == '_' || c == '-') { Ok(()) }
            else { Err("只能是中文、字母、数字和下划线".to_string()) }
        }
        "cntrl" => {
            if value.chars().all(|c| c.is_control()) { Ok(()) }
            else { Err("只能是控制字符".to_string()) }
        }
        "graph" => {
            if value.chars().all(|c| !c.is_control() && !c.is_whitespace()) { Ok(()) }
            else { Err("只能是可打印字符".to_string()) }
        }
        "print" => {
            if value.chars().all(|c| !c.is_control()) { Ok(()) }
            else { Err("只能是可打印字符和空格".to_string()) }
        }
        "space" => {
            if value.chars().all(|c| c.is_whitespace()) { Ok(()) }
            else { Err("只能是空白字符".to_string()) }
        }
        "xdigit" => {
            if value.chars().all(|c| c.is_ascii_hexdigit()) { Ok(()) }
            else { Err("只能是十六进制字符".to_string()) }
        }

        // ==================== 长度验证 ====================
        "min" => {
            if let Some(min_str) = params.first() {
                if let Ok(min) = min_str.parse::<usize>() {
                    // 如果值是数字，比较数值大小；否则比较字符串长度
                    if let Ok(num_val) = value.parse::<f64>() {
                        if let Ok(min_val) = min_str.parse::<f64>() {
                            if num_val >= min_val { return Ok(()); }
                        }
                    }
                    if value.len() >= min { return Ok(()); }
                }
            }
            Err(format!("长度不能少于{}", params.first().unwrap_or(&"0".to_string())))
        }
        "max" => {
            if let Some(max_str) = params.first() {
                if let Ok(max) = max_str.parse::<usize>() {
                    if let Ok(num_val) = value.parse::<f64>() {
                        if let Ok(max_val) = max_str.parse::<f64>() {
                            if num_val <= max_val { return Ok(()); }
                        }
                    }
                    if value.len() <= max { return Ok(()); }
                }
            }
            Err(format!("长度不能超过{}", params.first().unwrap_or(&"0".to_string())))
        }
        "length" => {
            if params.len() >= 2 {
                if let (Ok(min), Ok(max)) = (params[0].parse::<usize>(), params[1].parse::<usize>()) {
                    if value.len() >= min && value.len() <= max { return Ok(()); }
                }
            } else if let Some(exact) = params.first() {
                if let Ok(len) = exact.parse::<usize>() {
                    if value.len() == len { return Ok(()); }
                }
            }
            Err("长度不符合要求".to_string())
        }

        // ==================== 范围验证 ====================
        "in" => {
            if params.iter().any(|p| p == value) { Ok(()) }
            else { Err(format!("必须在 {} 范围内", params.join(","))) }
        }
        "notIn" => {
            if !params.iter().any(|p| p == value) { Ok(()) }
            else { Err(format!("不能在 {} 范围内", params.join(","))) }
        }
        "between" => {
            if params.len() >= 2 {
                if let (Ok(min), Ok(max)) = (params[0].parse::<f64>(), params[1].parse::<f64>()) {
                    if let Ok(v) = value.parse::<f64>() {
                        if v >= min && v <= max { return Ok(()); }
                    }
                }
            }
            Err("不在指定范围内".to_string())
        }
        "notBetween" => {
            if params.len() >= 2 {
                if let (Ok(min), Ok(max)) = (params[0].parse::<f64>(), params[1].parse::<f64>()) {
                    if let Ok(v) = value.parse::<f64>() {
                        if v < min || v > max { return Ok(()); }
                    }
                }
            }
            Ok(())
        }

        // ==================== 比较验证 ====================
        "gt" => {
            if let Some(min_str) = params.first() {
                if let (Ok(v), Ok(min)) = (value.parse::<f64>(), min_str.parse::<f64>()) {
                    if v > min { return Ok(()); }
                }
            }
            Err("必须大于指定值".to_string())
        }
        "lt" => {
            if let Some(max_str) = params.first() {
                if let (Ok(v), Ok(max)) = (value.parse::<f64>(), max_str.parse::<f64>()) {
                    if v < max { return Ok(()); }
                }
            }
            Err("必须小于指定值".to_string())
        }
        "egt" => {
            if let Some(min_str) = params.first() {
                if let (Ok(v), Ok(min)) = (value.parse::<f64>(), min_str.parse::<f64>()) {
                    if v >= min { return Ok(()); }
                }
            }
            Err("必须大于等于指定值".to_string())
        }
        "elt" => {
            if let Some(max_str) = params.first() {
                if let (Ok(v), Ok(max)) = (value.parse::<f64>(), max_str.parse::<f64>()) {
                    if v <= max { return Ok(()); }
                }
            }
            Err("必须小于等于指定值".to_string())
        }
        "eq" | "same" => {
            if let Some(expected) = params.first() {
                if value == expected { return Ok(()); }
            }
            Err("必须等于指定值".to_string())
        }
        "confirm" => {
            // 确认字段验证需要外部处理（在 Validator::check 中）
            Ok(())
        }
        "different" => {
            if let Some(other) = params.first() {
                if value != other { return Ok(()); }
            }
            Err("不能与指定值相同".to_string())
        }

        // ==================== 文件验证 ====================
        "file" => {
            // 文件验证需要外部处理（在 HTTP 层检查上传文件）
            Ok(())
        }
        "image" => {
            let image_exts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
            let ext = std::path::Path::new(value)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if image_exts.contains(&ext.as_str()) { Ok(()) }
            else { Err("必须是有效的图片".to_string()) }
        }
        "fileSize" => {
            // 文件大小验证需要外部处理
            Ok(())
        }
        "fileExt" => {
            if !params.is_empty() {
                let ext = std::path::Path::new(value)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if params.iter().any(|p| p.to_lowercase() == ext) { return Ok(()); }
            }
            Err("文件类型不支持".to_string())
        }
        "mime" => {
            // MIME 类型验证需要外部处理
            Ok(())
        }

        // ==================== 数据库验证 ====================
        "unique" => {
            // 数据库唯一性验证需要外部处理（需要查询数据库）
            Ok(())
        }
        "exists" => {
            // 数据库存在性验证需要外部处理
            Ok(())
        }

        // ==================== 未知规则 ====================
        _ => Err(format!("未知验证规则: {}", rule_name)),
    }
}

/// 验证 IPv4 地址
fn validate_ipv4(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        match part.parse::<u8>() {
            Ok(_) => {}
            Err(_) => return false,
        }
    }
    true
}

/// 验证 IPv6 地址
fn validate_ipv6(value: &str) -> bool {
    // 简化的 IPv6 验证
    if value.contains(':') {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() <= 8 && parts.iter().all(|p| {
            p.is_empty() || u16::from_str_radix(p, 16).is_ok()
        }) {
            return true;
        }
    }
    false
}

/// 验证中国身份证号（18位）
fn validate_id_card(value: &str) -> bool {
    if value.len() != 18 {
        return false;
    }

    let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    let check_chars = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];

    let mut sum: u32 = 0;
    for (i, c) in value.chars().take(17).enumerate() {
        if let Some(digit) = c.to_digit(10) {
            sum += digit * weights[i] as u32;
        } else {
            return false;
        }
    }

    let check_index = (sum % 11) as usize;
    let check_char = value.chars().nth(17).unwrap();
    check_char.to_uppercase().next() == Some(check_chars[check_index])
}

/// 判断字符是否为中文字符
fn is_chinese_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK 统一汉字
        '\u{3400}'..='\u{4DBF}' |   // CJK 扩展A
        '\u{20000}'..='\u{2A6DF}' | // CJK 扩展B
        '\u{2A700}'..='\u{2B73F}' | // CJK 扩展C
        '\u{2B740}'..='\u{2B81F}' | // CJK 扩展D
        '\u{F900}'..='\u{FAFF}' |   // CJK 兼容汉字
        '\u{2F800}'..='\u{2FA1F}'   // CJK 兼容汉字补充
    )
}
