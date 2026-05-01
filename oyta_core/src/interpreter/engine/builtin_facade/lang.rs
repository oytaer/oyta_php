//! Lang 门面类方法实现
//!
//! 提供语言操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Lang::get 方法实现
pub fn lang_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let value = crate::i18n::Lang::get(&key, None);
    Ok(Value::String(value))
}

/// Lang::set 方法实现
pub fn lang_set(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Lang::has 方法实现
pub fn lang_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(crate::i18n::Lang::has(&key)))
}

/// Lang::locale 方法实现
pub fn lang_locale(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::i18n::Lang::get_locale()))
}

/// Lang::setLocale 方法实现
pub fn lang_set_locale(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let locale = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    crate::i18n::Lang::set_locale(&locale);
    Ok(Value::Bool(true))
}

/// 获取所有 Lang 门面方法
pub fn get_lang_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("get", lang_get),
        ("set", lang_set),
        ("has", lang_has),
        ("locale", lang_locale),
        ("setLocale", lang_set_locale),
    ]
}
