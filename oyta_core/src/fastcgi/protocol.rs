//! FastCGI 协议定义
//!
//! 实现 FastCGI 协议的常量和数据结构
//! 参考：https://fastcgi-archives.github.io/FastCGI_Specification.html

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

/// FastCGI 协议版本
pub const VERSION: u8 = 1;

/// FastCGI 记录类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    /// 开始请求
    BeginRequest = 1,
    /// 中止请求
    AbortRequest = 2,
    /// 结束请求
    EndRequest = 3,
    /// 参数
    Params = 4,
    /// 标准输入
    Stdin = 5,
    /// 标准输出
    Stdout = 6,
    /// 标准错误
    Stderr = 7,
    /// 数据
    Data = 8,
    /// 获取值
    GetValues = 9,
    /// 获取值结果
    GetValuesResult = 10,
    /// 未知类型
    UnknownType = 11,
}

impl TryFrom<u8> for RecordType {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RecordType::BeginRequest),
            2 => Ok(RecordType::AbortRequest),
            3 => Ok(RecordType::EndRequest),
            4 => Ok(RecordType::Params),
            5 => Ok(RecordType::Stdin),
            6 => Ok(RecordType::Stdout),
            7 => Ok(RecordType::Stderr),
            8 => Ok(RecordType::Data),
            9 => Ok(RecordType::GetValues),
            10 => Ok(RecordType::GetValuesResult),
            11 => Ok(RecordType::UnknownType),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown record type: {}", value),
            )),
        }
    }
}

/// FastCGI 角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Role {
    /// 响应者（处理 HTTP 请求）
    Responder = 1,
    /// 授权者
    Authorizer = 2,
    /// 过滤器
    Filter = 3,
}

/// FastCGI 协议状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolStatus {
    /// 请求完成
    RequestComplete = 0,
    /// 无法多路复用
    CantMultiplex = 1,
    /// 过载
    Overloaded = 2,
    /// 角色未知
    UnknownRole = 3,
}

/// FastCGI 记录头
#[derive(Debug, Clone)]
pub struct RecordHeader {
    /// 协议版本
    pub version: u8,
    /// 记录类型
    pub record_type: RecordType,
    /// 请求 ID
    pub request_id: u16,
    /// 内容长度
    pub content_length: u16,
    /// 填充长度
    pub padding_length: u8,
    /// 保留字段
    pub reserved: u8,
}

impl RecordHeader {
    /// 记录头大小
    pub const SIZE: usize = 8;

    /// 从读取器解析记录头
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let version = reader.read_u8()?;
        let record_type_byte = reader.read_u8()?;
        let record_type = RecordType::try_from(record_type_byte)?;
        let request_id = reader.read_u16::<BigEndian>()?;
        let content_length = reader.read_u16::<BigEndian>()?;
        let padding_length = reader.read_u8()?;
        let reserved = reader.read_u8()?;

        Ok(RecordHeader {
            version,
            record_type,
            request_id,
            content_length,
            padding_length,
            reserved,
        })
    }

    /// 写入记录头
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u8(self.version)?;
        writer.write_u8(self.record_type as u8)?;
        writer.write_u16::<BigEndian>(self.request_id)?;
        writer.write_u16::<BigEndian>(self.content_length)?;
        writer.write_u8(self.padding_length)?;
        writer.write_u8(self.reserved)?;
        Ok(())
    }
}

/// 开始请求体
#[derive(Debug, Clone)]
pub struct BeginRequestBody {
    /// 角色
    pub role: Role,
    /// 标志
    pub flags: u8,
    /// 保留字段
    pub reserved: [u8; 5],
}

impl BeginRequestBody {
    /// 体大小
    pub const SIZE: usize = 8;

    /// 从字节解析
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "BeginRequestBody too short",
            ));
        }

        let role_byte = u16::from_be_bytes([data[0], data[1]]);
        let role = match role_byte {
            1 => Role::Responder,
            2 => Role::Authorizer,
            3 => Role::Filter,
            _ => Role::Responder,
        };

        Ok(BeginRequestBody {
            role,
            flags: data[2],
            reserved: [data[3], data[4], data[5], data[6], data[7]],
        })
    }
}

/// 结束请求体
#[derive(Debug, Clone)]
pub struct EndRequestBody {
    /// 应用状态
    pub app_status: u32,
    /// 协议状态
    pub protocol_status: ProtocolStatus,
    /// 保留字段
    pub reserved: [u8; 3],
}

impl EndRequestBody {
    /// 体大小
    pub const SIZE: usize = 8;

    /// 创建新的结束请求体
    pub fn new(app_status: u32, protocol_status: ProtocolStatus) -> Self {
        EndRequestBody {
            app_status,
            protocol_status,
            reserved: [0; 3],
        }
    }

    /// 转换为字节
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.app_status.to_be_bytes());
        bytes[4] = self.protocol_status as u8;
        bytes[5..8].copy_from_slice(&self.reserved);
        bytes
    }
}

/// FastCGI 记录
#[derive(Debug, Clone)]
pub struct Record {
    /// 记录头
    pub header: RecordHeader,
    /// 内容数据
    pub content: Vec<u8>,
}

impl Record {
    /// 创建新记录
    pub fn new(record_type: RecordType, request_id: u16, content: Vec<u8>) -> Self {
        let content_length = content.len() as u16;
        let padding_length = ((8 - (content_length % 8)) % 8) as u8;

        Record {
            header: RecordHeader {
                version: VERSION,
                record_type,
                request_id,
                content_length,
                padding_length,
                reserved: 0,
            },
            content,
        }
    }

    /// 从读取器解析记录
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let header = RecordHeader::read(reader)?;

        // 读取内容
        let mut content = vec![0u8; header.content_length as usize];
        if !content.is_empty() {
            reader.read_exact(&mut content)?;
        }

        // 读取填充
        if header.padding_length > 0 {
            let mut padding = vec![0u8; header.padding_length as usize];
            reader.read_exact(&mut padding)?;
        }

        Ok(Record { header, content })
    }

    /// 写入记录
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.header.write(writer)?;
        writer.write_all(&self.content)?;

        // 写入填充
        if self.header.padding_length > 0 {
            let padding = vec![0u8; self.header.padding_length as usize];
            writer.write_all(&padding)?;
        }

        Ok(())
    }
}

/// 解析 FastCGI 名值对
pub fn parse_name_value_pairs(data: &[u8]) -> io::Result<Vec<(String, String)>> {
    let mut pairs = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // 读取名称长度
        let name_len = if data[pos] & 0x80 != 0 {
            // 4 字节长度
            if pos + 4 > data.len() {
                break;
            }
            let len = u32::from_be_bytes([
                data[pos] & 0x7f,
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]);
            pos += 4;
            len as usize
        } else {
            // 1 字节长度
            let len = data[pos] as usize;
            pos += 1;
            len
        };

        // 读取值长度
        let value_len = if pos >= data.len() {
            break;
        } else if data[pos] & 0x80 != 0 {
            // 4 字节长度
            if pos + 4 > data.len() {
                break;
            }
            let len = u32::from_be_bytes([
                data[pos] & 0x7f,
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]);
            pos += 4;
            len as usize
        } else {
            // 1 字节长度
            let len = data[pos] as usize;
            pos += 1;
            len
        };

        // 读取名称
        if pos + name_len > data.len() {
            break;
        }
        let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
        pos += name_len;

        // 读取值
        if pos + value_len > data.len() {
            break;
        }
        let value = String::from_utf8_lossy(&data[pos..pos + value_len]).to_string();
        pos += value_len;

        pairs.push((name, value));
    }

    Ok(pairs)
}

/// 创建标准输出记录
pub fn create_stdout_record(request_id: u16, data: &[u8]) -> Record {
    Record::new(RecordType::Stdout, request_id, data.to_vec())
}

/// 创建标准错误记录
pub fn create_stderr_record(request_id: u16, data: &[u8]) -> Record {
    Record::new(RecordType::Stderr, request_id, data.to_vec())
}

/// 创建结束请求记录
pub fn create_end_request(request_id: u16, app_status: u32, protocol_status: ProtocolStatus) -> Record {
    let body = EndRequestBody::new(app_status, protocol_status);
    Record::new(RecordType::EndRequest, request_id, body.to_bytes().to_vec())
}

/// 创建空标准输出记录（用于结束输出）
pub fn create_empty_stdout_record(request_id: u16) -> Record {
    Record::new(RecordType::Stdout, request_id, Vec::new())
}
