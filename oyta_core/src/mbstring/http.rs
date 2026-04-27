//! HTTP 输入输出编码处理模块
//!
//! 提供完整的 PHP mbstring HTTP 编码处理功能

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::encoding::MbEncoding;

/// HTTP 输入输出编码处理器
///
/// 管理 HTTP 输入输出的字符编码转换
#[derive(Debug)]
pub struct MbHttpHandler {
    /// HTTP 输入编码
    pub http_input: HttpInputEncoding,
    /// HTTP 输出编码
    pub http_output: HttpOutputEncoding,
    /// 内部编码
    pub internal_encoding: String,
    /// HTTP 输出转换处理器
    pub output_handler: Option<String>,
}

impl MbHttpHandler {
    /// 创建新的 HTTP 处理器
    pub fn new() -> Self {
        Self {
            http_input: HttpInputEncoding::default(),
            http_output: HttpOutputEncoding::default(),
            internal_encoding: "UTF-8".to_string(),
            output_handler: None,
        }
    }
    
    /// 设置 HTTP 输入编码
    ///
    /// 对应 PHP 的 mb_http_input() 函数
    pub fn set_http_input(&mut self, encoding: &str, input_type: Option<HttpInputType>) -> bool {
        let enc = encoding.to_uppercase();
        
        match input_type {
            Some(HttpInputType::Get) => {
                self.http_input.get = Some(enc);
            }
            Some(HttpInputType::Post) => {
                self.http_input.post = Some(enc);
            }
            Some(HttpInputType::Cookie) => {
                self.http_input.cookie = Some(enc);
            }
            Some(HttpInputType::All) | None => {
                self.http_input.get = Some(enc.clone());
                self.http_input.post = Some(enc.clone());
                self.http_input.cookie = Some(enc);
            }
        }
        
        true
    }
    
    /// 获取 HTTP 输入编码
    pub fn get_http_input(&self, input_type: Option<HttpInputType>) -> Option<&str> {
        match input_type {
            Some(HttpInputType::Get) => self.http_input.get.as_deref(),
            Some(HttpInputType::Post) => self.http_input.post.as_deref(),
            Some(HttpInputType::Cookie) => self.http_input.cookie.as_deref(),
            None | Some(HttpInputType::All) => {
                // 返回第一个设置的编码
                self.http_input.get.as_deref()
                    .or(self.http_input.post.as_deref())
                    .or(self.http_input.cookie.as_deref())
            }
        }
    }
    
    /// 设置 HTTP 输出编码
    ///
    /// 对应 PHP 的 mb_http_output() 函数
    pub fn set_http_output(&mut self, encoding: &str) -> bool {
        self.http_output.encoding = Some(encoding.to_uppercase());
        true
    }
    
    /// 获取 HTTP 输出编码
    pub fn get_http_output(&self) -> Option<&str> {
        self.http_output.encoding.as_deref()
    }
    
    /// 设置内部编码
    pub fn set_internal_encoding(&mut self, encoding: &str) -> bool {
        self.internal_encoding = encoding.to_uppercase();
        true
    }
    
    /// 获取内部编码
    pub fn get_internal_encoding(&self) -> &str {
        &self.internal_encoding
    }
    
    /// 处理 HTTP 输入
    ///
    /// 将输入从 HTTP 输入编码转换为内部编码
    pub fn process_input(&self, input: &str, input_type: HttpInputType) -> String {
        // 获取输入编码
        let input_encoding = match input_type {
            HttpInputType::Get => self.http_input.get.as_deref(),
            HttpInputType::Post => self.http_input.post.as_deref(),
            HttpInputType::Cookie => self.http_input.cookie.as_deref(),
            HttpInputType::All => self.http_input.get.as_deref(),
        };
        
        // 如果没有设置输入编码，直接返回
        let input_enc = match input_encoding {
            Some(enc) => enc,
            None => return input.to_string(),
        };
        
        // 如果输入编码和内部编码相同，直接返回
        if input_enc.eq_ignore_ascii_case(&self.internal_encoding) {
            return input.to_string();
        }
        
        // 进行编码转换
        if let Ok(result) = super::convert::mb_convert_encoding(
            input,
            &self.internal_encoding,
            Some(input_enc),
        ) {
            result
        } else {
            input.to_string()
        }
    }
    
    /// 处理 HTTP 输出
    ///
    /// 将输出从内部编码转换为 HTTP 输出编码
    pub fn process_output(&self, output: &str) -> String {
        // 获取输出编码
        let output_encoding = match self.http_output.encoding.as_deref() {
            Some(enc) => enc,
            None => return output.to_string(),
        };
        
        // 如果输出编码和内部编码相同，直接返回
        if output_encoding.eq_ignore_ascii_case(&self.internal_encoding) {
            return output.to_string();
        }
        
        // 进行编码转换
        if let Ok(result) = super::convert::mb_convert_encoding(
            output,
            output_encoding,
            Some(&self.internal_encoding),
        ) {
            result
        } else {
            output.to_string()
        }
    }
    
    /// 获取输出处理器名称
    pub fn get_output_handler(&self) -> Option<&str> {
        self.output_handler.as_deref()
    }
    
    /// 设置输出处理器
    pub fn set_output_handler(&mut self, handler: &str) {
        self.output_handler = Some(handler.to_string());
    }
}

impl Default for MbHttpHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP 输入编码配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpInputEncoding {
    /// GET 请求编码
    pub get: Option<String>,
    /// POST 请求编码
    pub post: Option<String>,
    /// Cookie 编码
    pub cookie: Option<String>,
}

impl Default for HttpInputEncoding {
    fn default() -> Self {
        Self {
            get: Some("UTF-8".to_string()),
            post: Some("UTF-8".to_string()),
            cookie: Some("UTF-8".to_string()),
        }
    }
}

/// HTTP 输出编码配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpOutputEncoding {
    /// 输出编码
    pub encoding: Option<String>,
}

impl Default for HttpOutputEncoding {
    fn default() -> Self {
        Self {
            encoding: Some("UTF-8".to_string()),
        }
    }
}

/// HTTP 输入类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpInputType {
    /// GET 请求
    Get,
    /// POST 请求
    Post,
    /// Cookie
    Cookie,
    /// 所有类型
    All,
}

/// 全局 HTTP 处理器
pub struct GlobalMbHttpHandler {
    /// 内部处理器
    inner: Arc<RwLock<MbHttpHandler>>,
}

impl GlobalMbHttpHandler {
    /// 创建新的全局处理器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MbHttpHandler::new())),
        }
    }
    
    /// 获取内部处理器
    pub fn inner(&self) -> Arc<RwLock<MbHttpHandler>> {
        self.inner.clone()
    }
    
    /// 设置 HTTP 输入编码
    pub fn set_http_input(&self, encoding: &str, input_type: Option<HttpInputType>) -> bool {
        self.inner.write().set_http_input(encoding, input_type)
    }
    
    /// 获取 HTTP 输入编码
    pub fn get_http_input(&self, input_type: Option<HttpInputType>) -> Option<String> {
        self.inner.read().get_http_input(input_type).map(|s| s.to_string())
    }
    
    /// 设置 HTTP 输出编码
    pub fn set_http_output(&self, encoding: &str) -> bool {
        self.inner.write().set_http_output(encoding)
    }
    
    /// 获取 HTTP 输出编码
    pub fn get_http_output(&self) -> Option<String> {
        self.inner.read().get_http_output().map(|s| s.to_string())
    }
    
    /// 处理输入
    pub fn process_input(&self, input: &str, input_type: HttpInputType) -> String {
        self.inner.read().process_input(input, input_type)
    }
    
    /// 处理输出
    pub fn process_output(&self, output: &str) -> String {
        self.inner.read().process_output(output)
    }
}

impl Default for GlobalMbHttpHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalMbHttpHandler {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// 全局 HTTP 处理器实例
static GLOBAL_MB_HTTP_HANDLER: once_cell::sync::Lazy<GlobalMbHttpHandler> = 
    once_cell::sync::Lazy::new(GlobalMbHttpHandler::new);

/// 获取全局 HTTP 处理器
pub fn global_mb_http_handler() -> &'static GlobalMbHttpHandler {
    &GLOBAL_MB_HTTP_HANDLER
}

/// 设置 HTTP 输入编码
pub fn mb_http_input(encoding: &str, input_type: Option<HttpInputType>) -> bool {
    global_mb_http_handler().set_http_input(encoding, input_type)
}

/// 获取 HTTP 输入编码
pub fn mb_http_input_get(input_type: Option<HttpInputType>) -> Option<String> {
    global_mb_http_handler().get_http_input(input_type)
}

/// 设置 HTTP 输出编码
pub fn mb_http_output(encoding: &str) -> bool {
    global_mb_http_handler().set_http_output(encoding)
}

/// 获取 HTTP 输出编码
pub fn mb_http_output_get() -> Option<String> {
    global_mb_http_handler().get_http_output()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_handler_creation() {
        let handler = MbHttpHandler::new();
        
        assert_eq!(handler.get_internal_encoding(), "UTF-8");
    }
    
    #[test]
    fn test_set_http_input() {
        let mut handler = MbHttpHandler::new();
        
        handler.set_http_input("GBK", Some(HttpInputType::Get));
        assert_eq!(handler.get_http_input(Some(HttpInputType::Get)), Some("GBK"));
    }
    
    #[test]
    fn test_set_http_output() {
        let mut handler = MbHttpHandler::new();
        
        handler.set_http_output("GBK");
        assert_eq!(handler.get_http_output(), Some("GBK"));
    }
    
    #[test]
    fn test_process_input() {
        let handler = MbHttpHandler::new();
        
        let result = handler.process_input("Hello", HttpInputType::Get);
        assert_eq!(result, "Hello");
    }
    
    #[test]
    fn test_process_output() {
        let handler = MbHttpHandler::new();
        
        let result = handler.process_output("Hello");
        assert_eq!(result, "Hello");
    }
    
    #[test]
    fn test_global_handler() {
        let result = mb_http_input("UTF-8", Some(HttpInputType::All));
        assert!(result);
        
        let encoding = mb_http_input_get(Some(HttpInputType::Get));
        assert!(encoding.is_some());
    }
}
