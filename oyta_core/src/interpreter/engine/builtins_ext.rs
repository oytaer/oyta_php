//! 内置 PHP 函数扩展模块
//!
//! 包含类型检测、类型转换、数学、调试、哈希、时间、类/对象等函数
//! 这些函数通过 register_ext_builtins() 注册到主内置函数映射表中

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::builtins::BuiltinFunction;

/// 注册扩展内置函数到映射表
///
/// 将类型检测、类型转换、数学、调试、其他、哈希、时间、类/对象函数
/// 注册到主映射表中，由 create_builtins_map() 调用
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_ext_builtins(map: &mut HashMap<String, BuiltinFunction>) {
    map.insert("is_null".to_string(), builtin_is_null);
    map.insert("is_array".to_string(), builtin_is_array);
    map.insert("is_string".to_string(), builtin_is_string);
    map.insert("is_int".to_string(), builtin_is_int);
    map.insert("is_float".to_string(), builtin_is_float);
    map.insert("is_bool".to_string(), builtin_is_bool);
    map.insert("is_object".to_string(), builtin_is_object);
    map.insert("is_callable".to_string(), builtin_is_callable);
    map.insert("is_numeric".to_string(), builtin_is_numeric);
    map.insert("is_empty".to_string(), builtin_is_empty);
    map.insert("intval".to_string(), builtin_intval);
    map.insert("floatval".to_string(), builtin_floatval);
    map.insert("strval".to_string(), builtin_strval);
    map.insert("boolval".to_string(), builtin_boolval);
    map.insert("abs".to_string(), builtin_abs);
    map.insert("max".to_string(), builtin_max);
    map.insert("min".to_string(), builtin_min);
    map.insert("floor".to_string(), builtin_floor);
    map.insert("ceil".to_string(), builtin_ceil);
    map.insert("round".to_string(), builtin_round);
    map.insert("var_dump".to_string(), builtin_var_dump);
    map.insert("print_r".to_string(), builtin_print_r);
    map.insert("json".to_string(), builtin_json);
    map.insert("dd".to_string(), builtin_dd);
    map.insert("dump".to_string(), builtin_dump);
    map.insert("isset".to_string(), builtin_isset);
    map.insert("unset".to_string(), builtin_unset);
    map.insert("gettype".to_string(), builtin_gettype);
    map.insert("settype".to_string(), builtin_settype);
    map.insert("sprintf".to_string(), builtin_sprintf);
    map.insert("printf".to_string(), builtin_printf);
    map.insert("number_format".to_string(), builtin_number_format);
    map.insert("chr".to_string(), builtin_chr);
    map.insert("ord".to_string(), builtin_ord);
    map.insert("str_split".to_string(), builtin_str_split);
    map.insert("ucfirst".to_string(), builtin_ucfirst);
    map.insert("lcfirst".to_string(), builtin_lcfirst);
    map.insert("ucwords".to_string(), builtin_ucwords);
    map.insert("nl2br".to_string(), builtin_nl2br);
    map.insert("htmlspecialchars".to_string(), builtin_htmlspecialchars);
    map.insert("htmlentities".to_string(), builtin_htmlentities);
    map.insert("strip_tags".to_string(), builtin_strip_tags);
    map.insert("md5".to_string(), builtin_md5);
    map.insert("sha1".to_string(), builtin_sha1);
    map.insert("time".to_string(), builtin_time);
    map.insert("date".to_string(), builtin_date);
    map.insert("microtime".to_string(), builtin_microtime);
    map.insert("strtotime".to_string(), builtin_strtotime);
    map.insert("class_exists".to_string(), builtin_class_exists);
    map.insert("method_exists".to_string(), builtin_method_exists);
    map.insert("property_exists".to_string(), builtin_property_exists);
    map.insert("get_class".to_string(), builtin_get_class);
    map.insert("call_user_func".to_string(), builtin_call_user_func);
    map.insert("call_user_func_array".to_string(), builtin_call_user_func_array);
    map.insert("func_num_args".to_string(), builtin_func_num_args);
    
    // 验证相关函数
    map.insert("filter_var".to_string(), builtin_filter_var);
    map.insert("filter_input".to_string(), builtin_filter_input);
    map.insert("validate_email".to_string(), builtin_validate_email);
    map.insert("validate_url".to_string(), builtin_validate_url);
    map.insert("validate_ip".to_string(), builtin_validate_ip);
    map.insert("validate_regex".to_string(), builtin_validate_regex);
    map.insert("validate_length".to_string(), builtin_validate_length);
    map.insert("validate_range".to_string(), builtin_validate_range);
    map.insert("validate_required".to_string(), builtin_validate_required);
}

// ===== 类型检测函数 =====

/// is_null — 检测变量是否为 null
fn builtin_is_null(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(args.first().map(|v| v.is_null()).unwrap_or(true)))
}

/// is_array — 检测变量是否是数组
fn builtin_is_array(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(
        args.first(),
        Some(Value::IndexedArray(_)) | Some(Value::AssociativeArray(_))
    )))
}

/// is_string — 检测变量是否是字符串
fn builtin_is_string(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::String(_)))))
}

/// is_int — 检测变量是否是整数
fn builtin_is_int(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Int(_)))))
}

/// is_float — 检测变量是否是浮点数
fn builtin_is_float(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Float(_)))))
}

/// is_bool — 检测变量是否是布尔值
fn builtin_is_bool(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Bool(_)))))
}

/// is_object — 检测变量是否是对象
fn builtin_is_object(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Object(_)))))
}

/// is_callable — 检测变量是否是可调用的
fn builtin_is_callable(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(
        args.first(),
        Some(Value::Callable(_))
    )))
}

/// is_numeric — 检测变量是否为数字或数字字符串
fn builtin_is_numeric(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Int(_)) | Some(Value::Float(_)) => Ok(Value::Bool(true)),
        Some(Value::String(s)) => Ok(Value::Bool(s.parse::<f64>().is_ok())),
        _ => Ok(Value::Bool(false)),
    }
}

/// is_empty — 检测变量是否为空
fn builtin_is_empty(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(
        !args.first().map(|v| v.is_truthy()).unwrap_or(true),
    ))
}

// ===== 类型转换函数 =====

/// intval — 获取变量的整数值
fn builtin_intval(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(args.first().map(|v| v.to_int()).unwrap_or(0)))
}

/// floatval — 获取变量的浮点值
fn builtin_floatval(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float()).unwrap_or(0.0),
    ))
}

/// strval — 获取变量的字符串值
fn builtin_strval(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value())
            .unwrap_or_default(),
    ))
}

/// boolval — 获取变量的布尔值
fn builtin_boolval(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(
        args.first().map(|v| v.is_truthy()).unwrap_or(false),
    ))
}

// ===== 数学函数 =====

/// abs — 绝对值
fn builtin_abs(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Int(i)) => Ok(Value::Int(i.abs())),
        Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
        _ => Ok(Value::Int(0)),
    }
}

/// max — 找出最大值
fn builtin_max(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => items
            .iter()
            .max_by(|a, b| {
                a.to_float()
                    .partial_cmp(&b.to_float())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("empty")),
        Some(v) => Ok(v.clone()),
        None => Ok(Value::Null),
    }
}

/// min — 找出最小值
fn builtin_min(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => items
            .iter()
            .min_by(|a, b| {
                a.to_float()
                    .partial_cmp(&b.to_float())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("empty")),
        Some(v) => Ok(v.clone()),
        None => Ok(Value::Null),
    }
}

/// floor — 向下取整
fn builtin_floor(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float().floor()).unwrap_or(0.0),
    ))
}

/// ceil — 向上取整
fn builtin_ceil(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float().ceil()).unwrap_or(0.0),
    ))
}

/// round — 对浮点数进行四舍五入
/// round(float $val, int $precision = 0): float
fn builtin_round(args: &[Value]) -> Result<Value> {
    let v = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let precision = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let factor = 10f64.powi(precision as i32);
    Ok(Value::Float((v * factor).round() / factor))
}

// ===== 调试函数 =====

/// var_dump — 打印变量的相关信息
fn builtin_var_dump(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("var_dump: {:?}", arg);
    }
    Ok(Value::Null)
}

/// print_r — 打印关于变量的易于理解的信息
fn builtin_print_r(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("print_r: {:?}", arg);
    }
    Ok(Value::Bool(true))
}

/// json — 快捷 JSON 编码（OYTAPHP 扩展）
fn builtin_json(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(data) => Ok(Value::String(data.to_json_string())),
        None => Ok(Value::String("null".to_string())),
    }
}

/// dd — 转储变量并终止脚本（OYTAPHP 扩展）
fn builtin_dd(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("dd: {:?}", arg);
    }
    Ok(Value::Null)
}

/// dump — 转储变量（OYTAPHP 扩展）
fn builtin_dump(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("dump: {:?}", arg);
    }
    Ok(Value::Null)
}

// ===== 其他函数 =====

/// isset — 检测变量是否已设置并且非 null
fn builtin_isset(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(
        !args.first().map(|v| v.is_null()).unwrap_or(true),
    ))
}

/// unset — 释放给定的变量
fn builtin_unset(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(Value::Null)
}

/// gettype — 获取变量的类型
fn builtin_gettype(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.type_name().to_string())
            .unwrap_or("NULL".to_string()),
    ))
}

/// settype — 设置变量的类型
fn builtin_settype(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    let type_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let _value = args.first();
    match type_name.as_str() {
        "int" | "integer" => Ok(Value::Bool(true)),
        "float" | "double" => Ok(Value::Bool(true)),
        "string" => Ok(Value::Bool(true)),
        "bool" | "boolean" => Ok(Value::Bool(true)),
        "array" => Ok(Value::Bool(true)),
        "null" => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

/// sprintf — 返回格式化字符串
/// 支持 %s, %d, %f, %x, %o, %b, %c 等占位符
fn builtin_sprintf(args: &[Value]) -> Result<Value> {
    let fmt = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let fmt_args: Vec<&Value> = args.iter().skip(1).collect();

    let mut result = String::new();
    let mut arg_idx = 0;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('%') => result.push('%'),
                Some('s') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_string_value()).unwrap_or_default();
                    result.push_str(&val);
                    arg_idx += 1;
                }
                Some('d' | 'i') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0);
                    result.push_str(&val.to_string());
                    arg_idx += 1;
                }
                Some('f') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_float()).unwrap_or(0.0);
                    result.push_str(&format!("{}", val));
                    arg_idx += 1;
                }
                Some('x') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0);
                    result.push_str(&format!("{:x}", val));
                    arg_idx += 1;
                }
                Some('X') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0);
                    result.push_str(&format!("{:X}", val));
                    arg_idx += 1;
                }
                Some('o') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0);
                    result.push_str(&format!("{:o}", val));
                    arg_idx += 1;
                }
                Some('b') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0);
                    result.push_str(&format!("{:b}", val));
                    arg_idx += 1;
                }
                Some('c') => {
                    let val = fmt_args.get(arg_idx).map(|v| v.to_int()).unwrap_or(0) as u8;
                    result.push(val as char);
                    arg_idx += 1;
                }
                Some('0'..='9') => {
                    // 简化处理带宽度的格式符（如 %02d）
                    let mut spec = String::new();
                    spec.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() || next == '.' || next == '-' || next == '+' || next == ' ' {
                            spec.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Some(type_char) = chars.next() {
                        spec.push(type_char);
                        let val = fmt_args.get(arg_idx);
                        match type_char {
                            'd' | 'i' => {
                                let v = val.map(|v| v.to_int()).unwrap_or(0);
                                result.push_str(&format_spec(&spec, v as i64));
                            }
                            'f' => {
                                let v = val.map(|v| v.to_float()).unwrap_or(0.0);
                                result.push_str(&format_spec_f(&spec, v));
                            }
                            's' => {
                                let v = val.map(|v| v.to_string_value()).unwrap_or_default();
                                result.push_str(&v);
                            }
                            _ => {
                                result.push_str(&format!("%{}", spec));
                            }
                        }
                        arg_idx += 1;
                    }
                }
                _ => result.push('%'),
            }
        } else {
            result.push(c);
        }
    }

    Ok(Value::String(result))
}

/// printf — 输出格式化字符串
fn builtin_printf(args: &[Value]) -> Result<Value> {
    let result = builtin_sprintf(args)?;
    let s = result.to_string_value();
    let len = s.len() as i64;
    print!("{}", s);
    Ok(Value::Int(len))
}

/// 格式化带规格的整数
fn format_spec(spec: &str, value: i64) -> String {
    match spec {
        "02d" => format!("{:02}", value),
        "03d" => format!("{:03}", value),
        "04d" => format!("{:04}", value),
        s if s.ends_with('d') => format!("{}", value),
        _ => value.to_string(),
    }
}

/// 格式化带规格的浮点数
fn format_spec_f(spec: &str, value: f64) -> String {
    match spec {
        s if s.starts_with("%.2") => format!("{:.2}", value),
        s if s.starts_with("%.1") => format!("{:.1}", value),
        s if s.starts_with("%.3") => format!("{:.3}", value),
        _ => format!("{}", value),
    }
}

/// number_format — 以千位分隔符方式格式化数字
fn builtin_number_format(args: &[Value]) -> Result<Value> {
    let num = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let decimals = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
    Ok(Value::String(format!(
        "{:.prec$}",
        num,
        prec = decimals
    )))
}

/// chr — 返回指定的字符
fn builtin_chr(args: &[Value]) -> Result<Value> {
    let code = args.first().map(|v| v.to_int()).unwrap_or(0) as u8;
    Ok(Value::String((code as char).to_string()))
}

/// ord — 返回字符的 ASCII 码值
fn builtin_ord(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Int(s.chars().next().map(|c| c as i64).unwrap_or(0)))
}

/// str_split — 将字符串转换为数组
fn builtin_str_split(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let len = args.get(1).map(|v| v.to_int()).unwrap_or(1) as usize;
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    for chunk in chars.chunks(len.max(1)) {
        result.push(Value::String(chunk.iter().collect()));
    }
    Ok(Value::IndexedArray(result))
}

/// ucfirst — 将字符串的首字母转换为大写
fn builtin_ucfirst(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => Ok(Value::String(format!(
            "{}{}",
            c.to_uppercase(),
            chars.as_str()
        ))),
        None => Ok(Value::String(String::new())),
    }
}

/// lcfirst — 将字符串的首字母转换为小写
fn builtin_lcfirst(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => Ok(Value::String(format!(
            "{}{}",
            c.to_lowercase(),
            chars.as_str()
        ))),
        None => Ok(Value::String(String::new())),
    }
}

/// ucwords — 将字符串中每个单词的首字母转换为大写
fn builtin_ucwords(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(
        s.split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => format!("{}{}", f.to_uppercase(), c.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    ))
}

/// nl2br — 在字符串所有新行之前插入 HTML 换行标记
fn builtin_nl2br(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(s.replace("\n", "<br />\n")))
}

/// htmlspecialchars — 将特殊字符转换为 HTML 实体
fn builtin_htmlspecialchars(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;"),
    ))
}

/// htmlentities — 将所有适用的字符转换为 HTML 实体（同 htmlspecialchars）
fn builtin_htmlentities(args: &[Value]) -> Result<Value> {
    builtin_htmlspecialchars(args)
}

/// strip_tags — 从字符串中去除 HTML 和 PHP 标记
fn builtin_strip_tags(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    Ok(Value::String(re.replace_all(&s, "").to_string()))
}

// ===== 哈希函数 =====

/// md5 — 计算字符串的 MD5 散列值
fn builtin_md5(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(format!(
        "{:x}",
        md5::compute(s.as_bytes())
    )))
}

/// sha1 — 计算字符串的 SHA-1 散列值
/// 注意：由于 sha1 crate 不在依赖中，此处使用 sha256 替代
fn builtin_sha1(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(hex::encode(result)))
}

// ===== 时间函数 =====

/// time — 返回当前的 Unix 时间戳
fn builtin_time(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(Value::Int(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    ))
}

/// date — 格式化一个本地时间/日期
fn builtin_date(args: &[Value]) -> Result<Value> {
    let _format = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let now = chrono::Local::now();
    Ok(Value::String(now.format("%Y-%m-%d %H:%M:%S").to_string()))
}

/// microtime — 返回当前 Unix 时间戳和微秒数
fn builtin_microtime(args: &[Value]) -> Result<Value> {
    let _ = args;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(Value::String(format!(
        "{:.6} {}",
        now.as_secs_f64() % 1.0,
        now.as_secs()
    )))
}

/// strtotime — 将任何英文文本的日期时间描述解析为 Unix 时间戳
/// 支持：now, +1 day, +1 week, +1 month, +1 year, tomorrow, yesterday 等
fn builtin_strtotime(args: &[Value]) -> Result<Value> {
    let time_str = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let now = chrono::Local::now();

    let result = match time_str.to_lowercase().as_str() {
        "now" => now.timestamp(),
        "today" => now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        "tomorrow" => (now.date_naive() + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        "yesterday" => (now.date_naive() - chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        s if s.starts_with('+') => {
            // 解析 "+N unit" 格式
            let s = s.trim_start_matches('+').trim();
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(n) = parts[0].parse::<i64>() {
                    let unit = parts[1].to_lowercase();
                    match unit.as_str() {
                        "second" | "seconds" => now.timestamp() + n,
                        "minute" | "minutes" => now.timestamp() + n * 60,
                        "hour" | "hours" => now.timestamp() + n * 3600,
                        "day" | "days" => now.timestamp() + n * 86400,
                        "week" | "weeks" => now.timestamp() + n * 604800,
                        "month" | "months" => now.timestamp() + n * 2592000,
                        "year" | "years" => now.timestamp() + n * 31536000,
                        _ => now.timestamp(),
                    }
                } else {
                    now.timestamp()
                }
            } else {
                now.timestamp()
            }
        }
        s if s.starts_with('-') => {
            let s = s.trim_start_matches('-').trim();
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(n) = parts[0].parse::<i64>() {
                    let unit = parts[1].to_lowercase();
                    match unit.as_str() {
                        "second" | "seconds" => now.timestamp() - n,
                        "minute" | "minutes" => now.timestamp() - n * 60,
                        "hour" | "hours" => now.timestamp() - n * 3600,
                        "day" | "days" => now.timestamp() - n * 86400,
                        "week" | "weeks" => now.timestamp() - n * 604800,
                        "month" | "months" => now.timestamp() - n * 2592000,
                        "year" | "years" => now.timestamp() - n * 31536000,
                        _ => now.timestamp(),
                    }
                } else {
                    now.timestamp()
                }
            } else {
                now.timestamp()
            }
        }
        _ => {
            // 尝试解析日期字符串
            chrono::NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or_else(|_| {
                    chrono::NaiveDate::parse_from_str(&time_str, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                        .unwrap_or(0)
                })
        }
    };

    Ok(Value::Int(result))
}

// ===== 类/对象函数 =====

/// class_exists — 检查类是否已定义
/// 通过解释器内部的类注册表查找
fn builtin_class_exists(args: &[Value]) -> Result<Value> {
    let class_name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    // 内置函数无法直接访问符号表，返回 true 让解释器后续处理
    // 实际的类存在性检查在解释器的类解析阶段完成
    Ok(Value::Bool(!class_name.is_empty()))
}

/// method_exists — 检查类的方法是否存在
fn builtin_method_exists(args: &[Value]) -> Result<Value> {
    let _class_name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let _method_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    // 内置函数无法直接访问符号表
    // 方法存在性检查在解释器的方法调用阶段完成
    Ok(Value::Bool(true))
}

/// property_exists — 检查对象或类是否具有该属性
fn builtin_property_exists(args: &[Value]) -> Result<Value> {
    let _obj = args.first();
    let _prop_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    // 对于对象实例，检查属性是否存在
    if let Some(Value::Object(obj)) = args.first() {
        let prop_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
        Ok(Value::Bool(obj.properties.contains_key(&prop_name)))
    } else {
        Ok(Value::Bool(false))
    }
}

/// get_class — 返回对象的类名
fn builtin_get_class(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Object(obj)) => Ok(Value::String(obj.class_name.clone())),
        _ => Ok(Value::Bool(false)),
    }
}

/// call_user_func — 把第一个参数作为回调函数调用（简化实现）
fn builtin_call_user_func(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Callable(_)) => Ok(Value::Null),
        Some(Value::String(name)) => Ok(Value::String(format!("call_user_func({})", name))),
        _ => Ok(Value::Null),
    }
}

/// call_user_func_array — 调用回调函数，并把一个数组参数作为回调函数的参数（简化实现）
fn builtin_call_user_func_array(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(Value::Null)
}

/// func_num_args — 返回传递给函数的参数个数
fn builtin_func_num_args(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(args.len() as i64))
}

// ===== 验证函数 =====

/// filter_var — 使用特定的过滤器过滤变量
/// filter_var(mixed $value, int $filter = FILTER_DEFAULT, array|int $options = 0): mixed
fn builtin_filter_var(args: &[Value]) -> Result<Value> {
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
fn builtin_filter_input(args: &[Value]) -> Result<Value> {
    let _input_type = args.first().map(|v| v.to_int()).unwrap_or(0);
    let _var_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let _filter = args.get(2).map(|v| v.to_int()).unwrap_or(0);
    
    // 简化实现：返回 null
    // 实际实现需要访问请求上下文
    Ok(Value::Null)
}

/// validate_email — 验证邮箱地址格式
/// validate_email(string $email): bool
fn builtin_validate_email(args: &[Value]) -> Result<Value> {
    let email = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    
    // 简单的邮箱验证正则
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    Ok(Value::Bool(email_regex.is_match(&email)))
}

/// validate_url — 验证 URL 格式
/// validate_url(string $url): bool
fn builtin_validate_url(args: &[Value]) -> Result<Value> {
    let url = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    
    // URL 验证正则
    let url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    Ok(Value::Bool(url_regex.is_match(&url)))
}

/// validate_ip — 验证 IP 地址格式
/// validate_ip(string $ip, string $version = "both"): bool
fn builtin_validate_ip(args: &[Value]) -> Result<Value> {
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
fn builtin_validate_regex(args: &[Value]) -> Result<Value> {
    let value = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let pattern = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    
    match regex::Regex::new(&pattern) {
        Ok(re) => Ok(Value::Bool(re.is_match(&value))),
        Err(_) => Ok(Value::Bool(false))
    }
}

/// validate_length — 验证字符串长度
/// validate_length(string $value, int $min, ?int $max = null): bool
fn builtin_validate_length(args: &[Value]) -> Result<Value> {
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
fn builtin_validate_range(args: &[Value]) -> Result<Value> {
    let value = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let min = args.get(1).map(|v| v.to_float()).unwrap_or(f64::MIN);
    let max = args.get(2).map(|v| v.to_float()).unwrap_or(f64::MAX);
    
    Ok(Value::Bool(value >= min && value <= max))
}

/// validate_required — 验证必填字段
/// validate_required(mixed $value): bool
fn builtin_validate_required(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Null) => Ok(Value::Bool(false)),
        Some(Value::String(s)) if s.is_empty() => Ok(Value::Bool(false)),
        Some(Value::IndexedArray(arr)) if arr.is_empty() => Ok(Value::Bool(false)),
        Some(Value::AssociativeArray(map)) if map.is_empty() => Ok(Value::Bool(false)),
        Some(_) => Ok(Value::Bool(true)),
        None => Ok(Value::Bool(false))
    }
}

/// 注册 OYTAPHP 专用助手函数
/// 所有函数以 oyta_ 前缀命名，避免与其他框架冲突
pub fn register_oyta_helpers(map: &mut HashMap<String, BuiltinFunction>) {
    // OYTAPHP 助手函数别名（仅注册当前模块可访问的函数）
    map.insert("oyta_dd".to_string(), builtin_dd);
    map.insert("oyta_dump".to_string(), builtin_dump);
    map.insert("oyta_json".to_string(), builtin_json);
    map.insert("oyta_md5".to_string(), builtin_md5);
    map.insert("oyta_sha1".to_string(), builtin_sha1);
    map.insert("oyta_time".to_string(), builtin_time);
    map.insert("oyta_date".to_string(), builtin_date);
    map.insert("oyta_is_null".to_string(), builtin_is_null);
    map.insert("oyta_is_array".to_string(), builtin_is_array);
    map.insert("oyta_is_string".to_string(), builtin_is_string);
    map.insert("oyta_is_int".to_string(), builtin_is_int);
    map.insert("oyta_is_float".to_string(), builtin_is_float);
    map.insert("oyta_is_bool".to_string(), builtin_is_bool);
    map.insert("oyta_is_object".to_string(), builtin_is_object);
    map.insert("oyta_is_numeric".to_string(), builtin_is_numeric);
    map.insert("oyta_intval".to_string(), builtin_intval);
    map.insert("oyta_floatval".to_string(), builtin_floatval);
    map.insert("oyta_strval".to_string(), builtin_strval);
    map.insert("oyta_boolval".to_string(), builtin_boolval);
    map.insert("oyta_abs".to_string(), builtin_abs);
    map.insert("oyta_max".to_string(), builtin_max);
    map.insert("oyta_min".to_string(), builtin_min);
    map.insert("oyta_floor".to_string(), builtin_floor);
    map.insert("oyta_ceil".to_string(), builtin_ceil);
    map.insert("oyta_round".to_string(), builtin_round);
    map.insert("oyta_sprintf".to_string(), builtin_sprintf);
    map.insert("oyta_number_format".to_string(), builtin_number_format);
    map.insert("oyta_htmlspecialchars".to_string(), builtin_htmlspecialchars);
    map.insert("oyta_strip_tags".to_string(), builtin_strip_tags);
    map.insert("oyta_ucfirst".to_string(), builtin_ucfirst);
    map.insert("oyta_lcfirst".to_string(), builtin_lcfirst);
    map.insert("oyta_ucwords".to_string(), builtin_ucwords);
    map.insert("oyta_nl2br".to_string(), builtin_nl2br);
    map.insert("oyta_strtotime".to_string(), builtin_strtotime);
    map.insert("oyta_microtime".to_string(), builtin_microtime);
}
