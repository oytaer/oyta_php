//! 字符串扩展函数模块
//!
//! 包含 sprintf、printf、number_format、chr、ord 等字符串处理函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// sprintf — 返回格式化字符串
/// 支持 %s, %d, %f, %x, %o, %b, %c 等占位符
pub fn builtin_sprintf(args: &[Value]) -> Result<Value> {
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
pub fn builtin_printf(args: &[Value]) -> Result<Value> {
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
pub fn builtin_number_format(args: &[Value]) -> Result<Value> {
    let num = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let decimals = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
    Ok(Value::String(format!(
        "{:.prec$}",
        num,
        prec = decimals
    )))
}

/// chr — 返回指定的字符
pub fn builtin_chr(args: &[Value]) -> Result<Value> {
    let code = args.first().map(|v| v.to_int()).unwrap_or(0) as u8;
    Ok(Value::String((code as char).to_string()))
}

/// ord — 返回字符的 ASCII 码值
pub fn builtin_ord(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Int(s.chars().next().map(|c| c as i64).unwrap_or(0)))
}

/// str_split — 将字符串转换为数组
pub fn builtin_str_split(args: &[Value]) -> Result<Value> {
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
pub fn builtin_ucfirst(args: &[Value]) -> Result<Value> {
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
pub fn builtin_lcfirst(args: &[Value]) -> Result<Value> {
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
pub fn builtin_ucwords(args: &[Value]) -> Result<Value> {
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
pub fn builtin_nl2br(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(s.replace("\n", "<br />\n")))
}

/// htmlspecialchars — 将特殊字符转换为 HTML 实体
pub fn builtin_htmlspecialchars(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;"),
    ))
}

/// strip_tags — 从字符串中去除 HTML 和 PHP 标记
pub fn builtin_strip_tags(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    Ok(Value::String(re.replace_all(&s, "").to_string()))
}
