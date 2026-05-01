//! Model 内置类模块
//!
//! 实现 PHP Model 基类 - ThinkPHP 8.0 ORM 模型基类
//! PHP 代码可以通过 class User extends Model 来继承此基类

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

// ============================================================================
// Model 静态方法实现
// ============================================================================

/// Model::find 方法实现
///
/// 根据主键查询单条记录
///
/// # 参数
/// - `instance`: 模型实例（包含类名）
/// - `args`: 主键值参数
///
/// # 返回
/// 模型实例或 null
pub fn model_find(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取主键值
    let id = args.first().cloned().unwrap_or(Value::Null);
    // 获取表名（从实例的 class_name 推断）
    let table = get_model_table_from_instance(instance);
    // 获取主键字段名
    let pk = instance.properties.get("pk")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "id".to_string());
    
    // 构建查询 SQL
    let sql = format!("SELECT * FROM {} WHERE {} = ?", table, pk);
    
    // 执行查询
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::query_with_params(&sql, &[id]).await
    });
    
    match result {
        Ok(query_result) if !query_result.rows.is_empty() => {
            // 创建模型实例并设置数据
            let mut model_instance = create_model_instance(instance, query_result.rows.into_iter().next().unwrap());
            model_instance.properties.insert("exists".to_string(), Value::Bool(true));
            Ok(model_instance_to_value(model_instance))
        }
        _ => Ok(Value::Null),
    }
}

/// Model::findOrFail 方法实现
///
/// 根据主键查询，不存在则抛出异常
///
/// # 参数
/// - `instance`: 模型实例
/// - `args`: 主键值参数
///
/// # 返回
/// 模型实例
pub fn model_find_or_fail(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let result = model_find(instance, args)?;
    if result == Value::Null {
        anyhow::bail!("模型未找到");
    }
    Ok(result)
}

/// Model::first 方法实现
///
/// 查询第一条记录
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 模型实例或 null
pub fn model_first(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    
    let sql = format!("SELECT * FROM {} LIMIT 1", table);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::query(&sql).await
    });
    
    match result {
        Ok(query_result) if !query_result.rows.is_empty() => {
            let mut model_instance = create_model_instance(instance, query_result.rows.into_iter().next().unwrap());
            model_instance.properties.insert("exists".to_string(), Value::Bool(true));
            Ok(model_instance_to_value(model_instance))
        }
        _ => Ok(Value::Null),
    }
}

/// Model::all 方法实现
///
/// 查询所有记录
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 模型集合
pub fn model_all(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    
    let sql = format!("SELECT * FROM {}", table);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::query(&sql).await
    });
    
    match result {
        Ok(query_result) => {
            let models: Vec<Value> = query_result.rows.into_iter().map(|data| {
                let mut model_instance = create_model_instance(instance, data);
                model_instance.properties.insert("exists".to_string(), Value::Bool(true));
                model_instance_to_value(model_instance)
            }).collect();
            Ok(Value::IndexedArray(models))
        }
        _ => Ok(Value::IndexedArray(vec![])),
    }
}

/// Model::create 方法实现
///
/// 创建新记录
///
/// # 参数
/// - `instance`: 模型实例
/// - `args`: 数据参数
///
/// # 返回
/// 新创建的模型实例
pub fn model_create(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    let data = args.first().cloned();
    
    let mut insert_data = HashMap::new();
    if let Some(Value::AssociativeArray(items)) = data {
        for (key, value) in items {
            insert_data.insert(key, value);
        }
    }
    
    // 构建 INSERT SQL
    let fields: Vec<&str> = insert_data.keys().map(|s| s.as_str()).collect();
    let placeholders: Vec<&str> = fields.iter().map(|_| "?").collect();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        fields.join(", "),
        placeholders.join(", ")
    );
    
    let values: Vec<Value> = insert_data.values().cloned().collect();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(async {
        crate::database::Db::execute_with_params(&sql, &values).await
    });
    
    let mut model_instance = create_model_instance(instance, insert_data);
    model_instance.properties.insert("exists".to_string(), Value::Bool(true));
    Ok(model_instance_to_value(model_instance))
}

/// Model::count 方法实现
///
/// 统计数量
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 数量
pub fn model_count(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    
    let sql = format!("SELECT COUNT(*) as count FROM {}", table);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::query(&sql).await
    });
    
    match result {
        Ok(query_result) if !query_result.rows.is_empty() => {
            if let Some(Value::Int(count)) = query_result.rows.into_iter().next().unwrap().get("count") {
                Ok(Value::Int(*count))
            } else {
                Ok(Value::Int(0))
            }
        }
        _ => Ok(Value::Int(0)),
    }
}

/// Model::where 方法实现
///
/// 条件查询（返回查询构建器）
///
/// # 参数
/// - `instance`: 模型实例
/// - `args`: 条件参数
///
/// # 返回
/// 查询构建器
pub fn model_where(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    
    // 返回一个查询构建器对象
    Ok(Value::AssociativeArray(vec![
        ("__class".to_string(), Value::String(instance.class_name.clone())),
        ("__type".to_string(), Value::String("ModelQueryBuilder".to_string())),
        ("__table".to_string(), Value::String(table)),
        ("__method".to_string(), Value::String("where".to_string())),
        ("__args".to_string(), Value::IndexedArray(args.to_vec())),
    ]))
}

/// Model::paginate 方法实现
///
/// 分页查询
///
/// # 参数
/// - `instance`: 模型实例
/// - `args`: 页码和每页数量参数
///
/// # 返回
/// 分页结果
pub fn model_paginate(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    let page = args.first().map(|v| v.to_int()).unwrap_or(1) as u64;
    let per_page = args.get(1).map(|v| v.to_int()).unwrap_or(15) as u64;
    
    let offset = (page - 1) * per_page;
    
    // 查询总数
    let count_sql = format!("SELECT COUNT(*) as count FROM {}", table);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let count_result = rt.block_on(async {
        crate::database::Db::query(&count_sql).await
    });
    
    let total = match count_result {
        Ok(query_result) if !query_result.rows.is_empty() => {
            if let Some(Value::Int(count)) = query_result.rows.into_iter().next().unwrap().get("count") {
                *count as u64
            } else {
                0
            }
        }
        _ => 0,
    };
    
    // 查询数据
    let data_sql = format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, per_page, offset);
    let data_result = rt.block_on(async {
        crate::database::Db::query(&data_sql).await
    });
    
    let models = match data_result {
        Ok(query_result) => {
            query_result.rows.into_iter().map(|data| {
                let mut model_instance = create_model_instance(instance, data);
                model_instance.properties.insert("exists".to_string(), Value::Bool(true));
                model_instance_to_value(model_instance)
            }).collect()
        }
        _ => vec![],
    };
    
    let last_page = if total > 0 { (total + per_page - 1) / per_page } else { 1 };
    
    Ok(Value::AssociativeArray(vec![
        ("current_page".to_string(), Value::Int(page as i64)),
        ("per_page".to_string(), Value::Int(per_page as i64)),
        ("total".to_string(), Value::Int(total as i64)),
        ("last_page".to_string(), Value::Int(last_page as i64)),
        ("data".to_string(), Value::IndexedArray(models)),
    ]))
}

// ============================================================================
// Model 实例方法实现
// ============================================================================

/// $model->save() 方法实现
///
/// 保存模型
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 是否成功
pub fn model_instance_save(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    let exists = instance.properties.get("exists")
        .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
        .unwrap_or(false);
    
    // 收集数据
    let mut data = HashMap::new();
    for (key, value) in &instance.properties {
        if !key.starts_with("__") && key != "exists" && key != "table" && key != "pk" && key != "connection" {
            data.insert(key.clone(), value.clone());
        }
    }
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    if exists {
        // 更新
        let pk = instance.properties.get("pk")
            .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "id".to_string());
        
        if let Some(Value::Int(id)) = instance.properties.get(&pk) {
            let set_clause: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();
            let sql = format!("UPDATE {} SET {} WHERE {} = ?", table, set_clause.join(", "), pk);
            
            let mut values: Vec<Value> = data.values().cloned().collect();
            values.push(Value::Int(*id));
            
            let _ = rt.block_on(async {
                crate::database::Db::execute_with_params(&sql, &values).await
            });
        }
    } else {
        // 插入
        let fields: Vec<&str> = data.keys().map(|s| s.as_str()).collect();
        let placeholders: Vec<&str> = fields.iter().map(|_| "?").collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            fields.join(", "),
            placeholders.join(", ")
        );
        
        let values: Vec<Value> = data.values().cloned().collect();
        let _ = rt.block_on(async {
            crate::database::Db::execute_with_params(&sql, &values).await
        });
    }
    
    Ok(Value::Bool(true))
}

/// $model->delete() 方法实现
///
/// 删除模型
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 是否成功
pub fn model_instance_delete(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let table = get_model_table_from_instance(instance);
    let pk = instance.properties.get("pk")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "id".to_string());
    
    if let Some(Value::Int(id)) = instance.properties.get(&pk) {
        let sql = format!("DELETE FROM {} WHERE {} = ?", table, pk);
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async {
            crate::database::Db::execute_with_params(&sql, &[Value::Int(*id)]).await
        });
    }
    
    Ok(Value::Bool(true))
}

/// $model->toArray() 方法实现
///
/// 转换为数组
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// 数组
pub fn model_instance_to_array(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut result = Vec::new();
    for (key, value) in &instance.properties {
        if !key.starts_with("__") && key != "exists" {
            result.push((key.clone(), value.clone()));
        }
    }
    Ok(Value::AssociativeArray(result))
}

/// $model->toJson() 方法实现
///
/// 转换为 JSON
///
/// # 参数
/// - `instance`: 模型实例
///
/// # 返回
/// JSON 字符串
pub fn model_instance_to_json(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut result = serde_json::Map::new();
    for (key, value) in &instance.properties {
        if !key.starts_with("__") && key != "exists" {
            result.insert(key.clone(), value_to_json(value));
        }
    }
    Ok(Value::String(serde_json::to_string(&result).unwrap_or_default()))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 从实例获取模型表名
fn get_model_table_from_instance(instance: &ObjectInstance) -> String {
    // 首先检查是否设置了 table 属性
    if let Some(Value::String(table)) = instance.properties.get("table") {
        if !table.is_empty() {
            return table.clone();
        }
    }
    
    // 否则从类名推断表名（驼峰转下划线）
    let class_name = &instance.class_name;
    let mut table_name = String::new();
    for (i, c) in class_name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                table_name.push('_');
            }
            table_name.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            table_name.push(c);
        }
    }
    table_name
}

/// 创建模型实例
fn create_model_instance(template: &ObjectInstance, data: HashMap<String, Value>) -> ObjectInstance {
    let mut properties = template.properties.clone();
    
    // 将数据添加到属性
    for (key, value) in data {
        properties.insert(key, value);
    }
    
    ObjectInstance {
        class_name: template.class_name.clone(),
        properties,
    }
}

/// 将模型实例转换为 Value
fn model_instance_to_value(instance: ObjectInstance) -> Value {
    let mut data = Vec::new();
    for (key, value) in &instance.properties {
        if !key.starts_with("__") {
            data.push((key.clone(), value.clone()));
        }
    }
    data.push(("__class".to_string(), Value::String(instance.class_name.clone())));
    data.push(("__instance".to_string(), Value::Bool(true)));
    Value::AssociativeArray(data)
}

/// 将 Value 转换为 JSON 值
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::IndexedArray(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::AssociativeArray(obj) => {
            serde_json::Value::Object(obj.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect())
        }
        _ => serde_json::Value::Null,
    }
}
