//! 相对时间解析器实现
//!
//! 提供完整的 PHP 相对时间格式解析功能
//! 支持：+1 day, next Monday, last day of December 等

use chrono::{DateTime as ChronoDateTime, Datelike, Duration, TimeZone, Timelike, Weekday, Utc};
use chrono_tz::Tz;
use regex::Regex;

/// 相对时间解析器
///
/// 解析 PHP 风格的相对时间字符串
pub struct RelativeTimeParser {
    /// 相对时间模式列表
    patterns: Vec<RelativePattern>,
}

/// 相对时间模式
struct RelativePattern {
    /// 正则表达式
    regex: Regex,
    /// 模式类型
    pattern_type: RelativePatternType,
}

/// 相对时间模式类型
#[derive(Debug, Clone)]
enum RelativePatternType {
    /// 数值偏移（如 +1 day, -2 hours）
    NumericOffset {
        /// 单位类型
        unit: TimeUnit,
    },
    /// 下一个工作日（如 next Monday）
    NextWeekday,
    /// 上一个工作日（如 last Monday）
    LastWeekday,
    /// 月初（如 first day of January）
    FirstDayOfMonth,
    /// 月末（如 last day of January）
    LastDayOfMonth,
    /// 年初
    FirstDayOfYear,
    /// 年末
    LastDayOfYear,
    /// 特殊时间（如 noon, midnight）
    SpecialTime,
}

/// 时间单位
#[derive(Debug, Clone, Copy)]
enum TimeUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
}

impl RelativeTimeParser {
    /// 创建新的相对时间解析器
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }
    
    /// 修改日期时间
    ///
    /// # 参数
    /// - `base`: 基准日期时间
    /// - `modify`: 修改字符串
    ///
    /// # 返回
    /// 成功返回新的日期时间，失败返回错误
    pub fn modify(&self, base: &ChronoDateTime<Utc>, modify: &str) -> Result<ChronoDateTime<Utc>, String> {
        let s = modify.trim().to_lowercase();
        
        // 遍历所有模式进行匹配
        for pattern in &self.patterns {
            if let Some(caps) = pattern.regex.captures(&s) {
                match &pattern.pattern_type {
                    RelativePatternType::NumericOffset { unit } => {
                        return self.apply_numeric_offset(base, &caps, *unit);
                    }
                    RelativePatternType::NextWeekday => {
                        return self.apply_next_weekday(base, &caps);
                    }
                    RelativePatternType::LastWeekday => {
                        return self.apply_last_weekday(base, &caps);
                    }
                    RelativePatternType::FirstDayOfMonth => {
                        return self.apply_first_day_of_month(base, &caps);
                    }
                    RelativePatternType::LastDayOfMonth => {
                        return self.apply_last_day_of_month(base, &caps);
                    }
                    RelativePatternType::FirstDayOfYear => {
                        return self.apply_first_day_of_year(base, &caps);
                    }
                    RelativePatternType::LastDayOfYear => {
                        return self.apply_last_day_of_year(base, &caps);
                    }
                    RelativePatternType::SpecialTime => {
                        return self.apply_special_time(base, &caps);
                    }
                }
            }
        }
        
        // 尝试解析复合表达式
        self.parse_complex_expression(base, &s)
    }
    
    /// 初始化相对时间模式
    fn init_patterns() -> Vec<RelativePattern> {
        vec![
            // 数值偏移模式
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*years?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Year },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*months?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Month },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*weeks?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Week },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*days?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Day },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*hours?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Hour },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*minutes?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Minute },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*seconds?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Second },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*milliseconds?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Millisecond },
            },
            RelativePattern {
                regex: Regex::new(r"^([+-]?\d+)\s*microseconds?$").unwrap(),
                pattern_type: RelativePatternType::NumericOffset { unit: TimeUnit::Microsecond },
            },
            
            // 下一个/上一个工作日
            RelativePattern {
                regex: Regex::new(r"^next\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$").unwrap(),
                pattern_type: RelativePatternType::NextWeekday,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$").unwrap(),
                pattern_type: RelativePatternType::LastWeekday,
            },
            
            // 月初/月末
            RelativePattern {
                regex: Regex::new(r"^first\s+day\s+of\s+(january|february|march|april|may|june|july|august|september|october|november|december)$").unwrap(),
                pattern_type: RelativePatternType::FirstDayOfMonth,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+day\s+of\s+(january|february|march|april|may|june|july|august|september|october|november|december)$").unwrap(),
                pattern_type: RelativePatternType::LastDayOfMonth,
            },
            RelativePattern {
                regex: Regex::new(r"^first\s+day\s+of\s+this\s+month$").unwrap(),
                pattern_type: RelativePatternType::FirstDayOfMonth,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+day\s+of\s+this\s+month$").unwrap(),
                pattern_type: RelativePatternType::LastDayOfMonth,
            },
            RelativePattern {
                regex: Regex::new(r"^first\s+day\s+of\s+next\s+month$").unwrap(),
                pattern_type: RelativePatternType::FirstDayOfMonth,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+day\s+of\s+next\s+month$").unwrap(),
                pattern_type: RelativePatternType::LastDayOfMonth,
            },
            
            // 年初/年末
            RelativePattern {
                regex: Regex::new(r"^first\s+day\s+of\s+this\s+year$").unwrap(),
                pattern_type: RelativePatternType::FirstDayOfYear,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+day\s+of\s+this\s+year$").unwrap(),
                pattern_type: RelativePatternType::LastDayOfYear,
            },
            RelativePattern {
                regex: Regex::new(r"^first\s+day\s+of\s+next\s+year$").unwrap(),
                pattern_type: RelativePatternType::FirstDayOfYear,
            },
            RelativePattern {
                regex: Regex::new(r"^last\s+day\s+of\s+next\s+year$").unwrap(),
                pattern_type: RelativePatternType::LastDayOfYear,
            },
            
            // 特殊时间
            RelativePattern {
                regex: Regex::new(r"^noon$").unwrap(),
                pattern_type: RelativePatternType::SpecialTime,
            },
            RelativePattern {
                regex: Regex::new(r"^midnight$").unwrap(),
                pattern_type: RelativePatternType::SpecialTime,
            },
        ]
    }
    
    /// 应用数值偏移
    fn apply_numeric_offset(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
        unit: TimeUnit,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let value: i64 = caps[1].parse().map_err(|_| "无效的数值")?;
        
        let result = match unit {
            TimeUnit::Year => {
                // 年需要特殊处理（考虑闰年）
                let current_year = base.year();
                let new_year = current_year + value as i32;
                base.with_year(new_year)
                    .ok_or_else(|| "年份调整失败".to_string())?
            }
            TimeUnit::Month => {
                // 月需要特殊处理（考虑不同月份的天数）
                let current_month = base.month() as i32;
                let current_year = base.year();
                let total_months = current_year * 12 + current_month + value as i32;
                let new_year = total_months / 12;
                let new_month = (total_months % 12).max(1) as u32;
                
                // 调整日期（处理月末）
                let max_day = Self::days_in_month(new_year, new_month);
                let new_day = base.day().min(max_day);
                
                base.with_year(new_year)
                    .and_then(|d| d.with_month(new_month))
                    .and_then(|d| d.with_day(new_day))
                    .ok_or_else(|| "月份调整失败".to_string())?
            }
            TimeUnit::Week => {
                base.checked_add_signed(Duration::try_weeks(value).unwrap_or_default())
                    .ok_or_else(|| "周调整失败".to_string())?
            }
            TimeUnit::Day => {
                base.checked_add_signed(Duration::try_days(value).unwrap_or_default())
                    .ok_or_else(|| "日调整失败".to_string())?
            }
            TimeUnit::Hour => {
                base.checked_add_signed(Duration::try_hours(value).unwrap_or_default())
                    .ok_or_else(|| "小时调整失败".to_string())?
            }
            TimeUnit::Minute => {
                base.checked_add_signed(Duration::try_minutes(value).unwrap_or_default())
                    .ok_or_else(|| "分钟调整失败".to_string())?
            }
            TimeUnit::Second => {
                base.checked_add_signed(Duration::try_seconds(value).unwrap_or_default())
                    .ok_or_else(|| "秒调整失败".to_string())?
            }
            TimeUnit::Millisecond => {
                base.checked_add_signed(Duration::try_milliseconds(value).unwrap_or_default())
                    .ok_or_else(|| "毫秒调整失败".to_string())?
            }
            TimeUnit::Microsecond => {
                base.checked_add_signed(Duration::microseconds(value))
                    .ok_or_else(|| "微秒调整失败".to_string())?
            }
        };
        
        Ok(result)
    }
    
    /// 应用"下一个工作日"
    fn apply_next_weekday(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let weekday_name = &caps[1];
        let target_weekday = Self::parse_weekday(weekday_name)?;
        
        let current_weekday = base.weekday();
        let days_ahead = (target_weekday.num_days_from_monday() as i32 
            - current_weekday.num_days_from_monday() as i32 + 7) % 7;
        let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };
        
        base.checked_add_signed(Duration::try_days(days_ahead as i64).unwrap_or_default())
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用"上一个工作日"
    fn apply_last_weekday(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let weekday_name = &caps[1];
        let target_weekday = Self::parse_weekday(weekday_name)?;
        
        let current_weekday = base.weekday();
        let days_behind = (current_weekday.num_days_from_monday() as i32 
            - target_weekday.num_days_from_monday() as i32 + 7) % 7;
        let days_behind = if days_behind == 0 { 7 } else { days_behind };
        
        base.checked_sub_signed(Duration::try_days(days_behind as i64).unwrap_or_default())
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用"月初"
    fn apply_first_day_of_month(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 检查是否指定了月份
        let (year, month) = if caps.len() > 1 {
            // 指定了月份名称
            let month_name = &caps[1];
            let month = Self::parse_month(month_name)?;
            (base.year(), month)
        } else {
            // 使用当前月份或下个月
            let s = caps.get(0).unwrap().as_str();
            if s.contains("next") {
                // 下个月
                let current_month = base.month();
                if current_month == 12 {
                    (base.year() + 1, 1)
                } else {
                    (base.year(), current_month + 1)
                }
            } else {
                (base.year(), base.month())
            }
        };
        
        base.with_year(year)
            .and_then(|d| d.with_month(month))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用"月末"
    fn apply_last_day_of_month(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 计算年份和月份
        let (year, month) = {
            let s = caps.get(0).unwrap().as_str();
            if s.contains("next") {
                // 下个月
                let current_month = base.month();
                if current_month == 12 {
                    (base.year() + 1, 1)
                } else {
                    (base.year(), current_month + 1)
                }
            } else if caps.len() > 1 {
                // 指定了月份名称
                let month_name = &caps[1];
                let month = Self::parse_month(month_name)?;
                (base.year(), month)
            } else {
                (base.year(), base.month())
            }
        };
        
        let last_day = Self::days_in_month(year, month);
        
        base.with_year(year)
            .and_then(|d| d.with_month(month))
            .and_then(|d| d.with_day(last_day))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用"年初"
    fn apply_first_day_of_year(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let s = caps.get(0).unwrap().as_str();
        let year = if s.contains("next") {
            base.year() + 1
        } else {
            base.year()
        };
        
        base.with_year(year)
            .and_then(|d| d.with_month(1))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用"年末"
    fn apply_last_day_of_year(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let s = caps.get(0).unwrap().as_str();
        let year = if s.contains("next") {
            base.year() + 1
        } else {
            base.year()
        };
        
        base.with_year(year)
            .and_then(|d| d.with_month(12))
            .and_then(|d| d.with_day(31))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .ok_or_else(|| "日期计算失败".to_string())
    }
    
    /// 应用特殊时间
    fn apply_special_time(
        &self,
        base: &ChronoDateTime<Utc>,
        caps: &regex::Captures,
    ) -> Result<ChronoDateTime<Utc>, String> {
        let s = caps.get(0).unwrap().as_str();
        
        match s {
            "noon" => {
                // 中午 12:00
                base.with_hour(12)
                    .and_then(|d| d.with_minute(0))
                    .and_then(|d| d.with_second(0))
                    .ok_or_else(|| "时间计算失败".to_string())
            }
            "midnight" => {
                // 午夜 00:00
                base.with_hour(0)
                    .and_then(|d| d.with_minute(0))
                    .and_then(|d| d.with_second(0))
                    .ok_or_else(|| "时间计算失败".to_string())
            }
            _ => Err("未知的特殊时间".to_string())
        }
    }
    
    /// 解析复合表达式
    ///
    /// 处理包含多个偏移的表达式，如 "+1 day +2 hours"
    fn parse_complex_expression(
        &self,
        base: &ChronoDateTime<Utc>,
        s: &str,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 分割表达式
        let parts: Vec<&str> = s.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err("空表达式".to_string());
        }
        
        let mut current = *base;
        let mut i = 0;
        
        while i < parts.len() {
            let part = parts[i];
            
            // 尝试匹配数值偏移
            if part.starts_with('+') || part.starts_with('-') {
                // 检查是否有单位
                if i + 1 < parts.len() {
                    let unit = parts[i + 1];
                    let combined = format!("{} {}", part, unit);
                    
                    // 尝试解析组合表达式
                    if let Ok(result) = self.modify(&current, &combined) {
                        current = result;
                        i += 2;
                        continue;
                    }
                }
                
                // 尝试单独解析数值
                if let Ok(result) = self.modify(&current, part) {
                    current = result;
                }
            } else {
                // 尝试解析其他表达式
                if let Ok(result) = self.modify(&current, part) {
                    current = result;
                }
            }
            
            i += 1;
        }
        
        Ok(current)
    }
    
    /// 解析星期名称
    fn parse_weekday(name: &str) -> Result<Weekday, String> {
        match name.to_lowercase().as_str() {
            "monday" | "mon" => Ok(Weekday::Mon),
            "tuesday" | "tue" | "tues" => Ok(Weekday::Tue),
            "wednesday" | "wed" => Ok(Weekday::Wed),
            "thursday" | "thu" | "thurs" => Ok(Weekday::Thu),
            "friday" | "fri" => Ok(Weekday::Fri),
            "saturday" | "sat" => Ok(Weekday::Sat),
            "sunday" | "sun" => Ok(Weekday::Sun),
            _ => Err(format!("未知的星期名称: {}", name))
        }
    }
    
    /// 解析月份名称
    fn parse_month(name: &str) -> Result<u32, String> {
        match name.to_lowercase().as_str() {
            "january" | "jan" => Ok(1),
            "february" | "feb" => Ok(2),
            "march" | "mar" => Ok(3),
            "april" | "apr" => Ok(4),
            "may" => Ok(5),
            "june" | "jun" => Ok(6),
            "july" | "jul" => Ok(7),
            "august" | "aug" => Ok(8),
            "september" | "sep" | "sept" => Ok(9),
            "october" | "oct" => Ok(10),
            "november" | "nov" => Ok(11),
            "december" | "dec" => Ok(12),
            _ => Err(format!("未知的月份名称: {}", name))
        }
    }
    
    /// 获取指定月份的天数
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }
}

impl Default for RelativeTimeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    
    fn get_test_datetime() -> ChronoDateTime<Utc> {
        // 2024-01-15 10:30:00 (周一)
        Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap()
    }
    
    #[test]
    fn test_modify_days() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 +1 day
        let result = parser.modify(&base, "+1 day").unwrap();
        assert_eq!(result.day(), 16);
        
        // 测试 -1 day
        let result = parser.modify(&base, "-1 day").unwrap();
        assert_eq!(result.day(), 14);
    }
    
    #[test]
    fn test_modify_weeks() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 +1 week
        let result = parser.modify(&base, "+1 week").unwrap();
        assert_eq!(result.day(), 22);
    }
    
    #[test]
    fn test_modify_months() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 +1 month
        let result = parser.modify(&base, "+1 month").unwrap();
        assert_eq!(result.month(), 2);
    }
    
    #[test]
    fn test_modify_years() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 +1 year
        let result = parser.modify(&base, "+1 year").unwrap();
        assert_eq!(result.year(), 2025);
    }
    
    #[test]
    fn test_next_weekday() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime(); // 周一
        
        // 测试 next Monday
        let result = parser.modify(&base, "next Monday").unwrap();
        assert_eq!(result.day(), 22); // 下周一
        
        // 测试 next Friday
        let result = parser.modify(&base, "next Friday").unwrap();
        assert_eq!(result.day(), 19); // 本周五
    }
    
    #[test]
    fn test_last_weekday() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime(); // 周一
        
        // 测试 last Monday
        let result = parser.modify(&base, "last Monday").unwrap();
        assert_eq!(result.day(), 8); // 上周一
    }
    
    #[test]
    fn test_first_day_of_month() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 first day of this month
        let result = parser.modify(&base, "first day of this month").unwrap();
        assert_eq!(result.day(), 1);
    }
    
    #[test]
    fn test_last_day_of_month() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 last day of this month
        let result = parser.modify(&base, "last day of this month").unwrap();
        assert_eq!(result.day(), 31); // 1月有31天
    }
    
    #[test]
    fn test_special_time() {
        let parser = RelativeTimeParser::new();
        let base = get_test_datetime();
        
        // 测试 noon
        let result = parser.modify(&base, "noon").unwrap();
        assert_eq!(result.hour(), 12);
        assert_eq!(result.minute(), 0);
        
        // 测试 midnight
        let result = parser.modify(&base, "midnight").unwrap();
        assert_eq!(result.hour(), 0);
        assert_eq!(result.minute(), 0);
    }
}
