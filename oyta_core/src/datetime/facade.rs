//! 日期时间门面
//!
//! 提供静态方法访问日期时间功能
//! 对应 ThinkPHP 8.0 的日期时间处理功能
//!
//! # 使用示例
//! ```php
//! // 获取当前时间
//! $now = Date::now();
//!
//! // 格式化日期
//! echo Date::format('Y-m-d H:i:s');
//!
//! // 解析日期字符串
//! $date = Date::parse('2024-01-15');
//! ```

use chrono::{DateTime, Utc, Local, NaiveDate, NaiveDateTime, Duration, TimeZone, Datelike, Timelike};

/// Date 门面结构体
///
/// 提供静态方法访问日期时间功能
/// 所有方法都是无状态的，不需要初始化
pub struct Date;

impl Date {
    /// 获取当前时间（UTC）
    ///
    /// 返回 UTC 时区的当前时间
    ///
    /// # 返回
    /// UTC 时区的当前时间
    ///
    /// # 示例
    /// ```php
    /// $now = Date::now();
    /// ```
    pub fn now() -> DateTime<Utc> {
        // 返回 UTC 时区的当前时间
        Utc::now()
    }

    /// 获取当前本地时间
    ///
    /// 返回本地时区的当前时间
    ///
    /// # 返回
    /// 本地时区的当前时间
    pub fn now_local() -> DateTime<Local> {
        // 返回本地时区的当前时间
        Local::now()
    }

    /// 格式化当前时间
    ///
    /// 使用指定格式格式化当前时间
    ///
    /// # 参数
    /// - `format`: 格式字符串，如 "Y-m-d H:i:s"
    ///
    /// # 返回
    /// 格式化后的时间字符串
    ///
    /// # 示例
    /// ```php
    /// echo Date::format('Y-m-d H:i:s');  // 2024-01-15 10:30:00
    /// ```
    pub fn format(format: &str) -> String {
        // 将 PHP 格式转换为 chrono 格式
        let chrono_format = Self::convert_format(format);
        // 格式化当前本地时间
        Local::now().format(&chrono_format).to_string()
    }

    /// 格式化指定时间
    ///
    /// 使用指定格式格式化给定的时间戳
    ///
    /// # 参数
    /// - `timestamp`: Unix 时间戳（秒）
    /// - `format`: 格式字符串
    ///
    /// # 返回
    /// 格式化后的时间字符串
    pub fn format_timestamp(timestamp: i64, format: &str) -> String {
        // 将 PHP 格式转换为 chrono 格式
        let chrono_format = Self::convert_format(format);
        // 从时间戳创建 DateTime
        match Utc.timestamp_opt(timestamp, 0) {
            // 转换成功
            chrono::offset::LocalResult::Single(dt) => dt.format(&chrono_format).to_string(),
            // 转换失败返回空字符串
            _ => String::new(),
        }
    }

    /// 解析日期时间字符串
    ///
    /// 将日期时间字符串解析为 DateTime 对象
    ///
    /// # 参数
    /// - `date_str`: 日期时间字符串
    /// - `format`: 格式字符串（可选，默认自动检测）
    ///
    /// # 返回
    /// 解析后的 DateTime，失败返回 None
    ///
    /// # 示例
    /// ```php
    /// $date = Date::parse('2024-01-15 10:30:00');
    /// ```
    pub fn parse(date_str: &str) -> Option<DateTime<Utc>> {
        // 尝试多种常见格式解析
        // 格式列表
        let formats = [
            // ISO 8601 格式
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S",
            // 常见日期时间格式
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d %H:%M",
            "%Y-%m-%d",
            "%Y/%m/%d",
        ];
        // 遍历所有格式尝试解析
        for fmt in formats {
            // 尝试解析为 UTC 时间
            if let Ok(dt) = DateTime::parse_from_str(date_str, fmt) {
                // 转换为 UTC 时区
                return Some(dt.with_timezone(&Utc));
            }
            // 尝试解析为本地时间
            if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, fmt) {
                // 假设为 UTC 时间
                return Some(Utc.from_utc_datetime(&dt));
            }
            // 尝试仅解析日期
            if let Ok(d) = NaiveDate::parse_from_str(date_str, fmt) {
                // 创建午夜时间
                let dt = d.and_hms_opt(0, 0, 0)?;
                return Some(Utc.from_utc_datetime(&dt));
            }
        }
        // 所有格式都失败
        None
    }

    /// 使用指定格式解析日期时间
    ///
    /// # 参数
    /// - `date_str`: 日期时间字符串
    /// - `format`: 格式字符串（PHP 格式）
    ///
    /// # 返回
    /// 解析后的 DateTime
    pub fn parse_with_format(date_str: &str, format: &str) -> Option<DateTime<Utc>> {
        // 将 PHP 格式转换为 chrono 格式
        let chrono_format = Self::convert_format(format);
        // 尝试解析
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, &chrono_format) {
            // 转换为 UTC 时间
            return Some(Utc.from_utc_datetime(&dt));
        }
        // 尝试仅解析日期
        if let Ok(d) = NaiveDate::parse_from_str(date_str, &chrono_format) {
            // 创建午夜时间
            let dt = d.and_hms_opt(0, 0, 0)?;
            return Some(Utc.from_utc_datetime(&dt));
        }
        // 解析失败
        None
    }

    /// 获取当前时间戳
    ///
    /// 返回当前 Unix 时间戳（秒）
    ///
    /// # 返回
    /// Unix 时间戳
    pub fn timestamp() -> i64 {
        // 返回当前 UTC 时间的 Unix 时间戳
        Utc::now().timestamp()
    }

    /// 获取当前毫秒时间戳
    ///
    /// 返回当前 Unix 时间戳（毫秒）
    ///
    /// # 返回
    /// 毫秒时间戳
    pub fn timestamp_ms() -> i64 {
        // 返回当前 UTC 时间的毫秒时间戳
        Utc::now().timestamp_millis()
    }

    /// 从时间戳创建 DateTime
    ///
    /// # 参数
    /// - `timestamp`: Unix 时间戳（秒）
    ///
    /// # 返回
    /// DateTime 对象
    pub fn from_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
        // 从时间戳创建 DateTime
        match Utc.timestamp_opt(timestamp, 0) {
            // 转换成功
            chrono::offset::LocalResult::Single(dt) => Some(dt),
            // 转换失败返回 None
            _ => None,
        }
    }

    /// 从毫秒时间戳创建 DateTime
    ///
    /// # 参数
    /// - `timestamp_ms`: 毫秒时间戳
    ///
    /// # 返回
    /// DateTime 对象
    pub fn from_timestamp_ms(timestamp_ms: i64) -> Option<DateTime<Utc>> {
        // 计算秒和纳秒部分
        let secs = timestamp_ms / 1000;
        let nanos = (timestamp_ms % 1000) * 1_000_000;
        // 从时间戳创建 DateTime
        match Utc.timestamp_opt(secs, nanos as u32) {
            // 转换成功
            chrono::offset::LocalResult::Single(dt) => Some(dt),
            // 转换失败返回 None
            _ => None,
        }
    }

    /// 计算两个时间的差值
    ///
    /// 返回两个时间之间的秒数差
    ///
    /// # 参数
    /// - `start`: 开始时间
    /// - `end`: 结束时间
    ///
    /// # 返回
    /// 秒数差（正数表示 end 在 start 之后）
    pub fn diff(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
        // 计算时间差
        (end - start).num_seconds()
    }

    /// 添加天数
    ///
    /// 在指定日期上添加天数
    ///
    /// # 参数
    /// - `date`: 原始日期
    /// - `days`: 要添加的天数（可为负数）
    ///
    /// # 返回
    /// 新的 DateTime
    pub fn add_days(date: DateTime<Utc>, days: i64) -> DateTime<Utc> {
        // 添加天数
        date + Duration::days(days)
    }

    /// 添加小时
    ///
    /// 在指定时间上添加小时
    ///
    /// # 参数
    /// - `date`: 原始时间
    /// - `hours`: 要添加的小时数
    ///
    /// # 返回
    /// 新的 DateTime
    pub fn add_hours(date: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
        // 添加小时
        date + Duration::hours(hours)
    }

    /// 添加分钟
    ///
    /// 在指定时间上添加分钟
    ///
    /// # 参数
    /// - `date`: 原始时间
    /// - `minutes`: 要添加的分钟数
    ///
    /// # 返回
    /// 新的 DateTime
    pub fn add_minutes(date: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
        // 添加分钟
        date + Duration::minutes(minutes)
    }

    /// 添加秒数
    ///
    /// 在指定时间上添加秒数
    ///
    /// # 参数
    /// - `date`: 原始时间
    /// - `seconds`: 要添加的秒数
    ///
    /// # 返回
    /// 新的 DateTime
    pub fn add_seconds(date: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
        // 添加秒数
        date + Duration::seconds(seconds)
    }

    /// 判断是否为今天
    ///
    /// 检查指定日期是否为今天
    ///
    /// # 参数
    /// - `date`: 要检查的日期
    ///
    /// # 返回
    /// 如果是今天返回 true
    pub fn is_today(date: DateTime<Utc>) -> bool {
        // 获取今天的日期
        let today = Utc::now().date_naive();
        // 比较日期部分
        date.date_naive() == today
    }

    /// 判断是否为昨天
    ///
    /// 检查指定日期是否为昨天
    ///
    /// # 参数
    /// - `date`: 要检查的日期
    ///
    /// # 返回
    /// 如果是昨天返回 true
    pub fn is_yesterday(date: DateTime<Utc>) -> bool {
        // 获取昨天的日期
        let yesterday = (Utc::now() - Duration::days(1)).date_naive();
        // 比较日期部分
        date.date_naive() == yesterday
    }

    /// 判断是否为明天
    ///
    /// 检查指定日期是否为明天
    ///
    /// # 参数
    /// - `date`: 要检查的日期
    ///
    /// # 返回
    /// 如果是明天返回 true
    pub fn is_tomorrow(date: DateTime<Utc>) -> bool {
        // 获取明天的日期
        let tomorrow = (Utc::now() + Duration::days(1)).date_naive();
        // 比较日期部分
        date.date_naive() == tomorrow
    }

    /// 获取年份
    ///
    /// 返回指定日期的年份
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 年份
    pub fn year(date: DateTime<Utc>) -> i32 {
        // 使用 Datelike trait 的 year 方法
        date.year()
    }

    /// 获取月份
    ///
    /// 返回指定日期的月份（1-12）
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 月份
    pub fn month(date: DateTime<Utc>) -> u32 {
        // 使用 Datelike trait 的 month 方法
        date.month()
    }

    /// 获取日期
    ///
    /// 返回指定日期的日（1-31）
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 日
    pub fn day(date: DateTime<Utc>) -> u32 {
        // 使用 Datelike trait 的 day 方法
        date.day()
    }

    /// 获取小时
    ///
    /// 返回指定时间的小时（0-23）
    ///
    /// # 参数
    /// - `date`: 时间
    ///
    /// # 返回
    /// 小时
    pub fn hour(date: DateTime<Utc>) -> u32 {
        // 使用 Timelike trait 的 hour 方法
        date.hour()
    }

    /// 获取分钟
    ///
    /// 返回指定时间的分钟（0-59）
    ///
    /// # 参数
    /// - `date`: 时间
    ///
    /// # 返回
    /// 分钟
    pub fn minute(date: DateTime<Utc>) -> u32 {
        // 使用 Timelike trait 的 minute 方法
        date.minute()
    }

    /// 获取秒数
    ///
    /// 返回指定时间的秒数（0-59）
    ///
    /// # 参数
    /// - `date`: 时间
    ///
    /// # 返回
    /// 秒数
    pub fn second(date: DateTime<Utc>) -> u32 {
        // 使用 Timelike trait 的 second 方法
        date.second()
    }

    /// 获取星期几
    ///
    /// 返回星期几（1-7，周一为 1）
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 星期几
    pub fn weekday(date: DateTime<Utc>) -> u32 {
        // chrono 的 weekday() 返回 0-6（周一为 0）
        // 转换为 1-7（周一为 1）
        // 使用 Datelike trait 的 weekday 方法
        date.weekday().number_from_monday()
    }

    /// 获取一年中的第几天
    ///
    /// 返回一年中的第几天（1-366）
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 第几天
    pub fn day_of_year(date: DateTime<Utc>) -> u32 {
        // 使用 Datelike trait 的 ordinal 方法
        date.ordinal()
    }

    /// 获取一年中的第几周
    ///
    /// 返回一年中的第几周（ISO 周）
    ///
    /// # 参数
    /// - `date`: 日期
    ///
    /// # 返回
    /// 第几周
    pub fn week_of_year(date: DateTime<Utc>) -> u32 {
        // 使用 Datelike trait 的 iso_week 方法
        date.iso_week().week()
    }

    /// 将 PHP 日期格式转换为 chrono 格式
    ///
    /// 内部方法，用于格式转换
    ///
    /// # 参数
    /// - `php_format`: PHP 日期格式字符串
    ///
    /// # 返回
    /// chrono 格式字符串
    fn convert_format(php_format: &str) -> String {
        // PHP 格式到 chrono 格式的映射
        let mut result = String::new();
        // 遍历字符进行转换
        let chars: Vec<char> = php_format.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            // 根据字符进行转换
            match c {
                // 年
                'Y' => result.push_str("%Y"),  // 4 位年
                'y' => result.push_str("%y"),  // 2 位年
                // 月
                'm' => result.push_str("%m"),  // 2 位月
                'n' => result.push_str("%-m"), // 无前导零月
                'M' => result.push_str("%b"),  // 月份缩写
                'F' => result.push_str("%B"),  // 月份全称
                // 日
                'd' => result.push_str("%d"),  // 2 位日
                'j' => result.push_str("%-d"), // 无前导零日
                // 时
                'H' => result.push_str("%H"),  // 24 小时制
                'h' => result.push_str("%I"),  // 12 小时制
                'G' => result.push_str("%-H"), // 无前导零 24 小时
                'g' => result.push_str("%-I"), // 无前导零 12 小时
                // 分
                'i' => result.push_str("%M"),  // 分钟
                // 秒
                's' => result.push_str("%S"),  // 秒
                // 星期
                'w' => result.push_str("%w"),  // 星期几（0-6）
                'N' => result.push_str("%u"),  // ISO 星期几（1-7）
                'D' => result.push_str("%a"),  // 星期缩写
                'l' => result.push_str("%A"),  // 星期全称
                // 其他
                'a' => result.push_str("%P"),  // am/pm 小写
                'A' => result.push_str("%p"),  // AM/PM 大写
                'z' => result.push_str("%j"),  // 一年中的第几天
                'W' => result.push_str("%V"),  // ISO 周数
                // 字面量
                _ => result.push(c),
            }
            i += 1;
        }
        result
    }
}
