//! 数学函数模块
//!
//! 包含 abs、max、min、floor、ceil、round 等数学函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// abs — 绝对值
pub fn builtin_abs(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Int(i)) => Ok(Value::Int(i.abs())),
        Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
        _ => Ok(Value::Int(0)),
    }
}

/// max — 找出最大值
pub fn builtin_max(args: &[Value]) -> Result<Value> {
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
pub fn builtin_min(args: &[Value]) -> Result<Value> {
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
pub fn builtin_floor(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float().floor()).unwrap_or(0.0),
    ))
}

/// ceil — 向上取整
pub fn builtin_ceil(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float().ceil()).unwrap_or(0.0),
    ))
}

/// round — 对浮点数进行四舍五入
/// round(float $val, int $precision = 0): float
pub fn builtin_round(args: &[Value]) -> Result<Value> {
    let v = args.first().map(|v| v.to_float()).unwrap_or(0.0);
    let precision = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let factor = 10f64.powi(precision as i32);
    Ok(Value::Float((v * factor).round() / factor))
}
