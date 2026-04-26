//! Cron 表达式解析模块
//!
//! 提供标准 Cron 表达式的解析和计算功能

use anyhow::{anyhow, Result};
// 导入 chrono 的日期时间相关 trait
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};

/// Cron 表达式解析器
///
/// 支持标准的 5 字段 Cron 表达式：
/// - 分钟 (0-59)
/// - 小时 (0-23)
/// - 日 (1-31)
/// - 月 (1-12)
/// - 星期 (0-7, 0 和 7 都表示周日)
///
/// # 示例
/// - `* * * * *` - 每分钟
/// - `0 * * * *` - 每小时
/// - `0 0 * * *` - 每天 00:00
/// - `0 0 * * 0` - 每周日 00:00
/// - `0 0 1 * *` - 每月 1 日 00:00
#[derive(Debug, Clone)]
pub struct CronExpression {
    /// 分钟字段
    minute: CronField,
    /// 小时字段
    hour: CronField,
    /// 日字段
    day: CronField,
    /// 月字段
    month: CronField,
    /// 星期字段
    weekday: CronField,
}

/// Cron 字段
#[derive(Debug, Clone)]
struct CronField {
    /// 允许的值集合
    values: Vec<u8>,
}

impl CronField {
    /// 创建新的 Cron 字段
    fn new(values: Vec<u8>) -> Self {
        Self { values }
    }

    /// 检查值是否匹配
    fn matches(&self, value: u8) -> bool {
        self.values.contains(&value)
    }

    /// 获取下一个匹配值
    fn next_match(&self, current: u8, max: u8, wrap: bool) -> Option<u8> {
        // 在当前值之后查找匹配
        for &v in &self.values {
            if v > current {
                return Some(v);
            }
        }

        // 如果允许回绕，返回第一个匹配值
        if wrap && !self.values.is_empty() {
            return Some(self.values[0]);
        }

        None
    }
}

impl CronExpression {
    /// 解析 Cron 表达式
    ///
    /// # 参数
    /// - `expression`: Cron 表达式字符串
    ///
    /// # 返回值
    /// 解析后的 CronExpression 实例
    pub fn parse(expression: &str) -> Result<Self> {
        // 分割表达式
        let parts: Vec<&str> = expression.split_whitespace().collect();

        // 检查字段数量
        if parts.len() != 5 {
            return Err(anyhow!(
                "Cron 表达式必须有 5 个字段，当前有 {} 个",
                parts.len()
            ));
        }

        // 解析各字段
        let minute = Self::parse_field(parts[0], 0, 59)?;
        let hour = Self::parse_field(parts[1], 0, 23)?;
        let day = Self::parse_field(parts[2], 1, 31)?;
        let month = Self::parse_field(parts[3], 1, 12)?;
        let weekday = Self::parse_field(parts[4], 0, 7)?;

        Ok(Self {
            minute,
            hour,
            day,
            month,
            weekday,
        })
    }

    /// 解析单个字段
    ///
    /// # 参数
    /// - `field`: 字段字符串
    /// - `min`: 最小值
    /// - `max`: 最大值
    ///
    /// # 返回值
    /// CronField 实例
    fn parse_field(field: &str, min: u8, max: u8) -> Result<CronField> {
        let mut values = Vec::new();

        // 处理逗号分隔的多个值
        for part in field.split(',') {
            let part = part.trim();

            if part == "*" {
                // 通配符：所有值
                for v in min..=max {
                    values.push(v);
                }
            } else if part.contains('/') {
                // 步长表达式：*/n 或 start-end/n
                let parts: Vec<&str> = part.split('/').collect();
                if parts.len() != 2 {
                    return Err(anyhow!("无效的步长表达式: {}", part));
                }

                let step: u8 = parts[1].parse()?;
                let range_parts: Vec<&str> = parts[0].split('-').collect();

                let (start, end) = if range_parts.len() == 2 {
                    (range_parts[0].parse()?, range_parts[1].parse()?)
                } else if range_parts[0] == "*" {
                    (min, max)
                } else {
                    (range_parts[0].parse()?, max)
                };

                // 生成范围内的值
                let mut v = start;
                while v <= end {
                    if v >= min && v <= max {
                        values.push(v);
                    }
                    v = v.saturating_add(step);
                    if v > max {
                        break;
                    }
                }
            } else if part.contains('-') {
                // 范围表达式：start-end
                let parts: Vec<&str> = part.split('-').collect();
                if parts.len() != 2 {
                    return Err(anyhow!("无效的范围表达式: {}", part));
                }

                let start: u8 = parts[0].parse()?;
                let end: u8 = parts[1].parse()?;

                for v in start..=end {
                    if v >= min && v <= max {
                        values.push(v);
                    }
                }
            } else {
                // 单个值
                let value: u8 = part.parse()?;
                if value < min || value > max {
                    return Err(anyhow!("值 {} 超出范围 {}-{}", value, min, max));
                }
                values.push(value);
            }
        }

        // 排序并去重
        values.sort();
        values.dedup();

        Ok(CronField::new(values))
    }

    /// 计算下次执行时间
    ///
    /// # 参数
    /// - `from`: 起始时间
    ///
    /// # 返回值
    /// 下次执行时间
    pub fn next_run(&self, from: &DateTime<Local>) -> Option<DateTime<Local>> {
        // 从下一分钟开始计算
        let mut next = from.clone()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            + chrono::Duration::minutes(1);

        // 最多尝试 366 天（一年）
        for _ in 0..525600 {
            // 检查月份
            if !self.month.matches(next.month() as u8) {
                // 跳到下个月
                next = next
                    .with_day(1)
                    .unwrap()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    + chrono::Duration::days(31);
                continue;
            }

            // 检查日和星期（两者都需要匹配）
            let day_matches = self.day.matches(next.day() as u8);
            let weekday_matches = self.weekday.matches(next.weekday().num_days_from_sunday() as u8);

            // 如果两个字段都有非通配符值，则使用 OR 逻辑
            // 否则使用 AND 逻辑
            let day_field_has_values = !self.day.values.is_empty() && self.day.values.len() < 31;
            let weekday_field_has_values = !self.weekday.values.is_empty() && self.weekday.values.len() < 7;

            let date_matches = if day_field_has_values && weekday_field_has_values {
                day_matches || weekday_matches
            } else {
                day_matches && weekday_matches
            };

            if !date_matches {
                // 跳到下一天
                next = next
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    + chrono::Duration::days(1);
                continue;
            }

            // 检查小时
            if !self.hour.matches(next.hour() as u8) {
                // 跳到下一小时
                next = next
                    .with_minute(0)
                    .unwrap()
                    + chrono::Duration::hours(1);
                continue;
            }

            // 检查分钟
            if !self.minute.matches(next.minute() as u8) {
                // 跳到下一分钟
                next = next + chrono::Duration::minutes(1);
                continue;
            }

            // 所有条件都满足
            return Some(next);
        }

        None
    }

    /// 获取字段的可读描述
    ///
    /// # 返回值
    /// 描述字符串
    pub fn description(&self) -> String {
        // 简单描述生成
        if self.minute.values.len() == 60
            && self.hour.values.len() == 24
            && self.day.values.len() == 31
            && self.month.values.len() == 12
        {
            return "每分钟".to_string();
        }

        if self.minute.values.len() == 1 && self.minute.values[0] == 0 {
            if self.hour.values.len() == 24 {
                return "每小时整点".to_string();
            }
            if self.hour.values.len() == 1 {
                return format!("每天 {:02}:00", self.hour.values[0]);
            }
        }

        "自定义计划".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试解析每分钟表达式
    #[test]
    fn test_parse_every_minute() {
        let cron = CronExpression::parse("* * * * *").unwrap();
        assert_eq!(cron.description(), "每分钟");
    }

    /// 测试解析每小时表达式
    #[test]
    fn test_parse_hourly() {
        let cron = CronExpression::parse("0 * * * *").unwrap();
        assert_eq!(cron.description(), "每小时整点");
    }

    /// 测试解析每天表达式
    #[test]
    fn test_parse_daily() {
        let cron = CronExpression::parse("0 0 * * *").unwrap();
        assert!(cron.description().contains("每天"));
    }

    /// 测试计算下次执行时间
    #[test]
    fn test_next_run() {
        let cron = CronExpression::parse("0 * * * *").unwrap();
        let now = Local::now();
        let next = cron.next_run(&now);

        assert!(next.is_some());
        if let Some(n) = next {
            assert_eq!(n.minute(), 0);
        }
    }

    /// 测试范围表达式
    #[test]
    fn test_range_expression() {
        let cron = CronExpression::parse("0 9-17 * * *").unwrap();
        assert!(cron.hour.matches(9));
        assert!(cron.hour.matches(12));
        assert!(cron.hour.matches(17));
        assert!(!cron.hour.matches(8));
        assert!(!cron.hour.matches(18));
    }

    /// 测试步长表达式
    #[test]
    fn test_step_expression() {
        let cron = CronExpression::parse("*/15 * * * *").unwrap();
        assert!(cron.minute.matches(0));
        assert!(cron.minute.matches(15));
        assert!(cron.minute.matches(30));
        assert!(cron.minute.matches(45));
        assert!(!cron.minute.matches(10));
    }
}
