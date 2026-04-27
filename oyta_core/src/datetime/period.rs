//! DatePeriod 时间周期类实现
//!
//! 提供完整的 PHP DatePeriod 功能
//! 支持：创建时间周期、迭代日期序列等

use chrono::{DateTime as ChronoDateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::datetime::{DateTimeValue, DateTimeImmutableValue};
use super::interval::DateIntervalValue;

/// DatePeriod 值类型
///
/// 表示一个时间周期，可用于迭代日期序列
/// 对应 PHP 的 DatePeriod 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatePeriodValue {
    /// 开始日期
    pub start: DateTimeValue,
    /// 时间间隔
    pub interval: DateIntervalValue,
    /// 结束日期（可选）
    /// 如果设置了结束日期，则 recurrences 被忽略
    pub end: Option<DateTimeValue>,
    /// 重复次数（可选）
    /// 如果设置了重复次数，则迭代指定次数后停止
    pub recurrences: Option<i32>,
    /// 当前迭代位置
    pub current_index: i32,
    /// 是否包含结束日期
    pub include_start_date: bool,
    /// 是否排除开始日期
    pub exclude_start_date: bool,
}

impl DatePeriodValue {
    /// 创建新的时间周期（使用重复次数）
    ///
    /// # 参数
    /// - `start`: 开始日期
    /// - `interval`: 时间间隔
    /// - `recurrences`: 重复次数
    ///
    /// # 返回
    /// DatePeriodValue 实例
    pub fn new_with_recurrences(
        start: DateTimeValue,
        interval: DateIntervalValue,
        recurrences: i32,
    ) -> Self {
        Self {
            start,
            interval,
            end: None,
            recurrences: Some(recurrences.max(1)),
            current_index: 0,
            include_start_date: true,
            exclude_start_date: false,
        }
    }
    
    /// 创建新的时间周期（使用结束日期）
    ///
    /// # 参数
    /// - `start`: 开始日期
    /// - `interval`: 时间间隔
    /// - `end`: 结束日期
    ///
    /// # 返回
    /// DatePeriodValue 实例
    pub fn new_with_end(
        start: DateTimeValue,
        interval: DateIntervalValue,
        end: DateTimeValue,
    ) -> Self {
        Self {
            start,
            interval,
            end: Some(end),
            recurrences: None,
            current_index: 0,
            include_start_date: true,
            exclude_start_date: false,
        }
    }
    
    /// 从 ISO 8601 字符串创建时间周期
    ///
    /// # 参数
    /// - `isostr`: ISO 8601 时间周期字符串
    ///
    /// # 支持的格式
    /// - "R4/2012-07-01T00:00:00Z/P7D": 重复 4 次，从指定日期开始，间隔 7 天
    /// - "2012-07-01T00:00:00Z/P7D/R4": 同上（PHP 风格）
    ///
    /// # 返回
    /// 成功返回 DatePeriodValue，失败返回错误信息
    pub fn from_iso8601(isostr: &str) -> Result<Self, String> {
        Self::parse_iso8601(isostr)
    }
    
    /// 获取开始日期
    pub fn get_start_date(&self) -> &DateTimeValue {
        &self.start
    }
    
    /// 获取结束日期
    pub fn get_end_date(&self) -> Option<&DateTimeValue> {
        self.end.as_ref()
    }
    
    /// 获取时间间隔
    pub fn get_interval(&self) -> &DateIntervalValue {
        &self.interval
    }
    
    /// 获取重复次数
    pub fn get_recurrences(&self) -> Option<i32> {
        self.recurrences
    }
    
    /// 设置是否包含开始日期
    pub fn set_include_start_date(&mut self, include: bool) -> &mut Self {
        self.include_start_date = include;
        self
    }
    
    /// 设置是否排除开始日期
    pub fn set_exclude_start_date(&mut self, exclude: bool) -> &mut Self {
        self.exclude_start_date = exclude;
        self
    }
    
    /// 获取迭代器
    ///
    /// # 返回
    /// DatePeriodIterator 迭代器
    pub fn iter(&self) -> DatePeriodIterator {
        DatePeriodIterator {
            period: self.clone(),
            current: None,
            index: 0,
            finished: false,
        }
    }
    
    /// 转换为日期列表
    ///
    /// # 返回
    /// 包含所有日期的 Vec
    pub fn to_date_list(&self) -> Vec<DateTimeValue> {
        self.iter().collect()
    }
    
    /// 计算日期数量
    ///
    /// # 返回
    /// 周期内的日期数量
    pub fn count(&self) -> i32 {
        if let Some(recurrences) = self.recurrences {
            // 使用重复次数计算
            if self.exclude_start_date {
                recurrences
            } else {
                recurrences + 1
            }
        } else if let Some(end) = &self.end {
            // 使用结束日期计算
            let mut count = 0;
            let mut current = self.start.clone();
            
            while current.inner <= end.inner {
                if !self.exclude_start_date || count > 0 {
                    count += 1;
                }
                current.add(&self.interval);
                
                // 防止无限循环
                if count > 1000000 {
                    break;
                }
            }
            
            count
        } else {
            0
        }
    }
    
    /// 解析 ISO 8601 时间周期字符串
    fn parse_iso8601(isostr: &str) -> Result<Self, String> {
        let s = isostr.trim();
        
        // 尝试解析 Rn/Start/Interval 格式
        if s.starts_with('R') {
            return Self::parse_repeating_format(s);
        }
        
        // 尝试解析 Start/Interval/Rn 格式（PHP 风格）
        if s.contains("/R") {
            return Self::parse_php_format(s);
        }
        
        // 尝试解析 Start/Interval/End 格式
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            let start = DateTimeValue::new(Some(parts[0]), None)
                .map_err(|e| format!("解析开始日期失败: {}", e))?;
            let interval = DateIntervalValue::new(parts[1])
                .map_err(|e| format!("解析间隔失败: {}", e))?;
            let end = DateTimeValue::new(Some(parts[2]), None)
                .map_err(|e| format!("解析结束日期失败: {}", e))?;
            
            return Ok(Self::new_with_end(start, interval, end));
        }
        
        Err(format!("无效的 ISO 8601 时间周期格式: {}", isostr))
    }
    
    /// 解析重复格式 Rn/Start/Interval
    fn parse_repeating_format(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(format!("无效的重复格式: {}", s));
        }
        
        // 解析重复次数（R 后面的数字）
        let recurrences_str = &parts[0][1..]; // 跳过 R
        let recurrences: i32 = recurrences_str.parse()
            .map_err(|_| format!("无效的重复次数: {}", recurrences_str))?;
        
        // 解析开始日期
        let start = DateTimeValue::new(Some(parts[1]), None)
            .map_err(|e| format!("解析开始日期失败: {}", e))?;
        
        // 解析间隔
        let interval = DateIntervalValue::new(parts[2])
            .map_err(|e| format!("解析间隔失败: {}", e))?;
        
        Ok(Self::new_with_recurrences(start, interval, recurrences))
    }
    
    /// 解析 PHP 风格格式 Start/Interval/Rn
    fn parse_php_format(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(format!("无效的 PHP 格式: {}", s));
        }
        
        // 解析开始日期
        let start = DateTimeValue::new(Some(parts[0]), None)
            .map_err(|e| format!("解析开始日期失败: {}", e))?;
        
        // 解析间隔
        let interval = DateIntervalValue::new(parts[1])
            .map_err(|e| format!("解析间隔失败: {}", e))?;
        
        // 解析重复次数
        let recurrences_str = &parts[2][1..]; // 跳过 R
        let recurrences: i32 = recurrences_str.parse()
            .map_err(|_| format!("无效的重复次数: {}", recurrences_str))?;
        
        Ok(Self::new_with_recurrences(start, interval, recurrences))
    }
}

impl fmt::Display for DatePeriodValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(end) = &self.end {
            write!(
                f,
                "DatePeriod({}, {}, {})",
                self.start.format("Y-m-d H:i:s"),
                self.interval.to_iso8601_string(),
                end.format("Y-m-d H:i:s")
            )
        } else if let Some(recurrences) = self.recurrences {
            write!(
                f,
                "DatePeriod({}, {}, {})",
                self.start.format("Y-m-d H:i:s"),
                self.interval.to_iso8601_string(),
                recurrences
            )
        } else {
            write!(
                f,
                "DatePeriod({}, {})",
                self.start.format("Y-m-d H:i:s"),
                self.interval.to_iso8601_string()
            )
        }
    }
}

/// DatePeriod 迭代器
///
/// 用于遍历时间周期中的所有日期
#[derive(Debug, Clone)]
pub struct DatePeriodIterator {
    /// 时间周期引用
    period: DatePeriodValue,
    /// 当前日期
    current: Option<DateTimeValue>,
    /// 当前索引
    index: i32,
    /// 是否已完成
    finished: bool,
}

impl DatePeriodIterator {
    /// 创建新的迭代器
    pub fn new(period: DatePeriodValue) -> Self {
        Self {
            period,
            current: None,
            index: 0,
            finished: false,
        }
    }
    
    /// 重置迭代器
    pub fn reset(&mut self) {
        self.current = None;
        self.index = 0;
        self.finished = false;
    }
    
    /// 获取当前索引
    pub fn current_index(&self) -> i32 {
        self.index
    }
    
    /// 判断是否已完成
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

impl Iterator for DatePeriodIterator {
    type Item = DateTimeValue;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 如果已完成，返回 None
        if self.finished {
            return None;
        }
        
        // 第一次迭代
        if self.current.is_none() {
            // 设置当前日期为开始日期
            self.current = Some(self.period.start.clone());
            
            // 如果排除开始日期，跳到下一个
            if self.period.exclude_start_date {
                let mut next = self.period.start.clone();
                next.add(&self.period.interval);
                self.current = Some(next);
            }
            
            self.index = 0;
            return self.current.clone();
        }
        
        // 检查是否达到重复次数限制
        if let Some(recurrences) = self.period.recurrences {
            // 计算最大索引
            let max_index = if self.period.exclude_start_date {
                recurrences - 1
            } else {
                recurrences
            };
            
            if self.index >= max_index {
                self.finished = true;
                return None;
            }
        }
        
        // 计算下一个日期
        let mut next = self.current.clone()?;
        next.add(&self.period.interval);
        
        // 检查是否超过结束日期
        if let Some(end) = &self.period.end {
            if next.inner > end.inner {
                self.finished = true;
                return None;
            }
        }
        
        // 更新当前日期
        self.current = Some(next.clone());
        self.index += 1;
        
        Some(next)
    }
}

impl DoubleEndedIterator for DatePeriodIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        // 反向迭代：从结束日期开始
        if self.finished {
            return None;
        }
        
        // 如果有结束日期，从结束日期开始反向迭代
        if let Some(end) = &self.period.end {
            // 计算从开始到结束的总步数
            let mut count = 0;
            let mut temp = self.period.start.clone();
            while temp.inner <= end.inner {
                temp.add(&self.period.interval);
                count += 1;
                if count > 1000000 {
                    break;
                }
            }
            
            // 从结束位置开始返回
            if self.index == 0 {
                self.index = count;
            }
            
            if self.index <= 0 {
                self.finished = true;
                return None;
            }
            
            self.index -= 1;
            
            // 计算对应位置的日期
            let mut result = self.period.start.clone();
            for _ in 0..self.index {
                result.add(&self.period.interval);
            }
            
            return Some(result);
        }
        
        None
    }
}

/// DatePeriod 常量
///
/// 对应 PHP 的 DatePeriod 类常量
pub struct DatePeriod;

impl DatePeriod {
    /// 排除开始日期
    pub const EXCLUDE_START_DATE: i32 = 1;
    /// 包含结束日期
    pub const INCLUDE_END_DATE: i32 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_period_with_recurrences() {
        // 测试使用重复次数创建
        let start = DateTimeValue::new(Some("2024-01-01"), None).unwrap();
        let interval = DateIntervalValue::new("P1D").unwrap();
        
        let period = DatePeriodValue::new_with_recurrences(start, interval, 3);
        
        let dates: Vec<_> = period.iter().collect();
        assert_eq!(dates.len(), 4); // 开始日期 + 3 次重复
        assert_eq!(dates[0].format("Y-m-d"), "2024-01-01");
        assert_eq!(dates[1].format("Y-m-d"), "2024-01-02");
        assert_eq!(dates[2].format("Y-m-d"), "2024-01-03");
        assert_eq!(dates[3].format("Y-m-d"), "2024-01-04");
    }
    
    #[test]
    fn test_period_with_end() {
        // 测试使用结束日期创建
        let start = DateTimeValue::new(Some("2024-01-01"), None).unwrap();
        let interval = DateIntervalValue::new("P1D").unwrap();
        let end = DateTimeValue::new(Some("2024-01-05"), None).unwrap();
        
        let period = DatePeriodValue::new_with_end(start, interval, end);
        
        let dates: Vec<_> = period.iter().collect();
        assert_eq!(dates.len(), 5);
    }
    
    #[test]
    fn test_period_from_iso8601() {
        // 测试从 ISO 8601 字符串创建
        let period = DatePeriodValue::from_iso8601("R3/2024-01-01/P1D").unwrap();
        
        assert_eq!(period.recurrences, Some(3));
        assert_eq!(period.interval.days, 1);
    }
    
    #[test]
    fn test_period_exclude_start() {
        // 测试排除开始日期
        let start = DateTimeValue::new(Some("2024-01-01"), None).unwrap();
        let interval = DateIntervalValue::new("P1D").unwrap();
        
        let mut period = DatePeriodValue::new_with_recurrences(start, interval, 3);
        period.set_exclude_start_date(true);
        
        let dates: Vec<_> = period.iter().collect();
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0].format("Y-m-d"), "2024-01-02");
    }
    
    #[test]
    fn test_period_count() {
        // 测试计数
        let start = DateTimeValue::new(Some("2024-01-01"), None).unwrap();
        let interval = DateIntervalValue::new("P1D").unwrap();
        
        let period = DatePeriodValue::new_with_recurrences(start, interval, 5);
        assert_eq!(period.count(), 6);
    }
}
