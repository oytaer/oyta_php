//! DateTimeZone 时区类实现
//!
//! 提供完整的 PHP DateTimeZone 功能
//! 支持：时区创建、时区列表、时区转换等

use chrono::{Datelike, FixedOffset, TimeZone, Utc};
use chrono_tz::{Tz, TZ_VARIANTS, OffsetComponents};
use serde::{Deserialize, Serialize};
use std::fmt;

/// DateTimeZone 值类型
///
/// 表示一个时区对象
/// 对应 PHP 的 DateTimeZone 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimeZoneValue {
    /// 时区类型
    pub(crate) timezone_type: TimezoneType,
    /// 时区标识符
    /// - 对于 IANA 时区：如 "Asia/Shanghai"
    /// - 对于偏移量：如 "+08:00"
    /// - 对于缩写：如 "CST"
    pub(crate) identifier: String,
    /// 内部的 chrono-tz 时区
    #[serde(skip)]
    pub(crate) tz: Option<Tz>,
    /// 固定偏移量（仅用于偏移量类型时区）
    #[serde(skip)]
    pub(crate) offset: Option<i32>,
}

/// 时区类型枚举
///
/// PHP 定义了三种时区类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimezoneType {
    /// IANA 时区标识符，如 "Asia/Shanghai"
    /// 这是最推荐的时区表示方式
    Identifier = 3,
    
    /// 时区缩写，如 "CST", "EST"
    /// 不推荐使用，因为缩写可能有歧义
    Abbreviation = 2,
    
    /// UTC 偏移量，如 "+08:00", "-05:00"
    /// 适用于没有时区规则的简单场景
    Offset = 1,
}

impl DateTimeZoneValue {
    /// 创建新的时区对象
    ///
    /// # 参数
    /// - `timezone`: 时区标识符字符串
    ///
    /// # 支持的格式
    /// - IANA 时区：如 "Asia/Shanghai", "America/New_York"
    /// - UTC 偏移：如 "+08:00", "-05:00"
    /// - 时区缩写：如 "UTC", "GMT"
    ///
    /// # 返回
    /// 成功返回 DateTimeZoneValue，失败返回错误信息
    pub fn new(timezone: &str) -> Result<Self, String> {
        // 尝试解析为 IANA 时区
        if let Ok(tz) = timezone.parse::<Tz>() {
            return Ok(Self {
                timezone_type: TimezoneType::Identifier,
                identifier: timezone.to_string(),
                tz: Some(tz),
                offset: None,
            });
        }
        
        // 尝试解析为 UTC 偏移量
        if timezone.starts_with('+') || timezone.starts_with('-') {
            if let Some(offset) = Self::parse_offset(timezone) {
                return Ok(Self {
                    timezone_type: TimezoneType::Offset,
                    identifier: timezone.to_string(),
                    tz: None,
                    offset: Some(offset),
                });
            }
        }
        
        // 处理特殊时区名称
        match timezone.to_uppercase().as_str() {
            "UTC" | "GMT" => {
                return Ok(Self {
                    timezone_type: TimezoneType::Abbreviation,
                    identifier: timezone.to_uppercase(),
                    tz: Some(Tz::UTC),
                    offset: Some(0),
                });
            }
            "Z" => {
                return Ok(Self {
                    timezone_type: TimezoneType::Offset,
                    identifier: "+00:00".to_string(),
                    tz: Some(Tz::UTC),
                    offset: Some(0),
                });
            }
            _ => {}
        }
        
        // 尝试模糊匹配 IANA 时区
        let normalized = Self::normalize_timezone_name(timezone);
        if let Ok(tz) = normalized.parse::<Tz>() {
            return Ok(Self {
                timezone_type: TimezoneType::Identifier,
                identifier: normalized,
                tz: Some(tz),
                offset: None,
            });
        }
        
        Err(format!("未知的时区: {}", timezone))
    }
    
    /// 创建 UTC 时区
    pub fn utc() -> Self {
        Self {
            timezone_type: TimezoneType::Abbreviation,
            identifier: "UTC".to_string(),
            tz: Some(Tz::UTC),
            offset: Some(0),
        }
    }
    
    /// 创建系统默认时区
    pub fn system_default() -> Self {
        // 尝试获取系统时区
        // 在无法获取时使用 UTC
        Self::utc()
    }
    
    /// 获取时区名称
    ///
    /// # 返回
    /// 时区标识符字符串
    pub fn get_name(&self) -> &str {
        &self.identifier
    }
    
    /// 获取时区类型
    ///
    /// # 返回
    /// 时区类型枚举值
    pub fn get_type(&self) -> TimezoneType {
        self.timezone_type
    }
    
    /// 获取时区偏移量
    ///
    /// # 参数
    /// - `datetime`: 可选的日期时间，用于计算 DST（夏令时）偏移
    ///
    /// # 返回
    /// 相对于 UTC 的偏移量（秒）
    pub fn get_offset(&self, datetime: Option<&chrono::DateTime<Utc>>) -> i32 {
        match self.timezone_type {
            // IANA 时区需要考虑 DST
            TimezoneType::Identifier | TimezoneType::Abbreviation => {
                if let Some(tz) = self.tz {
                    if let Some(dt) = datetime {
                        // 计算指定时间在该时区的偏移
                        let local = dt.with_timezone(&tz);
                        let base = local.offset().base_utc_offset();
                        let dst = local.offset().dst_offset();
                        return (base + dst).num_seconds() as i32;
                    } else {
                        // 使用当前时间计算偏移
                        let now = Utc::now();
                        let local = now.with_timezone(&tz);
                        let base = local.offset().base_utc_offset();
                        let dst = local.offset().dst_offset();
                        return (base + dst).num_seconds() as i32;
                    }
                }
                0
            }
            // 固定偏移量
            TimezoneType::Offset => {
                self.offset.unwrap_or(0)
            }
        }
    }
    
    /// 获取时区偏移量的字符串表示
    ///
    /// # 参数
    /// - `datetime`: 可选的日期时间
    ///
    /// # 返回
    /// 偏移量字符串，如 "+08:00"
    pub fn get_offset_string(&self, datetime: Option<&chrono::DateTime<Utc>>) -> String {
        let offset_seconds = self.get_offset(datetime);
        Self::format_offset(offset_seconds)
    }
    
    /// 获取所有时区标识符列表
    ///
    /// # 参数
    /// - `what`: 时区过滤器（PHP 常量）
    /// - `country`: 可选的国家代码
    ///
    /// # 返回
    /// 时区标识符列表
    pub fn list_identifiers(what: Option<i32>, country: Option<&str>) -> Vec<String> {
        let filter = what.unwrap_or(DateTimeZone::ALL_WITH_BC);
        
        let mut result = Vec::new();
        
        // 遍历所有 IANA 时区
        for tz in TZ_VARIANTS.iter() {
            let name = tz.name();
            
            // 根据过滤器筛选
            match filter {
                DateTimeZone::ALL | DateTimeZone::ALL_WITH_BC => {
                    result.push(name.to_string());
                }
                DateTimeZone::AFRICA => {
                    if name.starts_with("Africa/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::AMERICA => {
                    if name.starts_with("America/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::ANTARCTICA => {
                    if name.starts_with("Antarctica/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::ARCTIC => {
                    if name.starts_with("Arctic/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::ASIA => {
                    if name.starts_with("Asia/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::ATLANTIC => {
                    if name.starts_with("Atlantic/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::AUSTRALIA => {
                    if name.starts_with("Australia/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::EUROPE => {
                    if name.starts_with("Europe/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::INDIAN => {
                    if name.starts_with("Indian/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::PACIFIC => {
                    if name.starts_with("Pacific/") {
                        result.push(name.to_string());
                    }
                }
                DateTimeZone::UTC => {
                    if name == "UTC" {
                        result.push(name.to_string());
                    }
                }
                _ => {
                    // 默认包含所有时区
                    result.push(name.to_string());
                }
            }
        }
        
        // 如果指定了国家，进一步过滤
        if let Some(country_code) = country {
            result = Self::filter_by_country(&result, country_code);
        }
        
        // 排序结果
        result.sort();
        
        result
    }
    
    /// 获取时区所在的大洲
    ///
    /// # 返回
    /// 大洲名称，如 "Asia", "Europe"
    pub fn get_location(&self) -> Option<&str> {
        if self.timezone_type == TimezoneType::Identifier {
            // 从标识符中提取大洲部分
            if let Some(pos) = self.identifier.find('/') {
                return Some(&self.identifier[..pos]);
            }
        }
        None
    }
    
    /// 获取时区所在的城市
    ///
    /// # 返回
    /// 城市名称，如 "Shanghai", "New_York"
    pub fn get_city(&self) -> Option<&str> {
        if self.timezone_type == TimezoneType::Identifier {
            // 从标识符中提取城市部分
            if let Some(pos) = self.identifier.find('/') {
                return Some(&self.identifier[pos + 1..]);
            }
        }
        None
    }
    
    /// 判断是否为 UTC 时区
    pub fn is_utc(&self) -> bool {
        match self.timezone_type {
            TimezoneType::Abbreviation => self.identifier == "UTC" || self.identifier == "GMT",
            TimezoneType::Offset => self.offset == Some(0),
            TimezoneType::Identifier => self.identifier == "UTC",
        }
    }
    
    /// 获取内部的 chrono-tz 时区
    ///
    /// 用于与其他 DateTime 操作集成
    pub(crate) fn get_chrono_tz(&self) -> Tz {
        self.tz.unwrap_or(Tz::UTC)
    }
    
    /// 解析偏移量字符串
    ///
    /// 支持格式："+08:00", "-05:00", "+0800", "-0500"
    fn parse_offset(offset_str: &str) -> Option<i32> {
        let s = offset_str.trim();
        
        // 确定符号
        let (sign, rest) = if s.starts_with('+') {
            (1, &s[1..])
        } else if s.starts_with('-') {
            (-1, &s[1..])
        } else {
            return None;
        };
        
        // 解析小时和分钟
        let (hours, minutes) = if rest.contains(':') {
            // 格式：HH:MM
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() != 2 {
                return None;
            }
            let h: i32 = parts[0].parse().ok()?;
            let m: i32 = parts[1].parse().ok()?;
            (h, m)
        } else if rest.len() == 4 {
            // 格式：HHMM
            let h: i32 = rest[0..2].parse().ok()?;
            let m: i32 = rest[2..4].parse().ok()?;
            (h, m)
        } else if rest.len() == 2 {
            // 格式：HH
            let h: i32 = rest.parse().ok()?;
            (h, 0)
        } else {
            return None;
        };
        
        // 验证范围
        if hours > 14 || minutes > 59 {
            return None;
        }
        
        // 计算总秒数
        Some(sign * (hours * 3600 + minutes * 60))
    }
    
    /// 格式化偏移量为字符串
    fn format_offset(offset_seconds: i32) -> String {
        let sign = if offset_seconds >= 0 { "+" } else { "-" };
        let abs_offset = offset_seconds.abs();
        let hours = abs_offset / 3600;
        let minutes = (abs_offset % 3600) / 60;
        
        format!("{}{:02}:{:02}", sign, hours, minutes)
    }
    
    /// 标准化时区名称
    ///
    /// 处理常见的时区名称变体
    fn normalize_timezone_name(name: &str) -> String {
        // 处理常见的别名
        match name {
            "China" | "PRC" | "Beijing" => "Asia/Shanghai".to_string(),
            "Hongkong" | "Hong_Kong" => "Asia/Hong_Kong".to_string(),
            "Taipei" => "Asia/Taipei".to_string(),
            "Tokyo" => "Asia/Tokyo".to_string(),
            "Singapore" => "Asia/Singapore".to_string(),
            "New_York" | "NY" => "America/New_York".to_string(),
            "Los_Angeles" | "LA" => "America/Los_Angeles".to_string(),
            "London" => "Europe/London".to_string(),
            "Paris" => "Europe/Paris".to_string(),
            "Sydney" => "Australia/Sydney".to_string(),
            _ => name.to_string(),
        }
    }
    
    /// 根据国家代码过滤时区
    fn filter_by_country(timezones: &[String], country_code: &str) -> Vec<String> {
        // 国家代码到时区的映射
        let country_timezones: std::collections::HashMap<&str, &[&str]> = [
            ("CN", &["Asia/Shanghai", "Asia/Urumqi"] as &[&str]),
            ("US", &["America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles"]),
            ("GB", &["Europe/London"]),
            ("JP", &["Asia/Tokyo"]),
            ("KR", &["Asia/Seoul"]),
            ("AU", &["Australia/Sydney", "Australia/Melbourne", "Australia/Perth"]),
            ("DE", &["Europe/Berlin"]),
            ("FR", &["Europe/Paris"]),
            ("SG", &["Asia/Singapore"]),
            ("HK", &["Asia/Hong_Kong"]),
            ("TW", &["Asia/Taipei"]),
        ].iter().cloned().collect();
        
        let upper_code = country_code.to_uppercase();
        
        if let Some(tzs) = country_timezones.get(upper_code.as_str()) {
            timezones.iter()
                .filter(|tz| tzs.contains(&tz.as_str()))
                .cloned()
                .collect()
        } else {
            timezones.to_vec()
        }
    }
}

impl fmt::Display for DateTimeZoneValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

impl Default for DateTimeZoneValue {
    fn default() -> Self {
        Self::utc()
    }
}

/// DateTimeZone 常量
///
/// 对应 PHP 的 DateTimeZone 类常量
pub struct DateTimeZone;

impl DateTimeZone {
    /// 非洲时区
    pub const AFRICA: i32 = 1;
    /// 美洲时区
    pub const AMERICA: i32 = 2;
    /// 南极洲时区
    pub const ANTARCTICA: i32 = 4;
    /// 北极时区
    pub const ARCTIC: i32 = 8;
    /// 亚洲时区
    pub const ASIA: i32 = 16;
    /// 大西洋时区
    pub const ATLANTIC: i32 = 32;
    /// 澳大利亚时区
    pub const AUSTRALIA: i32 = 64;
    /// 欧洲时区
    pub const EUROPE: i32 = 128;
    /// 印度洋时区
    pub const INDIAN: i32 = 256;
    /// 太平洋时区
    pub const PACIFIC: i32 = 512;
    /// UTC 时区
    pub const UTC: i32 = 1024;
    /// 所有时区
    pub const ALL: i32 = 2047;
    /// 所有时区（包括向后兼容）
    pub const ALL_WITH_BC: i32 = 4095;
    /// 每个国家一个时区
    pub const PER_COUNTRY: i32 = 4096;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timezone_creation() {
        // 测试 IANA 时区
        let tz = DateTimeZoneValue::new("Asia/Shanghai").unwrap();
        assert_eq!(tz.get_name(), "Asia/Shanghai");
        assert_eq!(tz.get_type(), TimezoneType::Identifier);
    }
    
    #[test]
    fn test_timezone_offset() {
        // 测试偏移量时区
        let tz = DateTimeZoneValue::new("+08:00").unwrap();
        assert_eq!(tz.get_type(), TimezoneType::Offset);
        assert_eq!(tz.get_offset(None), 8 * 3600);
    }
    
    #[test]
    fn test_timezone_utc() {
        // 测试 UTC 时区
        let tz = DateTimeZoneValue::utc();
        assert!(tz.is_utc());
        assert_eq!(tz.get_offset(None), 0);
    }
    
    #[test]
    fn test_list_identifiers() {
        // 测试获取时区列表
        let asia_tzs = DateTimeZoneValue::list_identifiers(Some(DateTimeZone::ASIA), None);
        assert!(!asia_tzs.is_empty());
        assert!(asia_tzs.contains(&"Asia/Shanghai".to_string()));
    }
    
    #[test]
    fn test_timezone_offset_string() {
        let tz = DateTimeZoneValue::new("+08:00").unwrap();
        assert_eq!(tz.get_offset_string(None), "+08:00");
        
        let tz2 = DateTimeZoneValue::new("-05:00").unwrap();
        assert_eq!(tz2.get_offset_string(None), "-05:00");
    }
}
