//! 图像变换模块
//! 
//! 本模块实现 GD 图像的变换功能，包括：
//! - 缩放
//! - 旋转
//! - 裁剪
//! - 翻转
//! - 仿射变换

use crate::gd::color::GdColor;
use crate::gd::image::{GdImage, GdInterpolationMethod};

/// GD 图像变换工具
pub struct GdTransform;

impl GdTransform {
    /// 缩放图像
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `new_width`: 新宽度
    /// - `new_height`: 新高度
    /// 
    /// # 返回
    /// 返回缩放后的新图像
    pub fn scale(image: &GdImage, new_width: u32, new_height: u32) -> GdImage {
        if new_width == 0 || new_height == 0 {
            return GdImage::new_true_color(1, 1);
        }
        let mut result = GdImage::new_true_color(new_width, new_height);
        let x_ratio = image.width() as f64 / new_width as f64;
        let y_ratio = image.height() as f64 / new_height as f64;
        let interpolation = image.get_interpolation();
        for y in 0..new_height {
            for x in 0..new_width {
                let color = match interpolation {
                    GdInterpolationMethod::NearestNeighbour => {
                        let src_x = (x as f64 * x_ratio) as i32;
                        let src_y = (y as f64 * y_ratio) as i32;
                        image.get_pixel_color(src_x, src_y)
                    }
                    GdInterpolationMethod::Bilinear => {
                        Self::bilinear_interpolate(image, x as f64 * x_ratio, y as f64 * y_ratio)
                    }
                    GdInterpolationMethod::Bicubic => {
                        Self::bicubic_interpolate(image, x as f64 * x_ratio, y as f64 * y_ratio)
                    }
                    _ => {
                        let src_x = (x as f64 * x_ratio) as i32;
                        let src_y = (y as f64 * y_ratio) as i32;
                        image.get_pixel_color(src_x, src_y)
                    }
                };
                result.set_pixel(x as i32, y as i32, color.to_int());
            }
        }
        result
    }

    /// 双线性插值
    fn bilinear_interpolate(image: &GdImage, x: f64, y: f64) -> GdColor {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;
        let x_frac = x - x0 as f64;
        let y_frac = y - y0 as f64;
        let c00 = image.get_pixel_color(x0.clamp(0, image.width() as i32 - 1), y0.clamp(0, image.height() as i32 - 1));
        let c10 = image.get_pixel_color(x1.clamp(0, image.width() as i32 - 1), y0.clamp(0, image.height() as i32 - 1));
        let c01 = image.get_pixel_color(x0.clamp(0, image.width() as i32 - 1), y1.clamp(0, image.height() as i32 - 1));
        let c11 = image.get_pixel_color(x1.clamp(0, image.width() as i32 - 1), y1.clamp(0, image.height() as i32 - 1));
        let r = (c00.red as f64 * (1.0 - x_frac) * (1.0 - y_frac)
            + c10.red as f64 * x_frac * (1.0 - y_frac)
            + c01.red as f64 * (1.0 - x_frac) * y_frac
            + c11.red as f64 * x_frac * y_frac) as u8;
        let g = (c00.green as f64 * (1.0 - x_frac) * (1.0 - y_frac)
            + c10.green as f64 * x_frac * (1.0 - y_frac)
            + c01.green as f64 * (1.0 - x_frac) * y_frac
            + c11.green as f64 * x_frac * y_frac) as u8;
        let b = (c00.blue as f64 * (1.0 - x_frac) * (1.0 - y_frac)
            + c10.blue as f64 * x_frac * (1.0 - y_frac)
            + c01.blue as f64 * (1.0 - x_frac) * y_frac
            + c11.blue as f64 * x_frac * y_frac) as u8;
        let a = (c00.alpha as f64 * (1.0 - x_frac) * (1.0 - y_frac)
            + c10.alpha as f64 * x_frac * (1.0 - y_frac)
            + c01.alpha as f64 * (1.0 - x_frac) * y_frac
            + c11.alpha as f64 * x_frac * y_frac) as u8;
        GdColor::new_with_alpha(r, g, b, a)
    }

    /// 双三次插值
    fn bicubic_interpolate(image: &GdImage, x: f64, y: f64) -> GdColor {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x_frac = x - x0 as f64;
        let y_frac = y - y0 as f64;
        let mut r = 0.0f64;
        let mut g = 0.0f64;
        let mut b = 0.0f64;
        let mut a = 0.0f64;
        for j in -1..=2 {
            for i in -1..=2 {
                let sx = (x0 + i).clamp(0, image.width() as i32 - 1);
                let sy = (y0 + j).clamp(0, image.height() as i32 - 1);
                let color = image.get_pixel_color(sx, sy);
                let weight = Self::cubic_weight(x_frac - i as f64) * Self::cubic_weight(y_frac - j as f64);
                r += color.red as f64 * weight;
                g += color.green as f64 * weight;
                b += color.blue as f64 * weight;
                a += color.alpha as f64 * weight;
            }
        }
        GdColor::new_with_alpha(
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
            a.clamp(0.0, 127.0) as u8,
        )
    }

    /// 三次权重函数
    fn cubic_weight(t: f64) -> f64 {
        let a = -0.5;
        let t = t.abs();
        if t < 1.0 {
            (a + 2.0) * t * t * t - (a + 3.0) * t * t + 1.0
        } else if t < 2.0 {
            a * t * t * t - 5.0 * a * t * t + 8.0 * a * t - 4.0 * a
        } else {
            0.0
        }
    }

    /// 按比例缩放
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `ratio`: 缩放比例
    /// 
    /// # 返回
    /// 返回缩放后的新图像
    pub fn scale_by_ratio(image: &GdImage, ratio: f64) -> GdImage {
        let new_width = (image.width() as f64 * ratio) as u32;
        let new_height = (image.height() as f64 * ratio) as u32;
        Self::scale(image, new_width.max(1), new_height.max(1))
    }

    /// 缩放到适应尺寸
    /// 
    /// 保持宽高比缩放到指定尺寸内
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `max_width`: 最大宽度
    /// - `max_height`: 最大高度
    /// 
    /// # 返回
    /// 返回缩放后的新图像
    pub fn scale_to_fit(image: &GdImage, max_width: u32, max_height: u32) -> GdImage {
        let width_ratio = max_width as f64 / image.width() as f64;
        let height_ratio = max_height as f64 / image.height() as f64;
        let ratio = width_ratio.min(height_ratio);
        if ratio >= 1.0 {
            return image.clone();
        }
        Self::scale_by_ratio(image, ratio)
    }

    /// 缩放到填充尺寸
    /// 
    /// 保持宽高比缩放到完全填充指定尺寸
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `target_width`: 目标宽度
    /// - `target_height`: 目标高度
    /// 
    /// # 返回
    /// 返回缩放后的新图像
    pub fn scale_to_fill(image: &GdImage, target_width: u32, target_height: u32) -> GdImage {
        let width_ratio = target_width as f64 / image.width() as f64;
        let height_ratio = target_height as f64 / image.height() as f64;
        let ratio = width_ratio.max(height_ratio);
        Self::scale_by_ratio(image, ratio)
    }

    /// 旋转图像
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `angle`: 旋转角度（度）
    /// - `bg_color`: 背景颜色
    /// 
    /// # 返回
    /// 返回旋转后的新图像
    pub fn rotate(image: &GdImage, angle: f64, bg_color: GdColor) -> GdImage {
        let rad = angle.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        // 计算旋转后的图像尺寸
        let width = image.width() as f64;
        let height = image.height() as f64;
        let corners = [
            (0.0, 0.0),
            (width, 0.0),
            (0.0, height),
            (width, height),
        ];
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        let cx = width / 2.0;
        let cy = height / 2.0;
        for (x, y) in corners {
            let dx = x - cx;
            let dy = y - cy;
            let rx = dx * cos_a - dy * sin_a + cx;
            let ry = dx * sin_a + dy * cos_a + cy;
            min_x = min_x.min(rx);
            max_x = max_x.max(rx);
            min_y = min_y.min(ry);
            max_y = max_y.max(ry);
        }
        let new_width = (max_x - min_x).ceil() as u32;
        let new_height = (max_y - min_y).ceil() as u32;
        let mut result = GdImage::new_true_color(new_width, new_height);
        // 填充背景
        for y in 0..new_height {
            for x in 0..new_width {
                result.set_pixel(x as i32, y as i32, bg_color.to_int());
            }
        }
        // 旋转像素
        let new_cx = new_width as f64 / 2.0;
        let new_cy = new_height as f64 / 2.0;
        for y in 0..new_height {
            for x in 0..new_width {
                let dx = x as f64 - new_cx;
                let dy = y as f64 - new_cy;
                let src_x = dx * cos_a + dy * sin_a + cx;
                let src_y = -dx * sin_a + dy * cos_a + cy;
                if src_x >= 0.0 && src_x < width && src_y >= 0.0 && src_y < height {
                    let color = Self::bilinear_interpolate(image, src_x, src_y);
                    result.set_pixel(x as i32, y as i32, color.to_int());
                }
            }
        }
        result
    }

    /// 旋转 90 度
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `clockwise`: 是否顺时针
    /// 
    /// # 返回
    /// 返回旋转后的新图像
    pub fn rotate_90(image: &GdImage, clockwise: bool) -> GdImage {
        let new_width = image.height();
        let new_height = image.width();
        let mut result = GdImage::new_true_color(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let (src_x, src_y) = if clockwise {
                    (y as i32, (new_width - 1 - x) as i32)
                } else {
                    ((new_height - 1 - y) as i32, x as i32)
                };
                let color = image.get_pixel_color(src_x, src_y);
                result.set_pixel(x as i32, y as i32, color.to_int());
            }
        }
        result
    }

    /// 旋转 180 度
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// 
    /// # 返回
    /// 返回旋转后的新图像
    pub fn rotate_180(image: &GdImage) -> GdImage {
        let mut result = GdImage::new_true_color(image.width(), image.height());
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(width - 1 - x, height - 1 - y);
                result.set_pixel(x, y, color.to_int());
            }
        }
        result
    }

    /// 水平翻转
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// 
    /// # 返回
    /// 返回翻转后的新图像
    pub fn flip_horizontal(image: &GdImage) -> GdImage {
        let mut result = GdImage::new_true_color(image.width(), image.height());
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(width - 1 - x, y);
                result.set_pixel(x, y, color.to_int());
            }
        }
        result
    }

    /// 垂直翻转
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// 
    /// # 返回
    /// 返回翻转后的新图像
    pub fn flip_vertical(image: &GdImage) -> GdImage {
        let mut result = GdImage::new_true_color(image.width(), image.height());
        let width = image.width() as i32;
        let height = image.height() as i32;
        for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, height - 1 - y);
                result.set_pixel(x, y, color.to_int());
            }
        }
        result
    }

    /// 双向翻转
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// 
    /// # 返回
    /// 返回翻转后的新图像
    pub fn flip_both(image: &GdImage) -> GdImage {
        Self::rotate_180(image)
    }

    /// 裁剪图像
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `x`: 起始 X 坐标
    /// - `y`: 起始 Y 坐标
    /// - `width`: 裁剪宽度
    /// - `height`: 裁剪高度
    /// 
    /// # 返回
    /// 返回裁剪后的新图像
    pub fn crop(image: &GdImage, x: i32, y: i32, width: u32, height: u32) -> GdImage {
        let mut result = GdImage::new_true_color(width, height);
        for py in 0..height {
            for px in 0..width {
                let src_x = x + px as i32;
                let src_y = y + py as i32;
                if src_x >= 0 && src_x < image.width() as i32 && src_y >= 0 && src_y < image.height() as i32 {
                    let color = image.get_pixel_color(src_x, src_y);
                    result.set_pixel(px as i32, py as i32, color.to_int());
                }
            }
        }
        result
    }

    /// 自动裁剪
    /// 
    /// 自动裁剪图像边缘的空白区域
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `threshold`: 颜色差异阈值
    /// 
    /// # 返回
    /// 返回裁剪后的新图像
    pub fn auto_crop(image: &GdImage, threshold: u8) -> GdImage {
        let width = image.width() as i32;
        let height = image.height() as i32;
        // 获取左上角颜色作为背景色
        let bg_color = image.get_pixel_color(0, 0);
        // 找到边界
        let mut left = 0;
        let mut right = width - 1;
        let mut top = 0;
        let mut bottom = height - 1;
        // 找左边界
        'outer: for x in 0..width {
            for y in 0..height {
                let color = image.get_pixel_color(x, y);
                if Self::color_diff(&color, &bg_color) > threshold {
                    left = x;
                    break 'outer;
                }
            }
        }
        // 找右边界
        'outer: for x in (0..width).rev() {
            for y in 0..height {
                let color = image.get_pixel_color(x, y);
                if Self::color_diff(&color, &bg_color) > threshold {
                    right = x;
                    break 'outer;
                }
            }
        }
        // 找上边界
        'outer: for y in 0..height {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                if Self::color_diff(&color, &bg_color) > threshold {
                    top = y;
                    break 'outer;
                }
            }
        }
        // 找下边界
        'outer: for y in (0..height).rev() {
            for x in 0..width {
                let color = image.get_pixel_color(x, y);
                if Self::color_diff(&color, &bg_color) > threshold {
                    bottom = y;
                    break 'outer;
                }
            }
        }
        Self::crop(image, left, top, (right - left + 1) as u32, (bottom - top + 1) as u32)
    }

    /// 计算颜色差异
    fn color_diff(c1: &GdColor, c2: &GdColor) -> u8 {
        let dr = (c1.red as i32 - c2.red as i32).abs() as u8;
        let dg = (c1.green as i32 - c2.green as i32).abs() as u8;
        let db = (c1.blue as i32 - c2.blue as i32).abs() as u8;
        dr.max(dg).max(db)
    }

    /// 仿射变换
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `matrix`: 仿射变换矩阵 [a, b, c, d, e, f]
    /// - `bg_color`: 背景颜色
    /// 
    /// # 返回
    /// 返回变换后的新图像
    pub fn affine(image: &GdImage, matrix: [f64; 6], bg_color: GdColor) -> GdImage {
        let width = image.width();
        let height = image.height();
        let mut result = GdImage::new_true_color(width, height);
        // 填充背景
        for y in 0..height {
            for x in 0..width {
                result.set_pixel(x as i32, y as i32, bg_color.to_int());
            }
        }
        // 计算逆变换
        let det = matrix[0] * matrix[3] - matrix[1] * matrix[2];
        if det.abs() < 1e-10 {
            return result;
        }
        let inv_matrix = [
            matrix[3] / det,
            -matrix[1] / det,
            -matrix[2] / det,
            matrix[0] / det,
            (matrix[2] * matrix[5] - matrix[3] * matrix[4]) / det,
            (matrix[1] * matrix[4] - matrix[0] * matrix[5]) / det,
        ];
        // 应用逆变换
        for y in 0..height {
            for x in 0..width {
                let src_x = inv_matrix[0] * x as f64 + inv_matrix[2] * y as f64 + inv_matrix[4];
                let src_y = inv_matrix[1] * x as f64 + inv_matrix[3] * y as f64 + inv_matrix[5];
                if src_x >= 0.0 && src_x < width as f64 && src_y >= 0.0 && src_y < height as f64 {
                    let color = Self::bilinear_interpolate(image, src_x, src_y);
                    result.set_pixel(x as i32, y as i32, color.to_int());
                }
            }
        }
        result
    }

    /// 创建旋转矩阵
    /// 
    /// # 参数
    /// - `angle`: 旋转角度（度）
    /// - `cx`: 旋转中心 X
    /// - `cy`: 旋转中心 Y
    /// 
    /// # 返回
    /// 返回仿射变换矩阵
    pub fn rotation_matrix(angle: f64, cx: f64, cy: f64) -> [f64; 6] {
        let rad = angle.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        [
            cos_a, sin_a, -sin_a, cos_a,
            cx - cos_a * cx + sin_a * cy,
            cy - sin_a * cx - cos_a * cy,
        ]
    }

    /// 创建缩放矩阵
    /// 
    /// # 参数
    /// - `sx`: X 缩放比例
    /// - `sy`: Y 缩放比例
    /// 
    /// # 返回
    /// 返回仿射变换矩阵
    pub fn scale_matrix(sx: f64, sy: f64) -> [f64; 6] {
        [sx, 0.0, 0.0, sy, 0.0, 0.0]
    }

    /// 创建平移矩阵
    /// 
    /// # 参数
    /// - `tx`: X 平移量
    /// - `ty`: Y 平移量
    /// 
    /// # 返回
    /// 返回仿射变换矩阵
    pub fn translation_matrix(tx: f64, ty: f64) -> [f64; 6] {
        [1.0, 0.0, 0.0, 1.0, tx, ty]
    }

    /// 创建剪切矩阵
    /// 
    /// # 参数
    /// - `shx`: X 剪切因子
    /// - `shy`: Y 剪切因子
    /// 
    /// # 返回
    /// 返回仿射变换矩阵
    pub fn shear_matrix(shx: f64, shy: f64) -> [f64; 6] {
        [1.0, shy, shx, 1.0, 0.0, 0.0]
    }

    /// 组合变换矩阵
    /// 
    /// # 参数
    /// - `m1`: 第一个矩阵
    /// - `m2`: 第二个矩阵
    /// 
    /// # 返回
    /// 返回组合后的矩阵
    pub fn combine_matrices(m1: [f64; 6], m2: [f64; 6]) -> [f64; 6] {
        [
            m1[0] * m2[0] + m1[2] * m2[1],
            m1[1] * m2[0] + m1[3] * m2[1],
            m1[0] * m2[2] + m1[2] * m2[3],
            m1[1] * m2[2] + m1[3] * m2[3],
            m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
            m1[1] * m2[4] + m1[3] * m2[5] + m1[5],
        ]
    }

    /// 填充边缘
    /// 
    /// 将图像填充到指定尺寸
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `target_width`: 目标宽度
    /// - `target_height`: 目标高度
    /// - `bg_color`: 背景颜色
    /// - `alignment`: 对齐方式
    /// 
    /// # 返回
    /// 返回填充后的新图像
    pub fn pad(
        image: &GdImage,
        target_width: u32,
        target_height: u32,
        bg_color: GdColor,
        alignment: (GdTextAlignment, GdTextAlignment),
    ) -> GdImage {
        let mut result = GdImage::new_true_color(target_width, target_height);
        // 填充背景
        for y in 0..target_height {
            for x in 0..target_width {
                result.set_pixel(x as i32, y as i32, bg_color.to_int());
            }
        }
        // 计算起始位置
        let x = match alignment.0 {
            GdTextAlignment::Left => 0,
            GdTextAlignment::Center => ((target_width - image.width()) / 2) as i32,
            GdTextAlignment::Right => (target_width - image.width()) as i32,
            _ => 0,
        };
        let y = match alignment.1 {
            GdTextAlignment::Top => 0,
            GdTextAlignment::Center => ((target_height - image.height()) / 2) as i32,
            GdTextAlignment::Bottom => (target_height - image.height()) as i32,
            _ => 0,
        };
        // 复制图像
        for py in 0..image.height() {
            for px in 0..image.width() {
                let color = image.get_pixel_color(px as i32, py as i32);
                result.set_pixel(x + px as i32, y + py as i32, color.to_int());
            }
        }
        result
    }

    /// 创建缩略图
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `max_width`: 最大宽度
    /// - `max_height`: 最大高度
    /// 
    /// # 返回
    /// 返回缩略图
    pub fn thumbnail(image: &GdImage, max_width: u32, max_height: u32) -> GdImage {
        Self::scale_to_fit(image, max_width, max_height)
    }

    /// 创建圆形裁剪
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `cx`: 圆心 X
    /// - `cy`: 圆心 Y
    /// - `radius`: 半径
    /// 
    /// # 返回
    /// 返回圆形裁剪后的图像
    pub fn circular_crop(image: &GdImage, cx: i32, cy: i32, radius: i32) -> GdImage {
        let diameter = (radius * 2) as u32;
        let mut result = GdImage::new_true_color(diameter, diameter);
        for y in 0..diameter {
            for x in 0..diameter {
                let dx = x as i32 - radius;
                let dy = y as i32 - radius;
                let dist = ((dx * dx + dy * dy) as f64).sqrt() as i32;
                if dist <= radius {
                    let src_x = cx + dx;
                    let src_y = cy + dy;
                    if src_x >= 0 && src_x < image.width() as i32 && src_y >= 0 && src_y < image.height() as i32 {
                        let color = image.get_pixel_color(src_x, src_y);
                        result.set_pixel(x as i32, y as i32, color.to_int());
                    }
                }
            }
        }
        result
    }

    /// 创建圆角矩形裁剪
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `corner_radius`: 圆角半径
    /// 
    /// # 返回
    /// 返回圆角裁剪后的图像
    pub fn rounded_corners(image: &GdImage, corner_radius: u32) -> GdImage {
        let width = image.width();
        let height = image.height();
        let mut result = GdImage::new_true_color(width, height);
        let r = corner_radius as i32;
        for y in 0..height {
            for x in 0..width {
                let in_corner = if (x as i32) < r && (y as i32) < r {
                    // 左上角
                    let dx = r - x as i32;
                    let dy = r - y as i32;
                    dx * dx + dy * dy <= r * r
                } else if (x as i32) >= (width as i32 - r) && (y as i32) < r {
                    // 右上角
                    let dx = x as i32 - (width as i32 - r);
                    let dy = r - y as i32;
                    dx * dx + dy * dy <= r * r
                } else if (x as i32) < r && (y as i32) >= (height as i32 - r) {
                    // 左下角
                    let dx = r - x as i32;
                    let dy = y as i32 - (height as i32 - r);
                    dx * dx + dy * dy <= r * r
                } else if (x as i32) >= (width as i32 - r) && (y as i32) >= (height as i32 - r) {
                    // 右下角
                    let dx = x as i32 - (width as i32 - r);
                    let dy = y as i32 - (height as i32 - r);
                    dx * dx + dy * dy <= r * r
                } else {
                    true
                };
                if in_corner {
                    let color = image.get_pixel_color(x as i32, y as i32);
                    result.set_pixel(x as i32, y as i32, color.to_int());
                }
            }
        }
        result
    }
}

/// 文字对齐方式（用于填充对齐）
use crate::gd::text::GdTextAlignment;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        let image = GdImage::new_true_color(100, 100);
        let scaled = GdTransform::scale(&image, 50, 50);
        assert_eq!(scaled.width(), 50);
        assert_eq!(scaled.height(), 50);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(255, 0, 0);
        image.set_pixel(0, 0, color);
        let flipped = GdTransform::flip_horizontal(&image);
        assert_eq!(flipped.get_pixel(9, 0), color);
    }

    #[test]
    fn test_flip_vertical() {
        let mut image = GdImage::new_true_color(10, 10);
        let color = image.color_allocate(0, 255, 0);
        image.set_pixel(0, 0, color);
        let flipped = GdTransform::flip_vertical(&image);
        assert_eq!(flipped.get_pixel(0, 9), color);
    }

    #[test]
    fn test_crop() {
        let mut image = GdImage::new_true_color(100, 100);
        let color = image.color_allocate(0, 0, 255);
        image.fill(color);
        let cropped = GdTransform::crop(&image, 10, 10, 50, 50);
        assert_eq!(cropped.width(), 50);
        assert_eq!(cropped.height(), 50);
    }

    #[test]
    fn test_rotate_90() {
        let mut image = GdImage::new_true_color(10, 20);
        let color = image.color_allocate(255, 255, 0);
        image.set_pixel(0, 0, color);
        let rotated = GdTransform::rotate_90(&image, true);
        assert_eq!(rotated.width(), 20);
        assert_eq!(rotated.height(), 10);
    }

    #[test]
    fn test_rotation_matrix() {
        let matrix = GdTransform::rotation_matrix(90.0, 50.0, 50.0);
        assert!((matrix[0] - 0.0).abs() < 1e-10);
        assert!((matrix[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_matrix() {
        let matrix = GdTransform::scale_matrix(2.0, 0.5);
        assert_eq!(matrix[0], 2.0);
        assert_eq!(matrix[3], 0.5);
    }
}
