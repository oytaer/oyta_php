//! 图像滤镜模块
//! 
//! 本模块实现 GD 图像的滤镜效果，包括：
//! - 基本滤镜（模糊、锐化、边缘检测等）
//! - 颜色滤镜（灰度、反色、亮度、对比度等）
//! - 特效滤镜（浮雕、马赛克、像素化等）
//! - 卷积滤镜

use crate::gd::color::GdColor;
use crate::gd::image::GdImage;

/// GD 滤镜工具
/// 
/// 提供各种图像滤镜效果
pub struct GdFilter;

impl GdFilter {
    /// 应用卷积滤镜
    /// 
    /// 使用卷积矩阵对图像进行滤波
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `matrix`: 卷积矩阵
    /// - `divisor`: 除数
    /// - `offset`: 偏移量
    pub fn convolution(image: &mut GdImage, matrix: &[f32], divisor: f32, offset: f32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        let matrix_size = (matrix.len() as f64).sqrt() as i32;
        if matrix_size * matrix_size != matrix.len() as i32 {
            return;
        }
        let half = matrix_size / 2;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                for my in 0..matrix_size {
                    for mx in 0..matrix_size {
                        let sx = (x + mx - half).clamp(0, width - 1);
                        let sy = (y + my - half).clamp(0, height - 1);
                        let color = image.get_pixel_color(sx, sy);
                        let weight = matrix[(my * matrix_size + mx) as usize];
                        r += color.red as f32 * weight;
                        g += color.green as f32 * weight;
                        b += color.blue as f32 * weight;
                    }
                }
                if divisor != 0.0 {
                    r = r / divisor + offset;
                    g = g / divisor + offset;
                    b = b / divisor + offset;
                }
                let idx = (y * width + x) as usize;
                result[idx] = GdColor::new(
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                );
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }

    /// 高斯模糊
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `radius`: 模糊半径
    pub fn gaussian_blur(image: &mut GdImage, radius: i32) {
        let radius = radius.max(1);
        let size = radius * 2 + 1;
        let mut matrix = vec![0.0f32; (size * size) as usize];
        let sigma = radius as f32 / 3.0;
        let mut sum = 0.0f32;
        for y in -radius..=radius {
            for x in -radius..=radius {
                let val = (-(x * x + y * y) as f32 / (2.0 * sigma * sigma)).exp();
                let idx = ((y + radius) * size + (x + radius)) as usize;
                matrix[idx] = val;
                sum += val;
            }
        }
        // 归一化
        for val in &mut matrix {
            *val /= sum;
        }
        Self::convolution(image, &matrix, 1.0, 0.0);
    }

    /// 均值模糊
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `radius`: 模糊半径
    pub fn mean_blur(image: &mut GdImage, radius: i32) {
        let radius = radius.max(1);
        let size = radius * 2 + 1;
        let count = (size * size) as f32;
        let matrix = vec![1.0 / count; (size * size) as usize];
        Self::convolution(image, &matrix, 1.0, 0.0);
    }

    /// 运动模糊
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `distance`: 模糊距离
    /// - `angle`: 模糊角度（度）
    pub fn motion_blur(image: &mut GdImage, distance: i32, angle: i32) {
        let distance = distance.max(1);
        let rad = (angle as f32).to_radians();
        let dx = rad.cos();
        let dy = rad.sin();
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut count = 0u32;
                for i in 0..distance {
                    let sx = (x as f32 + i as f32 * dx) as i32;
                    let sy = (y as f32 + i as f32 * dy) as i32;
                    if sx >= 0 && sx < width && sy >= 0 && sy < height {
                        let color = image.get_pixel_color(sx, sy);
                        r += color.red as u32;
                        g += color.green as u32;
                        b += color.blue as u32;
                        count += 1;
                    }
                }
                let idx = (y * width + x) as usize;
                if count > 0 {
                    result[idx] = GdColor::new(
                        (r / count) as u8,
                        (g / count) as u8,
                        (b / count) as u8,
                    );
                }
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }

    /// 锐化
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `amount`: 锐化强度 (0-100)
    pub fn sharpen(image: &mut GdImage, amount: i32) {
        let amount = amount.clamp(0, 100) as f32 / 100.0;
        let matrix = vec![
            0.0, -amount, 0.0,
            -amount, 1.0 + 4.0 * amount, -amount,
            0.0, -amount, 0.0,
        ];
        Self::convolution(image, &matrix, 1.0, 0.0);
    }

    /// 边缘检测（Sobel）
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn edge_detect(image: &mut GdImage) {
        let gx = vec![
            -1.0, 0.0, 1.0,
            -2.0, 0.0, 2.0,
            -1.0, 0.0, 1.0,
        ];
        let gy = vec![
            -1.0, -2.0, -1.0,
            0.0, 0.0, 0.0,
            1.0, 2.0, 1.0,
        ];
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut rx = 0.0f32;
                let mut gx_val = 0.0f32;
                let mut bx = 0.0f32;
                let mut ry = 0.0f32;
                let mut gy_val = 0.0f32;
                let mut by = 0.0f32;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let color = image.get_pixel_color(x + dx, y + dy);
                        let idx = ((dy + 1) * 3 + (dx + 1)) as usize;
                        let weight_x = gx[idx];
                        let weight_y = gy[idx];
                        rx += color.red as f32 * weight_x;
                        gx_val += color.green as f32 * weight_x;
                        bx += color.blue as f32 * weight_x;
                        ry += color.red as f32 * weight_y;
                        gy_val += color.green as f32 * weight_y;
                        by += color.blue as f32 * weight_y;
                    }
                }
                let r = ((rx * rx + ry * ry).sqrt() / 2.0).clamp(0.0, 255.0) as u8;
                let g = ((gx_val * gx_val + gy_val * gy_val).sqrt() / 2.0).clamp(0.0, 255.0) as u8;
                let b = ((bx * bx + by * by).sqrt() / 2.0).clamp(0.0, 255.0) as u8;
                let idx = (y * width + x) as usize;
                result[idx] = GdColor::new(r, g, b);
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }

    /// 浮雕效果
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `direction`: 浮雕方向 (0-7)
    pub fn emboss(image: &mut GdImage, direction: i32) {
        let matrix = match direction % 8 {
            0 => vec![0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0],
            1 => vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0],
            2 => vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0],
            3 => vec![0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0],
            4 => vec![0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0],
            5 => vec![-1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            6 => vec![0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            _ => vec![0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        };
        Self::convolution(image, &matrix, 1.0, 128.0);
    }

    /// 灰度化
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn grayscale(image: &mut GdImage) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let gray = color.to_grayscale();
                image.set_pixel(x, y, gray.to_int());
            }
        }
    }

    /// 反色
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn negate(image: &mut GdImage) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let inverted = color.invert();
                image.set_pixel(x, y, inverted.to_int());
            }
        }
    }

    /// 亮度调整
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `brightness`: 亮度调整值 (-255 到 255)
    pub fn brightness(image: &mut GdImage, brightness: i32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let adjusted = color.adjust_brightness(brightness);
                image.set_pixel(x, y, adjusted.to_int());
            }
        }
    }

    /// 对比度调整
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `contrast`: 对比度调整值 (-100 到 100)
    pub fn contrast(image: &mut GdImage, contrast: i32) {
        let contrast = contrast.clamp(-100, 100) as f32 / 100.0;
        let factor = (259.0 * (contrast * 255.0 + 255.0)) / (255.0 * (259.0 - contrast * 255.0));
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let r = (factor * (color.red as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
                let g = (factor * (color.green as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
                let b = (factor * (color.blue as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
                image.set_pixel(x, y, GdColor::new_with_alpha(r, g, b, color.alpha).to_int());
            }
        }
    }

    /// 颜色平衡
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `red`: 红色调整 (-255 到 255)
    /// - `green`: 绿色调整 (-255 到 255)
    /// - `blue`: 蓝色调整 (-255 到 255)
    pub fn color_balance(image: &mut GdImage, red: i32, green: i32, blue: i32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let r = (color.red as i32 + red).clamp(0, 255) as u8;
                let g = (color.green as i32 + green).clamp(0, 255) as u8;
                let b = (color.blue as i32 + blue).clamp(0, 255) as u8;
                image.set_pixel(x, y, GdColor::new_with_alpha(r, g, b, color.alpha).to_int());
            }
        }
    }

    /// 色相/饱和度/亮度调整
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `hue`: 色相调整 (-180 到 180)
    /// - `saturation`: 饱和度调整 (-100 到 100)
    /// - `lightness`: 亮度调整 (-100 到 100)
    pub fn hsl_adjust(image: &mut GdImage, hue: i32, saturation: i32, lightness: i32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let (h, s, l) = color.to_hsl();
                let new_h = (h + hue as f32 + 360.0) % 360.0;
                let new_s = (s + saturation as f32).clamp(0.0, 100.0);
                let new_l = (l + lightness as f32).clamp(0.0, 100.0);
                let new_color = GdColor::from_hsl(new_h, new_s, new_l).with_alpha(color.alpha);
                image.set_pixel(x, y, new_color.to_int());
            }
        }
    }

    /// 马赛克效果
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `block_size`: 块大小
    pub fn pixelate(image: &mut GdImage, block_size: i32) {
        let block_size = block_size.max(1);
        let width = image.width() as i32;
        let height = image.height() as i32;
        for by in (0..height).step_by(block_size as usize) {
            for bx in (0..width).step_by(block_size as usize) {
                // 计算块的平均颜色
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut count = 0u32;
                for dy in 0..block_size {
                    for dx in 0..block_size {
                        let x = bx + dx;
                        let y = by + dy;
                        if x < width && y < height {
                            let color = image.get_pixel_color(x, y);
                            r += color.red as u32;
                            g += color.green as u32;
                            b += color.blue as u32;
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    let avg_color = GdColor::new(
                        (r / count) as u8,
                        (g / count) as u8,
                        (b / count) as u8,
                    );
                    // 填充块
                    for dy in 0..block_size {
                        for dx in 0..block_size {
                            let x = bx + dx;
                            let y = by + dy;
                            if x < width && y < height {
                                image.set_pixel(x, y, avg_color.to_int());
                            }
                        }
                    }
                }
            }
        }
    }

    /// 油画效果
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `radius`: 效果半径
    /// - `intensity`: 强度
    pub fn oil_paint(image: &mut GdImage, radius: i32, intensity: i32) {
        let radius = radius.max(1);
        let intensity = intensity.max(1).min(256) as usize;
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let mut intensity_count = vec![0usize; intensity];
                let mut avg_r = vec![0u32; intensity];
                let mut avg_g = vec![0u32; intensity];
                let mut avg_b = vec![0u32; intensity];
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let sx = (x + dx).clamp(0, width - 1);
                        let sy = (y + dy).clamp(0, height - 1);
                        let color = image.get_pixel_color(sx, sy);
                        let intensity_idx = ((color.red as u32 + color.green as u32 + color.blue as u32) / 3 * (intensity - 1) as u32 / 255) as usize;
                        intensity_count[intensity_idx] += 1;
                        avg_r[intensity_idx] += color.red as u32;
                        avg_g[intensity_idx] += color.green as u32;
                        avg_b[intensity_idx] += color.blue as u32;
                    }
                }
                // 找到出现次数最多的强度
                let mut max_count = 0;
                let mut max_idx = 0;
                for (i, &count) in intensity_count.iter().enumerate() {
                    if count > max_count {
                        max_count = count;
                        max_idx = i;
                    }
                }
                let idx = (y * width + x) as usize;
                if max_count > 0 {
                    result[idx] = GdColor::new(
                        (avg_r[max_idx] / max_count as u32) as u8,
                        (avg_g[max_idx] / max_count as u32) as u8,
                        (avg_b[max_idx] / max_count as u32) as u8,
                    );
                }
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }

    /// 色调分离
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `levels`: 色阶数
    pub fn posterize(image: &mut GdImage, levels: i32) {
        let levels = levels.max(2).min(255);
        let step = 255 / (levels - 1);
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let r = ((color.red as i32 / step) * step).min(255) as u8;
                let g = ((color.green as i32 / step) * step).min(255) as u8;
                let b = ((color.blue as i32 / step) * step).min(255) as u8;
                image.set_pixel(x, y, GdColor::new_with_alpha(r, g, b, color.alpha).to_int());
            }
        }
    }

    /// 阈值化
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `threshold`: 阈值 (0-255)
    pub fn threshold(image: &mut GdImage, threshold: i32) {
        let threshold = threshold.clamp(0, 255) as u8;
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let gray = (color.red as u16 + color.green as u16 + color.blue as u16) / 3;
                let val = if gray as u8 > threshold { 255 } else { 0 };
                image.set_pixel(x, y, GdColor::new(val, val, val).to_int());
            }
        }
    }

    /// 添加噪点
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `amount`: 噪点量 (0-100)
    pub fn noise(image: &mut GdImage, amount: i32) {
        let amount = amount.clamp(0, 100);
        let width = image.width() as i32;
        let height = image.height() as i32;
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        let mut rng = SimpleRng::new(seed);
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                let noise = ((rng.next() % (2 * amount as u64 + 1)) as i32 - amount) as i32;
                let r = (color.red as i32 + noise).clamp(0, 255) as u8;
                let g = (color.green as i32 + noise).clamp(0, 255) as u8;
                let b = (color.blue as i32 + noise).clamp(0, 255) as u8;
                image.set_pixel(x, y, GdColor::new_with_alpha(r, g, b, color.alpha).to_int());
            }
        }
    }

    /// 散射效果
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `radius`: 散射半径
    pub fn scatter(image: &mut GdImage, radius: i32) {
        let radius = radius.max(1);
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        let mut rng = SimpleRng::new(seed);
        for y in 0..height {
            for x in 0..width {
                let dx = (rng.next() % (2 * radius as u64 + 1)) as i32 - radius;
                let dy = (rng.next() % (2 * radius as u64 + 1)) as i32 - radius;
                let sx = (x + dx).clamp(0, width - 1);
                let sy = (y + dy).clamp(0, height - 1);
                let idx = (y * width + x) as usize;
                result[idx] = image.get_pixel_color(sx, sy);
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }

    /// 查找边缘
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn find_edges(image: &mut GdImage) {
        let matrix = vec![
            -1.0, -1.0, -1.0,
            -1.0, 8.0, -1.0,
            -1.0, -1.0, -1.0,
        ];
        Self::convolution(image, &matrix, 1.0, 0.0);
    }

    /// 浮雕效果（简化版）
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn emboss_simple(image: &mut GdImage) {
        let matrix = vec![
            -2.0, -1.0, 0.0,
            -1.0, 1.0, 1.0,
            0.0, 1.0, 2.0,
        ];
        Self::convolution(image, &matrix, 1.0, 128.0);
    }

    /// 柔化
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn smooth(image: &mut GdImage) {
        let matrix = vec![
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
        ];
        Self::convolution(image, &matrix, 9.0, 0.0);
    }

    /// 去除均值
    /// 
    /// # 参数
    /// - `image`: 目标图像
    pub fn mean_removal(image: &mut GdImage) {
        let matrix = vec![
            -1.0, -1.0, -1.0,
            -1.0, 9.0, -1.0,
            -1.0, -1.0, -1.0,
        ];
        Self::convolution(image, &matrix, 1.0, 0.0);
    }

    /// 选择性模糊
    /// 
    /// 只模糊相似颜色的区域
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `radius`: 模糊半径
    /// - `threshold`: 颜色相似阈值
    pub fn selective_blur(image: &mut GdImage, radius: i32, threshold: i32) {
        let radius = radius.max(1);
        let threshold = threshold as f32;
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut result = vec![GdColor::black(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let center_color = image.get_pixel_color(x, y);
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut count = 0u32;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let sx = (x + dx).clamp(0, width - 1);
                        let sy = (y + dy).clamp(0, height - 1);
                        let color = image.get_pixel_color(sx, sy);
                        let distance = center_color.distance(&color);
                        if distance <= threshold {
                            r += color.red as u32;
                            g += color.green as u32;
                            b += color.blue as u32;
                            count += 1;
                        }
                    }
                }
                let idx = (y * width + x) as usize;
                if count > 0 {
                    result[idx] = GdColor::new(
                        (r / count) as u8,
                        (g / count) as u8,
                        (b / count) as u8,
                    );
                } else {
                    result[idx] = center_color;
                }
            }
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                image.set_pixel(x, y, result[idx].to_int());
            }
        }
    }
}

/// 简单的伪随机数生成器
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
}

/// 预定义滤镜类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// 高斯模糊
    GaussianBlur,
    /// 均值模糊
    MeanBlur,
    /// 锐化
    Sharpen,
    /// 边缘检测
    EdgeDetect,
    /// 浮雕
    Emboss,
    /// 灰度
    Grayscale,
    /// 反色
    Negate,
    /// 柔化
    Smooth,
    /// 去除均值
    MeanRemoval,
}

impl FilterType {
    /// 应用滤镜
    pub fn apply(&self, image: &mut GdImage, arg1: i32, arg2: i32, arg3: i32) {
        match self {
            FilterType::GaussianBlur => GdFilter::gaussian_blur(image, arg1.max(1)),
            FilterType::MeanBlur => GdFilter::mean_blur(image, arg1.max(1)),
            FilterType::Sharpen => GdFilter::sharpen(image, arg1),
            FilterType::EdgeDetect => GdFilter::edge_detect(image),
            FilterType::Emboss => GdFilter::emboss(image, arg1),
            FilterType::Grayscale => GdFilter::grayscale(image),
            FilterType::Negate => GdFilter::negate(image),
            FilterType::Smooth => GdFilter::smooth(image),
            FilterType::MeanRemoval => GdFilter::mean_removal(image),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_filter() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(255, 128, 64);
        image.fill(color);
        GdFilter::grayscale(&mut image);
        let result = image.get_pixel_color(5, 5);
        assert_eq!(result.red, result.green);
        assert_eq!(result.green, result.blue);
    }

    #[test]
    fn test_negate_filter() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(100, 150, 200);
        image.fill(color);
        GdFilter::negate(&mut image);
        let result = image.get_pixel_color(5, 5);
        assert_eq!(result.red, 155);
        assert_eq!(result.green, 105);
        assert_eq!(result.blue, 55);
    }

    #[test]
    fn test_brightness_filter() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(100, 100, 100);
        image.fill(color);
        GdFilter::brightness(&mut image, 50);
        let result = image.get_pixel_color(5, 5);
        assert_eq!(result.red, 150);
    }

    #[test]
    fn test_pixelate_filter() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(255, 0, 0);
        image.fill(color);
        GdFilter::pixelate(&mut image, 5);
        let result = image.get_pixel_color(2, 2);
        assert_eq!(result.red, 255);
    }
}
