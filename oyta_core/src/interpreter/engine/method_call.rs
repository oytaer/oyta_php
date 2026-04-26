//! 字符串和数组方法调用模块
//!
//! 实现 PHP 中通过 -> 运算符对字符串和数组调用的链式方法
//! 这些不是真正的 PHP 语法，而是 OYTAPHP 扩展的便捷方法
//! 同时也支持 PHP 原生的字符串/数组函数式调用

use anyhow::Result;

use crate::interpreter::value::Value;

use super::Interpreter;
use super::binary_op::values_equal;

impl Interpreter {
    /// 调用字符串方法
    ///
    /// 当对字符串值使用 -> 运算符时调用
    /// 支持的方法：
    /// - length/strlen: 获取长度
    /// - toLowercase/strtolower: 转小写
    /// - toUppercase/strtoupper: 转大写
    /// - trim: 去除首尾空白
    /// - contains: 是否包含子串
    /// - startsWith: 是否以指定前缀开头
    /// - endsWith: 是否以指定后缀结尾
    /// - substring/substr: 截取子串
    /// - replace: 替换子串
    /// - split: 分割字符串
    /// - indexOf: 查找子串位置
    ///
    /// # 参数
    /// - `s`: 字符串值
    /// - `method`: 方法名
    /// - `args`: 方法参数列表
    pub(crate) fn call_string_method(
        &mut self,
        s: &str,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value> {
        match method {
            /// 获取字符串长度
            "length" | "strlen" => Ok(Value::Int(s.len() as i64)),
            /// 转换为小写
            "toLowerCase" | "strtolower" => Ok(Value::String(s.to_lowercase())),
            /// 转换为大写
            "toUpperCase" | "strtoupper" => Ok(Value::String(s.to_uppercase())),
            /// 去除首尾空白
            "trim" => Ok(Value::String(s.trim().to_string())),
            /// 判断是否包含子串
            "contains" => {
                if let Some(Value::String(needle)) = args.first() {
                    Ok(Value::Bool(s.contains(needle.as_str())))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            /// 判断是否以指定前缀开头
            "startsWith" => {
                if let Some(Value::String(prefix)) = args.first() {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            /// 判断是否以指定后缀结尾
            "endsWith" => {
                if let Some(Value::String(suffix)) = args.first() {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            /// 截取子串
            "substring" | "substr" => {
                let start = args.first().map(|v| v.to_int()).unwrap_or(0) as usize;
                let len = args.get(1).map(|v| v.to_int()).unwrap_or(-1);
                if len < 0 {
                    Ok(Value::String(s.chars().skip(start).collect()))
                } else {
                    Ok(Value::String(
                        s.chars().skip(start).take(len as usize).collect(),
                    ))
                }
            }
            /// 替换子串
            "replace" => {
                let search = args
                    .first()
                    .map(|v| v.to_string_value())
                    .unwrap_or_default();
                let replace = args
                    .get(1)
                    .map(|v| v.to_string_value())
                    .unwrap_or_default();
                Ok(Value::String(s.replace(&search, &replace)))
            }
            /// 分割字符串
            "split" => {
                let delim = args
                    .first()
                    .map(|v| v.to_string_value())
                    .unwrap_or_default();
                Ok(Value::IndexedArray(
                    s.split(&delim)
                        .map(|p| Value::String(p.to_string()))
                        .collect(),
                ))
            }
            /// 查找子串位置
            "indexOf" => {
                let needle = args
                    .first()
                    .map(|v| v.to_string_value())
                    .unwrap_or_default();
                Ok(Value::Int(
                    s.find(&needle).map(|i| i as i64).unwrap_or(-1),
                ))
            }
            /// 未知方法返回 Null
            _ => Ok(Value::Null),
        }
    }

    /// 调用数组方法
    ///
    /// 当对数组值使用 -> 运算符时调用
    /// 支持的方法：
    /// - push: 添加元素到末尾
    /// - pop: 弹出最后一个元素
    /// - count/length/size: 获取元素数量
    /// - keys: 获取所有键名
    /// - values: 获取所有值
    /// - has/contains: 判断是否包含指定值
    /// - merge: 合并数组
    /// - map/filter: 映射/过滤（简化实现）
    /// - join/implode: 用分隔符连接为字符串
    /// - slice: 截取子数组
    /// - reverse: 反转数组
    /// - unique: 去重
    /// - toJson/json: 转换为 JSON 字符串
    ///
    /// # 参数
    /// - `arr`: 数组值
    /// - `method`: 方法名
    /// - `args`: 方法参数列表
    pub(crate) fn call_array_method(
        &mut self,
        arr: &Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value> {
        match method {
            /// 添加元素到数组末尾
            "push" => {
                if let Value::IndexedArray(ref mut items) = arr.clone() {
                    for arg in args {
                        items.push(arg);
                    }
                    Ok(Value::Int(items.len() as i64))
                } else {
                    Ok(Value::Null)
                }
            }
            /// 弹出数组最后一个元素
            "pop" => {
                if let Value::IndexedArray(ref mut items) = arr.clone() {
                    Ok(items.pop().unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            /// 获取数组元素数量
            "count" | "length" | "size" => Ok(Value::Int(arr.count() as i64)),
            /// 获取所有键名
            "keys" => match arr {
                Value::AssociativeArray(map) => Ok(Value::IndexedArray(
                    map.iter().map(|(k, _)| Value::String(k.clone())).collect(),
                )),
                Value::IndexedArray(items) => Ok(Value::IndexedArray(
                    (0..items.len()).map(|i| Value::Int(i as i64)).collect(),
                )),
                _ => Ok(Value::IndexedArray(Vec::new())),
            },
            /// 获取所有值
            "values" => match arr {
                Value::IndexedArray(items) => Ok(Value::IndexedArray(items.clone())),
                Value::AssociativeArray(map) => Ok(Value::IndexedArray(
                    map.iter().map(|(_, v)| v.clone()).collect(),
                )),
                _ => Ok(Value::IndexedArray(Vec::new())),
            },
            /// 判断数组是否包含指定值
            "has" | "contains" => {
                if let Some(needle) = args.first() {
                    match arr {
                        Value::IndexedArray(items) => {
                            Ok(Value::Bool(items.iter().any(|v| values_equal(v, needle))))
                        }
                        Value::AssociativeArray(map) => Ok(Value::Bool(
                            map.iter().any(|(_, v)| values_equal(v, needle)),
                        )),
                        _ => Ok(Value::Bool(false)),
                    }
                } else {
                    Ok(Value::Bool(false))
                }
            }
            /// 合并数组
            "merge" => match arr {
                Value::IndexedArray(items) => {
                    let mut result = items.clone();
                    for arg in &args {
                        if let Value::IndexedArray(other) = arg {
                            result.extend(other.iter().cloned());
                        }
                    }
                    Ok(Value::IndexedArray(result))
                }
                Value::AssociativeArray(map) => {
                    let mut result = map.clone();
                    for arg in &args {
                        if let Value::AssociativeArray(other) = arg {
                            for (k, v) in other {
                                if let Some(entry) = result.iter_mut().find(|(key, _)| key == k) {
                                    entry.1 = v.clone();
                                } else {
                                    result.push((k.clone(), v.clone()));
                                }
                            }
                        }
                    }
                    Ok(Value::AssociativeArray(result))
                }
                _ => Ok(arr.clone()),
            },
            /// 映射（简化实现，返回原数组）
            "map" => Ok(arr.clone()),
            /// 过滤（简化实现，返回原数组）
            "filter" => Ok(arr.clone()),
            /// 用分隔符连接为字符串
            "join" | "implode" => {
                let glue = args
                    .first()
                    .map(|v| v.to_string_value())
                    .unwrap_or_default();
                match arr {
                    Value::IndexedArray(items) => Ok(Value::String(
                        items
                            .iter()
                            .map(|v| v.to_string_value())
                            .collect::<Vec<_>>()
                            .join(&glue),
                    )),
                    _ => Ok(Value::String(String::new())),
                }
            }
            /// 截取子数组
            "slice" => {
                let offset = args.first().map(|v| v.to_int()).unwrap_or(0) as usize;
                let length = args.get(1).map(|v| v.to_int()).unwrap_or(-1);
                match arr {
                    Value::IndexedArray(items) => {
                        if length < 0 {
                            Ok(Value::IndexedArray(
                                items.iter().skip(offset).cloned().collect(),
                            ))
                        } else {
                            Ok(Value::IndexedArray(
                                items
                                    .iter()
                                    .skip(offset)
                                    .take(length as usize)
                                    .cloned()
                                    .collect(),
                            ))
                        }
                    }
                    _ => Ok(Value::IndexedArray(Vec::new())),
                }
            }
            /// 反转数组
            "reverse" => match arr {
                Value::IndexedArray(items) => {
                    Ok(Value::IndexedArray(items.iter().rev().cloned().collect()))
                }
                _ => Ok(arr.clone()),
            },
            /// 去重
            "unique" => match arr {
                Value::IndexedArray(items) => {
                    let mut seen = Vec::new();
                    let mut result = Vec::new();
                    for item in items {
                        if !seen.iter().any(|v: &Value| values_equal(v, item)) {
                            seen.push(item.clone());
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::IndexedArray(result))
                }
                _ => Ok(arr.clone()),
            },
            /// 转换为 JSON 字符串
            "toJson" | "json" => Ok(Value::String(arr.to_json_string())),
            /// 未知方法返回 Null
            _ => Ok(Value::Null),
        }
    }
}
