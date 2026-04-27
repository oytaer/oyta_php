//! 输出处理器标志定义模块
//!
//! 定义 PHP 输出缓冲控制相关的标志常量
//! 对应 PHP 的 PHP_OUTPUT_HANDLER_* 常量

use serde::{Deserialize, Serialize};
use std::fmt;

/// 输出处理器标志
///
/// 使用位标志设计，支持组合多个标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutputHandlerFlags(pub u32);

impl OutputHandlerFlags {
    // ============ 处理器状态标志 ============
    
    /// PHP_OUTPUT_HANDLER_START
    /// 表示缓冲区刚刚启动
    pub const START: OutputHandlerFlags = OutputHandlerFlags(1);
    
    /// PHP_OUTPUT_HANDLER_WRITE
    /// 表示处理器正在写入数据
    pub const WRITE: OutputHandlerFlags = OutputHandlerFlags(2);
    
    /// PHP_OUTPUT_HANDLER_FLUSH
    /// 表示处理器正在刷新数据
    pub const FLUSH: OutputHandlerFlags = OutputHandlerFlags(4);
    
    /// PHP_OUTPUT_HANDLER_CLEAN
    /// 表示处理器正在清空数据
    pub const CLEAN: OutputHandlerFlags = OutputHandlerFlags(8);
    
    /// PHP_OUTPUT_HANDLER_FINAL
    /// 表示处理器即将被移除
    pub const FINAL: OutputHandlerFlags = OutputHandlerFlags(16);
    
    /// PHP_OUTPUT_HANDLER_CONT
    /// 表示处理器将继续处理
    pub const CONT: OutputHandlerFlags = OutputHandlerFlags(32);
    
    // ============ 处理器能力标志 ============
    
    /// PHP_OUTPUT_HANDLER_REMOVABLE
    /// 表示处理器可以被移除
    pub const REMOVABLE: OutputHandlerFlags = OutputHandlerFlags(64);
    
    /// PHP_OUTPUT_HANDLER_CLEANABLE
    /// 表示处理器可以被清空
    pub const CLEANABLE: OutputHandlerFlags = OutputHandlerFlags(128);
    
    /// PHP_OUTPUT_HANDLER_FLUSHABLE
    /// 表示处理器可以被刷新
    pub const FLUSHABLE: OutputHandlerFlags = OutputHandlerFlags(256);
    
    // ============ 预定义组合标志 ============
    
    /// PHP_OUTPUT_HANDLER_STDFLAGS
    /// 标准标志组合：START | WRITE | CLEAN | FLUSH | REMOVABLE | CLEANABLE | FLUSHABLE
    pub const STDFLAGS: OutputHandlerFlags = OutputHandlerFlags(
        Self::START.0 | Self::WRITE.0 | Self::CLEAN.0 | Self::FLUSH.0 
        | Self::REMOVABLE.0 | Self::CLEANABLE.0 | Self::FLUSHABLE.0
    );
    
    /// PHP_OUTPUT_HANDLER_STARTED
    /// 表示处理器已启动
    pub const STARTED: OutputHandlerFlags = OutputHandlerFlags(4096);
    
    /// PHP_OUTPUT_HANDLER_DISABLED
    /// 表示处理器已禁用
    pub const DISABLED: OutputHandlerFlags = OutputHandlerFlags(8192);
    
    /// PHP_OUTPUT_HANDLER_PROCESSED
    /// 表示处理器已处理
    pub const PROCESSED: OutputHandlerFlags = OutputHandlerFlags(16384);
    
    // ============ 方法实现 ============
    
    /// 创建新的标志组合
    pub fn new(flags: u32) -> Self {
        OutputHandlerFlags(flags)
    }
    
    /// 空标志
    pub fn empty() -> Self {
        OutputHandlerFlags(0)
    }
    
    /// 检查是否包含指定标志
    pub fn contains(&self, other: OutputHandlerFlags) -> bool {
        (self.0 & other.0) != 0
    }
    
    /// 添加标志
    pub fn add(&self, other: OutputHandlerFlags) -> Self {
        OutputHandlerFlags(self.0 | other.0)
    }
    
    /// 移除标志
    pub fn remove(&self, other: OutputHandlerFlags) -> Self {
        OutputHandlerFlags(self.0 & !other.0)
    }
    
    /// 切换标志
    pub fn toggle(&self, other: OutputHandlerFlags) -> Self {
        OutputHandlerFlags(self.0 ^ other.0)
    }
    
    /// 获取原始值
    pub fn value(&self) -> u32 {
        self.0
    }
    
    /// 检查是否可移除
    pub fn is_removable(&self) -> bool {
        self.contains(Self::REMOVABLE)
    }
    
    /// 检查是否可清空
    pub fn is_cleanable(&self) -> bool {
        self.contains(Self::CLEANABLE)
    }
    
    /// 检查是否可刷新
    pub fn is_flushable(&self) -> bool {
        self.contains(Self::FLUSHABLE)
    }
    
    /// 检查是否已启动
    pub fn is_started(&self) -> bool {
        self.contains(Self::STARTED)
    }
    
    /// 检查是否已禁用
    pub fn is_disabled(&self) -> bool {
        self.contains(Self::DISABLED)
    }
    
    /// 获取标志名称列表
    pub fn flag_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        
        if self.contains(Self::START) { names.push("START"); }
        if self.contains(Self::WRITE) { names.push("WRITE"); }
        if self.contains(Self::FLUSH) { names.push("FLUSH"); }
        if self.contains(Self::CLEAN) { names.push("CLEAN"); }
        if self.contains(Self::FINAL) { names.push("FINAL"); }
        if self.contains(Self::CONT) { names.push("CONT"); }
        if self.contains(Self::REMOVABLE) { names.push("REMOVABLE"); }
        if self.contains(Self::CLEANABLE) { names.push("CLEANABLE"); }
        if self.contains(Self::FLUSHABLE) { names.push("FLUSHABLE"); }
        if self.contains(Self::STARTED) { names.push("STARTED"); }
        if self.contains(Self::DISABLED) { names.push("DISABLED"); }
        if self.contains(Self::PROCESSED) { names.push("PROCESSED"); }
        
        names
    }
}

impl fmt::Display for OutputHandlerFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.flag_names();
        if names.is_empty() {
            write!(f, "NONE")
        } else {
            write!(f, "{}", names.join(" | "))
        }
    }
}

impl Default for OutputHandlerFlags {
    fn default() -> Self {
        Self::STDFLAGS
    }
}

impl From<u32> for OutputHandlerFlags {
    fn from(value: u32) -> Self {
        OutputHandlerFlags(value)
    }
}

impl From<OutputHandlerFlags> for u32 {
    fn from(flags: OutputHandlerFlags) -> Self {
        flags.0
    }
}

impl std::ops::BitOr for OutputHandlerFlags {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::BitAnd for OutputHandlerFlags {
    type Output = Self;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        OutputHandlerFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for OutputHandlerFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for OutputHandlerFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// 输出处理器状态
///
/// 用于传递给回调函数的状态信息
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutputHandlerStatus(pub u32);

impl OutputHandlerStatus {
    /// PHP_OUTPUT_HANDLER_START
    /// 输出缓冲开始
    pub const START: OutputHandlerStatus = OutputHandlerStatus(1);
    
    /// PHP_OUTPUT_HANDLER_WRITE
    /// 输出缓冲写入
    pub const WRITE: OutputHandlerStatus = OutputHandlerStatus(2);
    
    /// PHP_OUTPUT_HANDLER_FLUSH
    /// 输出缓冲刷新
    pub const FLUSH: OutputHandlerStatus = OutputHandlerStatus(4);
    
    /// PHP_OUTPUT_HANDLER_CLEAN
    /// 输出缓冲清空
    pub const CLEAN: OutputHandlerStatus = OutputHandlerStatus(8);
    
    /// PHP_OUTPUT_HANDLER_FINAL
    /// 输出缓冲结束
    pub const FINAL: OutputHandlerStatus = OutputHandlerStatus(16);
    
    /// 获取原始值
    pub fn value(&self) -> u32 {
        self.0
    }
    
    /// 检查是否为开始状态
    pub fn is_start(&self) -> bool {
        self.0 == Self::START.0
    }
    
    /// 检查是否为写入状态
    pub fn is_write(&self) -> bool {
        self.0 == Self::WRITE.0
    }
    
    /// 检查是否为刷新状态
    pub fn is_flush(&self) -> bool {
        self.0 == Self::FLUSH.0
    }
    
    /// 检查是否为清空状态
    pub fn is_clean(&self) -> bool {
        self.0 == Self::CLEAN.0
    }
    
    /// 检查是否为结束状态
    pub fn is_final(&self) -> bool {
        self.0 == Self::FINAL.0
    }
}

impl fmt::Display for OutputHandlerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::START => write!(f, "START"),
            Self::WRITE => write!(f, "WRITE"),
            Self::FLUSH => write!(f, "FLUSH"),
            Self::CLEAN => write!(f, "CLEAN"),
            Self::FINAL => write!(f, "FINAL"),
            _ => write!(f, "UNKNOWN({})", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flags_contains() {
        let flags = OutputHandlerFlags::STDFLAGS;
        
        assert!(flags.contains(OutputHandlerFlags::START));
        assert!(flags.contains(OutputHandlerFlags::REMOVABLE));
        assert!(flags.contains(OutputHandlerFlags::CLEANABLE));
    }
    
    #[test]
    fn test_flags_add_remove() {
        let flags = OutputHandlerFlags::START;
        
        // 添加标志
        let flags = flags.add(OutputHandlerFlags::WRITE);
        assert!(flags.contains(OutputHandlerFlags::START));
        assert!(flags.contains(OutputHandlerFlags::WRITE));
        
        // 移除标志
        let flags = flags.remove(OutputHandlerFlags::START);
        assert!(!flags.contains(OutputHandlerFlags::START));
        assert!(flags.contains(OutputHandlerFlags::WRITE));
    }
    
    #[test]
    fn test_flags_bitor() {
        let flags = OutputHandlerFlags::START | OutputHandlerFlags::WRITE;
        
        assert!(flags.contains(OutputHandlerFlags::START));
        assert!(flags.contains(OutputHandlerFlags::WRITE));
    }
    
    #[test]
    fn test_flags_is_removable() {
        let flags = OutputHandlerFlags::STDFLAGS;
        assert!(flags.is_removable());
        assert!(flags.is_cleanable());
        assert!(flags.is_flushable());
    }
    
    #[test]
    fn test_flags_display() {
        let flags = OutputHandlerFlags::START | OutputHandlerFlags::WRITE;
        let display = format!("{}", flags);
        
        assert!(display.contains("START"));
        assert!(display.contains("WRITE"));
    }
    
    #[test]
    fn test_handler_status() {
        let status = OutputHandlerStatus::START;
        assert!(status.is_start());
        assert!(!status.is_flush());
        
        let status = OutputHandlerStatus::FLUSH;
        assert!(status.is_flush());
        assert!(!status.is_clean());
    }
    
    #[test]
    fn test_default_flags() {
        let flags = OutputHandlerFlags::default();
        
        // 默认标志应该包含标准能力
        assert!(flags.is_removable());
        assert!(flags.is_cleanable());
        assert!(flags.is_flushable());
    }
}
