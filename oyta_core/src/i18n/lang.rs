//! 语言包管理

use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;

/// 语言包存储
static LANG_PACKS: Lazy<RwLock<HashMap<String, DashMap<String, String>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// 当前语言
static CURRENT_LANG: Lazy<RwLock<String>> = Lazy::new(|| {
    RwLock::new("zh-cn".to_string())
});

/// 加载语言包
pub fn load(lang: &str, pack_name: &str, translations: HashMap<String, String>) {
    let mut packs = LANG_PACKS.write();
    let lang_packs = packs.entry(lang.to_string()).or_insert_with(DashMap::new);
    for (key, value) in translations {
        let full_key = format!("{}.{}", pack_name, key);
        lang_packs.insert(full_key, value);
    }
}

/// 获取翻译
pub fn trans(key: &str, lang: Option<&str>) -> String {
    let lang_str;
    let lang = match lang {
        Some(l) => l,
        None => {
            let current = CURRENT_LANG.read();
            lang_str = current.clone();
            &lang_str
        }
    };

    let packs = LANG_PACKS.read();
    if let Some(lang_packs) = packs.get(lang) {
        lang_packs.get(key).map(|v| v.value().clone()).unwrap_or_else(|| key.to_string())
    } else {
        key.to_string()
    }
}

/// 设置当前语言
pub fn set_locale(lang: &str) {
    let mut current = CURRENT_LANG.write();
    *current = lang.to_string();
}

/// 获取当前语言
pub fn get_locale() -> String {
    let current = CURRENT_LANG.read();
    current.clone()
}
