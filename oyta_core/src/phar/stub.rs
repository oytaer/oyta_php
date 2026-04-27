//! Phar Stub 模块
//! 
//! 本模块实现 Phar Stub 的处理，包括：
//! - Stub 生成
//! - Stub 解析
//! - 默认 Stub 模板

use std::io::{Read, Write};

/// Phar Stub
/// 
/// 表示 Phar 文件的启动存根
#[derive(Debug, Clone)]
pub struct PharStub {
    /// Stub 代码
    code: Vec<u8>,
    /// 是否使用默认 stub
    use_default: bool,
    /// 入口文件
    entry_point: Option<String>,
    /// 映射文件
    map: Vec<(String, String)>,
}

impl PharStub {
    /// 创建新的 Stub
    pub fn new() -> Self {
        PharStub {
            code: Vec::new(),
            use_default: true,
            entry_point: None,
            map: Vec::new(),
        }
    }

    /// 从代码创建 Stub
    /// 
    /// # 参数
    /// - `code`: Stub 代码
    /// 
    /// # 返回
    /// 返回 Stub 对象
    pub fn from_code(code: Vec<u8>) -> Self {
        PharStub {
            code,
            use_default: false,
            entry_point: None,
            map: Vec::new(),
        }
    }

    /// 设置入口文件
    /// 
    /// # 参数
    /// - `entry`: 入口文件路径
    pub fn set_entry_point(&mut self, entry: &str) {
        self.entry_point = Some(entry.to_string());
    }

    /// 获取入口文件
    pub fn entry_point(&self) -> Option<&str> {
        self.entry_point.as_deref()
    }

    /// 添加映射
    /// 
    /// # 参数
    /// - `alias`: 别名
    /// - `file`: 文件路径
    pub fn add_map(&mut self, alias: &str, file: &str) {
        self.map.push((alias.to_string(), file.to_string()));
    }

    /// 获取映射
    pub fn maps(&self) -> &[(String, String)] {
        &self.map
    }

    /// 生成默认 Stub
    /// 
    /// # 返回
    /// 返回默认 Stub 代码
    pub fn generate_default() -> Vec<u8> {
        Self::default_stub_template().into_bytes()
    }

    /// 生成带入口的 Stub
    /// 
    /// # 参数
    /// - `entry`: 入口文件
    /// 
    /// # 返回
    /// 返回 Stub 代码
    pub fn generate_with_entry(entry: &str) -> Vec<u8> {
        let template = Self::default_stub_template_with_entry(entry);
        template.into_bytes()
    }

    /// 默认 Stub 模板
    fn default_stub_template() -> String {
        r#"<?php
$web = 'index.php';
if (in_array('phar', stream_get_wrappers()) && class_exists('Phar', 0)) {
    Phar::mapPhar('phar://' . __FILE__);
    include 'phar://' . __FILE__ . '/' . $web;
} else {
    $tmp = tempnam(sys_get_temp_dir(), 'phar');
    file_put_contents($tmp, file_get_contents(__FILE__, false, null, __COMPILER_HALT_OFFSET__));
    include $tmp;
    unlink($tmp);
}
__HALT_COMPILER();
"#.to_string()
    }

    /// 带入口的 Stub 模板
    fn default_stub_template_with_entry(entry: &str) -> String {
        format!(
            r#"<?php
$web = '{entry}';
if (in_array('phar', stream_get_wrappers()) && class_exists('Phar', 0)) {{
    Phar::mapPhar('phar://' . __FILE__);
    include 'phar://' . __FILE__ . '/' . $web;
}} else {{
    $tmp = tempnam(sys_get_temp_dir(), 'phar');
    file_put_contents($tmp, file_get_contents(__FILE__, false, null, __COMPILER_HALT_OFFSET__));
    include $tmp;
    unlink($tmp);
}}
__HALT_COMPILER();
"#,
            entry = entry
        )
    }

    /// 获取 Stub 代码
    pub fn code(&self) -> &[u8] {
        if self.use_default {
            if let Some(ref entry) = self.entry_point {
                // 返回带入口的默认 stub
                // 注意：这里需要缓存生成的代码
                return &self.code;
            }
        }
        &self.code
    }

    /// 序列化 Stub
    /// 
    /// # 返回
    /// 返回序列化后的字节
    pub fn serialize(&self) -> Vec<u8> {
        if self.use_default {
            if let Some(ref entry) = self.entry_point {
                Self::generate_with_entry(entry)
            } else {
                Self::generate_default()
            }
        } else {
            self.code.clone()
        }
    }

    /// 解析 Stub
    /// 
    /// # 参数
    /// - `data`: 数据
    /// 
    /// # 返回
    /// 成功返回 Stub 和剩余数据，失败返回错误信息
    pub fn parse(data: &[u8]) -> Result<(Self, &[u8]), String> {
        // 查找 __HALT_COMPILER();
        let halt_marker = b"__HALT_COMPILER();";
        if let Some(pos) = data.windows(halt_marker.len()).position(|w| w == halt_marker) {
            let end_pos = pos + halt_marker.len();
            // 跳过可能的空白和注释
            let mut remaining_start = end_pos;
            while remaining_start < data.len() {
                match data[remaining_start] {
                    b' ' | b'\t' | b'\n' | b'\r' | b'?' | b'>' => remaining_start += 1,
                    _ => break,
                }
            }
            // 查找结束标记 ?>
            let end_marker = b"?>";
            if remaining_start + 2 <= data.len() && &data[remaining_start..remaining_start + 2] == end_marker {
                remaining_start += 2;
            }
            let stub_code = data[..remaining_start].to_vec();
            let stub = PharStub::from_code(stub_code);
            Ok((stub, &data[remaining_start..]))
        } else {
            Err("未找到 __HALT_COMPILER() 标记".to_string())
        }
    }

    /// 检查是否为有效的 Stub
    /// 
    /// # 参数
    /// - `data`: Stub 数据
    /// 
    /// # 返回
    /// 如果是有效的 Stub 返回 true
    pub fn is_valid_stub(data: &[u8]) -> bool {
        // 检查是否包含 PHP 开始标记
        if !data.starts_with(b"<?php") && !data.starts_with(b"<?") {
            return false;
        }
        // 检查是否包含 __HALT_COMPILER()
        let halt_marker = b"__HALT_COMPILER();";
        data.windows(halt_marker.len()).any(|w| w == halt_marker)
    }

    /// 获取 Stub 长度
    /// 
    /// # 返回
    /// 返回 Stub 长度
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// 检查 Stub 是否为空
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

impl Default for PharStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Phar Stub 构建器
/// 
/// 用于构建自定义 Stub
pub struct PharStubBuilder {
    stub: PharStub,
}

impl PharStubBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        PharStubBuilder {
            stub: PharStub::new(),
        }
    }

    /// 设置 Stub 代码
    /// 
    /// # 参数
    /// - `code`: Stub 代码
    pub fn code(mut self, code: Vec<u8>) -> Self {
        self.stub.code = code;
        self.stub.use_default = false;
        self
    }

    /// 设置入口文件
    /// 
    /// # 参数
    /// - `entry`: 入口文件路径
    pub fn entry_point(mut self, entry: &str) -> Self {
        self.stub.entry_point = Some(entry.to_string());
        self
    }

    /// 添加映射
    /// 
    /// # 参数
    /// - `alias`: 别名
    /// - `file`: 文件路径
    pub fn map(mut self, alias: &str, file: &str) -> Self {
        self.stub.map.push((alias.to_string(), file.to_string()));
        self
    }

    /// 使用默认 Stub
    pub fn use_default(mut self) -> Self {
        self.stub.use_default = true;
        self
    }

    /// 构建 Stub
    pub fn build(mut self) -> PharStub {
        if self.stub.use_default && self.stub.code.is_empty() {
            self.stub.code = PharStub::generate_default();
        }
        self.stub
    }
}

impl Default for PharStubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_creation() {
        let stub = PharStub::new();
        assert!(stub.use_default);
    }

    #[test]
    fn test_stub_from_code() {
        let code = b"<?php __HALT_COMPILER();".to_vec();
        let stub = PharStub::from_code(code.clone());
        assert_eq!(stub.code(), code);
    }

    #[test]
    fn test_stub_parse() {
        let data = b"<?php __HALT_COMPILER(); ?>manifest_data";
        let (stub, remaining) = PharStub::parse(data).unwrap();
        assert!(stub.code().starts_with(b"<?php"));
        assert!(remaining.starts_with(b"manifest"));
    }

    #[test]
    fn test_is_valid_stub() {
        let valid = b"<?php __HALT_COMPILER(); ?>";
        assert!(PharStub::is_valid_stub(valid));
        let invalid = b"Not a valid stub";
        assert!(!PharStub::is_valid_stub(invalid));
    }

    #[test]
    fn test_stub_builder() {
        let stub = PharStubBuilder::new()
            .entry_point("index.php")
            .build();
        assert_eq!(stub.entry_point(), Some("index.php"));
    }
}
