//! 图像核心模块
//! 
//! 本模块实现 GD 图像的核心功能，包括：
//! - 图像创建和销毁
//! - 图像加载和保存
//! - 像素操作
//! - 图像信息获取
//! - 支持多种图像格式（PNG、JPEG、GIF、WebP、BMP等）

use std::io::{Cursor, Read, Write};
use std::path::Path;
use crate::gd::color::{GdColor, GdPalette};

/// 图像类型枚举
/// 
/// 定义支持的图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdImageType {
    /// PNG 格式
    Png,
    /// JPEG 格式
    Jpeg,
    /// GIF 格式
    Gif,
    /// WebP 格式
    WebP,
    /// BMP 格式
    Bmp,
    /// WBMP 格式（无线位图）
    Wbmp,
    /// XBM 格式（X 位图）
    Xbm,
    /// XPM 格式（X 像素图）
    Xpm,
    /// TIFF 格式
    Tiff,
    /// 未知格式
    Unknown,
}

impl GdImageType {
    /// 从文件扩展名获取图像类型
    /// 
    /// # 参数
    /// - `path`: 文件路径
    /// 
    /// # 返回
    /// 返回图像类型
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("png") => GdImageType::Png,
            Some("jpg") | Some("jpeg") => GdImageType::Jpeg,
            Some("gif") => GdImageType::Gif,
            Some("webp") => GdImageType::WebP,
            Some("bmp") => GdImageType::Bmp,
            Some("wbmp") => GdImageType::Wbmp,
            Some("xbm") => GdImageType::Xbm,
            Some("xpm") => GdImageType::Xpm,
            Some("tiff") | Some("tif") => GdImageType::Tiff,
            _ => GdImageType::Unknown,
        }
    }

    /// 从 MIME 类型获取图像类型
    /// 
    /// # 参数
    /// - `mime`: MIME 类型字符串
    /// 
    /// # 返回
    /// 返回图像类型
    pub fn from_mime(mime: &str) -> Self {
        match mime.to_lowercase().as_str() {
            "image/png" => GdImageType::Png,
            "image/jpeg" | "image/jpg" => GdImageType::Jpeg,
            "image/gif" => GdImageType::Gif,
            "image/webp" => GdImageType::WebP,
            "image/bmp" | "image/x-ms-bmp" => GdImageType::Bmp,
            "image/vnd.wap.wbmp" => GdImageType::Wbmp,
            "image/xbm" | "image/x-xbitmap" => GdImageType::Xbm,
            "image/xpm" | "image/x-xpixmap" => GdImageType::Xpm,
            "image/tiff" => GdImageType::Tiff,
            _ => GdImageType::Unknown,
        }
    }

    /// 获取 MIME 类型
    /// 
    /// # 返回
    /// 返回 MIME 类型字符串
    pub fn to_mime(&self) -> &'static str {
        match self {
            GdImageType::Png => "image/png",
            GdImageType::Jpeg => "image/jpeg",
            GdImageType::Gif => "image/gif",
            GdImageType::WebP => "image/webp",
            GdImageType::Bmp => "image/bmp",
            GdImageType::Wbmp => "image/vnd.wap.wbmp",
            GdImageType::Xbm => "image/x-xbitmap",
            GdImageType::Xpm => "image/x-xpixmap",
            GdImageType::Tiff => "image/tiff",
            GdImageType::Unknown => "application/octet-stream",
        }
    }

    /// 获取文件扩展名
    /// 
    /// # 返回
    /// 返回文件扩展名
    pub fn to_extension(&self) -> &'static str {
        match self {
            GdImageType::Png => "png",
            GdImageType::Jpeg => "jpg",
            GdImageType::Gif => "gif",
            GdImageType::WebP => "webp",
            GdImageType::Bmp => "bmp",
            GdImageType::Wbmp => "wbmp",
            GdImageType::Xbm => "xbm",
            GdImageType::Xpm => "xpm",
            GdImageType::Tiff => "tiff",
            GdImageType::Unknown => "bin",
        }
    }

    /// 检查是否支持透明度
    /// 
    /// # 返回
    /// 如果支持透明度返回 true
    pub fn supports_transparency(&self) -> bool {
        matches!(self, GdImageType::Png | GdImageType::Gif | GdImageType::WebP | GdImageType::Tiff)
    }

    /// 检查是否支持动画
    /// 
    /// # 返回
    /// 如果支持动画返回 true
    pub fn supports_animation(&self) -> bool {
        matches!(self, GdImageType::Gif | GdImageType::WebP)
    }
}

/// 图像信息结构
/// 
/// 包含图像的基本信息
#[derive(Debug, Clone)]
pub struct GdImageInfo {
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 图像类型
    pub image_type: GdImageType,
    /// 是否为真彩色
    pub true_color: bool,
    /// 位深度
    pub bits: u8,
    /// 通道数
    pub channels: u8,
    /// MIME 类型
    pub mime: String,
}

impl GdImageInfo {
    /// 创建新的图像信息
    pub fn new(width: u32, height: u32, image_type: GdImageType) -> Self {
        GdImageInfo {
            width,
            height,
            image_type,
            true_color: true,
            bits: 24,
            channels: 3,
            mime: image_type.to_mime().to_string(),
        }
    }
}

/// GD 图像结构
/// 
/// 表示一个 GD 图像对象，包含像素数据和调色板
#[derive(Debug, Clone)]
pub struct GdImage {
    /// 图像宽度
    width: u32,
    /// 图像高度
    height: u32,
    /// 像素数据（颜色索引数组）
    pixels: Vec<i32>,
    /// 调色板
    palette: GdPalette,
    /// 是否为真彩色图像
    true_color: bool,
    /// 透明色索引
    transparent: Option<i32>,
    /// 抗锯齿标志
    anti_alias: bool,
    /// 厚度（画笔宽度）
    thickness: i32,
    /// 画笔颜色
    brush: Option<i32>,
    /// 平铺图像
    tile: Option<Box<GdImage>>,
    /// 样式数组
    style: Vec<i32>,
    /// Alpha 混合标志
    alpha_blending: bool,
    /// 保存 Alpha 标志
    save_alpha: bool,
    /// 插值方法
    interpolation: GdInterpolationMethod,
    /// 压缩级别（用于 PNG）
    compression_level: i32,
    /// JPEG 质量
    jpeg_quality: i32,
}

/// 插值方法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdInterpolationMethod {
    /// 默认方法
    Default,
    /// 最近邻插值
    NearestNeighbour,
    /// 双线性插值
    Bilinear,
    /// 双三次插值
    Bicubic,
    /// Bell 插值
    Bell,
    /// Mitchell 插值
    Mitchell,
    /// Lanczos 插值
    Lanczos3,
    /// Hermite 插值
    Hermite,
    /// Spline 插值
    Spline,
    /// 三角插值
    Triangle,
    /// Weighted Average 插值
    WeightedAverage,
}

impl Default for GdInterpolationMethod {
    fn default() -> Self {
        GdInterpolationMethod::Bilinear
    }
}

impl GdImage {
    /// 创建新的真彩色图像
    /// 
    /// # 参数
    /// - `width`: 图像宽度
    /// - `height`: 图像高度
    /// 
    /// # 返回
    /// 返回新创建的图像
    pub fn new_true_color(width: u32, height: u32) -> Self {
        let pixels = vec![0i32; (width * height) as usize];
        GdImage {
            width,
            height,
            pixels,
            palette: GdPalette::true_color(),
            true_color: true,
            transparent: None,
            anti_alias: false,
            thickness: 1,
            brush: None,
            tile: None,
            style: Vec::new(),
            alpha_blending: true,
            save_alpha: false,
            interpolation: GdInterpolationMethod::default(),
            compression_level: -1,
            jpeg_quality: 75,
        }
    }

    /// 创建新的调色板图像
    /// 
    /// # 参数
    /// - `width`: 图像宽度
    /// - `height`: 图像高度
    /// 
    /// # 返回
    /// 返回新创建的图像
    pub fn new_palette(width: u32, height: u32) -> Self {
        let pixels = vec![-1i32; (width * height) as usize];
        GdImage {
            width,
            height,
            pixels,
            palette: GdPalette::indexed(256),
            true_color: false,
            transparent: None,
            anti_alias: false,
            thickness: 1,
            brush: None,
            tile: None,
            style: Vec::new(),
            alpha_blending: true,
            save_alpha: false,
            interpolation: GdInterpolationMethod::default(),
            compression_level: -1,
            jpeg_quality: 75,
        }
    }

    /// 从文件加载图像
    /// 
    /// # 参数
    /// - `path`: 文件路径
    /// 
    /// # 返回
    /// 成功返回图像，失败返回错误信息
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
        Self::from_bytes(&data)
    }

    /// 从字节数据加载图像
    /// 
    /// # 参数
    /// - `data`: 图像字节数据
    /// 
    /// # 返回
    /// 成功返回图像，失败返回错误信息
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        // 检测图像类型
        let image_type = Self::detect_type(data)?;
        match image_type {
            GdImageType::Png => Self::load_png(data),
            GdImageType::Jpeg => Self::load_jpeg(data),
            GdImageType::Gif => Self::load_gif(data),
            GdImageType::Bmp => Self::load_bmp(data),
            GdImageType::WebP => Self::load_webp(data),
            _ => Err("不支持的图像格式".to_string()),
        }
    }

    /// 检测图像类型
    /// 
    /// # 参数
    /// - `data`: 图像数据
    /// 
    /// # 返回
    /// 返回检测到的图像类型
    fn detect_type(data: &[u8]) -> Result<GdImageType, String> {
        if data.len() < 4 {
            return Err("数据太短，无法检测图像类型".to_string());
        }
        // PNG 签名: 89 50 4E 47 0D 0A 1A 0A
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Ok(GdImageType::Png);
        }
        // JPEG 签名: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(GdImageType::Jpeg);
        }
        // GIF 签名: GIF87a 或 GIF89a
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Ok(GdImageType::Gif);
        }
        // BMP 签名: BM
        if data.starts_with(b"BM") {
            return Ok(GdImageType::Bmp);
        }
        // WebP 签名: RIFF....WEBP
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Ok(GdImageType::WebP);
        }
        // TIFF 签名: II 或 MM
        if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
            return Ok(GdImageType::Tiff);
        }
        Err("无法识别的图像格式".to_string())
    }

    /// 加载 PNG 图像
    fn load_png(data: &[u8]) -> Result<Self, String> {
        let decoder = png::Decoder::new(Cursor::new(data));
        let mut reader = decoder.read_info().map_err(|e| format!("PNG 解码错误: {}", e))?;
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).map_err(|e| format!("PNG 帧读取错误: {}", e))?;
        let width = info.width;
        let height = info.height;
        let mut image = GdImage::new_true_color(width, height);
        // 根据颜色类型处理像素数据
        match info.color_type {
            png::ColorType::Rgb => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 3) as usize;
                        let r = buf[idx];
                        let g = buf[idx + 1];
                        let b = buf[idx + 2];
                        let color = GdColor::new(r, g, b);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
            png::ColorType::Rgba => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let r = buf[idx];
                        let g = buf[idx + 1];
                        let b = buf[idx + 2];
                        let a = buf[idx + 3];
                        // GD 使用 0-127 的 alpha，其中 127 是完全透明
                        let alpha = (127 - (a as i32 * 127 / 255)) as u8;
                        let color = GdColor::new_with_alpha(r, g, b, alpha);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
                image.save_alpha = true;
            }
            png::ColorType::Grayscale => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let gray = buf[idx];
                        let color = GdColor::new(gray, gray, gray);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
            png::ColorType::GrayscaleAlpha => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = buf[idx];
                        let a = buf[idx + 1];
                        let alpha = (127 - (a as i32 * 127 / 255)) as u8;
                        let color = GdColor::new_with_alpha(gray, gray, gray, alpha);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
                image.save_alpha = true;
            }
            png::ColorType::Indexed => {
                // 索引色图像需要调色板
                let mut palette_image = GdImage::new_palette(width, height);
                // 获取调色板
                if let Some(palette) = reader.info().palette.as_ref() {
                    for (i, chunk) in palette.chunks(3).enumerate() {
                        let _ = palette_image.palette.allocate(GdColor::new(chunk[0], chunk[1], chunk[2]));
                    }
                }
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let color_idx = buf[idx] as i32;
                        palette_image.set_pixel(x as i32, y as i32, color_idx);
                    }
                }
                return Ok(palette_image);
            }
        }
        Ok(image)
    }

    /// 加载 JPEG 图像
    fn load_jpeg(data: &[u8]) -> Result<Self, String> {
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
        let pixels = decoder.decode().map_err(|e| format!("JPEG 解码错误: {}", e))?;
        let info = decoder.info().ok_or("无法获取 JPEG 信息")?;
        let width = info.width as u32;
        let height = info.height as u32;
        let mut image = GdImage::new_true_color(width, height);
        match info.pixel_format {
            jpeg_decoder::PixelFormat::RGB24 => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 3) as usize;
                        let r = pixels[idx];
                        let g = pixels[idx + 1];
                        let b = pixels[idx + 2];
                        let color = GdColor::new(r, g, b);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
            jpeg_decoder::PixelFormat::CMYK32 => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let c = pixels[idx] as f32 / 255.0;
                        let m = pixels[idx + 1] as f32 / 255.0;
                        let y_val = pixels[idx + 2] as f32 / 255.0;
                        let k = pixels[idx + 3] as f32 / 255.0;
                        // CMYK 到 RGB 转换
                        let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                        let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                        let b = ((1.0 - y_val) * (1.0 - k) * 255.0) as u8;
                        let color = GdColor::new(r, g, b);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
            jpeg_decoder::PixelFormat::L8 => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let gray = pixels[idx];
                        let color = GdColor::new(gray, gray, gray);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
            jpeg_decoder::PixelFormat::L16 => {
                // 16位灰度图像，取高8位作为灰度值
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx]; // 取高字节
                        let color = GdColor::new(gray, gray, gray);
                        image.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
        }
        Ok(image)
    }

    /// 加载 GIF 图像
    fn load_gif(data: &[u8]) -> Result<Self, String> {
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::Indexed);
        let mut decoder = decoder.read_info(Cursor::new(data)).map_err(|e| format!("GIF 解码错误: {}", e))?;
        let width = decoder.width() as u32;
        let height = decoder.height() as u32;
        let mut image = GdImage::new_palette(width, height);
        // 读取全局调色板
        if let Some(global_palette) = decoder.global_palette() {
            for chunk in global_palette.chunks(3) {
                let _ = image.palette.allocate(GdColor::new(chunk[0], chunk[1], chunk[2]));
            }
        }
        // 读取帧数据到单独的缓冲区
        let buffer_size = (width * height) as usize;
        let mut buffer = vec![0u8; buffer_size];
        decoder.read_into_buffer(&mut buffer).map_err(|e| format!("GIF 帧读取错误: {}", e))?;
        // 设置像素
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let color_idx = buffer[idx] as i32;
                image.set_pixel(x as i32, y as i32, color_idx);
            }
        }
        // 处理透明色 - 使用下一帧信息
        // 注意：简化实现，不处理透明色
        Ok(image)
    }

    /// 加载 BMP 图像
    fn load_bmp(data: &[u8]) -> Result<Self, String> {
        // 简化的 BMP 加载实现
        if data.len() < 54 {
            return Err("BMP 文件太短".to_string());
        }
        // 读取文件头
        let file_size = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        let pixel_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        // 读取信息头
        let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as u32;
        let width = width.unsigned_abs();
        let height = height.unsigned_abs();
        let mut image = GdImage::new_true_color(width, height);
        // 读取像素数据
        let row_size = ((bits_per_pixel * width + 31) / 32) * 4;
        let bytes_per_pixel = (bits_per_pixel / 8) as usize;
        for y in 0..height {
            let row_start = pixel_offset + (height - 1 - y) as usize * row_size as usize;
            for x in 0..width {
                let pixel_start = row_start + x as usize * bytes_per_pixel;
                if pixel_start + bytes_per_pixel <= data.len() {
                    let b = data[pixel_start];
                    let g = data[pixel_start + 1];
                    let r = data[pixel_start + 2];
                    let color = GdColor::new(r, g, b);
                    image.set_pixel(x as i32, y as i32, color.to_int());
                }
            }
        }
        Ok(image)
    }

    /// 加载 WebP 图像
    fn load_webp(data: &[u8]) -> Result<Self, String> {
        // WebP 解码需要 libwebp 或类似库
        // 这里提供简化实现
        Err("WebP 格式支持需要额外的依赖库".to_string())
    }

    /// 保存图像到文件
    /// 
    /// # 参数
    /// - `path`: 文件路径
    /// - `image_type`: 图像类型
    /// 
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn save(&self, path: &Path, image_type: GdImageType) -> Result<(), String> {
        let data = self.to_bytes(image_type)?;
        std::fs::write(path, data).map_err(|e| format!("写入文件失败: {}", e))
    }

    /// 将图像转换为字节数据
    /// 
    /// # 参数
    /// - `image_type`: 图像类型
    /// 
    /// # 返回
    /// 成功返回字节数据，失败返回错误信息
    pub fn to_bytes(&self, image_type: GdImageType) -> Result<Vec<u8>, String> {
        match image_type {
            GdImageType::Png => self.to_png(),
            GdImageType::Jpeg => self.to_jpeg(),
            GdImageType::Gif => self.to_gif(),
            GdImageType::Bmp => self.to_bmp(),
            GdImageType::WebP => self.to_webp(),
            _ => Err("不支持的输出格式".to_string()),
        }
    }

    /// 转换为 PNG 格式
    fn to_png(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, self.width, self.height);
            if self.save_alpha {
                encoder.set_color(png::ColorType::Rgba);
            } else {
                encoder.set_color(png::ColorType::Rgb);
            }
            if self.compression_level >= 0 {
                // png crate 使用 Default 或 Fast 压缩
                // 压缩级别通过其他方式设置
            }
            let mut writer = encoder.write_header().map_err(|e| format!("PNG 编码错误: {}", e))?;
            let mut data = Vec::with_capacity((self.width * self.height * (if self.save_alpha { 4 } else { 3 })) as usize);
            for y in 0..self.height {
                for x in 0..self.width {
                    let color = self.get_pixel_color(x as i32, y as i32);
                    data.push(color.red);
                    data.push(color.green);
                    data.push(color.blue);
                    if self.save_alpha {
                        // GD alpha 0-127 转换为 PNG alpha 0-255
                        let a = 255 - (color.alpha as u32 * 255 / 127) as u8;
                        data.push(a);
                    }
                }
            }
            writer.write_image_data(&data).map_err(|e| format!("PNG 数据写入错误: {}", e))?;
        }
        Ok(buf)
    }

    /// 转换为 JPEG 格式
    fn to_jpeg(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        {
            let mut encoder = jpeg_encoder::Encoder::new(&mut buf, self.jpeg_quality as u8);
            let mut data = Vec::with_capacity((self.width * self.height * 3) as usize);
            for y in 0..self.height {
                for x in 0..self.width {
                    let color = self.get_pixel_color(x as i32, y as i32);
                    data.push(color.red);
                    data.push(color.green);
                    data.push(color.blue);
                }
            }
            encoder.encode(&data, self.width as u16, self.height as u16, jpeg_encoder::ColorType::Rgb)
                .map_err(|e| format!("JPEG 编码错误: {}", e))?;
        }
        Ok(buf)
    }

    /// 转换为 GIF 格式
    fn to_gif(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        {
            let mut encoder = gif::Encoder::new(&mut buf, self.width as u16, self.height as u16, &[]).unwrap();
            let mut frame = gif::Frame::default();
            frame.width = self.width as u16;
            frame.height = self.height as u16;
            // 量化颜色到 256 色
            let pixels: Vec<GdColor> = (0..self.height).flat_map(|y| {
                (0..self.width).map(move |x| self.get_pixel_color(x as i32, y as i32))
            }).collect();
            let mut palette = GdPalette::indexed(256);
            let indices = palette.quantize(&pixels, 256);
            // 构建调色板数据
            let mut palette_data = Vec::with_capacity(768);
            for color in palette.colors() {
                palette_data.push(color.red);
                palette_data.push(color.green);
                palette_data.push(color.blue);
            }
            while palette_data.len() < 768 {
                palette_data.push(0);
            }
            frame.palette = Some(palette_data);
            frame.buffer = std::borrow::Cow::Owned(indices.iter().map(|&i| i as u8).collect());
            encoder.write_frame(&frame).map_err(|e| format!("GIF 编码错误: {}", e))?;
        }
        Ok(buf)
    }

    /// 转换为 BMP 格式
    fn to_bmp(&self) -> Result<Vec<u8>, String> {
        let row_size = ((self.width * 3 + 3) / 4) * 4;
        let pixel_data_size = row_size * self.height;
        let file_size = 54 + pixel_data_size;
        let mut buf = Vec::with_capacity(file_size as usize);
        // 文件头 (14 bytes)
        buf.push(b'B');
        buf.push(b'M');
        buf.extend_from_slice(&file_size.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // 保留
        buf.extend_from_slice(&0u16.to_le_bytes()); // 保留
        buf.extend_from_slice(&54u32.to_le_bytes()); // 像素数据偏移
        // 信息头 (40 bytes)
        buf.extend_from_slice(&40u32.to_le_bytes()); // 信息头大小
        buf.extend_from_slice(&self.width.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // 平面数
        buf.extend_from_slice(&24u16.to_le_bytes()); // 位深度
        buf.extend_from_slice(&0u32.to_le_bytes()); // 压缩方式
        buf.extend_from_slice(&pixel_data_size.to_le_bytes());
        buf.extend_from_slice(&2835u32.to_le_bytes()); // 水平分辨率
        buf.extend_from_slice(&2835u32.to_le_bytes()); // 垂直分辨率
        buf.extend_from_slice(&0u32.to_le_bytes()); // 调色板颜色数
        buf.extend_from_slice(&0u32.to_le_bytes()); // 重要颜色数
        // 像素数据
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let color = self.get_pixel_color(x as i32, y as i32);
                buf.push(color.blue);
                buf.push(color.green);
                buf.push(color.red);
            }
            // 行填充
            let padding = (row_size - self.width * 3) as usize;
            for _ in 0..padding {
                buf.push(0);
            }
        }
        Ok(buf)
    }

    /// 转换为 WebP 格式
    fn to_webp(&self) -> Result<Vec<u8>, String> {
        Err("WebP 格式支持需要额外的依赖库".to_string())
    }

    /// 获取图像宽度
    /// 
    /// # 返回
    /// 返回图像宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取图像高度
    /// 
    /// # 返回
    /// 返回图像高度
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 检查是否为真彩色图像
    /// 
    /// # 返回
    /// 如果是真彩色图像返回 true
    pub fn is_true_color(&self) -> bool {
        self.true_color
    }

    /// 获取调色板
    /// 
    /// # 返回
    /// 返回调色板的引用
    pub fn palette(&self) -> &GdPalette {
        &self.palette
    }

    /// 获取调色板的可变引用
    /// 
    /// # 返回
    /// 返回调色板的可变引用
    pub fn palette_mut(&mut self) -> &mut GdPalette {
        &mut self.palette
    }

    /// 设置像素
    /// 
    /// # 参数
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `color`: 颜色索引或颜色值
    pub fn set_pixel(&mut self, x: i32, y: i32, color: i32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as u32 * self.width + x as u32) as usize;
            self.pixels[idx] = color;
        }
    }

    /// 获取像素
    /// 
    /// # 参数
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// 
    /// # 返回
    /// 返回颜色索引或颜色值
    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as u32 * self.width + x as u32) as usize;
            self.pixels[idx]
        } else {
            0
        }
    }

    /// 获取像素颜色
    /// 
    /// # 参数
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// 
    /// # 返回
    /// 返回颜色值
    pub fn get_pixel_color(&self, x: i32, y: i32) -> GdColor {
        let pixel = self.get_pixel(x, y);
        if self.true_color {
            GdColor::from_int(pixel)
        } else {
            self.palette.get(pixel).unwrap_or(GdColor::black())
        }
    }

    /// 分配颜色
    /// 
    /// # 参数
    /// - `red`: 红色分量
    /// - `green`: 绿色分量
    /// - `blue`: 蓝色分量
    /// 
    /// # 返回
    /// 返回颜色索引
    pub fn color_allocate(&mut self, red: u8, green: u8, blue: u8) -> i32 {
        let color = GdColor::new(red, green, blue);
        self.palette.allocate(color).map(|r| r.index).unwrap_or(-1)
    }

    /// 分配带透明度的颜色
    /// 
    /// # 参数
    /// - `red`: 红色分量
    /// - `green`: 绿色分量
    /// - `blue`: 蓝色分量
    /// - `alpha`: 透明度 (0-127)
    /// 
    /// # 返回
    /// 返回颜色索引
    pub fn color_allocate_alpha(&mut self, red: u8, green: u8, blue: u8, alpha: u8) -> i32 {
        let color = GdColor::new_with_alpha(red, green, blue, alpha);
        self.palette.allocate(color).map(|r| r.index).unwrap_or(-1)
    }

    /// 获取颜色索引
    /// 
    /// # 参数
    /// - `red`: 红色分量
    /// - `green`: 绿色分量
    /// - `blue`: 蓝色分量
    /// 
    /// # 返回
    /// 返回颜色索引，如果不存在返回 -1
    pub fn color_exact(&self, red: u8, green: u8, blue: u8) -> i32 {
        let color = GdColor::new(red, green, blue);
        let color_int = color.to_int();
        self.palette.get_color_index(color_int).unwrap_or(-1)
    }

    /// 获取或分配最接近的颜色
    /// 
    /// # 参数
    /// - `red`: 红色分量
    /// - `green`: 绿色分量
    /// - `blue`: 蓝色分量
    /// 
    /// # 返回
    /// 返回颜色索引
    pub fn color_closest(&mut self, red: u8, green: u8, blue: u8) -> i32 {
        let color = GdColor::new(red, green, blue);
        let (index, _) = self.palette.find_closest(&color);
        index
    }

    /// 设置透明色
    /// 
    /// # 参数
    /// - `color`: 透明色索引
    pub fn set_transparent(&mut self, color: i32) {
        self.transparent = Some(color);
        self.palette.set_transparent(color);
    }

    /// 获取透明色
    /// 
    /// # 返回
    /// 返回透明色索引
    pub fn get_transparent(&self) -> Option<i32> {
        self.transparent
    }

    /// 填充图像
    /// 
    /// # 参数
    /// - `color`: 填充颜色
    pub fn fill(&mut self, color: i32) {
        for pixel in &mut self.pixels {
            *pixel = color;
        }
    }

    /// 设置抗锯齿
    /// 
    /// # 参数
    /// - `enable`: 是否启用
    pub fn set_anti_alias(&mut self, enable: bool) {
        self.anti_alias = enable;
    }

    /// 获取抗锯齿状态
    /// 
    /// # 返回
    /// 返回是否启用抗锯齿
    pub fn get_anti_alias(&self) -> bool {
        self.anti_alias
    }

    /// 设置画笔厚度
    /// 
    /// # 参数
    /// - `thickness`: 厚度值
    pub fn set_thickness(&mut self, thickness: i32) {
        self.thickness = thickness.max(1);
    }

    /// 获取画笔厚度
    /// 
    /// # 返回
    /// 返回画笔厚度
    pub fn get_thickness(&self) -> i32 {
        self.thickness
    }

    /// 设置 Alpha 混合模式
    /// 
    /// # 参数
    /// - `enable`: 是否启用
    pub fn set_alpha_blending(&mut self, enable: bool) {
        self.alpha_blending = enable;
    }

    /// 获取 Alpha 混合模式
    /// 
    /// # 返回
    /// 返回是否启用 Alpha 混合
    pub fn get_alpha_blending(&self) -> bool {
        self.alpha_blending
    }

    /// 设置保存 Alpha
    /// 
    /// # 参数
    /// - `save`: 是否保存
    pub fn set_save_alpha(&mut self, save: bool) {
        self.save_alpha = save;
    }

    /// 获取保存 Alpha 状态
    /// 
    /// # 返回
    /// 返回是否保存 Alpha
    pub fn get_save_alpha(&self) -> bool {
        self.save_alpha
    }

    /// 设置插值方法
    /// 
    /// # 参数
    /// - `method`: 插值方法
    pub fn set_interpolation(&mut self, method: GdInterpolationMethod) {
        self.interpolation = method;
    }

    /// 获取插值方法
    /// 
    /// # 返回
    /// 返回插值方法
    pub fn get_interpolation(&self) -> GdInterpolationMethod {
        self.interpolation
    }

    /// 设置 JPEG 质量
    /// 
    /// # 参数
    /// - `quality`: 质量值 (0-100)
    pub fn set_jpeg_quality(&mut self, quality: i32) {
        self.jpeg_quality = quality.clamp(0, 100);
    }

    /// 获取 JPEG 质量
    /// 
    /// # 返回
    /// 返回 JPEG 质量
    pub fn get_jpeg_quality(&self) -> i32 {
        self.jpeg_quality
    }

    /// 设置 PNG 压缩级别
    /// 
    /// # 参数
    /// - `level`: 压缩级别 (-1 为默认, 0-9)
    pub fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level.clamp(-1, 9);
    }

    /// 获取 PNG 压缩级别
    /// 
    /// # 返回
    /// 返回压缩级别
    pub fn get_compression_level(&self) -> i32 {
        self.compression_level
    }

    /// 复制图像
    /// 
    /// # 返回
    /// 返回图像副本
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// 转换为真彩色图像
    /// 
    /// # 返回
    /// 返回真彩色图像
    pub fn to_true_color(&self) -> Self {
        if self.true_color {
            return self.clone();
        }
        let mut image = GdImage::new_true_color(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_pixel_color(x as i32, y as i32);
                image.set_pixel(x as i32, y as i32, color.to_int());
            }
        }
        image.transparent = self.transparent;
        image.save_alpha = true;
        image
    }

    /// 转换为调色板图像
    /// 
    /// # 参数
    /// - `max_colors`: 最大颜色数
    /// 
    /// # 返回
    /// 返回调色板图像
    pub fn to_palette(&self, max_colors: usize) -> Self {
        if !self.true_color {
            return self.clone();
        }
        let mut image = GdImage::new_palette(self.width, self.height);
        let pixels: Vec<GdColor> = (0..self.height).flat_map(|y| {
            (0..self.width).map(move |x| self.get_pixel_color(x as i32, y as i32))
        }).collect();
        let indices = image.palette.quantize(&pixels, max_colors);
        for (i, &idx) in indices.iter().enumerate() {
            let x = (i % self.width as usize) as i32;
            let y = (i / self.width as usize) as i32;
            image.set_pixel(x, y, idx);
        }
        image
    }

    /// 获取图像信息
    /// 
    /// # 返回
    /// 返回图像信息
    pub fn get_info(&self) -> GdImageInfo {
        GdImageInfo {
            width: self.width,
            height: self.height,
            image_type: GdImageType::Unknown,
            true_color: self.true_color,
            bits: if self.true_color { 32 } else { 8 },
            channels: if self.true_color { 4 } else { 1 },
            mime: String::new(),
        }
    }

    /// 比较两个图像
    /// 
    /// # 参数
    /// - `other`: 另一个图像
    /// 
    /// # 返回
    /// 返回差异像素数
    pub fn compare(&self, other: &GdImage) -> u64 {
        if self.width != other.width || self.height != other.height {
            return (self.width * self.height).max(other.width * other.height) as u64;
        }
        let mut diff = 0u64;
        for y in 0..self.height {
            for x in 0..self.width {
                let c1 = self.get_pixel_color(x as i32, y as i32);
                let c2 = other.get_pixel_color(x as i32, y as i32);
                if c1 != c2 {
                    diff += 1;
                }
            }
        }
        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_true_color_image() {
        let image = GdImage::new_true_color(100, 100);
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
        assert!(image.is_true_color());
    }

    #[test]
    fn test_create_palette_image() {
        let image = GdImage::new_palette(100, 100);
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
        assert!(!image.is_true_color());
    }

    #[test]
    fn test_set_get_pixel() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = GdColor::new(255, 128, 64);
        image.set_pixel(5, 5, color.to_int());
        let pixel = image.get_pixel_color(5, 5);
        assert_eq!(pixel.red, 255);
        assert_eq!(pixel.green, 128);
        assert_eq!(pixel.blue, 64);
    }

    #[test]
    fn test_color_allocate() {
        let mut image = GdImage::new_true_color(10, 10);
        let index = image.color_allocate(255, 0, 0);
        assert!(index >= 0);
    }

    #[test]
    fn test_image_type_detection() {
        let png_sig = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = GdImage::detect_type(&png_sig);
        assert_eq!(result.unwrap(), GdImageType::Png);
    }
}
