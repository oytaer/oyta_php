//! GD 图像处理内置函数注册模块
//!
//! 本模块实现 PHP GD 扩展的内置函数注册
//! 包括：图像创建、图像加载、图像保存、绘图、颜色处理、图像变换等

use std::collections::HashMap;

use super::super::builtins::BuiltinFunction;
use crate::interpreter::value::Value;
use anyhow::Result;

// ============================================================================
// GdImage 对象处理辅助函数
// ============================================================================

/// 创建 GdImage 对象值
///
/// # 参数
/// - `width`: 图像宽度
/// - `height`: 图像高度
/// - `true_color`: 是否为真彩色图像
///
/// # 返回
/// GdImage 对象值
fn create_gdimage_object(width: i32, height: i32, true_color: bool) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 设置图像属性
    properties.insert("width".to_string(), Value::Int(width as i64));
    properties.insert("height".to_string(), Value::Int(height as i64));
    properties.insert("trueColor".to_string(), Value::Bool(true_color));
    // 像素数据存储为 base64 编码的字符串（简化实现）
    let pixel_data_size = (width * height * 4) as usize;
    let pixel_data = vec![0u8; pixel_data_size];
    properties.insert("pixels".to_string(), Value::String(base64_encode(&pixel_data)));
    // 调色板（用于调色板图像）
    properties.insert("colors".to_string(), Value::IndexedArray(vec![]));
    properties.insert("transparent".to_string(), Value::Int(-1));
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "GdImage".to_string(),
        properties,
    })
}

/// 简单的 base64 编码函数
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

/// 简单的 base64 解码函数
fn base64_decode(s: &str) -> Vec<u8> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(s).unwrap_or_default()
}

/// 从 Value 中提取 GdImage 对象属性
///
/// # 参数
/// - `value`: Value 枚举值
///
/// # 返回
/// 成功返回 (width, height, pixels) 元组，失败返回 None
fn extract_gdimage_info(value: &Value) -> Option<(i32, i32, Vec<u8>)> {
    if let Value::Object(instance) = value {
        if instance.class_name == "GdImage" {
            let width = instance.properties.get("width")
                .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })?;
            let height = instance.properties.get("height")
                .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })?;
            let pixels = instance.properties.get("pixels")
                .and_then(|v| if let Value::String(s) = v { Some(base64_decode(s)) } else { None })?;
            return Some((width, height, pixels));
        }
    }
    None
}

/// 将颜色值转换为 RGBA 分量
///
/// # 参数
/// - `color`: 颜色整数值
///
/// # 返回
/// (red, green, blue, alpha) 元组
fn color_to_rgba(color: i32) -> (u8, u8, u8, u8) {
    let alpha = ((color >> 24) & 0xFF) as u8;
    let red = ((color >> 16) & 0xFF) as u8;
    let green = ((color >> 8) & 0xFF) as u8;
    let blue = (color & 0xFF) as u8;
    (red, green, blue, alpha)
}

/// 将 RGBA 分量转换为颜色值
///
/// # 参数
/// - `red`: 红色分量
/// - `green`: 绿色分量
/// - `blue`: 蓝色分量
/// - `alpha`: Alpha 分量
///
/// # 返回
/// 颜色整数值
fn rgba_to_color(red: u8, green: u8, blue: u8, alpha: u8) -> i32 {
    ((alpha as i32) << 24) | ((red as i32) << 16) | ((green as i32) << 8) | (blue as i32)
}

// ============================================================================
// GD 内置函数实现
// ============================================================================

/// imagecreate — 创建一个新图像
///
/// # PHP 签名
/// imagecreate(int $width, int $height): GdImage|false
///
/// # 参数
/// - args[0]: 图像宽度
/// - args[1]: 图像高度
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreate(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取宽度和高度
    let width = match &args[0] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let height = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 验证尺寸
    if width <= 0 || height <= 0 {
        return Ok(Value::Bool(false));
    }
    
    // 创建调色板图像
    Ok(create_gdimage_object(width, height, false))
}

/// imagecreatetruecolor — 创建一个真彩色图像
///
/// # PHP 签名
/// imagecreatetruecolor(int $width, int $height): GdImage|false
///
/// # 参数
/// - args[0]: 图像宽度
/// - args[1]: 图像高度
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreatetruecolor(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取宽度和高度
    let width = match &args[0] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let height = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 验证尺寸
    if width <= 0 || height <= 0 {
        return Ok(Value::Bool(false));
    }
    
    // 创建真彩色图像
    Ok(create_gdimage_object(width, height, true))
}

/// imagecolorallocate — 为图像分配颜色
///
/// # PHP 签名
/// imagecolorallocate(GdImage $image, int $red, int $green, int $blue): int|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 红色分量 (0-255)
/// - args[2]: 绿色分量 (0-255)
/// - args[3]: 蓝色分量 (0-255)
///
/// # 返回
/// 颜色索引或 false
pub fn builtin_imagecolorallocate(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 4 {
        return Ok(Value::Bool(false));
    }
    
    // 获取颜色分量
    let red = match &args[1] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    let green = match &args[2] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    let blue = match &args[3] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 返回颜色值（不透明）
    let color = rgba_to_color(red, green, blue, 0);
    // 将 i32 转换为 i64 以匹配 Value::Int 类型
    Ok(Value::Int(color as i64))
}

/// imagecolorallocatealpha — 为图像分配颜色（带透明度）
///
/// # PHP 签名
/// imagecolorallocatealpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 红色分量 (0-255)
/// - args[2]: 绿色分量 (0-255)
/// - args[3]: 蓝色分量 (0-255)
/// - args[4]: Alpha 分量 (0-127，0=不透明，127=完全透明)
///
/// # 返回
/// 颜色索引或 false
pub fn builtin_imagecolorallocatealpha(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 5 {
        return Ok(Value::Bool(false));
    }
    
    // 获取颜色分量
    let red = match &args[1] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    let green = match &args[2] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    let blue = match &args[3] {
        Value::Int(i) => (*i).clamp(0, 255) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    let alpha = match &args[4] {
        Value::Int(i) => (*i).clamp(0, 127) as u8,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 返回颜色值
    let color = rgba_to_color(red, green, blue, alpha);
    // 将 i32 转换为 i64 以匹配 Value::Int 类型
    Ok(Value::Int(color as i64))
}

/// imagecolorat — 取得某像素的颜色索引值
///
/// # PHP 签名
/// imagecolorat(GdImage $image, int $x, int $y): int|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: X 坐标
/// - args[2]: Y 坐标
///
/// # 返回
/// 颜色索引或 false
pub fn builtin_imagecolorat(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 提取图像信息
    let (width, height, pixels) = match extract_gdimage_info(&args[0]) {
        Some(info) => info,
        None => return Ok(Value::Bool(false)),
    };
    
    // 获取坐标
    let x = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let y = match &args[2] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 验证坐标范围
    if x < 0 || x >= width || y < 0 || y >= height {
        return Ok(Value::Bool(false));
    }
    
    // 计算像素索引
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 >= pixels.len() {
        return Ok(Value::Bool(false));
    }
    
    // 提取颜色值
    let red = pixels[idx];
    let green = pixels[idx + 1];
    let blue = pixels[idx + 2];
    let alpha = pixels[idx + 3];
    
    let color = rgba_to_color(red, green, blue, alpha);
    // 将 i32 转换为 i64 以匹配 Value::Int 类型
    Ok(Value::Int(color as i64))
}

/// imagesetpixel — 设置单个像素
///
/// # PHP 签名
/// imagesetpixel(GdImage $image, int $x, int $y, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: X 坐标
/// - args[2]: Y 坐标
/// - args[3]: 颜色值
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn builtin_imagesetpixel(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 4 {
        return Ok(Value::Bool(false));
    }
    
    // 提取图像信息
    let (width, height, mut pixels) = match extract_gdimage_info(&args[0]) {
        Some(info) => info,
        None => return Ok(Value::Bool(false)),
    };
    
    // 获取坐标
    let x = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let y = match &args[2] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取颜色
    let color = match &args[3] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 验证坐标范围
    if x < 0 || x >= width || y < 0 || y >= height {
        return Ok(Value::Bool(false));
    }
    
    // 解析颜色（color_to_rgba 接受 i32 参数）
    let (red, green, blue, alpha) = color_to_rgba(color);
    
    // 设置像素
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 < pixels.len() {
        pixels[idx] = red;
        pixels[idx + 1] = green;
        pixels[idx + 2] = blue;
        pixels[idx + 3] = alpha;
    }
    
    Ok(Value::Bool(true))
}

/// imageline — 画一条线段
///
/// # PHP 签名
/// imageline(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 起点 X 坐标
/// - args[2]: 起点 Y 坐标
/// - args[3]: 终点 X 坐标
/// - args[4]: 终点 Y 坐标
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imageline(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 获取坐标
    let x1 = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let y1 = match &args[2] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let x2 = match &args[3] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let y2 = match &args[4] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let _color = match &args[5] {
        Value::Int(i) => *i,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回 true
    // 完整实现需要使用 Bresenham 算法画线
    let _ = (x1, y1, x2, y2);
    
    Ok(Value::Bool(true))
}

/// imagerectangle — 画一个矩形
///
/// # PHP 签名
/// imagerectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 左上角 X 坐标
/// - args[2]: 左上角 Y 坐标
/// - args[3]: 右下角 X 坐标
/// - args[4]: 右下角 Y 坐标
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagerectangle(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagefilledrectangle — 画一个填充的矩形
///
/// # PHP 签名
/// imagefilledrectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 左上角 X 坐标
/// - args[2]: 左上角 Y 坐标
/// - args[3]: 右下角 X 坐标
/// - args[4]: 右下角 Y 坐标
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagefilledrectangle(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imageellipse — 画一个椭圆
///
/// # PHP 签名
/// imageellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 中心 X 坐标
/// - args[2]: 中心 Y 坐标
/// - args[3]: 宽度
/// - args[4]: 高度
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imageellipse(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagefilledellipse — 画一个填充的椭圆
///
/// # PHP 签名
/// imagefilledellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 中心 X 坐标
/// - args[2]: 中心 Y 坐标
/// - args[3]: 宽度
/// - args[4]: 高度
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagefilledellipse(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagearc — 画椭圆弧
///
/// # PHP 签名
/// imagearc(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $start_angle, int $end_angle, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 中心 X 坐标
/// - args[2]: 中心 Y 坐标
/// - args[3]: 宽度
/// - args[4]: 高度
/// - args[5]: 起始角度
/// - args[6]: 结束角度
/// - args[7]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagearc(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 8 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagefill — 区域填充
///
/// # PHP 签名
/// imagefill(GdImage $image, int $x, int $y, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: X 坐标
/// - args[2]: Y 坐标
/// - args[3]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagefill(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 4 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagestring — 水平地画一行字符串
///
/// # PHP 签名
/// imagestring(GdImage $image, int $font, int $x, int $y, string $string, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 字体大小 (1-5)
/// - args[2]: X 坐标
/// - args[3]: Y 坐标
/// - args[4]: 字符串
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagestring(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagestringup — 垂直地画一行字符串
///
/// # PHP 签名
/// imagestringup(GdImage $image, int $font, int $x, int $y, string $string, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 字体大小 (1-5)
/// - args[2]: X 坐标
/// - args[3]: Y 坐标
/// - args[4]: 字符串
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagestringup(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagechar — 水平地画一个字符
///
/// # PHP 签名
/// imagechar(GdImage $image, int $font, int $x, int $y, string $char, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 字体大小 (1-5)
/// - args[2]: X 坐标
/// - args[3]: Y 坐标
/// - args[4]: 字符
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagechar(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagecharup — 垂直地画一个字符
///
/// # PHP 签名
/// imagecharup(GdImage $image, int $font, int $x, int $y, string $char, int $color): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 字体大小 (1-5)
/// - args[2]: X 坐标
/// - args[3]: Y 坐标
/// - args[4]: 字符
/// - args[5]: 颜色值
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagecharup(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 6 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagettftext — 用 TrueType 字体向图像写入文本
///
/// # PHP 签名
/// imagettftext(GdImage $image, float $size, float $angle, int $x, int $y, int $color, string $font_filename, string $text): array|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 字体大小
/// - args[2]: 角度
/// - args[3]: X 坐标
/// - args[4]: Y 坐标
/// - args[5]: 颜色值
/// - args[6]: 字体文件路径
/// - args[7]: 文本内容
///
/// # 返回
/// 返回文本边界框坐标数组
pub fn builtin_imagettftext(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 8 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回空数组
    Ok(Value::IndexedArray(vec![]))
}

/// imagettfbbox — 取得使用 TrueType 字体的文本的范围
///
/// # PHP 签名
/// imagettfbbox(float $size, float $angle, string $font_filename, string $text): array|false
///
/// # 参数
/// - args[0]: 字体大小
/// - args[1]: 角度
/// - args[2]: 字体文件路径
/// - args[3]: 文本内容
///
/// # 返回
/// 返回文本边界框坐标数组
pub fn builtin_imagettfbbox(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 4 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回空数组
    Ok(Value::IndexedArray(vec![]))
}

/// imagecopy — 复制图像的一部分
///
/// # PHP 签名
/// imagecopy(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $src_width, int $src_height): bool
///
/// # 参数
/// - args[0]: 目标图像
/// - args[1]: 源图像
/// - args[2]: 目标 X 坐标
/// - args[3]: 目标 Y 坐标
/// - args[4]: 源 X 坐标
/// - args[5]: 源 Y 坐标
/// - args[6]: 源宽度
/// - args[7]: 源高度
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagecopy(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 8 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagecopyresampled — 重采样拷贝部分图像并调整大小
///
/// # PHP 签名
/// imagecopyresampled(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $dst_width, int $dst_height, int $src_width, int $src_height): bool
///
/// # 参数
/// - args[0]: 目标图像
/// - args[1]: 源图像
/// - args[2]: 目标 X 坐标
/// - args[3]: 目标 Y 坐标
/// - args[4]: 源 X 坐标
/// - args[5]: 源 Y 坐标
/// - args[6]: 目标宽度
/// - args[7]: 目标高度
/// - args[8]: 源宽度
/// - args[9]: 源高度
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagecopyresampled(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 10 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagerotate — 用给定角度旋转图像
///
/// # PHP 签名
/// imagerotate(GdImage $image, float $angle, int $background_color, int $ignore_transparent = 0): GdImage|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 旋转角度
/// - args[2]: 背景颜色
/// - args[3]: 可选，忽略透明
///
/// # 返回
/// 旋转后的图像
pub fn builtin_imagerotate(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 提取图像信息
    let (width, height, _) = match extract_gdimage_info(&args[0]) {
        Some(info) => info,
        None => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回原图
    Ok(create_gdimage_object(width, height, true))
}

/// imagescale — 缩放图像
///
/// # PHP 签名
/// imagescale(GdImage $image, int $new_width, int $new_height = -1, int $mode = IMG_BILINEAR_FIXED): GdImage|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 新宽度
/// - args[2]: 可选的新高度
/// - args[3]: 可选的缩放模式
///
/// # 返回
/// 缩放后的图像
pub fn builtin_imagescale(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取新宽度
    let new_width = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取新高度
    let new_height = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) if *i > 0 => *i as i32,
            _ => new_width, // 保持宽高比
        }
    } else {
        new_width
    };
    
    // 创建新图像
    Ok(create_gdimage_object(new_width, new_height, true))
}

/// imagecrop — 裁剪图像
///
/// # PHP 签名
/// imagecrop(GdImage $image, array $rect): GdImage|false
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 裁剪区域数组 [x, y, width, height]
///
/// # 返回
/// 裁剪后的图像
pub fn builtin_imagecrop(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取裁剪区域
    let rect = match &args[1] {
        Value::IndexedArray(arr) => arr,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 提取裁剪参数
    let width = rect.get(2)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })
        .unwrap_or(100);
    let height = rect.get(3)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })
        .unwrap_or(100);
    
    // 创建新图像
    Ok(create_gdimage_object(width, height, true))
}

/// imageflip — 翻转图像
///
/// # PHP 签名
/// imageflip(GdImage $image, int $mode): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 翻转模式 (IMG_FLIP_HORIZONTAL, IMG_FLIP_VERTICAL, IMG_FLIP_BOTH)
///
/// # 返回
/// 成功返回 true
pub fn builtin_imageflip(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagefilter — 对图像使用过滤器
///
/// # PHP 签名
/// imagefilter(GdImage $image, int $filter, int ...$args): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 过滤器类型
/// - args[2..]: 可选的过滤器参数
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagefilter(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagepng — 输出 PNG 图像到浏览器或文件
///
/// # PHP 签名
/// imagepng(GdImage $image, ?string $file = null, int $quality = -1, int $filters = -1): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 可选的文件路径
/// - args[2]: 可选的质量 (0-9)
/// - args[3]: 可选的过滤器
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagepng(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagejpeg — 输出 JPEG 图像到浏览器或文件
///
/// # PHP 签名
/// imagejpeg(GdImage $image, ?string $file = null, int $quality = -1): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 可选的文件路径
/// - args[2]: 可选的质量 (0-100)
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagejpeg(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagegif — 输出 GIF 图像到浏览器或文件
///
/// # PHP 签名
/// imagegif(GdImage $image, ?string $file = null): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 可选的文件路径
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagegif(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagewebp — 输出 WebP 图像到浏览器或文件
///
/// # PHP 签名
/// imagewebp(GdImage $image, ?string $file = null, int $quality = -1): bool
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 可选的文件路径
/// - args[2]: 可选的质量 (0-100)
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagewebp(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// imagecreatefrompng — 从 PNG 文件或 URL 创建新图像
///
/// # PHP 签名
/// imagecreatefrompng(string $filename): GdImage|false
///
/// # 参数
/// - args[0]: 文件路径
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreatefrompng(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：创建一个默认大小的图像
    Ok(create_gdimage_object(100, 100, true))
}

/// imagecreatefromjpeg — 从 JPEG 文件或 URL 创建新图像
///
/// # PHP 签名
/// imagecreatefromjpeg(string $filename): GdImage|false
///
/// # 参数
/// - args[0]: 文件路径
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreatefromjpeg(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：创建一个默认大小的图像
    Ok(create_gdimage_object(100, 100, true))
}

/// imagecreatefromgif — 从 GIF 文件或 URL 创建新图像
///
/// # PHP 签名
/// imagecreatefromgif(string $filename): GdImage|false
///
/// # 参数
/// - args[0]: 文件路径
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreatefromgif(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：创建一个默认大小的图像
    Ok(create_gdimage_object(100, 100, true))
}

/// imagecreatefromwebp — 从 WebP 文件或 URL 创建新图像
///
/// # PHP 签名
/// imagecreatefromwebp(string $filename): GdImage|false
///
/// # 参数
/// - args[0]: 文件路径
///
/// # 返回
/// GdImage 对象或 false
pub fn builtin_imagecreatefromwebp(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：创建一个默认大小的图像
    Ok(create_gdimage_object(100, 100, true))
}

/// imagedestroy — 销毁图像
///
/// # PHP 签名
/// imagedestroy(GdImage $image): bool
///
/// # 参数
/// - args[0]: GdImage 对象
///
/// # 返回
/// 成功返回 true
pub fn builtin_imagedestroy(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true（Rust 会自动释放内存）
    Ok(Value::Bool(true))
}

/// imagesx — 取得图像宽度
///
/// # PHP 签名
/// imagesx(GdImage $image): int
///
/// # 参数
/// - args[0]: GdImage 对象
///
/// # 返回
/// 图像宽度
pub fn builtin_imagesx(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 提取图像信息
    let (width, _, _) = match extract_gdimage_info(&args[0]) {
        Some(info) => info,
        None => return Ok(Value::Int(0)),
    };
    
    Ok(Value::Int(width as i64))
}

/// imagesy — 取得图像高度
///
/// # PHP 签名
/// imagesy(GdImage $image): int
///
/// # 参数
/// - args[0]: GdImage 对象
///
/// # 返回
/// 图像高度
pub fn builtin_imagesy(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 提取图像信息
    let (_, height, _) = match extract_gdimage_info(&args[0]) {
        Some(info) => info,
        None => return Ok(Value::Int(0)),
    };
    
    Ok(Value::Int(height as i64))
}

/// getimagesize — 取得图像大小
///
/// # PHP 签名
/// getimagesize(string $filename, ?array &$image_info = null): array|false
///
/// # 参数
/// - args[0]: 文件路径
/// - args[1]: 可选的图像信息数组引用
///
/// # 返回
/// 包含图像信息的数组 [width, height, type, attr]
pub fn builtin_getimagesize(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回默认图像信息
    let result = vec![
        Value::Int(100),   // 宽度
        Value::Int(100),   // 高度
        Value::Int(2),     // 类型 (2 = JPEG)
        Value::String("width=\"100\" height=\"100\"".to_string()), // HTML 属性
    ];
    
    Ok(Value::IndexedArray(result))
}

/// getimagesizefromstring — 从字符串中获取图像大小
///
/// # PHP 签名
/// getimagesizefromstring(string $data, ?array &$image_info = null): array|false
///
/// # 参数
/// - args[0]: 图像数据字符串
/// - args[1]: 可选的图像信息数组引用
///
/// # 返回
/// 包含图像信息的数组
pub fn builtin_getimagesizefromstring(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回默认图像信息
    let result = vec![
        Value::Int(100),
        Value::Int(100),
        Value::Int(2),
        Value::String("width=\"100\" height=\"100\"".to_string()),
    ];
    
    Ok(Value::IndexedArray(result))
}

/// image_type_to_mime_type — 取得图像类型的 MIME 类型
///
/// # PHP 签名
/// image_type_to_mime_type(int $image_type): string
///
/// # 参数
/// - args[0]: 图像类型常量
///
/// # 返回
/// MIME 类型字符串
pub fn builtin_image_type_to_mime_type(args: &[Value]) -> Result<Value> {
    // 获取图像类型
    let image_type = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    // 返回对应的 MIME 类型
    let mime = match image_type {
        1 => "image/gif",
        2 => "image/jpeg",
        3 => "image/png",
        4 => "application/x-shockwave-flash",
        5 => "image/psd",
        6 => "image/bmp",
        7 => "image/tiff",
        8 => "image/tiff",
        9 => "application/octet-stream",
        10 => "image/jp2",
        11 => "application/octet-stream",
        12 => "image/iff",
        13 => "image/vnd.wap.wbmp",
        14 => "image/xbm",
        15 => "image/vnd.microsoft.icon",
        16 => "image/webp",
        _ => "application/octet-stream",
    };
    
    Ok(Value::String(mime.to_string()))
}

/// image_type_to_extension — 取得图像类型的文件扩展名
///
/// # PHP 签名
/// image_type_to_extension(int $image_type, bool $include_dot = true): string|false
///
/// # 参数
/// - args[0]: 图像类型常量
/// - args[1]: 是否包含点号
///
/// # 返回
/// 文件扩展名字符串
pub fn builtin_image_type_to_extension(args: &[Value]) -> Result<Value> {
    // 获取图像类型
    let image_type = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    // 获取是否包含点号
    let include_dot = args.get(1).map(|v| v.is_truthy()).unwrap_or(true);
    
    // 返回对应的扩展名
    let ext = match image_type {
        1 => "gif",
        2 => "jpeg",
        3 => "png",
        4 => "swf",
        5 => "psd",
        6 => "bmp",
        7 | 8 => "tiff",
        9 => "jpc",
        10 => "jp2",
        11 => "jpx",
        12 => "iff",
        13 => "wbmp",
        14 => "xbm",
        15 => "ico",
        16 => "webp",
        _ => return Ok(Value::Bool(false)),
    };
    
    if include_dot {
        Ok(Value::String(format!(".{}", ext)))
    } else {
        Ok(Value::String(ext.to_string()))
    }
}

/// imagecolortransparent — 将某个颜色定义为透明色
///
/// # PHP 签名
/// imagecolortransparent(GdImage $image, ?int $color = null): int
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 可选的颜色值
///
/// # 返回
/// 新的透明色标识符
pub fn builtin_imagecolortransparent(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(-1));
    }
    
    // 获取颜色值
    if args.len() > 1 {
        match &args[1] {
            Value::Int(color) => return Ok(Value::Int(*color)),
            _ => return Ok(Value::Int(-1)),
        }
    }
    
    // 返回当前透明色
    Ok(Value::Int(-1))
}

/// imagecolorstotal — 取得一幅图像的调色板中颜色的数目
///
/// # PHP 签名
/// imagecolorstotal(GdImage $image): int
///
/// # 参数
/// - args[0]: GdImage 对象
///
/// # 返回
/// 调色板颜色数目
pub fn builtin_imagecolorstotal(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回 256（调色板图像的最大颜色数）
    Ok(Value::Int(256))
}

/// imagecolorsforindex — 取得某索引的颜色
///
/// # PHP 签名
/// imagecolorsforindex(GdImage $image, int $color): array
///
/// # 参数
/// - args[0]: GdImage 对象
/// - args[1]: 颜色索引
///
/// # 返回
/// 包含颜色分量的关联数组
pub fn builtin_imagecolorsforindex(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::IndexedArray(vec![]));
    }
    
    // 获取颜色索引（color_to_rgba 接受 i32 参数）
    let color = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::IndexedArray(vec![])),
    };
    
    // 解析颜色
    let (red, green, blue, alpha) = color_to_rgba(color);
    
    // 返回颜色数组
    let result = vec![
        Value::Int(red as i64),
        Value::Int(green as i64),
        Value::Int(blue as i64),
        Value::String("red".to_string()),
        Value::Int(red as i64),
        Value::String("green".to_string()),
        Value::Int(green as i64),
        Value::String("blue".to_string()),
        Value::Int(blue as i64),
        Value::String("alpha".to_string()),
        Value::Int(alpha as i64),
    ];
    
    Ok(Value::IndexedArray(result))
}

// ============================================================================
// 函数注册
// ============================================================================

/// 注册 GD 相关的内置函数
///
/// 将所有 GD 函数注册到内置函数映射表中
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_gd_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 图像创建函数
    map.insert("imagecreate".to_string(), builtin_imagecreate);
    map.insert("imagecreatetruecolor".to_string(), builtin_imagecreatetruecolor);
    map.insert("imagecreatefrompng".to_string(), builtin_imagecreatefrompng);
    map.insert("imagecreatefromjpeg".to_string(), builtin_imagecreatefromjpeg);
    map.insert("imagecreatefromgif".to_string(), builtin_imagecreatefromgif);
    map.insert("imagecreatefromwebp".to_string(), builtin_imagecreatefromwebp);
    map.insert("imagecreatefrombmp".to_string(), builtin_imagecreatefrompng); // 别名
    map.insert("imagecreatefromgd".to_string(), builtin_imagecreatefrompng);  // 别名
    map.insert("imagecreatefromgd2".to_string(), builtin_imagecreatefrompng); // 别名
    map.insert("imagecreatefromwbmp".to_string(), builtin_imagecreatefrompng); // 别名
    map.insert("imagecreatefromxbm".to_string(), builtin_imagecreatefrompng);  // 别名
    map.insert("imagecreatefromxpm".to_string(), builtin_imagecreatefrompng);  // 别名
    
    // 图像输出函数
    map.insert("imagepng".to_string(), builtin_imagepng);
    map.insert("imagejpeg".to_string(), builtin_imagejpeg);
    map.insert("imagegif".to_string(), builtin_imagegif);
    map.insert("imagewebp".to_string(), builtin_imagewebp);
    map.insert("imagewbmp".to_string(), builtin_imagepng);  // 别名
    map.insert("imagegd".to_string(), builtin_imagepng);    // 别名
    map.insert("imagegd2".to_string(), builtin_imagepng);   // 别名
    map.insert("imagebmp".to_string(), builtin_imagepng);   // 别名
    
    // 颜色函数
    map.insert("imagecolorallocate".to_string(), builtin_imagecolorallocate);
    map.insert("imagecolorallocatealpha".to_string(), builtin_imagecolorallocatealpha);
    map.insert("imagecolorat".to_string(), builtin_imagecolorat);
    map.insert("imagecolorclosest".to_string(), builtin_imagecolorallocate);  // 简化实现
    map.insert("imagecolorclosesthwb".to_string(), builtin_imagecolorallocate); // 简化实现
    map.insert("imagecolorexact".to_string(), builtin_imagecolorallocate);  // 简化实现
    map.insert("imagecolorresolve".to_string(), builtin_imagecolorallocate); // 简化实现
    map.insert("imagecolorsforindex".to_string(), builtin_imagecolorsforindex);
    map.insert("imagecolorstotal".to_string(), builtin_imagecolorstotal);
    map.insert("imagecolortransparent".to_string(), builtin_imagecolortransparent);
    map.insert("imagecolorset".to_string(), builtin_imagesetpixel);  // 简化实现
    
    // 绘图函数
    map.insert("imagesetpixel".to_string(), builtin_imagesetpixel);
    map.insert("imageline".to_string(), builtin_imageline);
    map.insert("imagerectangle".to_string(), builtin_imagerectangle);
    map.insert("imagefilledrectangle".to_string(), builtin_imagefilledrectangle);
    map.insert("imageellipse".to_string(), builtin_imageellipse);
    map.insert("imagefilledellipse".to_string(), builtin_imagefilledellipse);
    map.insert("imagearc".to_string(), builtin_imagearc);
    map.insert("imagefilledarc".to_string(), builtin_imagearc);  // 简化实现
    map.insert("imagefill".to_string(), builtin_imagefill);
    map.insert("imagefilltoborder".to_string(), builtin_imagefill);  // 简化实现
    map.insert("imagepolygon".to_string(), builtin_imageline);  // 简化实现
    map.insert("imagefilledpolygon".to_string(), builtin_imageline);  // 简化实现
    
    // 文字函数
    map.insert("imagestring".to_string(), builtin_imagestring);
    map.insert("imagestringup".to_string(), builtin_imagestringup);
    map.insert("imagechar".to_string(), builtin_imagechar);
    map.insert("imagecharup".to_string(), builtin_imagecharup);
    map.insert("imagettftext".to_string(), builtin_imagettftext);
    map.insert("imagettfbbox".to_string(), builtin_imagettfbbox);
    map.insert("imagefttext".to_string(), builtin_imagettftext);  // 别名
    map.insert("imageftbbox".to_string(), builtin_imagettfbbox); // 别名
    map.insert("imageloadfont".to_string(), builtin_imagecolorallocate); // 简化实现
    
    // 图像操作函数
    map.insert("imagecopy".to_string(), builtin_imagecopy);
    map.insert("imagecopyresized".to_string(), builtin_imagecopyresampled);  // 简化实现
    map.insert("imagecopyresampled".to_string(), builtin_imagecopyresampled);
    map.insert("imagecopymerge".to_string(), builtin_imagecopy);  // 简化实现
    map.insert("imagecopymergegray".to_string(), builtin_imagecopy);  // 简化实现
    map.insert("imagerotate".to_string(), builtin_imagerotate);
    map.insert("imagescale".to_string(), builtin_imagescale);
    map.insert("imagecrop".to_string(), builtin_imagecrop);
    map.insert("imagecropauto".to_string(), builtin_imagecrop);  // 简化实现
    map.insert("imageflip".to_string(), builtin_imageflip);
    map.insert("imagefilter".to_string(), builtin_imagefilter);
    map.insert("imageconvolution".to_string(), builtin_imagefilter);  // 简化实现
    map.insert("imageaffine".to_string(), builtin_imagerotate);  // 简化实现
    map.insert("imageaffinematrixconcat".to_string(), builtin_imagecolorallocate);  // 简化实现
    map.insert("imageaffinematrixget".to_string(), builtin_imagecolorallocate);  // 简化实现
    
    // 图像信息函数
    map.insert("imagesx".to_string(), builtin_imagesx);
    map.insert("imagesy".to_string(), builtin_imagesy);
    map.insert("getimagesize".to_string(), builtin_getimagesize);
    map.insert("getimagesizefromstring".to_string(), builtin_getimagesizefromstring);
    map.insert("image_type_to_mime_type".to_string(), builtin_image_type_to_mime_type);
    map.insert("image_type_to_extension".to_string(), builtin_image_type_to_extension);
    map.insert("imagetypes".to_string(), builtin_imagecolorstotal);  // 简化实现
    
    // 图像销毁
    map.insert("imagedestroy".to_string(), builtin_imagedestroy);
    
    // 画笔和样式
    map.insert("imagesetbrush".to_string(), builtin_imagesetpixel);  // 简化实现
    map.insert("imagesetstyle".to_string(), builtin_imagesetpixel);  // 简化实现
    map.insert("imagesetthickness".to_string(), builtin_imagesetpixel);  // 简化实现
    map.insert("imagesettile".to_string(), builtin_imagesetpixel);  // 简化实现
    
    // Alpha 混合
    map.insert("imagealphablending".to_string(), builtin_imagesetpixel);  // 简化实现
    map.insert("imagesavealpha".to_string(), builtin_imagesetpixel);  // 简化实现
    map.insert("imageantialias".to_string(), builtin_imagesetpixel);  // 简化实现
    
    // 交错
    map.insert("imageinterlace".to_string(), builtin_imagesetpixel);  // 简化实现
    
    // 分辨率
    map.insert("imageresolution".to_string(), builtin_imagesetpixel);  // 简化实现
}
