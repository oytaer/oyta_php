//! 国际化门面

use super::lang;

/// Lang 门面
pub struct Lang;

impl Lang {
    /// 获取翻译
    pub fn get(key: &str) -> String {
        lang::trans(key, None)
    }

    /// 获取翻译（指定语言）
    pub fn get_with_lang(key: &str, lang: &str) -> String {
        lang::trans(key, Some(lang))
    }

    /// 设置当前语言
    pub fn set_locale(lang: &str) {
        lang::set_locale(lang);
    }

    /// 获取当前语言
    pub fn get_locale() -> String {
        lang::get_locale()
    }
}
