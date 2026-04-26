//! FastCGI 请求处理器
//!
//! 将 FastCGI 请求转换为 HTTP 请求并处理

use anyhow::Result;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use crate::http::state::AppState;
use crate::http::request::Request;
use crate::http::response::OytaResponse;
use crate::fastcgi::protocol::*;

/// FastCGI 请求处理器
pub struct FastCGIHandler {
    /// 应用状态
    app_state: Arc<AppState>,
    /// 项目根目录
    project_root: PathBuf,
}

impl FastCGIHandler {
    /// 创建新的处理器
    pub fn new(app_state: Arc<AppState>, project_root: PathBuf) -> Self {
        FastCGIHandler {
            app_state,
            project_root,
        }
    }

    /// 处理 FastCGI 连接
    pub fn handle_connection<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> Result<()> {
        // 读取请求
        let mut request_id: u16 = 0;
        let mut params: HashMap<String, String> = HashMap::new();
        let mut stdin_data: Vec<u8> = Vec::new();
        let mut request_started = false;

        loop {
            // 读取记录
            let record = match Record::read(&mut reader) {
                Ok(r) => r,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    tracing::error!("读取 FastCGI 记录失败: {}", e);
                    return Err(e.into());
                }
            };

            match record.header.record_type {
                RecordType::BeginRequest => {
                    let body = BeginRequestBody::from_bytes(&record.content)?;
                    request_id = record.header.request_id;
                    request_started = true;
                    tracing::debug!("开始 FastCGI 请求: id={}, role={:?}", request_id, body.role);
                }
                RecordType::Params => {
                    if !record.content.is_empty() {
                        let pairs = parse_name_value_pairs(&record.content)?;
                        for (name, value) in pairs {
                            params.insert(name, value);
                        }
                    }
                }
                RecordType::Stdin => {
                    if !record.content.is_empty() {
                        stdin_data.extend_from_slice(&record.content);
                    } else {
                        // 空的 Stdin 表示请求体结束
                        break;
                    }
                }
                RecordType::AbortRequest => {
                    tracing::debug!("收到中止请求: id={}", record.header.request_id);
                    return Ok(());
                }
                _ => {
                    tracing::debug!("忽略记录类型: {:?}", record.header.record_type);
                }
            }
        }

        if !request_started {
            return Ok(());
        }

        // 处理请求
        let response = self.process_request(params, stdin_data)?;

        // 发送响应
        self.send_response(&mut writer, request_id, &response)?;

        Ok(())
    }

    /// 处理请求
    fn process_request(
        &self,
        params: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Result<OytaResponse> {
        // 从 FastCGI 参数构建请求
        let method = params.get("REQUEST_METHOD").cloned().unwrap_or_else(|| "GET".to_string());
        let uri = params.get("REQUEST_URI").cloned().unwrap_or_else(|| "/".to_string());
        let query_string = params.get("QUERY_STRING").cloned();

        // 构建请求头
        let mut headers = HashMap::new();
        for (key, value) in &params {
            if key.starts_with("HTTP_") {
                let header_name = key[5..].replace('_', "-").to_lowercase();
                headers.insert(header_name, value.clone());
            } else {
                match key.as_str() {
                    "CONTENT_TYPE" => {
                        headers.insert("content-type".to_string(), value.clone());
                    }
                    "CONTENT_LENGTH" => {
                        headers.insert("content-length".to_string(), value.clone());
                    }
                    "REMOTE_ADDR" => {
                        headers.insert("x-real-ip".to_string(), value.clone());
                    }
                    "REMOTE_PORT" => {
                        headers.insert("x-real-port".to_string(), value.clone());
                    }
                    "SERVER_NAME" => {
                        headers.insert("host".to_string(), value.clone());
                    }
                    "SERVER_PORT" => {
                        headers.insert("x-server-port".to_string(), value.clone());
                    }
                    "HTTPS" => {
                        if value == "on" {
                            headers.insert("x-forwarded-proto".to_string(), "https".to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        // 解析查询参数
        let mut query_params = HashMap::new();
        if let Some(ref qs) = query_string {
            for pair in qs.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    query_params.insert(
                        urlencoding::decode(key).unwrap_or_default().to_string(),
                        urlencoding::decode(value).unwrap_or_default().to_string(),
                    );
                }
            }
        }

        // 解析路径
        let path = uri.split('?').next().unwrap_or("/").to_string();

        // 创建请求对象
        let mut request = Request::new();
        request.method = method.clone();
        request.uri = uri.clone();
        request.path = path;
        request.query_string = query_string;
        request.headers = headers;
        request.query_params = query_params;
        if !body.is_empty() {
            request.body = Some(body);
        }

        // 设置客户端 IP
        if let Some(ip) = params.get("REMOTE_ADDR") {
            request.client_ip = Some(ip.clone());
        }

        // 使用路由分发请求
        let method_enum = crate::router::types::HttpMethod::from_str(&method);
        let response = crate::http::handler::dispatch_request(&self.app_state, &request, &method_enum)?;

        Ok(response)
    }

    /// 发送响应
    fn send_response<W: Write>(
        &self,
        writer: &mut W,
        request_id: u16,
        response: &OytaResponse,
    ) -> Result<()> {
        // 获取状态文本
        let status_text = get_status_text(response.status_code);
        
        // 构建 HTTP 响应
        let status_line = format!("Status: {} {}\r\n", response.status_code, status_text);
        let mut response_data = status_line.into_bytes();

        // 添加响应头
        for (name, value) in &response.headers {
            response_data.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }
        
        // 添加 Content-Type 头
        response_data.extend_from_slice(format!("Content-Type: {}\r\n", response.content_type).as_bytes());
        response_data.extend_from_slice(b"\r\n");

        // 添加响应体
        response_data.extend_from_slice(response.body.as_bytes());

        // 分块发送（FastCGI 限制每条记录最大 65535 字节）
        const CHUNK_SIZE: usize = 65535;
        let total_len = response_data.len();
        let mut offset = 0;

        while offset < total_len {
            let end = std::cmp::min(offset + CHUNK_SIZE, total_len);
            let chunk = &response_data[offset..end];

            let record = create_stdout_record(request_id, chunk);
            record.write(writer)?;

            offset = end;
        }

        // 发送空的 Stdout 记录表示输出结束
        let empty_record = create_empty_stdout_record(request_id);
        empty_record.write(writer)?;

        // 发送结束请求记录
        let end_record = create_end_request(request_id, 0, ProtocolStatus::RequestComplete);
        end_record.write(writer)?;

        writer.flush()?;

        Ok(())
    }
}

/// 获取 HTTP 状态码对应的文本描述
fn get_status_text(status_code: u16) -> &'static str {
    match status_code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}
