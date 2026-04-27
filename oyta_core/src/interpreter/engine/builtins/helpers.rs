//! 辅助函数模块
//!
//! 包含一些辅助性的内置函数

use std::collections::HashMap;

use anyhow::Result;

use crate::interpreter::value::Value;

use super::types::BuiltinFunction;

/// 注册辅助函数到映射表
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_helper_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 变量打印
    map.insert("print_r".to_string(), builtin_print_r);
    map.insert("var_dump".to_string(), builtin_var_dump);
    map.insert("var_export".to_string(), builtin_var_export);
    // 调试
    map.insert("debug_zval_dump".to_string(), builtin_debug_zval_dump);
    // 内存
    map.insert("memory_get_usage".to_string(), builtin_memory_get_usage);
    map.insert("memory_get_peak_usage".to_string(), builtin_memory_get_peak_usage);
}

/// print_r — 以易于理解的格式打印变量
pub fn builtin_print_r(args: &[Value]) -> Result<Value> {
    let value = args.first().unwrap_or(&Value::Null);
    let return_value = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);

    // 格式化输出
    let output = format_value_recursive(value, 0);

    if return_value {
        Ok(Value::String(output))
    } else {
        println!("{}", output);
        Ok(Value::Bool(true))
    }
}

/// var_dump — 打印变量的相关信息
pub fn builtin_var_dump(args: &[Value]) -> Result<Value> {
    for value in args {
        let output = format_value_debug(value, 0);
        println!("{}", output);
    }
    Ok(Value::Null)
}

/// var_export — 输出或返回一个变量的字符串表示
pub fn builtin_var_export(args: &[Value]) -> Result<Value> {
    let value = args.first().unwrap_or(&Value::Null);
    let return_value = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);

    // 生成 PHP 代码格式的字符串
    let output = format_value_php(value, 0);

    if return_value {
        Ok(Value::String(output))
    } else {
        println!("{}", output);
        Ok(Value::Null)
    }
}

/// debug_zval_dump — 将内部 zval 结构转储为字符串
pub fn builtin_debug_zval_dump(args: &[Value]) -> Result<Value> {
    // 简化实现：与 var_dump 相同
    builtin_var_dump(args)
}

/// memory_get_usage — 返回分配给 PHP 的内存量
pub fn builtin_memory_get_usage(args: &[Value]) -> Result<Value> {
    let real_usage = args.first().map(|v| v.is_truthy()).unwrap_or(false);

    // 使用系统调用获取内存使用
    let memory = if real_usage {
        // 尝试获取实际内存使用
        get_process_memory()
    } else {
        // 返回模拟值
        1024 * 1024 * 10 // 10MB
    };

    Ok(Value::Int(memory as i64))
}

/// memory_get_peak_usage — 返回分配给 PHP 内存的峰值
pub fn builtin_memory_get_peak_usage(args: &[Value]) -> Result<Value> {
    let real_usage = args.first().map(|v| v.is_truthy()).unwrap_or(false);

    // 返回模拟值
    let memory = if real_usage {
        get_process_memory() * 2
    } else {
        1024 * 1024 * 20 // 20MB
    };

    Ok(Value::Int(memory as i64))
}

/// 递归格式化值
fn format_value_recursive(value: &Value, indent: usize) -> String {
    let indent_str = " ".repeat(indent);

    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => if *b { "1" } else { "" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::IndexedArray(arr) => {
            let mut result = format!("{}Array\n{}(\n", indent_str, indent_str);
            for (i, v) in arr.iter().enumerate() {
                result.push_str(&format!(
                    "{}    [{}] => {}\n",
                    indent_str,
                    i,
                    format_value_recursive(v, indent + 4)
                ));
            }
            result.push_str(&format!("{})\n", indent_str));
            result
        }
        Value::AssociativeArray(map) => {
            let mut result = format!("{}Array\n{}(\n", indent_str, indent_str);
            for (k, v) in map {
                result.push_str(&format!(
                    "{}    [{}] => {}\n",
                    indent_str,
                    k,
                    format_value_recursive(v, indent + 4)
                ));
            }
            result.push_str(&format!("{})\n", indent_str));
            result
        }
        Value::Object(obj) => {
            let mut result = format!("{}{} Object\n{}(\n", indent_str, obj.class_name, indent_str);
            for (k, v) in &obj.properties {
                result.push_str(&format!(
                    "{}    [{}] => {}\n",
                    indent_str,
                    k,
                    format_value_recursive(v, indent + 4)
                ));
            }
            result.push_str(&format!("{})\n", indent_str));
            result
        }
        Value::Callable(_) => format!("{}Closure", indent_str),
        // Resource 类型显示为资源描述
        Value::Resource(r) => format!("{}Resource({})", indent_str, r),
        // Generator 类型显示为生成器描述
        Value::Generator(_) => format!("{}Generator", indent_str),
        // Fiber 类型显示为协程描述
        Value::Fiber(_) => format!("{}Fiber", indent_str),
    }
}

/// 格式化值（调试格式）
fn format_value_debug(value: &Value, indent: usize) -> String {
    let indent_str = " ".repeat(indent);

    match value {
        Value::Null => format!("{}NULL\n", indent_str),
        Value::Bool(b) => format!("{}bool({})\n", indent_str, if *b { "true" } else { "false" }),
        Value::Int(i) => format!("{}int({})\n", indent_str, i),
        Value::Float(f) => format!("{}float({})\n", indent_str, f),
        Value::String(s) => format!("{}string({}) \"{}\"\n", indent_str, s.len(), s),
        Value::IndexedArray(arr) => {
            let mut result = format!("{}array({}) {{\n", indent_str, arr.len());
            for (i, v) in arr.iter().enumerate() {
                result.push_str(&format!(
                    "{}  [{}]=>\n{}",
                    indent_str,
                    i,
                    format_value_debug(v, indent + 2)
                ));
            }
            result.push_str(&format!("{}}}\n", indent_str));
            result
        }
        Value::AssociativeArray(map) => {
            let mut result = format!("{}array({}) {{\n", indent_str, map.len());
            for (k, v) in map {
                result.push_str(&format!(
                    "{}  [\"{}\"]=>\n{}",
                    indent_str,
                    k,
                    format_value_debug(v, indent + 2)
                ));
            }
            result.push_str(&format!("{}}}\n", indent_str));
            result
        }
        Value::Object(obj) => {
            let mut result = format!("{}object({})#1 ({}) {{\n", indent_str, obj.class_name, obj.properties.len());
            for (k, v) in &obj.properties {
                result.push_str(&format!(
                    "{}  [\"{}\"]=>\n{}",
                    indent_str,
                    k,
                    format_value_debug(v, indent + 2)
                ));
            }
            result.push_str(&format!("{}}}\n", indent_str));
            result
        }
        Value::Callable(_) => format!("{}object(Closure)#1 (0) {{\n{}}}\n", indent_str, indent_str),
        // Resource 类型显示为资源描述
        Value::Resource(r) => format!("{}resource({})\n", indent_str, r),
        // Generator 类型显示为生成器描述
        Value::Generator(_) => format!("{}object(Generator)#1 (0) {{\n{}}}\n", indent_str, indent_str),
        // Fiber 类型显示为协程描述
        Value::Fiber(_) => format!("{}object(Fiber)#1 (0) {{\n{}}}\n", indent_str, indent_str),
    }
}

/// 格式化值（PHP 代码格式）
fn format_value_php(value: &Value, indent: usize) -> String {
    let indent_str = " ".repeat(indent);

    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
        Value::IndexedArray(arr) => {
            let mut result = "array(\n".to_string();
            for v in arr {
                result.push_str(&format!(
                    "{}  {},\n",
                    indent_str,
                    format_value_php(v, indent + 2)
                ));
            }
            result.push_str(&format!("{})", indent_str));
            result
        }
        Value::AssociativeArray(map) => {
            let mut result = "array(\n".to_string();
            for (k, v) in map {
                result.push_str(&format!(
                    "{}  '{}' => {},\n",
                    indent_str,
                    k.replace('\'', "\\'"),
                    format_value_php(v, indent + 2)
                ));
            }
            result.push_str(&format!("{})", indent_str));
            result
        }
        Value::Object(_) => "/* object */".to_string(),
        Value::Callable(_) => "/* closure */".to_string(),
        // Resource 类型显示为资源描述
        Value::Resource(r) => format!("/* resource: {} */", r),
        // Generator 类型显示为生成器描述
        Value::Generator(_) => "/* generator */".to_string(),
        // Fiber 类型显示为协程描述
        Value::Fiber(_) => "/* fiber */".to_string(),
    }
}

/// 获取进程内存使用（Linux）
#[cfg(target_os = "linux")]
fn get_process_memory() -> usize {
    use std::fs;

    // 读取 /proc/self/status
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // VmRSS: 12345 kB
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }

    0
}

#[cfg(not(target_os = "linux"))]
fn get_process_memory() -> usize {
    0
}
