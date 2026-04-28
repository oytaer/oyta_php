//! DateTime 字符串解析器实现
//!
//! 提供完整的 PHP 日期时间字符串解析功能
//! 支持：多种日期格式、相对时间、时间戳等

use chrono::{DateTime as ChronoDateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use regex::Regex;
use std::collections::HashMap;

use super::timezone::DateTimeZoneValue;

/// 日期时间解析器
///
/// 支持解析多种格式的日期时间字符串
pub struct DateTimeParser {
    /// 相对时间正则表达式缓存
    relative_patterns: Vec<(Regex, RelativePatternHandler)>,
    /// 格式字符映射
    format_chars: HashMap<char, FormatCharInfo>,
}

/// 相对时间模式处理函数类型
type RelativePatternHandler = fn(&regex::Captures, &ChronoDateTime<Utc>, &Tz) -> Option<ChronoDateTime<Utc>>;

/// 格式字符信息
struct FormatCharInfo {
    /// 格式字符描述
    description: &'static str,
    /// 是否支持解析
    can_parse: bool,
}

impl DateTimeParser {
    /// 创建新的解析器实例
    pub fn new() -> Self {
        Self {
            relative_patterns: Self::init_relative_patterns(),
            format_chars: Self::init_format_chars(),
        }
    }
    
    /// 解析日期时间字符串
    ///
    /// # 参数
    /// - `input`: 日期时间字符串
    /// - `timezone`: 时区对象
    ///
    /// # 返回
    /// 成功返回 (DateTime, warnings, errors)，失败返回错误
    pub fn parse(
        &self,
        input: &str,
        timezone: &DateTimeZoneValue,
    ) -> Result<(ChronoDateTime<Utc>, Vec<String>, Vec<String>), String> {
        let s = input.trim();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // 处理特殊值
        match s.to_lowercase().as_str() {
            // 当前时间
            "now" => {
                return Ok((Utc::now(), warnings, errors));
            }
            // 今天
            "today" => {
                let now = Utc::now();
                let date = now.date_naive();
                let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let naive = NaiveDateTime::new(date, time);
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);
                return Ok((result, warnings, errors));
            }
            // 明天
            "tomorrow" => {
                let now = Utc::now();
                let date = (now + chrono::Duration::try_days(1).unwrap()).date_naive();
                let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let naive = NaiveDateTime::new(date, time);
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);
                return Ok((result, warnings, errors));
            }
            // 昨天
            "yesterday" => {
                let now = Utc::now();
                let date = (now - chrono::Duration::try_days(1).unwrap()).date_naive();
                let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let naive = NaiveDateTime::new(date, time);
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);
                return Ok((result, warnings, errors));
            }
            _ => {}
        }
        
        // 尝试解析 Unix 时间戳（@1234567890 格式）
        if s.starts_with('@') {
            let timestamp_str = &s[1..];
            if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                let result = Utc.timestamp_opt(timestamp, 0).single()
                    .ok_or_else(|| format!("无效的时间戳: {}", timestamp))?;
                return Ok((result, warnings, errors));
            }
        }
        
        // 尝试解析相对时间
        if let Some(result) = self.parse_relative_time(s, &Utc::now(), &timezone.get_chrono_tz()) {
            return Ok((result, warnings, errors));
        }
        
        // 尝试解析 ISO 8601 格式
        if let Ok(result) = self.parse_iso8601(s) {
            return Ok((result, warnings, errors));
        }
        
        // 尝试解析常见格式
        if let Ok(result) = self.parse_common_formats(s, timezone) {
            return Ok((result, warnings, errors));
        }
        
        // 尝试解析英文日期格式
        if let Ok(result) = self.parse_english_date(s, timezone) {
            return Ok((result, warnings, errors));
        }
        
        // 尝试使用 chrono 的自动解析
        if let Ok(result) = self.parse_with_chrono(s, timezone) {
            return Ok((result, warnings, errors));
        }
        
        errors.push(format!("无法解析日期时间字符串: {}", input));
        Err(format!("无法解析日期时间字符串: {}", input))
    }
    
    /// 从格式字符串解析
    ///
    /// # 参数
    /// - `format`: 格式字符串
    /// - `datetime`: 日期时间字符串
    /// - `timezone`: 可选的时区
    ///
    /// # 返回
    /// 成功返回 DateTime，失败返回 None
    pub fn parse_from_format(
        &self,
        format: &str,
        datetime: &str,
        timezone: Option<DateTimeZoneValue>,
    ) -> Option<super::datetime::DateTimeValue> {
        // 解析格式字符串
        let parsed = self.parse_format_string(format, datetime)?;
        
        // 构建日期时间
        let tz = timezone.unwrap_or_else(DateTimeZoneValue::utc);
        
        // 创建 NaiveDateTime
        let date = NaiveDate::from_ymd_opt(
            parsed.year.unwrap_or(1970),
            parsed.month.unwrap_or(1),
            parsed.day.unwrap_or(1),
        )?;
        
        let time = NaiveTime::from_hms_micro_opt(
            parsed.hour.unwrap_or(0),
            parsed.minute.unwrap_or(0),
            parsed.second.unwrap_or(0),
            parsed.microsecond.unwrap_or(0),
        )?;
        
        let naive = NaiveDateTime::new(date, time);
        
        // 转换为时区时间
        let result = tz.get_chrono_tz().from_local_datetime(&naive)
            .single()?
            .with_timezone(&Utc);
        
        Some(super::datetime::DateTimeValue {
            inner: result,
            timezone: tz,
            is_immutable: false,
            microsecond: parsed.microsecond.unwrap_or(0),
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }
    
    /// 解析 ISO 8601 格式
    fn parse_iso8601(&self, s: &str) -> Result<ChronoDateTime<Utc>, String> {
        // 尝试解析带时区的 ISO 8601
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // 尝试解析 ISO 8601 日期时间
        if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f%:z") {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // 尝试解析 ISO 8601 日期时间（无时区）
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
            return Ok(Utc.from_utc_datetime(&naive));
        }
        
        // 尝试解析 ISO 8601 日期时间（带 Z）
        if s.ends_with('Z') {
            let without_z = &s[..s.len() - 1];
            if let Ok(naive) = NaiveDateTime::parse_from_str(without_z, "%Y-%m-%dT%H:%M:%S%.f") {
                return Ok(Utc.from_utc_datetime(&naive));
            }
        }
        
        Err(format!("无法解析 ISO 8601 格式: {}", s))
    }
    
    /// 解析常见格式
    fn parse_common_formats(
        &self,
        s: &str,
        timezone: &DateTimeZoneValue,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 定义常见格式列表
        let formats = [
            // 标准格式
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
            "%d-%m-%Y %H:%M:%S",
            "%d-%m-%Y",
            "%d/%m/%Y %H:%M:%S",
            "%d/%m/%Y",
            "%m-%d-%Y %H:%M:%S",
            "%m-%d-%Y",
            "%m/%d/%Y %H:%M:%S",
            "%m/%d/%Y",
            // 中文格式
            "%Y年%m月%d日 %H:%M:%S",
            "%Y年%m月%d日",
            // 时间格式
            "%H:%M:%S",
            "%H:%M",
            // 其他格式
            "%Y%m%d %H%M%S",
            "%Y%m%d",
        ];
        
        for fmt in formats {
            // 尝试解析日期时间
            if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .ok_or_else(|| format!("时区转换失败"))?
                    .with_timezone(&Utc);
                return Ok(result);
            }
            
            // 尝试解析日期
            if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
                let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let naive = NaiveDateTime::new(date, time);
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .ok_or_else(|| format!("时区转换失败"))?
                    .with_timezone(&Utc);
                return Ok(result);
            }
            
            // 尝试解析时间
            if let Ok(time) = NaiveTime::parse_from_str(s, fmt) {
                let now = Utc::now();
                let date = now.date_naive();
                let naive = NaiveDateTime::new(date, time);
                let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                    .single()
                    .ok_or_else(|| format!("时区转换失败"))?
                    .with_timezone(&Utc);
                return Ok(result);
            }
        }
        
        Err(format!("无法解析常见格式: {}", s))
    }
    
    /// 解析英文日期格式
    fn parse_english_date(
        &self,
        s: &str,
        timezone: &DateTimeZoneValue,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 英文月份映射
        let months = [
            ("january", 1), ("february", 2), ("march", 3), ("april", 4),
            ("may", 5), ("june", 6), ("july", 7), ("august", 8),
            ("september", 9), ("october", 10), ("november", 11), ("december", 12),
            ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4),
            ("jun", 6), ("jul", 7), ("aug", 8), ("sep", 9), ("sept", 9),
            ("oct", 10), ("nov", 11), ("dec", 12),
        ];
        
        let s_lower = s.to_lowercase();
        
        // 尝试匹配 "January 15, 2024" 或 "15 January 2024" 格式
        for (month_name, month_num) in months {
            if s_lower.contains(month_name) {
                // 提取日期和年份
                let re = regex::Regex::new(r"(\d{1,2})\s*,?\s*(\d{4})").unwrap();
                if let Some(caps) = re.captures(s) {
                    let day: u32 = caps[1].parse().unwrap_or(1);
                    let year: i32 = caps[2].parse().unwrap_or(1970);
                    
                    if let Some(date) = NaiveDate::from_ymd_opt(year, month_num, day) {
                        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                        let naive = NaiveDateTime::new(date, time);
                        let result = timezone.get_chrono_tz().from_local_datetime(&naive)
                            .single()
                            .ok_or_else(|| format!("时区转换失败"))?
                            .with_timezone(&Utc);
                        return Ok(result);
                    }
                }
            }
        }
        
        Err(format!("无法解析英文日期: {}", s))
    }
    
    /// 使用 chrono 自动解析
    fn parse_with_chrono(
        &self,
        s: &str,
        timezone: &DateTimeZoneValue,
    ) -> Result<ChronoDateTime<Utc>, String> {
        // 尝试使用 chrono 的 parse_from_str
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
            "%d %b %Y %H:%M:%S",
            "%d %b %Y",
            "%b %d, %Y",
        ];
        
        for fmt in formats {
            if let Ok(dt) = chrono::DateTime::parse_from_str(s, fmt) {
                return Ok(dt.with_timezone(&Utc));
            }
        }
        
        Err(format!("无法解析: {}", s))
    }
    
    /// 解析相对时间
    fn parse_relative_time(
        &self,
        s: &str,
        base: &ChronoDateTime<Utc>,
        tz: &Tz,
    ) -> Option<ChronoDateTime<Utc>> {
        let s_lower = s.to_lowercase();
        
        // 遍历所有相对时间模式
        for (pattern, handler) in &self.relative_patterns {
            if let Some(caps) = pattern.captures(&s_lower) {
                if let Some(result) = handler(&caps, base, tz) {
                    return Some(result);
                }
            }
        }
        
        None
    }
    
    /// 初始化相对时间模式
    fn init_relative_patterns() -> Vec<(Regex, RelativePatternHandler)> {
        vec![
            // "+N day", "-N day", "+N days"
            (Regex::new(r"^([+-]?\d+)\s*days?$").unwrap(), Self::handle_days),
            // "+N week", "-N week", "+N weeks"
            (Regex::new(r"^([+-]?\d+)\s*weeks?$").unwrap(), Self::handle_weeks),
            // "+N month", "-N month", "+N months"
            (Regex::new(r"^([+-]?\d+)\s*months?$").unwrap(), Self::handle_months),
            // "+N year", "-N year", "+N years"
            (Regex::new(r"^([+-]?\d+)\s*years?$").unwrap(), Self::handle_years),
            // "+N hour", "-N hour", "+N hours"
            (Regex::new(r"^([+-]?\d+)\s*hours?$").unwrap(), Self::handle_hours),
            // "+N minute", "-N minute", "+N minutes"
            (Regex::new(r"^([+-]?\d+)\s*minutes?$").unwrap(), Self::handle_minutes),
            // "+N second", "-N second", "+N seconds"
            (Regex::new(r"^([+-]?\d+)\s*seconds?$").unwrap(), Self::handle_seconds),
            // "next monday", "next tuesday", etc.
            (Regex::new(r"^next\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$").unwrap(), Self::handle_next_weekday),
            // "last monday", "last tuesday", etc.
            (Regex::new(r"^last\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$").unwrap(), Self::handle_last_weekday),
            // "first day of next month"
            (Regex::new(r"^first\s+day\s+of\s+next\s+month$").unwrap(), Self::handle_first_day_next_month),
            // "last day of this month"
            (Regex::new(r"^last\s+day\s+of\s+this\s+month$").unwrap(), Self::handle_last_day_this_month),
            // "last day of next month"
            (Regex::new(r"^last\s+day\s+of\s+next\s+month$").unwrap(), Self::handle_last_day_next_month),
        ]
    }
    
    /// 处理天数偏移
    fn handle_days(caps: &regex::Captures, base: &ChronoDateTime<Utc>, _tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let days: i64 = caps[1].parse().ok()?;
        base.checked_add_signed(chrono::Duration::try_days(days).unwrap_or_default())
    }
    
    /// 处理周数偏移
    fn handle_weeks(caps: &regex::Captures, base: &ChronoDateTime<Utc>, _tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let weeks: i64 = caps[1].parse().ok()?;
        base.checked_add_signed(chrono::Duration::try_weeks(weeks).unwrap_or_default())
    }
    
    /// 处理月数偏移
    fn handle_months(caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let months: i32 = caps[1].parse().ok()?;
        let local = base.with_timezone(tz);
        
        // 计算新的月份
        let current_month = local.month() as i32;
        let current_year = local.year();
        let total_months = current_year * 12 + current_month as i32 + months;
        let new_year = total_months / 12;
        let new_month = (total_months % 12).max(1) as u32;
        
        // 调整日期（处理月末）
        let new_day = local.day().min(Self::days_in_month(new_year, new_month));
        
        local
            .with_year(new_year)
            .and_then(|d| d.with_month(new_month))
            .and_then(|d| d.with_day(new_day))
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理年数偏移
    fn handle_years(caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let years: i32 = caps[1].parse().ok()?;
        let local = base.with_timezone(tz);
        
        local
            .with_year(local.year() + years)
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理小时偏移
    fn handle_hours(caps: &regex::Captures, base: &ChronoDateTime<Utc>, _tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let hours: i64 = caps[1].parse().ok()?;
        base.checked_add_signed(chrono::Duration::try_hours(hours).unwrap_or_default())
    }
    
    /// 处理分钟偏移
    fn handle_minutes(caps: &regex::Captures, base: &ChronoDateTime<Utc>, _tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let minutes: i64 = caps[1].parse().ok()?;
        base.checked_add_signed(chrono::Duration::try_minutes(minutes).unwrap_or_default())
    }
    
    /// 处理秒数偏移
    fn handle_seconds(caps: &regex::Captures, base: &ChronoDateTime<Utc>, _tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let seconds: i64 = caps[1].parse().ok()?;
        base.checked_add_signed(chrono::Duration::try_seconds(seconds).unwrap_or_default())
    }
    
    /// 处理"下一个周几"
    fn handle_next_weekday(caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let weekday_name = &caps[1];
        let target_weekday = Self::parse_weekday_name(weekday_name)?;
        let local = base.with_timezone(tz);
        let current_weekday = local.weekday();
        
        // 计算需要增加的天数
        let days_ahead = (target_weekday.num_days_from_monday() as i32 
            - current_weekday.num_days_from_monday() as i32 + 7) % 7;
        let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };
        
        local
            .checked_add_signed(chrono::Duration::try_days(days_ahead as i64).unwrap_or_default())
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理"上一个周几"
    fn handle_last_weekday(caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let weekday_name = &caps[1];
        let target_weekday = Self::parse_weekday_name(weekday_name)?;
        let local = base.with_timezone(tz);
        let current_weekday = local.weekday();
        
        // 计算需要减少的天数
        let days_behind = (current_weekday.num_days_from_monday() as i32 
            - target_weekday.num_days_from_monday() as i32 + 7) % 7;
        let days_behind = if days_behind == 0 { 7 } else { days_behind };
        
        local
            .checked_sub_signed(chrono::Duration::try_days(days_behind as i64).unwrap_or_default())
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理"下个月第一天"
    fn handle_first_day_next_month(_caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let local = base.with_timezone(tz);
        let current_month = local.month();
        let current_year = local.year();
        
        // 计算下个月
        let (new_year, new_month) = if current_month == 12 {
            (current_year + 1, 1)
        } else {
            (current_year, current_month + 1)
        };
        
        local
            .with_year(new_year)
            .and_then(|d| d.with_month(new_month))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理"本月最后一天"
    fn handle_last_day_this_month(_caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let local = base.with_timezone(tz);
        let last_day = Self::days_in_month(local.year(), local.month());
        
        local
            .with_day(last_day)
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 处理"下个月最后一天"
    fn handle_last_day_next_month(_caps: &regex::Captures, base: &ChronoDateTime<Utc>, tz: &Tz) -> Option<ChronoDateTime<Utc>> {
        let local = base.with_timezone(tz);
        let current_month = local.month();
        let current_year = local.year();
        
        // 计算下个月
        let (new_year, new_month) = if current_month == 12 {
            (current_year + 1, 1)
        } else {
            (current_year, current_month + 1)
        };
        
        let last_day = Self::days_in_month(new_year, new_month);
        
        local
            .with_year(new_year)
            .and_then(|d| d.with_month(new_month))
            .and_then(|d| d.with_day(last_day))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .map(|d| d.with_timezone(&Utc))
    }
    
    /// 解析星期名称
    fn parse_weekday_name(name: &str) -> Option<chrono::Weekday> {
        match name.to_lowercase().as_str() {
            "monday" | "mon" => Some(chrono::Weekday::Mon),
            "tuesday" | "tue" | "tues" => Some(chrono::Weekday::Tue),
            "wednesday" | "wed" => Some(chrono::Weekday::Wed),
            "thursday" | "thu" | "thurs" => Some(chrono::Weekday::Thu),
            "friday" | "fri" => Some(chrono::Weekday::Fri),
            "saturday" | "sat" => Some(chrono::Weekday::Sat),
            "sunday" | "sun" => Some(chrono::Weekday::Sun),
            _ => None,
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
    
    /// 初始化格式字符映射
    fn init_format_chars() -> HashMap<char, FormatCharInfo> {
        let mut map = HashMap::new();
        
        // 日期格式字符
        map.insert('Y', FormatCharInfo { description: "4位年份", can_parse: true });
        map.insert('y', FormatCharInfo { description: "2位年份", can_parse: true });
        map.insert('m', FormatCharInfo { description: "月份（01-12）", can_parse: true });
        map.insert('n', FormatCharInfo { description: "月份（1-12）", can_parse: true });
        map.insert('d', FormatCharInfo { description: "日期（01-31）", can_parse: true });
        map.insert('j', FormatCharInfo { description: "日期（1-31）", can_parse: true });
        map.insert('t', FormatCharInfo { description: "该月天数", can_parse: false });
        
        // 时间格式字符
        map.insert('H', FormatCharInfo { description: "小时（00-23）", can_parse: true });
        map.insert('h', FormatCharInfo { description: "小时（01-12）", can_parse: true });
        map.insert('G', FormatCharInfo { description: "小时（0-23）", can_parse: true });
        map.insert('g', FormatCharInfo { description: "小时（1-12）", can_parse: true });
        map.insert('i', FormatCharInfo { description: "分钟（00-59）", can_parse: true });
        map.insert('s', FormatCharInfo { description: "秒（00-59）", can_parse: true });
        map.insert('u', FormatCharInfo { description: "微秒", can_parse: true });
        
        map
    }
    
    /// 解析格式字符串
    fn parse_format_string(&self, format: &str, datetime: &str) -> Option<ParsedDateTime> {
        let mut parsed = ParsedDateTime::default();
        let mut format_chars = format.chars().peekable();
        let mut datetime_chars = datetime.chars().peekable();
        
        while let Some(&ch) = format_chars.peek() {
            if ch == '%' {
                format_chars.next(); // 跳过 %
                if let Some(&format_char) = format_chars.peek() {
                    format_chars.next(); // 跳过格式字符
                    
                    // 根据格式字符解析
                    match format_char {
                        'Y' => {
                            parsed.year = Some(self.parse_number(&mut datetime_chars, 4)?);
                        }
                        'y' => {
                            let year = self.parse_number(&mut datetime_chars, 2)?;
                            // 2位年份转换
                            parsed.year = Some(if year >= 70 { 1900 + year } else { 2000 + year });
                        }
                        'm' | 'n' => {
                            parsed.month = Some(self.parse_number_varlen(&mut datetime_chars, 1, 2)? as u32);
                        }
                        'd' | 'j' => {
                            parsed.day = Some(self.parse_number_varlen(&mut datetime_chars, 1, 2)? as u32);
                        }
                        'H' | 'h' | 'G' | 'g' => {
                            parsed.hour = Some(self.parse_number_varlen(&mut datetime_chars, 1, 2)? as u32);
                        }
                        'i' => {
                            parsed.minute = Some(self.parse_number(&mut datetime_chars, 2)? as u32);
                        }
                        's' => {
                            parsed.second = Some(self.parse_number(&mut datetime_chars, 2)? as u32);
                        }
                        'u' => {
                            parsed.microsecond = Some(self.parse_number(&mut datetime_chars, 6)? as u32);
                        }
                        _ => {
                            // 跳过未知格式字符
                        }
                    }
                }
            } else {
                // 匹配字面字符
                format_chars.next();
                if datetime_chars.next() != Some(ch) {
                    // 字符不匹配
                    return None;
                }
            }
        }
        
        Some(parsed)
    }
    
    /// 解析固定长度数字
    fn parse_number<I: Iterator<Item = char>>(&self, chars: &mut std::iter::Peekable<I>, len: usize) -> Option<i32> {
        let mut num_str = String::new();
        for _ in 0..len {
            if let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        num_str.parse().ok()
    }
    
    /// 解析可变长度数字
    fn parse_number_varlen<I: Iterator<Item = char>>(&self, chars: &mut std::iter::Peekable<I>, min: usize, max: usize) -> Option<i32> {
        let mut num_str = String::new();
        let mut count = 0;
        
        while count < max {
            if let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    chars.next();
                    count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if count >= min {
            num_str.parse().ok()
        } else {
            None
        }
    }
}

impl Default for DateTimeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析后的日期时间结构
#[derive(Debug, Default)]
struct ParsedDateTime {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
    microsecond: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_now() {
        let parser = DateTimeParser::new();
        let tz = DateTimeZoneValue::utc();
        let (result, _, _) = parser.parse("now", &tz).unwrap();
        assert!(result.timestamp() > 0);
    }
    
    #[test]
    fn test_parse_iso8601() {
        let parser = DateTimeParser::new();
        let tz = DateTimeZoneValue::utc();
        let (result, _, _) = parser.parse("2024-01-15T10:30:00Z", &tz).unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 15);
    }
    
    #[test]
    fn test_parse_relative() {
        let parser = DateTimeParser::new();
        let tz = DateTimeZoneValue::utc();
        let base = Utc::now();
        
        // 测试 "+1 day"
        if let Some(result) = parser.parse_relative_time("+1 day", &base, &Tz::UTC) {
            assert!(result > base);
        }
    }
    
    #[test]
    fn test_parse_timestamp() {
        let parser = DateTimeParser::new();
        let tz = DateTimeZoneValue::utc();
        let (result, _, _) = parser.parse("@1705315800", &tz).unwrap();
        assert_eq!(result.timestamp(), 1705315800);
    }
    
    #[test]
    fn test_parse_from_format() {
        let parser = DateTimeParser::new();
        // 使用正确的格式字符串（带 % 前缀）
        let result = parser.parse_from_format("%Y-%m-%d %H:%i:%s", "2024-01-15 10:30:00", None);
        assert!(result.is_some());
        
        let dt = result.unwrap();
        assert_eq!(dt.get_year(), 2024);
        assert_eq!(dt.get_month(), 1);
        assert_eq!(dt.get_day(), 15);
    }
}
