//! 文字渲染模块
//! 
//! 本模块实现 GD 图像的文字渲染功能，包括：
//! - 内置字体渲染
//! - TrueType 字体渲染
    /// - FreeType 字体支持
/// - 文字布局和对齐

use crate::gd::color::GdColor;
use crate::gd::image::GdImage;

/// 内置字体 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdFontId {
    /// 小字体 (4x6)
    Font1 = 1,
    /// 中小字体 (4x8)
    Font2 = 2,
    /// 中等字体 (6x10)
    Font3 = 3,
    /// 中大字体 (6x12)
    Font4 = 4,
    /// 大字体 (8x16)
    Font5 = 5,
}

impl GdFontId {
    /// 获取字体尺寸
    /// 
    /// # 返回
    /// 返回 (宽度, 高度)
    pub fn size(&self) -> (i32, i32) {
        match self {
            GdFontId::Font1 => (4, 6),
            GdFontId::Font2 => (4, 8),
            GdFontId::Font3 => (6, 10),
            GdFontId::Font4 => (6, 12),
            GdFontId::Font5 => (8, 16),
        }
    }
}

/// 文字对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdTextAlignment {
    /// 左对齐
    Left,
    /// 居中对齐
    Center,
    /// 右对齐
    Right,
    /// 顶部对齐
    Top,
    /// 底部对齐
    Bottom,
    /// 基线对齐
    Baseline,
}

/// GD 字体结构
/// 
/// 表示一个 GD 内置字体
#[derive(Debug, Clone)]
pub struct GdFont {
    /// 字体 ID
    pub id: GdFontId,
    /// 字体宽度
    pub width: i32,
    /// 字体高度
    pub height: i32,
    /// 字符位图数据
    pub char_data: Vec<Vec<Vec<bool>>>,
}

impl GdFont {
    /// 创建内置字体
    /// 
    /// # 参数
    /// - `id`: 字体 ID
    /// 
    /// # 返回
    /// 返回字体对象
    pub fn new(id: GdFontId) -> Self {
        let (width, height) = id.size();
        GdFont {
            id,
            width,
            height,
            char_data: Self::generate_builtin_font(id),
        }
    }

    /// 生成内置字体数据
    fn generate_builtin_font(id: GdFontId) -> Vec<Vec<Vec<bool>>> {
        // 这里提供简化的 ASCII 字符位图数据
        // 实际实现需要完整的字体位图数据
        let (width, height) = id.size();
        let mut chars = Vec::with_capacity(256);
        for _ in 0..256 {
            let char_bitmap = vec![vec![false; width as usize]; height as usize];
            chars.push(char_bitmap);
        }
        // 填充基本 ASCII 字符
        Self::fill_basic_chars(&mut chars, id);
        chars
    }

    /// 填充基本 ASCII 字符
    fn fill_basic_chars(chars: &mut [Vec<Vec<bool>>], id: GdFontId) {
        let (w, h) = id.size();
        // 简化实现：只填充几个基本字符
        // 实际实现需要完整的字体数据
        // 'A' (65)
        let a_data = Self::get_char_a(w, h);
        chars[65] = a_data;
        // 'B' (66)
        let b_data = Self::get_char_b(w, h);
        chars[66] = b_data;
        // 可以继续添加更多字符...
    }

    /// 获取字符 'A' 的位图
    fn get_char_a(w: i32, h: i32) -> Vec<Vec<bool>> {
        let mut data = vec![vec![false; w as usize]; h as usize];
        // 简化的 'A' 形状
        match (w, h) {
            (4, 6) => {
                data[0][1] = true; data[0][2] = true;
                data[1][0] = true; data[1][3] = true;
                data[2][0] = true; data[2][1] = true; data[2][2] = true; data[2][3] = true;
                data[3][0] = true; data[3][3] = true;
                data[4][0] = true; data[4][3] = true;
                data[5][0] = true; data[5][3] = true;
            }
            _ => {
                // 默认形状
                for y in 0..h {
                    for x in 0..w {
                        data[y as usize][x as usize] = (x + y) % 2 == 0;
                    }
                }
            }
        }
        data
    }

    /// 获取字符 'B' 的位图
    fn get_char_b(w: i32, h: i32) -> Vec<Vec<bool>> {
        let mut data = vec![vec![false; w as usize]; h as usize];
        // 简化的 'B' 形状
        match (w, h) {
            (4, 6) => {
                data[0][0] = true; data[0][1] = true; data[0][2] = true;
                data[1][0] = true; data[1][3] = true;
                data[2][0] = true; data[2][1] = true; data[2][2] = true;
                data[3][0] = true; data[3][3] = true;
                data[4][0] = true; data[4][1] = true; data[4][2] = true;
            }
            _ => {
                for y in 0..h {
                    for x in 0..w {
                        data[y as usize][x as usize] = (x + y + 1) % 2 == 0;
                    }
                }
            }
        }
        data
    }

    /// 获取字符位图
    /// 
    /// # 参数
    /// - `c`: 字符
    /// 
    /// # 返回
    /// 返回字符位图
    pub fn get_char(&self, c: char) -> &Vec<Vec<bool>> {
        let idx = c as usize;
        if idx < self.char_data.len() {
            &self.char_data[idx]
        } else {
            &self.char_data[0]
        }
    }

    /// 渲染字符
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `c`: 字符
    /// - `color`: 颜色
    pub fn render_char(&self, image: &mut GdImage, x: i32, y: i32, c: char, color: i32) {
        let char_data = self.get_char(c);
        for (dy, row) in char_data.iter().enumerate() {
            for (dx, &pixel) in row.iter().enumerate() {
                if pixel {
                    image.set_pixel(x + dx as i32, y + dy as i32, color);
                }
            }
        }
    }

    /// 渲染字符串
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 颜色
    pub fn render_text(&self, image: &mut GdImage, x: i32, y: i32, text: &str, color: i32) {
        let mut cx = x;
        for c in text.chars() {
            self.render_char(image, cx, y, c, color);
            cx += self.width;
        }
    }

    /// 计算文本宽度
    /// 
    /// # 参数
    /// - `text`: 文本
    /// 
    /// # 返回
    /// 返回文本宽度
    pub fn text_width(&self, text: &str) -> i32 {
        text.chars().count() as i32 * self.width
    }

    /// 计算文本高度
    /// 
    /// # 返回
    /// 返回文本高度
    pub fn text_height(&self) -> i32 {
        self.height
    }
}

/// GD 文字渲染工具
pub struct GdText;

impl GdText {
    /// 使用内置字体绘制字符
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `c`: 字符
    /// - `color`: 颜色
    pub fn char(image: &mut GdImage, font: &GdFont, x: i32, y: i32, c: char, color: i32) {
        font.render_char(image, x, y, c, color);
    }

    /// 使用内置字体绘制字符串
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 颜色
    pub fn string(image: &mut GdImage, font: &GdFont, x: i32, y: i32, text: &str, color: i32) {
        font.render_text(image, x, y, text, color);
    }

    /// 绘制垂直字符串
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 颜色
    pub fn string_up(image: &mut GdImage, font: &GdFont, x: i32, y: i32, text: &str, color: i32) {
        let mut cy = y;
        for c in text.chars().rev() {
            font.render_char(image, x, cy, c, color);
            cy += font.height;
        }
    }

    /// 获取字符串边界框
    /// 
    /// # 参数
    /// - `font`: 字体
    /// - `text`: 文本
    /// 
    /// # 返回
    /// 返回 (左, 上, 右, 下)
    pub fn bbox(font: &GdFont, text: &str) -> (i32, i32, i32, i32) {
        let width = font.text_width(text);
        let height = font.text_height();
        (0, 0, width, height)
    }

    /// 使用 TrueType 字体绘制文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font_path`: 字体文件路径
    /// - `size`: 字体大小
    /// - `angle`: 旋转角度（度）
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `color`: 颜色
    /// - `text`: 文本
    /// 
    /// # 返回
    /// 成功返回边界框，失败返回错误信息
    pub fn ttf_text(
        image: &mut GdImage,
        font_path: &str,
        size: f32,
        angle: f32,
        x: i32,
        y: i32,
        color: i32,
        text: &str,
    ) -> Result<(i32, i32, i32, i32, i32, i32, i32, i32), String> {
        // 简化实现：使用内置字体模拟
        // 实际实现需要 FreeType 或类似的字体渲染库
        let font = GdFont::new(GdFontId::Font5);
        let scaled_width = (font.width as f32 * size / 16.0) as i32;
        let scaled_height = (font.height as f32 * size / 16.0) as i32;
        // 渲染文本
        let mut cx = x;
        for c in text.chars() {
            Self::render_scaled_char(image, &font, cx, y, c, color, scaled_width, scaled_height);
            cx += scaled_width;
        }
        // 返回边界框
        let text_width = scaled_width * text.chars().count() as i32;
        Ok((x, y - scaled_height, x + text_width, y, x, y - scaled_height, x + text_width, y))
    }

    /// 渲染缩放字符
    fn render_scaled_char(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        c: char,
        color: i32,
        scaled_width: i32,
        scaled_height: i32,
    ) {
        let char_data = font.get_char(c);
        let x_ratio = font.width as f32 / scaled_width as f32;
        let y_ratio = font.height as f32 / scaled_height as f32;
        for dy in 0..scaled_height {
            for dx in 0..scaled_width {
                let src_x = (dx as f32 * x_ratio) as usize;
                let src_y = (dy as f32 * y_ratio) as usize;
                if src_y < char_data.len() && src_x < char_data[src_y].len() {
                    if char_data[src_y][src_x] {
                        image.set_pixel(x + dx, y + dy, color);
                    }
                }
            }
        }
    }

    /// 获取 TrueType 文本边界框
    /// 
    /// # 参数
    /// - `font_path`: 字体文件路径
    /// - `size`: 字体大小
    /// - `angle`: 旋转角度（度）
    /// - `text`: 文本
    /// 
    /// # 返回
    /// 成功返回边界框，失败返回错误信息
    pub fn ttf_bbox(
        font_path: &str,
        size: f32,
        angle: f32,
        text: &str,
    ) -> Result<(i32, i32, i32, i32, i32, i32, i32, i32), String> {
        let font = GdFont::new(GdFontId::Font5);
        let scaled_width = (font.width as f32 * size / 16.0) as i32;
        let scaled_height = (font.height as f32 * size / 16.0) as i32;
        let text_width = scaled_width * text.chars().count() as i32;
        Ok((0, -scaled_height, text_width, 0, 0, -scaled_height, text_width, 0))
    }

    /// 绘制带背景的文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 文本颜色
    /// - `bg_color`: 背景颜色
    pub fn string_with_bg(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        text: &str,
        color: i32,
        bg_color: i32,
    ) {
        let width = font.text_width(text);
        let height = font.text_height();
        // 绘制背景
        for dy in 0..height {
            for dx in 0..width {
                image.set_pixel(x + dx, y + dy, bg_color);
            }
        }
        // 绘制文本
        font.render_text(image, x, y, text, color);
    }

    /// 绘制带阴影的文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 文本颜色
    /// - `shadow_color`: 阴影颜色
    /// - `shadow_offset`: 阴影偏移 (dx, dy)
    pub fn string_with_shadow(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        text: &str,
        color: i32,
        shadow_color: i32,
        shadow_offset: (i32, i32),
    ) {
        // 绘制阴影
        font.render_text(image, x + shadow_offset.0, y + shadow_offset.1, text, shadow_color);
        // 绘制文本
        font.render_text(image, x, y, text, color);
    }

    /// 绘制对齐文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `width`: 宽度
    /// - `text`: 文本
    /// - `color`: 颜色
    /// - `alignment`: 对齐方式
    pub fn string_aligned(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        width: i32,
        text: &str,
        color: i32,
        alignment: GdTextAlignment,
    ) {
        let text_width = font.text_width(text);
        let start_x = match alignment {
            GdTextAlignment::Left => x,
            GdTextAlignment::Center => x + (width - text_width) / 2,
            GdTextAlignment::Right => x + width - text_width,
            _ => x,
        };
        font.render_text(image, start_x, y, text, color);
    }

    /// 绘制自动换行文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `max_width`: 最大宽度
    /// - `text`: 文本
    /// - `color`: 颜色
    /// 
    /// # 返回
    /// 返回实际绘制的高度
    pub fn string_wrapped(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        max_width: i32,
        text: &str,
        color: i32,
    ) -> i32 {
        let mut current_y = y;
        let mut current_line = String::new();
        let mut current_width = 0i32;
        for word in text.split_whitespace() {
            let word_width = font.text_width(word);
            let space_width = font.width;
            if current_width + word_width + (if current_line.is_empty() { 0 } else { space_width }) > max_width {
                if !current_line.is_empty() {
                    font.render_text(image, x, current_y, &current_line, color);
                    current_y += font.height;
                    current_line.clear();
                    current_width = 0;
                }
                if word_width > max_width {
                    // 单词太长，需要逐字符换行
                    for c in word.chars() {
                        let char_width = font.width;
                        if current_width + char_width > max_width {
                            font.render_text(image, x, current_y, &current_line, color);
                            current_y += font.height;
                            current_line.clear();
                            current_width = 0;
                        }
                        current_line.push(c);
                        current_width += char_width;
                    }
                } else {
                    current_line = word.to_string();
                    current_width = word_width;
                }
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += space_width;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }
        if !current_line.is_empty() {
            font.render_text(image, x, current_y, &current_line, color);
            current_y += font.height;
        }
        current_y - y
    }

    /// 加载字体文件
    /// 
    /// # 参数
    /// - `path`: 字体文件路径
    /// 
    /// # 返回
    /// 成功返回字体数据，失败返回错误信息
    pub fn load_font(path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("加载字体失败: {}", e))
    }

    /// 创建字体图像
    /// 
    /// 从字体创建包含所有字符的图像
    /// 
    /// # 参数
    /// - `font`: 字体
    /// - `color`: 颜色
    /// - `bg_color`: 背景颜色
    /// 
    /// # 返回
    /// 返回包含所有可打印 ASCII 字符的图像
    pub fn create_font_image(font: &GdFont, color: i32, bg_color: i32) -> GdImage {
        let chars_per_row = 16;
        let num_chars = 256 - 32; // 可打印 ASCII 字符
        let rows = (num_chars + chars_per_row - 1) / chars_per_row;
        let width = chars_per_row * font.width;
        let height = rows * font.height;
        let mut image = GdImage::new_true_color(width as u32, height as u32);
        // 填充背景
        for y in 0..height {
            for x in 0..width {
                image.set_pixel(x, y, bg_color);
            }
        }
        // 绘制字符
        for i in 0..num_chars {
            let c = (i + 32) as u8 as char;
            let row = i / chars_per_row;
            let col = i % chars_per_row;
            let x = col * font.width;
            let y = row * font.height;
            font.render_char(&mut image, x, y, c, color);
        }
        image
    }

    /// 测量文本
    /// 
    /// # 参数
    /// - `font`: 字体
    /// - `text`: 文本
    /// 
    /// # 返回
    /// 返回 (宽度, 高度)
    pub fn measure_text(font: &GdFont, text: &str) -> (i32, i32) {
        (font.text_width(text), font.text_height())
    }

    /// 绘制旋转文本
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `font`: 字体
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `text`: 文本
    /// - `color`: 颜色
    /// - `angle`: 旋转角度（度）
    pub fn string_rotated(
        image: &mut GdImage,
        font: &GdFont,
        x: i32,
        y: i32,
        text: &str,
        color: i32,
        angle: f32,
    ) {
        // 创建临时图像
        let text_width = font.text_width(text);
        let text_height = font.text_height();
        let mut temp = GdImage::new_true_color(text_width as u32, text_height as u32);
        font.render_text(&mut temp, 0, 0, text, color);
        // 旋转并复制到目标图像
        let rad = angle.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        let cx = text_width as f32 / 2.0;
        let cy = text_height as f32 / 2.0;
        for py in 0..image.height() as i32 {
            for px in 0..image.width() as i32 {
                let dx = px as f32 - x as f32 - cx;
                let dy = py as f32 - y as f32 - cy;
                let src_x = (dx * cos_a + dy * sin_a + cx) as i32;
                let src_y = (-dx * sin_a + dy * cos_a + cy) as i32;
                if src_x >= 0 && src_x < text_width && src_y >= 0 && src_y < text_height {
                    let pixel = temp.get_pixel(src_x, src_y);
                    if pixel != 0 {
                        image.set_pixel(px, py, pixel);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_creation() {
        let font = GdFont::new(GdFontId::Font1);
        assert_eq!(font.width, 4);
        assert_eq!(font.height, 6);
    }

    #[test]
    fn test_font_size() {
        let (w, h) = GdFontId::Font5.size();
        assert_eq!(w, 8);
        assert_eq!(h, 16);
    }

    #[test]
    fn test_text_width() {
        let font = GdFont::new(GdFontId::Font1);
        let width = font.text_width("Hello");
        assert_eq!(width, 5 * 4);
    }

    #[test]
    fn test_measure_text() {
        let font = GdFont::new(GdFontId::Font3);
        let (w, h) = GdText::measure_text(&font, "Test");
        assert_eq!(w, 4 * 6);
        assert_eq!(h, 10);
    }
}
