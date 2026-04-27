//! 颜色处理模块
//! 
//! 本模块实现 GD 图像的颜色处理功能，包括：
//! - RGB/RGBA 颜色表示
//! - 颜色分配和管理
//! - 调色板操作
//! - 颜色转换工具
//! - 透明度处理

use std::collections::HashMap;

/// RGB 颜色结构
/// 
/// 表示一个 RGB 颜色值，每个通道范围 0-255
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GdColor {
    /// 红色通道 (0-255)
    pub red: u8,
    /// 绿色通道 (0-255)
    pub green: u8,
    /// 蓝色通道 (0-255)
    pub blue: u8,
    /// Alpha 透明通道 (0-127, 0 表示完全不透明, 127 表示完全透明)
    pub alpha: u8,
}

impl GdColor {
    /// 创建新的 RGB 颜色（不透明）
    /// 
    /// # 参数
    /// - `red`: 红色通道值 (0-255)
    /// - `green`: 绿色通道值 (0-255)
    /// - `blue`: 蓝色通道值 (0-255)
    /// 
    /// # 返回
    /// 返回不透明的 RGB 颜色
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        GdColor {
            red,
            green,
            blue,
            alpha: 0,
        }
    }

    /// 创建新的 RGBA 颜色
    /// 
    /// # 参数
    /// - `red`: 红色通道值 (0-255)
    /// - `green`: 绿色通道值 (0-255)
    /// - `blue`: 蓝色通道值 (0-255)
    /// - `alpha`: Alpha 透明度 (0-127, 0 表示完全不透明)
    /// 
    /// # 返回
    /// 返回带透明度的 RGBA 颜色
    pub fn new_with_alpha(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        GdColor {
            red,
            green,
            blue,
            alpha: alpha.min(127),
        }
    }

    /// 从整数创建颜色
    /// 
    /// 将整数颜色值解析为 RGB 或 RGBA 颜色
    /// 
    /// # 参数
    /// - `color`: 整数颜色值
    /// 
    /// # 返回
    /// 返回解析后的颜色
    pub fn from_int(color: i32) -> Self {
        let alpha = ((color >> 24) & 0xFF) as u8;
        let red = ((color >> 16) & 0xFF) as u8;
        let green = ((color >> 8) & 0xFF) as u8;
        let blue = (color & 0xFF) as u8;
        GdColor {
            red,
            green,
            blue,
            alpha: alpha.min(127),
        }
    }

    /// 转换为整数颜色值
    /// 
    /// 将颜色转换为整数表示
    /// 
    /// # 返回
    /// 返回整数颜色值
    pub fn to_int(&self) -> i32 {
        let alpha = self.alpha as i32;
        let red = self.red as i32;
        let green = self.green as i32;
        let blue = self.blue as i32;
        (alpha << 24) | (red << 16) | (green << 8) | blue
    }

    /// 从十六进制字符串创建颜色
    /// 
    /// 支持格式：#RGB, #RRGGBB, #RRGGBBAA
    /// 
    /// # 参数
    /// - `hex`: 十六进制颜色字符串
    /// 
    /// # 返回
    /// 成功返回颜色，失败返回错误信息
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| "无效的十六进制颜色")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| "无效的十六进制颜色")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| "无效的十六进制颜色")?;
                Ok(GdColor::new(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "无效的十六进制颜色")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "无效的十六进制颜色")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "无效的十六进制颜色")?;
                Ok(GdColor::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "无效的十六进制颜色")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "无效的十六进制颜色")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "无效的十六进制颜色")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "无效的十六进制颜色")?;
                // 将 0-255 的 alpha 转换为 0-127
                let alpha = (a as f32 * 127.0 / 255.0) as u8;
                Ok(GdColor::new_with_alpha(r, g, b, alpha))
            }
            _ => Err("无效的十六进制颜色格式".to_string()),
        }
    }

    /// 转换为十六进制字符串
    /// 
    /// # 参数
    /// - `include_alpha`: 是否包含 alpha 通道
    /// 
    /// # 返回
    /// 返回十六进制颜色字符串
    pub fn to_hex(&self, include_alpha: bool) -> String {
        if include_alpha && self.alpha > 0 {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.red, self.green, self.blue, self.alpha)
        } else {
            format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
        }
    }

    /// 从 HSL 创建颜色
    /// 
    /// # 参数
    /// - `h`: 色相 (0-360)
    /// - `s`: 饱和度 (0-100)
    /// - `l`: 亮度 (0-100)
    /// 
    /// # 返回
    /// 返回 RGB 颜色
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let s = s / 100.0;
        let l = l / 100.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        GdColor::new(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// 转换为 HSL
    /// 
    /// # 返回
    /// 返回 (色相, 饱和度, 亮度) 元组
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.red as f32 / 255.0;
        let g = self.green as f32 / 255.0;
        let b = self.blue as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if max == min {
            return (0.0, 0.0, l * 100.0);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if max == r {
            60.0 * (((g - b) / d) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };
        (h, s * 100.0, l * 100.0)
    }

    /// 检查是否为透明色
    /// 
    /// # 返回
    /// 如果 alpha > 0 返回 true
    pub fn is_transparent(&self) -> bool {
        self.alpha > 0
    }

    /// 检查是否为完全透明
    /// 
    /// # 返回
    /// 如果 alpha == 127 返回 true
    pub fn is_fully_transparent(&self) -> bool {
        self.alpha == 127
    }

    /// 检查是否为完全不透明
    /// 
    /// # 返回
    /// 如果 alpha == 0 返回 true
    pub fn is_opaque(&self) -> bool {
        self.alpha == 0
    }

    /// 设置透明度
    /// 
    /// # 参数
    /// - `alpha`: 新的透明度值 (0-127)
    /// 
    /// # 返回
    /// 返回新的颜色
    pub fn with_alpha(&self, alpha: u8) -> Self {
        GdColor {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: alpha.min(127),
        }
    }

    /// 混合两个颜色
    /// 
    /// # 参数
    /// - `other`: 另一个颜色
    /// - `ratio`: 混合比例 (0.0-1.0, 0.0 表示完全使用 self, 1.0 表示完全使用 other)
    /// 
    /// # 返回
    /// 返回混合后的颜色
    pub fn blend(&self, other: &GdColor, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let r = (self.red as f32 * (1.0 - ratio) + other.red as f32 * ratio) as u8;
        let g = (self.green as f32 * (1.0 - ratio) + other.green as f32 * ratio) as u8;
        let b = (self.blue as f32 * (1.0 - ratio) + other.blue as f32 * ratio) as u8;
        let a = (self.alpha as f32 * (1.0 - ratio) + other.alpha as f32 * ratio) as u8;
        GdColor::new_with_alpha(r, g, b, a)
    }

    /// 调整亮度
    /// 
    /// # 参数
    /// - `amount`: 亮度调整量 (-100 到 100)
    /// 
    /// # 返回
    /// 返回调整后的颜色
    pub fn adjust_brightness(&self, amount: i32) -> Self {
        let adjust = |v: u8| -> u8 {
            let new_val = v as i32 + amount;
            new_val.clamp(0, 255) as u8
        };
        GdColor::new_with_alpha(adjust(self.red), adjust(self.green), adjust(self.blue), self.alpha)
    }

    /// 转换为灰度
    /// 
    /// # 返回
    /// 返回灰度颜色
    pub fn to_grayscale(&self) -> Self {
        // 使用加权平均法计算灰度值
        let gray = (self.red as f32 * 0.299 + self.green as f32 * 0.587 + self.blue as f32 * 0.114) as u8;
        GdColor::new_with_alpha(gray, gray, gray, self.alpha)
    }

    /// 反转颜色
    /// 
    /// # 返回
    /// 返回反转后的颜色
    pub fn invert(&self) -> Self {
        GdColor::new_with_alpha(255 - self.red, 255 - self.green, 255 - self.blue, self.alpha)
    }

    /// 计算与另一个颜色的距离
    /// 
    /// # 参数
    /// - `other`: 另一个颜色
    /// 
    /// # 返回
    /// 返回颜色距离 (0-441.67)
    pub fn distance(&self, other: &GdColor) -> f32 {
        let dr = self.red as f32 - other.red as f32;
        let dg = self.green as f32 - other.green as f32;
        let db = self.blue as f32 - other.blue as f32;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    /// 计算与另一个颜色的相似度
    /// 
    /// # 参数
    /// - `other`: 另一个颜色
    /// 
    /// # 返回
    /// 返回相似度 (0.0-1.0)
    pub fn similarity(&self, other: &GdColor) -> f32 {
        let max_distance = (255.0_f32 * 255.0 * 3.0).sqrt();
        1.0 - self.distance(other) / max_distance
    }

    /// 创建白色
    pub fn white() -> Self {
        GdColor::new(255, 255, 255)
    }

    /// 创建黑色
    pub fn black() -> Self {
        GdColor::new(0, 0, 0)
    }

    /// 创建红色
    pub fn red() -> Self {
        GdColor::new(255, 0, 0)
    }

    /// 创建绿色
    pub fn green() -> Self {
        GdColor::new(0, 255, 0)
    }

    /// 创建蓝色
    pub fn blue() -> Self {
        GdColor::new(0, 0, 255)
    }

    /// 创建透明色
    pub fn transparent() -> Self {
        GdColor::new_with_alpha(0, 0, 0, 127)
    }
}

impl Default for GdColor {
    fn default() -> Self {
        GdColor::black()
    }
}

/// 颜色分配结果
/// 
/// 表示颜色分配操作的结果
#[derive(Debug, Clone)]
pub struct GdColorAllocateResult {
    /// 颜色索引
    pub index: i32,
    /// 颜色值
    pub color: GdColor,
    /// 是否为新分配的颜色
    pub is_new: bool,
}

impl GdColorAllocateResult {
    /// 创建新的分配结果
    pub fn new(index: i32, color: GdColor, is_new: bool) -> Self {
        GdColorAllocateResult {
            index,
            color,
            is_new,
        }
    }
}

/// 调色板
/// 
/// 管理图像的颜色调色板，支持真彩色和索引色图像
#[derive(Debug, Clone)]
pub struct GdPalette {
    /// 颜色列表
    colors: Vec<GdColor>,
    /// 颜色到索引的映射
    color_map: HashMap<i32, i32>,
    /// 透明色索引
    transparent_index: Option<i32>,
    /// 是否为真彩色模式
    true_color: bool,
    /// 最大颜色数（仅对索引色图像有效）
    max_colors: usize,
}

impl GdPalette {
    /// 创建新的调色板
    /// 
    /// # 参数
    /// - `true_color`: 是否为真彩色模式
    /// - `max_colors`: 最大颜色数（仅对索引色图像有效）
    /// 
    /// # 返回
    /// 返回新的调色板
    pub fn new(true_color: bool, max_colors: usize) -> Self {
        GdPalette {
            colors: Vec::new(),
            color_map: HashMap::new(),
            transparent_index: None,
            true_color,
            max_colors: if true_color { 0 } else { max_colors.min(256) },
        }
    }

    /// 创建真彩色调色板
    pub fn true_color() -> Self {
        GdPalette::new(true, 0)
    }

    /// 创建索引色调色板
    pub fn indexed(max_colors: usize) -> Self {
        GdPalette::new(false, max_colors)
    }

    /// 分配颜色
    /// 
    /// # 参数
    /// - `color`: 要分配的颜色
    /// 
    /// # 返回
    /// 成功返回颜色分配结果，失败返回错误信息
    pub fn allocate(&mut self, color: GdColor) -> Result<GdColorAllocateResult, String> {
        let color_int = color.to_int();
        // 检查颜色是否已存在
        if let Some(&index) = self.color_map.get(&color_int) {
            return Ok(GdColorAllocateResult::new(index, color, false));
        }
        // 真彩色模式直接返回颜色值作为索引
        if self.true_color {
            let index = color_int;
            self.color_map.insert(color_int, index);
            return Ok(GdColorAllocateResult::new(index, color, true));
        }
        // 索引色模式检查是否超出最大颜色数
        if self.colors.len() >= self.max_colors {
            return Err("调色板已满，无法分配更多颜色".to_string());
        }
        let index = self.colors.len() as i32;
        self.colors.push(color);
        self.color_map.insert(color_int, index);
        Ok(GdColorAllocateResult::new(index, color, true))
    }

    /// 分配或获取最接近的颜色
    /// 
    /// # 参数
    /// - `color`: 要分配的颜色
    /// 
    /// # 返回
    /// 返回颜色分配结果
    pub fn allocate_or_closest(&mut self, color: GdColor) -> GdColorAllocateResult {
        match self.allocate(color.clone()) {
            Ok(result) => result,
            Err(_) => {
                // 调色板已满，查找最接近的颜色
                let (index, closest) = self.find_closest(&color);
                GdColorAllocateResult::new(index, closest, false)
            }
        }
    }

    /// 查找最接近的颜色
    /// 
    /// # 参数
    /// - `color`: 目标颜色
    /// 
    /// # 返回
    /// 返回最接近的颜色索引和颜色值
    pub fn find_closest(&self, color: &GdColor) -> (i32, GdColor) {
        if self.true_color {
            return (color.to_int(), color.clone());
        }
        let mut min_distance = f32::MAX;
        let mut closest_index = 0;
        let mut closest_color = GdColor::black();
        for (index, c) in self.colors.iter().enumerate() {
            let distance = color.distance(c);
            if distance < min_distance {
                min_distance = distance;
                closest_index = index as i32;
                closest_color = c.clone();
            }
        }
        (closest_index, closest_color)
    }

    /// 获取颜色
    /// 
    /// # 参数
    /// - `index`: 颜色索引
    /// 
    /// # 返回
    /// 返回颜色值，如果索引无效返回 None
    pub fn get(&self, index: i32) -> Option<GdColor> {
        if self.true_color {
            Some(GdColor::from_int(index))
        } else {
            self.colors.get(index as usize).cloned()
        }
    }

    /// 根据颜色整数值获取索引
    /// 
    /// # 参数
    /// - `color_int`: 颜色整数值
    /// 
    /// # 返回
    /// 返回颜色索引，如果不存在返回 None
    pub fn get_color_index(&self, color_int: i32) -> Option<i32> {
        self.color_map.get(&color_int).copied()
    }

    /// 设置透明色
    /// 
    /// # 参数
    /// - `index`: 透明色索引
    pub fn set_transparent(&mut self, index: i32) {
        self.transparent_index = Some(index);
    }

    /// 获取透明色索引
    /// 
    /// # 返回
    /// 返回透明色索引
    pub fn get_transparent(&self) -> Option<i32> {
        self.transparent_index
    }

    /// 清除透明色设置
    pub fn clear_transparent(&mut self) {
        self.transparent_index = None;
    }

    /// 获取颜色数量
    /// 
    /// # 返回
    /// 返回调色板中的颜色数量
    pub fn color_count(&self) -> usize {
        if self.true_color {
            self.color_map.len()
        } else {
            self.colors.len()
        }
    }

    /// 检查是否为真彩色模式
    /// 
    /// # 返回
    /// 如果是真彩色模式返回 true
    pub fn is_true_color(&self) -> bool {
        self.true_color
    }

    /// 释放颜色
    /// 
    /// # 参数
    /// - `index`: 要释放的颜色索引
    /// 
    /// # 返回
    /// 成功返回 true
    pub fn deallocate(&mut self, index: i32) -> bool {
        if self.true_color {
            return false;
        }
        if let Some(color) = self.colors.get(index as usize).cloned() {
            self.color_map.remove(&color.to_int());
            return true;
        }
        false
    }

    /// 清空调色板
    pub fn clear(&mut self) {
        self.colors.clear();
        self.color_map.clear();
        self.transparent_index = None;
    }

    /// 获取所有颜色
    /// 
    /// # 返回
    /// 返回颜色列表的引用
    pub fn colors(&self) -> &[GdColor] {
        &self.colors
    }

    /// 将颜色映射到调色板
    /// 
    /// 将图像的所有颜色映射到调色板中
    /// 
    /// # 参数
    /// - `pixels`: 像素数据
    /// 
    /// # 返回
    /// 返回映射后的像素索引
    pub fn map_to_palette(&mut self, pixels: &[GdColor]) -> Vec<i32> {
        pixels.iter().map(|c| {
            let result = self.allocate_or_closest(c.clone());
            result.index
        }).collect()
    }

    /// 量化颜色
    /// 
    /// 将真彩色图像量化为索引色图像
    /// 
    /// # 参数
    /// - `pixels`: 像素数据
    /// - `max_colors`: 最大颜色数
    /// 
    /// # 返回
    /// 返回量化后的像素索引
    pub fn quantize(&mut self, pixels: &[GdColor], max_colors: usize) -> Vec<i32> {
        self.true_color = false;
        self.max_colors = max_colors.min(256);
        self.colors.clear();
        self.color_map.clear();
        // 简单的中位切分量化算法
        self.median_cut_quantize(pixels, max_colors)
    }

    /// 中位切分量化算法
    fn median_cut_quantize(&mut self, pixels: &[GdColor], max_colors: usize) -> Vec<i32> {
        if pixels.is_empty() {
            return Vec::new();
        }
        // 初始颜色桶
        let mut buckets = vec![pixels.to_vec()];
        // 分割直到达到最大颜色数
        while buckets.len() < max_colors {
            // 找到范围最大的桶
            let mut max_range_idx = 0;
            let mut max_range = 0.0f32;
            for (i, bucket) in buckets.iter().enumerate() {
                let range = Self::bucket_range(bucket);
                if range > max_range {
                    max_range = range;
                    max_range_idx = i;
                }
            }
            if max_range == 0.0 {
                break;
            }
            // 分割桶
            let bucket = buckets.remove(max_range_idx);
            let (left, right) = Self::split_bucket(&bucket);
            if !left.is_empty() {
                buckets.push(left);
            }
            if !right.is_empty() {
                buckets.push(right);
            }
        }
        // 计算每个桶的平均颜色
        for bucket in &buckets {
            let avg = Self::average_color(bucket);
            let _ = self.allocate(avg);
        }
        // 映射像素到调色板
        pixels.iter().map(|c| {
            let (index, _) = self.find_closest(c);
            index
        }).collect()
    }

    /// 计算桶的颜色范围
    fn bucket_range(bucket: &[GdColor]) -> f32 {
        if bucket.is_empty() {
            return 0.0;
        }
        let mut min_r = 255u8;
        let mut max_r = 0u8;
        let mut min_g = 255u8;
        let mut max_g = 0u8;
        let mut min_b = 255u8;
        let mut max_b = 0u8;
        for c in bucket {
            min_r = min_r.min(c.red);
            max_r = max_r.max(c.red);
            min_g = min_g.min(c.green);
            max_g = max_g.max(c.green);
            min_b = min_b.min(c.blue);
            max_b = max_b.max(c.blue);
        }
        let range_r = (max_r - min_r) as f32;
        let range_g = (max_g - min_g) as f32;
        let range_b = (max_b - min_b) as f32;
        range_r.max(range_g).max(range_b)
    }

    /// 分割桶
    fn split_bucket(bucket: &[GdColor]) -> (Vec<GdColor>, Vec<GdColor>) {
        if bucket.len() < 2 {
            return (bucket.to_vec(), Vec::new());
        }
        // 找到范围最大的通道
        let mut min_r = 255u8;
        let mut max_r = 0u8;
        let mut min_g = 255u8;
        let mut max_g = 0u8;
        let mut min_b = 255u8;
        let mut max_b = 0u8;
        for c in bucket {
            min_r = min_r.min(c.red);
            max_r = max_r.max(c.red);
            min_g = min_g.min(c.green);
            max_g = max_g.max(c.green);
            min_b = min_b.min(c.blue);
            max_b = max_b.max(c.blue);
        }
        let range_r = (max_r - min_r) as f32;
        let range_g = (max_g - min_g) as f32;
        let range_b = (max_b - min_b) as f32;
        let mut sorted = bucket.to_vec();
        if range_r >= range_g && range_r >= range_b {
            sorted.sort_by_key(|c| c.red);
        } else if range_g >= range_b {
            sorted.sort_by_key(|c| c.green);
        } else {
            sorted.sort_by_key(|c| c.blue);
        }
        let mid = sorted.len() / 2;
        let left = sorted[..mid].to_vec();
        let right = sorted[mid..].to_vec();
        (left, right)
    }

    /// 计算桶的平均颜色
    fn average_color(bucket: &[GdColor]) -> GdColor {
        if bucket.is_empty() {
            return GdColor::black();
        }
        let sum: (u32, u32, u32, u32) = bucket.iter().fold((0, 0, 0, 0), |acc, c| {
            (acc.0 + c.red as u32, acc.1 + c.green as u32, acc.2 + c.blue as u32, acc.3 + c.alpha as u32)
        });
        let count = bucket.len() as u32;
        GdColor::new_with_alpha(
            (sum.0 / count) as u8,
            (sum.1 / count) as u8,
            (sum.2 / count) as u8,
            (sum.3 / count) as u8,
        )
    }
}

impl Default for GdPalette {
    fn default() -> Self {
        GdPalette::true_color()
    }
}

/// 预定义颜色常量
pub mod colors {
    use super::GdColor;

    /// 白色
    pub const WHITE: GdColor = GdColor { red: 255, green: 255, blue: 255, alpha: 0 };
    /// 黑色
    pub const BLACK: GdColor = GdColor { red: 0, green: 0, blue: 0, alpha: 0 };
    /// 红色
    pub const RED: GdColor = GdColor { red: 255, green: 0, blue: 0, alpha: 0 };
    /// 绿色
    pub const GREEN: GdColor = GdColor { red: 0, green: 255, blue: 0, alpha: 0 };
    /// 蓝色
    pub const BLUE: GdColor = GdColor { red: 0, green: 0, blue: 255, alpha: 0 };
    /// 黄色
    pub const YELLOW: GdColor = GdColor { red: 255, green: 255, blue: 0, alpha: 0 };
    /// 青色
    pub const CYAN: GdColor = GdColor { red: 0, green: 255, blue: 255, alpha: 0 };
    /// 品红色
    pub const MAGENTA: GdColor = GdColor { red: 255, green: 0, blue: 255, alpha: 0 };
    /// 灰色
    pub const GRAY: GdColor = GdColor { red: 128, green: 128, blue: 128, alpha: 0 };
    /// 深灰色
    pub const DARK_GRAY: GdColor = GdColor { red: 64, green: 64, blue: 64, alpha: 0 };
    /// 浅灰色
    pub const LIGHT_GRAY: GdColor = GdColor { red: 192, green: 192, blue: 192, alpha: 0 };
    /// 橙色
    pub const ORANGE: GdColor = GdColor { red: 255, green: 165, blue: 0, alpha: 0 };
    /// 粉色
    pub const PINK: GdColor = GdColor { red: 255, green: 192, blue: 203, alpha: 0 };
    /// 紫色
    pub const PURPLE: GdColor = GdColor { red: 128, green: 0, blue: 128, alpha: 0 };
    /// 棕色
    pub const BROWN: GdColor = GdColor { red: 139, green: 69, blue: 19, alpha: 0 };
    /// 透明色
    pub const TRANSPARENT: GdColor = GdColor { red: 0, green: 0, blue: 0, alpha: 127 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = GdColor::new(255, 128, 64);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
        assert_eq!(color.alpha, 0);
    }

    #[test]
    fn test_color_with_alpha() {
        let color = GdColor::new_with_alpha(255, 128, 64, 50);
        assert_eq!(color.alpha, 50);
    }

    #[test]
    fn test_color_from_hex() {
        let color = GdColor::from_hex("#FF8040").unwrap();
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
    }

    #[test]
    fn test_color_to_hex() {
        let color = GdColor::new(255, 128, 64);
        assert_eq!(color.to_hex(false), "#FF8040");
    }

    #[test]
    fn test_color_grayscale() {
        let color = GdColor::new(255, 128, 64);
        let gray = color.to_grayscale();
        assert_eq!(gray.red, gray.green);
        assert_eq!(gray.green, gray.blue);
    }

    #[test]
    fn test_color_invert() {
        let color = GdColor::new(255, 128, 64);
        let inverted = color.invert();
        assert_eq!(inverted.red, 0);
        assert_eq!(inverted.green, 127);
        assert_eq!(inverted.blue, 191);
    }

    #[test]
    fn test_palette_allocation() {
        let mut palette = GdPalette::indexed(256);
        let result = palette.allocate(GdColor::red()).unwrap();
        assert_eq!(result.index, 0);
        assert!(result.is_new);
    }

    #[test]
    fn test_palette_true_color() {
        let mut palette = GdPalette::true_color();
        let result = palette.allocate(GdColor::red()).unwrap();
        assert!(palette.is_true_color());
    }
}
