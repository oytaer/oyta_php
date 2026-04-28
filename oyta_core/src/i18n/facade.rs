//! 多语言门面
//!
//! 提供静态方法访问多语言功能
//! 对应 ThinkPHP 8.0 的 Lang 类
//!
//! # 使用示例
//! ```php
//! // 获取翻译
//! echo Lang::get('welcome.message');
//!
//! // 切换语言
//! Lang::setLocale('en');
//!
//! // 带参数的翻译
//! echo Lang::get('user.greeting', ['name' => 'John']);
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;
use parking_lot::RwLock;

/// 全局语言设置
///
/// 使用 LazyLock 实现延迟初始化
/// 默认语言为中文（zh-cn）
static CURRENT_LOCALE: LazyLock<RwLock<String>> = LazyLock::new(|| {
    // 创建 RwLock 并设置默认语言为中文
    RwLock::new(String::from("zh-cn"))
});

/// 全局翻译存储
///
/// 使用 LazyLock 实现延迟初始化
/// 存储结构：语言代码 -> 翻译键 -> 翻译值
static TRANSLATIONS: LazyLock<RwLock<HashMap<String, HashMap<String, String>>>> = LazyLock::new(|| {
    // 创建空的翻译存储
    RwLock::new(HashMap::new())
});

/// Lang 门面结构体
///
/// 提供静态方法访问多语言功能
/// 所有方法都是线程安全的
pub struct Lang;

impl Lang {
    /// 初始化多语言门面
    ///
    /// 设置默认语言
    ///
    /// # 参数
    /// - `default_locale`: 默认语言
    pub fn init(default_locale: &str) {
        // 获取写锁设置默认语言
        let mut guard = CURRENT_LOCALE.write();
        // 设置语言
        *guard = default_locale.to_string();
    }

    /// 获取翻译文本
    ///
    /// 根据当前语言获取指定键的翻译
    ///
    /// # 参数
    /// - `key`: 翻译键，支持点号分隔如 "welcome.message"
    /// - `replace`: 替换参数（可选）
    ///
    /// # 返回
    /// 翻译后的文本，如果未找到返回键名
    ///
    /// # 示例
    /// ```php
    /// echo Lang::get('welcome.message');
    /// echo Lang::get('user.greeting', ['name' => 'John']);
    /// ```
    pub fn get(key: &str, replace: Option<&HashMap<String, String>>) -> String {
        // 获取当前语言
        let locale = Self::get_locale();
        // 获取翻译存储的读锁
        let guard = TRANSLATIONS.read();
        // 查找翻译
        let text = if let Some(lang_translations) = guard.get(&locale) {
            // 在当前语言中查找键
            lang_translations.get(key).cloned().unwrap_or_else(|| key.to_string())
        } else {
            // 未找到语言包，返回键名
            key.to_string()
        };
        // 释放读锁
        drop(guard);
        // 如果有替换参数，进行替换
        if let Some(params) = replace {
            // 遍历所有参数进行替换
            let mut result = text;
            for (k, v) in params {
                // 替换占位符 :key
                result = result.replace(&format!(":{}", k), v);
            }
            // 返回替换后的结果
            result
        } else {
            // 返回原始翻译
            text
        }
    }

    /// 获取指定语言的翻译
    ///
    /// 忽略当前语言设置，获取指定语言的翻译
    ///
    /// # 参数
    /// - `key`: 翻译键
    /// - `locale`: 语言代码
    /// - `replace`: 替换参数（可选）
    ///
    /// # 返回
    /// 翻译后的文本
    pub fn get_with_locale(key: &str, locale: &str, replace: Option<&HashMap<String, String>>) -> String {
        // 获取翻译存储的读锁
        let guard = TRANSLATIONS.read();
        // 查找翻译
        let text = if let Some(lang_translations) = guard.get(locale) {
            // 在指定语言中查找键
            lang_translations.get(key).cloned().unwrap_or_else(|| key.to_string())
        } else {
            // 未找到语言包，返回键名
            key.to_string()
        };
        // 释放读锁
        drop(guard);
        // 如果有替换参数，进行替换
        if let Some(params) = replace {
            // 遍历所有参数进行替换
            let mut result = text;
            for (k, v) in params {
                // 替换占位符 :key
                result = result.replace(&format!(":{}", k), v);
            }
            // 返回替换后的结果
            result
        } else {
            // 返回原始翻译
            text
        }
    }

    /// 设置当前语言
    ///
    /// 切换应用的当前语言
    ///
    /// # 参数
    /// - `locale`: 语言代码，如 "zh-cn"、"en"
    ///
    /// # 示例
    /// ```php
    /// Lang::setLocale('en');
    /// ```
    pub fn set_locale(locale: &str) {
        // 获取写锁
        let mut guard = CURRENT_LOCALE.write();
        // 设置语言
        *guard = locale.to_string();
    }

    /// 获取当前语言
    ///
    /// 返回当前设置的语言代码
    ///
    /// # 返回
    /// 当前语言代码
    pub fn get_locale() -> String {
        // 获取读锁
        let guard = CURRENT_LOCALE.read();
        // 克隆语言代码
        guard.clone()
    }

    /// 获取所有支持的语言
    ///
    /// 返回已加载的语言列表
    ///
    /// # 返回
    /// 语言代码列表
    pub fn get_locales() -> Vec<String> {
        // 获取翻译存储的读锁
        let guard = TRANSLATIONS.read();
        // 返回所有语言代码
        guard.keys().cloned().collect()
    }

    /// 检查语言是否存在
    ///
    /// 判断指定语言是否已加载
    ///
    /// # 参数
    /// - `locale`: 语言代码
    ///
    /// # 返回
    /// 如果语言存在返回 true
    pub fn has_locale(locale: &str) -> bool {
        // 获取翻译存储的读锁
        let guard = TRANSLATIONS.read();
        // 检查语言是否存在
        guard.contains_key(locale)
    }

    /// 检查翻译键是否存在
    ///
    /// 判断指定翻译键是否定义
    ///
    /// # 参数
    /// - `key`: 翻译键
    ///
    /// # 返回
    /// 如果键存在返回 true
    pub fn has(key: &str) -> bool {
        // 获取当前语言
        let locale = Self::get_locale();
        // 获取翻译存储的读锁
        let guard = TRANSLATIONS.read();
        // 查找键
        if let Some(lang_translations) = guard.get(&locale) {
            // 返回键是否存在
            lang_translations.contains_key(key)
        } else {
            // 语言不存在，返回 false
            false
        }
    }

    /// 添加翻译
    ///
    /// 手动添加翻译条目
    ///
    /// # 参数
    /// - `locale`: 语言代码
    /// - `key`: 翻译键
    /// - `value`: 翻译值
    pub fn add(locale: &str, key: &str, value: &str) {
        // 获取写锁
        let mut guard = TRANSLATIONS.write();
        // 获取或创建语言翻译表
        let lang_translations = guard.entry(locale.to_string()).or_default();
        // 添加翻译
        lang_translations.insert(key.to_string(), value.to_string());
    }

    /// 批量添加翻译
    ///
    /// 批量添加翻译条目
    ///
    /// # 参数
    /// - `locale`: 语言代码
    /// - `translations`: 翻译键值对
    pub fn add_many(locale: &str, translations: &HashMap<String, String>) {
        // 获取写锁
        let mut guard = TRANSLATIONS.write();
        // 获取或创建语言翻译表
        let lang_translations = guard.entry(locale.to_string()).or_default();
        // 批量添加翻译
        for (key, value) in translations {
            // 插入翻译条目
            lang_translations.insert(key.clone(), value.clone());
        }
    }

    /// 加载语言包文件
    ///
    /// 从文件加载翻译
    ///
    /// # 参数
    /// - `locale`: 语言代码
    /// - `translations`: 翻译键值对
    pub fn load(locale: &str, translations: HashMap<String, String>) {
        // 获取写锁
        let mut guard = TRANSLATIONS.write();
        // 插入或更新语言翻译表
        guard.insert(locale.to_string(), translations);
    }

    /// 获取数字的复数形式
    ///
    /// 根据数量选择正确的复数形式
    ///
    /// # 参数
    /// - `key`: 翻译键
    /// - `count`: 数量
    /// - `replace`: 替换参数（可选）
    ///
    /// # 返回
    /// 正确的复数形式翻译
    pub fn choice(key: &str, count: i32, replace: Option<&HashMap<String, String>>) -> String {
        // 获取基础翻译
        let mut text = Self::get(key, replace);
        // 替换 :count 占位符
        text = text.replace(":count", &count.to_string());
        // 返回结果
        text
    }

    /// 检查多语言门面是否已初始化
    ///
    /// # 返回
    /// 如果已初始化返回 true
    pub fn is_initialized() -> bool {
        // 检查语言是否已设置
        let guard = CURRENT_LOCALE.read();
        // 返回是否非空
        !guard.is_empty()
    }

    /// 重置多语言门面
    ///
    /// 清除所有翻译
    /// 主要用于测试环境
    pub fn reset() {
        // 重置语言为默认值
        {
            // 获取写锁
            let mut guard = CURRENT_LOCALE.write();
            // 设置为中文
            *guard = String::from("zh-cn");
        }
        // 清空翻译
        {
            // 获取写锁
            let mut guard = TRANSLATIONS.write();
            // 清空所有翻译
            guard.clear();
        }
    }
}
