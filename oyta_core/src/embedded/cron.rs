//! Cron 表达式解析器
//!
//! 解析和计算 Cron 表达式
//! 支持标准的 5 字段 Cron 格式
//!
//! # 格式
//! ```text
//! *    *    *    *    *
//! ┬    ┬    ┬    ┬    ┬
//! │    │    │    │    └─── 星期几 (0-6, 0=周日)
//! │    │    │    └──────── 月份 (1-12)
//! │    │    └───────────── 日期 (1-31)
//! │    └────────────────── 小时 (0-23)
//! └─────────────────────── 分钟 (0-59)
//! ```
//!
//! # 特殊字符
//! - `*` - 任意值
//! - `,` - 值列表分隔符
//! - `-` - 范围
//! - `/` - 步长
//!
//! # 示例
//! - `* * * * *` - 每分钟
//! - `0 * * * *` - 每小时
//! - `0 0 * * *` - 每天午夜
//! - `0 2 * * *` - 每天凌晨 2 点
//! - `0 0 * * 0` - 每周日午夜
//! - `0 0 1 * *` - 每月 1 号午夜
//! - `*/15 * * * *` - 每 15 分钟
//! - `0 9-17 * * 1-5` - 周一到周五 9 点到 17 点

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, Timelike};
use std::str::FromStr;

/// Cron 表达式
#[derive(Debug, Clone)]
pub struct CronExpression {
    /// 分钟字段
    minute: CronField,
    /// 小时字段
    hour: CronField,
    /// 日期字段
    day_of_month: CronField,
    /// 月份字段
    month: CronField,
    /// 星期字段
    day_of_week: CronField,
}

/// Cron 字段
#[derive(Debug, Clone)]
pub struct CronField {
    /// 是否匹配所有值
    is_all: bool,
    /// 匹配的值集合
    values: Vec<u32>,
    /// 步长
    step: u32,
    /// 范围
    range: Option<(u32, u32)>,
}

impl CronField {
    /// 创建匹配所有值的字段
    pub fn all() -> Self {
        Self {
            is_all: true,
            values: Vec::new(),
            step: 1,
            range: None,
        }
    }

    /// 创建匹配特定值的字段
    pub fn values(vals: Vec<u32>) -> Self {
        Self {
            is_all: false,
            values: vals,
            step: 1,
            range: None,
        }
    }

    /// 创建带步长的字段
    pub fn step(step: u32) -> Self {
        Self {
            is_all: true,
            values: Vec::new(),
            step,
            range: None,
        }
    }

    /// 创建范围字段
    pub fn range(start: u32, end: u32) -> Self {
        Self {
            is_all: false,
            values: Vec::new(),
            step: 1,
            range: Some((start, end)),
        }
    }

    /// 检查值是否匹配
    pub fn matches(&self, value: u32, min: u32, _max: u32) -> bool {
        if self.is_all {
            if self.step > 1 {
                return (value - min) % self.step == 0;
            }
            return true;
        }

        if !self.values.is_empty() {
            return self.values.contains(&value);
        }

        if let Some((start, end)) = self.range {
            if self.step > 1 {
                for v in (start..=end).step_by(self.step as usize) {
                    if v == value {
                        return true;
                    }
                }
                return false;
            }
            return value >= start && value <= end;
        }

        false
    }
}

impl CronExpression {
    /// 解析 Cron 表达式
    ///
    /// # 参数
    /// - `expr`: Cron 表达式字符串
    ///
    /// # 返回
    /// 解析后的 CronExpression 结构
    ///
    /// # 示例
    /// ```ignore
    /// let cron = CronExpression::parse("0 2 * * *")?;
    /// ```
    pub fn parse(expr: &str) -> Result<Self> {
        let parts: Vec<&str> = expr.split_whitespace().collect();

        if parts.len() != 5 {
            anyhow::bail!("Cron 表达式必须有 5 个字段: {}", expr);
        }

        Ok(Self {
            minute: Self::parse_field(parts[0], 0, 59)?,
            hour: Self::parse_field(parts[1], 0, 23)?,
            day_of_month: Self::parse_field(parts[2], 1, 31)?,
            month: Self::parse_field(parts[3], 1, 12)?,
            day_of_week: Self::parse_field(parts[4], 0, 6)?,
        })
    }

    /// 解析单个字段
    fn parse_field(field: &str, min: u32, max: u32) -> Result<CronField> {
        // 处理 *
        if field == "*" {
            return Ok(CronField::all());
        }

        // 处理 */step
        if field.starts_with("*/") {
            let step = field[2..]
                .parse::<u32>()
                .with_context(|| format!("无效的步长: {}", field))?;
            return Ok(CronField { is_all: true, values: Vec::new(), step, range: None });
        }

        // 处理逗号分隔的值列表
        if field.contains(',') {
            let mut values = Vec::new();
            for part in field.split(',') {
                let val = Self::parse_single_value(part, min, max)?;
                values.push(val);
            }
            return Ok(CronField::values(values));
        }

        // 处理范围
        if field.contains('-') {
            let parts: Vec<&str> = field.split('-').collect();
            if parts.len() != 2 {
                anyhow::bail!("无效的范围: {}", field);
            }

            let start = Self::parse_single_value(parts[0], min, max)?;
            let end = Self::parse_single_value(parts[1], min, max)?;

            // 检查是否有步长
            if parts[1].contains('/') {
                let end_parts: Vec<&str> = parts[1].split('/').collect();
                let end_val = Self::parse_single_value(end_parts[0], min, max)?;
                let step = end_parts[1].parse::<u32>()?;
                return Ok(CronField {
                    is_all: false,
                    values: Vec::new(),
                    step,
                    range: Some((start, end_val)),
                });
            }

            return Ok(CronField::range(start, end));
        }

        // 处理单个值
        let value = Self::parse_single_value(field, min, max)?;
        Ok(CronField::values(vec![value]))
    }

    /// 解析单个值
    fn parse_single_value(value: &str, min: u32, max: u32) -> Result<u32> {
        let num = value
            .parse::<u32>()
            .with_context(|| format!("无效的值: {}", value))?;

        if num < min || num > max {
            anyhow::bail!("值 {} 超出范围 [{}, {}]", num, min, max);
        }

        Ok(num)
    }

    /// 检查指定时间是否匹配 Cron 表达式
    ///
    /// # 参数
    /// - `datetime`: 要检查的时间
    ///
    /// # 返回
    /// 是否匹配
    pub fn matches(&self, datetime: &DateTime<Local>) -> bool {
        // 检查分钟
        if !self.minute.matches(datetime.minute() as u32, 0, 59) {
            return false;
        }

        // 检查小时
        if !self.hour.matches(datetime.hour() as u32, 0, 23) {
            return false;
        }

        // 检查日期
        if !self.day_of_month.matches(datetime.day() as u32, 1, 31) {
            return false;
        }

        // 检查月份
        if !self.month.matches(datetime.month() as u32, 1, 12) {
            return false;
        }

        // 检查星期
        let weekday = datetime.weekday().num_days_from_sunday();
        if !self.day_of_week.matches(weekday, 0, 6) {
            return false;
        }

        true
    }

    /// 计算下一次执行时间
    ///
    /// # 参数
    /// - `from`: 起始时间
    ///
    /// # 返回
    /// 下一次匹配的时间
    pub fn next_run(&self, from: &DateTime<Local>) -> Option<DateTime<Local>> {
        let mut current = from.clone();

        // 最多检查 366 天
        for _ in 0..366 * 24 * 60 {
            current = current + chrono::Duration::minutes(1);

            if self.matches(&current) {
                return Some(current);
            }
        }

        None
    }

    /// 计算接下来 N 次执行时间
    ///
    /// # 参数
    /// - `from`: 起始时间
    /// - `count`: 次数
    ///
    /// # 返回
    /// 执行时间列表
    pub fn next_runs(&self, from: &DateTime<Local>, count: usize) -> Vec<DateTime<Local>> {
        let mut result = Vec::new();
        let mut current = from.clone();

        for _ in 0..count {
            if let Some(next) = self.next_run(&current) {
                result.push(next);
                current = next;
            } else {
                break;
            }
        }

        result
    }

    /// 获取人类可读的描述
    pub fn description(&self) -> String {
        // 简化的描述生成
        if self.minute.is_all && self.hour.is_all && self.day_of_month.is_all
            && self.month.is_all && self.day_of_week.is_all
        {
            return "每分钟".to_string();
        }

        if self.minute.values == vec![0] && self.hour.is_all {
            return "每小时整点".to_string();
        }

        if self.minute.values == vec![0] && self.hour.values.len() == 1 {
            return format!("每天 {}:00", self.hour.values[0]);
        }

        "自定义时间".to_string()
    }
}

impl FromStr for CronExpression {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// 预定义的 Cron 表达式
pub struct Cron;

impl Cron {
    /// 每分钟
    pub fn every_minute() -> CronExpression {
        CronExpression::parse("* * * * *").unwrap()
    }

    /// 每小时
    pub fn hourly() -> CronExpression {
        CronExpression::parse("0 * * * *").unwrap()
    }

    /// 每天午夜
    pub fn daily() -> CronExpression {
        CronExpression::parse("0 0 * * *").unwrap()
    }

    /// 每天指定时间
    pub fn daily_at(hour: u32, minute: u32) -> CronExpression {
        CronExpression::parse(&format!("{} {} * * *", minute, hour)).unwrap()
    }

    /// 每周
    pub fn weekly() -> CronExpression {
        CronExpression::parse("0 0 * * 0").unwrap()
    }

    /// 每周指定时间和星期
    pub fn weekly_on(weekday: u32, hour: u32, minute: u32) -> CronExpression {
        CronExpression::parse(&format!("{} {} * * {}", minute, hour, weekday)).unwrap()
    }

    /// 每月
    pub fn monthly() -> CronExpression {
        CronExpression::parse("0 0 1 * *").unwrap()
    }

    /// 每月指定日期和时间
    pub fn monthly_on(day: u32, hour: u32, minute: u32) -> CronExpression {
        CronExpression::parse(&format!("{} {} {} * *", minute, hour, day)).unwrap()
    }

    /// 每年
    pub fn yearly() -> CronExpression {
        CronExpression::parse("0 0 1 1 *").unwrap()
    }

    /// 工作日（周一到周五）指定时间
    pub fn weekdays_at(hour: u32, minute: u32) -> CronExpression {
        CronExpression::parse(&format!("{} {} * * 1-5", minute, hour)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron() {
        let cron = CronExpression::parse("0 2 * * *").unwrap();
        assert!(cron.hour.values.contains(&2));
    }

    #[test]
    fn test_cron_every_minute() {
        let cron = Cron::every_minute();
        let now = Local::now();
        let next = cron.next_run(&now);
        assert!(next.is_some());
    }

    #[test]
    fn test_cron_daily() {
        let cron = Cron::daily();
        let now = Local::now();
        let next = cron.next_run(&now);
        assert!(next.is_some());
    }

    #[test]
    fn test_cron_matches() {
        let cron = CronExpression::parse("30 14 * * *").unwrap();

        // 创建一个 14:30 的时间
        let time = Local::now()
            .with_hour(14)
            .unwrap()
            .with_minute(30)
            .unwrap();

        assert!(cron.matches(&time));
    }

    #[test]
    fn test_cron_description() {
        assert_eq!(Cron::every_minute().description(), "每分钟");
        assert_eq!(Cron::hourly().description(), "每小时整点");
    }
}
