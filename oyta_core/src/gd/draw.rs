//! 绘图操作模块
//! 
//! 本模块实现 GD 图像的绘图功能，包括：
//! - 基本图形绘制（点、线、矩形、多边形）
//! - 曲线绘制（弧、椭圆、圆）
//! - 填充操作
//! - 画笔和样式设置

use crate::gd::color::GdColor;
use crate::gd::image::GdImage;

/// GD 绘图工具
/// 
/// 提供各种绘图操作方法
pub struct GdDraw;

impl GdDraw {
    /// 绘制单个像素点
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// - `color`: 颜色索引或颜色值
    pub fn set_pixel(image: &mut GdImage, x: i32, y: i32, color: i32) {
        image.set_pixel(x, y, color);
    }

    /// 获取单个像素点颜色
    /// 
    /// # 参数
    /// - `image`: 源图像
    /// - `x`: X 坐标
    /// - `y`: Y 坐标
    /// 
    /// # 返回
    /// 返回颜色索引或颜色值
    pub fn get_pixel(image: &GdImage, x: i32, y: i32) -> i32 {
        image.get_pixel(x, y)
    }

    /// 绘制直线
    /// 
    /// 使用 Bresenham 直线算法绘制直线
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x1`: 起点 X 坐标
    /// - `y1`: 起点 Y 坐标
    /// - `x2`: 终点 X 坐标
    /// - `y2`: 终点 Y 坐标
    /// - `color`: 颜色索引或颜色值
    pub fn line(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32) {
        let thickness = image.get_thickness();
        if thickness <= 1 {
            Self::draw_line_bresenham(image, x1, y1, x2, y2, color);
        } else {
            Self::draw_thick_line(image, x1, y1, x2, y2, color, thickness);
        }
    }

    /// Bresenham 直线算法
    fn draw_line_bresenham(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;
        loop {
            image.set_pixel(x, y, color);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// 绘制粗直线
    fn draw_thick_line(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32, thickness: i32) {
        let half = thickness / 2;
        // 计算直线方向的垂直向量
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            Self::filled_rectangle(image, x1 - half, y1 - half, x1 + half, y1 + half, color);
            return;
        }
        let nx = -dy / len;
        let ny = dx / len;
        // 绘制多条平行线形成粗线
        for i in -half..=half {
            let offset_x = (nx * i as f64).round() as i32;
            let offset_y = (ny * i as f64).round() as i32;
            Self::draw_line_bresenham(image, x1 + offset_x, y1 + offset_y, x2 + offset_x, y2 + offset_y, color);
        }
    }

    /// 绘制虚线
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x1`: 起点 X 坐标
    /// - `y1`: 起点 Y 坐标
    /// - `x2`: 终点 X 坐标
    /// - `y2`: 终点 Y 坐标
    /// - `color`: 颜色索引或颜色值
    /// - `style`: 虚线样式数组
    pub fn dashed_line(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32, style: &[i32]) {
        if style.is_empty() {
            Self::line(image, x1, y1, x2, y2, color);
            return;
        }
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;
        let mut style_idx = 0usize;
        let mut step = 0i32;
        loop {
            // 根据样式决定是否绘制
            if style[style_idx] != -1 {
                image.set_pixel(x, y, if style[style_idx] >= 0 { style[style_idx] } else { color });
            }
            step += 1;
            style_idx = (step as usize) % style.len();
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// 绘制矩形
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x1`: 左上角 X 坐标
    /// - `y1`: 左上角 Y 坐标
    /// - `x2`: 右下角 X 坐标
    /// - `y2`: 右下角 Y 坐标
    /// - `color`: 颜色索引或颜色值
    pub fn rectangle(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32) {
        Self::line(image, x1, y1, x2, y1, color);
        Self::line(image, x2, y1, x2, y2, color);
        Self::line(image, x2, y2, x1, y2, color);
        Self::line(image, x1, y2, x1, y1, color);
    }

    /// 绘制填充矩形
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x1`: 左上角 X 坐标
    /// - `y1`: 左上角 Y 坐标
    /// - `x2`: 右下角 X 坐标
    /// - `y2`: 右下角 Y 坐标
    /// - `color`: 颜色索引或颜色值
    pub fn filled_rectangle(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: i32) {
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                image.set_pixel(x, y, color);
            }
        }
    }

    /// 绘制多边形
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `points`: 顶点坐标数组 (x, y)
    /// - `color`: 颜色索引或颜色值
    pub fn polygon(image: &mut GdImage, points: &[(i32, i32)], color: i32) {
        if points.len() < 2 {
            return;
        }
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            Self::line(image, x1, y1, x2, y2, color);
        }
    }

    /// 绘制填充多边形
    /// 
    /// 使用扫描线算法填充多边形
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `points`: 顶点坐标数组 (x, y)
    /// - `color`: 颜色索引或颜色值
    pub fn filled_polygon(image: &mut GdImage, points: &[(i32, i32)], color: i32) {
        if points.len() < 3 {
            return;
        }
        // 找到 Y 范围
        let min_y = points.iter().map(|p| p.1).min().unwrap();
        let max_y = points.iter().map(|p| p.1).max().unwrap();
        // 对每条扫描线进行处理
        for y in min_y..=max_y {
            // 找到与扫描线的交点
            let mut intersections = Vec::new();
            let n = points.len();
            for i in 0..n {
                let (x1, y1) = points[i];
                let (x2, y2) = points[(i + 1) % n];
                // 检查边是否与扫描线相交
                if (y1 <= y && y2 > y) || (y2 <= y && y1 > y) {
                    // 计算交点 X 坐标
                    let x = x1 as f64 + (y - y1) as f64 * (x2 - x1) as f64 / (y2 - y1) as f64;
                    intersections.push(x as i32);
                }
            }
            // 排序交点
            intersections.sort();
            // 填充交点对之间的区域
            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    let x1 = intersections[i];
                    let x2 = intersections[i + 1];
                    for x in x1..=x2 {
                        image.set_pixel(x, y, color);
                    }
                }
            }
        }
    }

    /// 绘制弧
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    /// - `start`: 起始角度（度）
    /// - `end`: 结束角度（度）
    /// - `color`: 颜色索引或颜色值
    /// - `style`: 弧样式
    pub fn arc(image: &mut GdImage, cx: i32, cy: i32, width: i32, height: i32, start: i32, end: i32, color: i32, style: ArcStyle) {
        let a = width as f64 / 2.0;
        let b = height as f64 / 2.0;
        let start_rad = (start as f64).to_radians();
        let end_rad = (end as f64).to_radians();
        let steps = ((width + height) as f64 * std::f64::consts::PI / 4.0) as i32;
        let step_angle = (end_rad - start_rad) / steps as f64;
        let mut prev_x = cx + (a * start_rad.cos()) as i32;
        let mut prev_y = cy + (b * start_rad.sin()) as i32;
        for i in 1..=steps {
            let angle = start_rad + step_angle * i as f64;
            let x = cx + (a * angle.cos()) as i32;
            let y = cy + (b * angle.sin()) as i32;
            if style.contains(ArcStyle::ARC) {
                Self::line(image, prev_x, prev_y, x, y, color);
            }
            prev_x = x;
            prev_y = y;
        }
        // 绘制弦
        if style.contains(ArcStyle::CHORD) {
            let x1 = cx + (a * start_rad.cos()) as i32;
            let y1 = cy + (b * start_rad.sin()) as i32;
            let x2 = cx + (a * end_rad.cos()) as i32;
            let y2 = cy + (b * end_rad.sin()) as i32;
            Self::line(image, x1, y1, x2, y2, color);
        }
        // 绘制扇形
        if style.contains(ArcStyle::PIE) {
            let x1 = cx + (a * start_rad.cos()) as i32;
            let y1 = cy + (b * start_rad.sin()) as i32;
            let x2 = cx + (a * end_rad.cos()) as i32;
            let y2 = cy + (b * end_rad.sin()) as i32;
            Self::line(image, cx, cy, x1, y1, color);
            Self::line(image, cx, cy, x2, y2, color);
        }
    }

    /// 绘制填充弧
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    /// - `start`: 起始角度（度）
    /// - `end`: 结束角度（度）
    /// - `color`: 颜色索引或颜色值
    /// - `style`: 弧样式
    pub fn filled_arc(image: &mut GdImage, cx: i32, cy: i32, width: i32, height: i32, start: i32, end: i32, color: i32, style: ArcStyle) {
        let a = width as f64 / 2.0;
        let b = height as f64 / 2.0;
        let start_rad = (start as f64).to_radians();
        let end_rad = (end as f64).to_radians();
        // 构建扇形顶点
        let mut points = vec![(cx, cy)];
        let steps = ((width + height) as f64 * std::f64::consts::PI / 4.0) as i32;
        let step_angle = (end_rad - start_rad) / steps as f64;
        for i in 0..=steps {
            let angle = start_rad + step_angle * i as f64;
            let x = cx + (a * angle.cos()) as i32;
            let y = cy + (b * angle.sin()) as i32;
            points.push((x, y));
        }
        Self::filled_polygon(image, &points, color);
        // 绘制边框
        if style.contains(ArcStyle::ARC) || style.contains(ArcStyle::CHORD) || style.contains(ArcStyle::PIE) {
            Self::arc(image, cx, cy, width, height, start, end, color, style);
        }
    }

    /// 绘制椭圆
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    /// - `color`: 颜色索引或颜色值
    pub fn ellipse(image: &mut GdImage, cx: i32, cy: i32, width: i32, height: i32, color: i32) {
        Self::arc(image, cx, cy, width, height, 0, 360, color, ArcStyle::ARC);
    }

    /// 绘制填充椭圆
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    /// - `color`: 颜色索引或颜色值
    pub fn filled_ellipse(image: &mut GdImage, cx: i32, cy: i32, width: i32, height: i32, color: i32) {
        let a = width as f64 / 2.0;
        let b = height as f64 / 2.0;
        // 使用扫描线算法填充
        let min_y = cy - (b as i32);
        let max_y = cy + (b as i32);
        for y in min_y..=max_y {
            let dy = (y - cy) as f64;
            let dx = (a * (1.0 - (dy * dy) / (b * b))).sqrt();
            let x1 = (cx as f64 - dx) as i32;
            let x2 = (cx as f64 + dx) as i32;
            for x in x1..=x2 {
                image.set_pixel(x, y, color);
            }
        }
    }

    /// 绘制圆
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `radius`: 半径
    /// - `color`: 颜色索引或颜色值
    pub fn circle(image: &mut GdImage, cx: i32, cy: i32, radius: i32, color: i32) {
        Self::ellipse(image, cx, cy, radius * 2, radius * 2, color);
    }

    /// 绘制填充圆
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `cx`: 圆心 X 坐标
    /// - `cy`: 圆心 Y 坐标
    /// - `radius`: 半径
    /// - `color`: 颜色索引或颜色值
    pub fn filled_circle(image: &mut GdImage, cx: i32, cy: i32, radius: i32, color: i32) {
        Self::filled_ellipse(image, cx, cy, radius * 2, radius * 2, color);
    }

    /// 洪水填充
    /// 
    /// 从指定点开始填充相邻的相同颜色区域
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x`: 起点 X 坐标
    /// - `y`: 起点 Y 坐标
    /// - `color`: 填充颜色
    pub fn fill(image: &mut GdImage, x: i32, y: i32, color: i32) {
        let target_color = image.get_pixel(x, y);
        if target_color == color {
            return;
        }
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut stack = vec![(x, y)];
        while let Some((px, py)) = stack.pop() {
            if px < 0 || px >= width || py < 0 || py >= height {
                continue;
            }
            let current = image.get_pixel(px, py);
            if current != target_color {
                continue;
            }
            image.set_pixel(px, py, color);
            stack.push((px + 1, py));
            stack.push((px - 1, py));
            stack.push((px, py + 1));
            stack.push((px, py - 1));
        }
    }

    /// 边界填充
    /// 
    /// 从指定点开始填充直到遇到边界颜色
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x`: 起点 X 坐标
    /// - `y`: 起点 Y 坐标
    /// - `color`: 填充颜色
    /// - `border`: 边界颜色
    pub fn fill_to_border(image: &mut GdImage, x: i32, y: i32, color: i32, border: i32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        let mut stack = vec![(x, y)];
        while let Some((px, py)) = stack.pop() {
            if px < 0 || px >= width || py < 0 || py >= height {
                continue;
            }
            let current = image.get_pixel(px, py);
            if current == border || current == color {
                continue;
            }
            image.set_pixel(px, py, color);
            stack.push((px + 1, py));
            stack.push((px - 1, py));
            stack.push((px, py + 1));
            stack.push((px, py - 1));
        }
    }

    /// 复制图像区域
    /// 
    /// # 参数
    /// - `dst`: 目标图像
    /// - `src`: 源图像
    /// - `dst_x`: 目标 X 坐标
    /// - `dst_y`: 目标 Y 坐标
    /// - `src_x`: 源 X 坐标
    /// - `src_y`: 源 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    pub fn copy(dst: &mut GdImage, src: &GdImage, dst_x: i32, dst_y: i32, src_x: i32, src_y: i32, width: i32, height: i32) {
        for y in 0..height {
            for x in 0..width {
                let sx = src_x + x;
                let sy = src_y + y;
                let dx = dst_x + x;
                let dy = dst_y + y;
                if sx >= 0 && sx < src.width() as i32 && sy >= 0 && sy < src.height() as i32 {
                    if dx >= 0 && dx < dst.width() as i32 && dy >= 0 && dy < dst.height() as i32 {
                        let color = src.get_pixel(sx, sy);
                        dst.set_pixel(dx, dy, color);
                    }
                }
            }
        }
    }

    /// 复制并调整大小的图像区域
    /// 
    /// # 参数
    /// - `dst`: 目标图像
    /// - `src`: 源图像
    /// - `dst_x`: 目标 X 坐标
    /// - `dst_y`: 目标 Y 坐标
    /// - `dst_w`: 目标宽度
    /// - `dst_h`: 目标高度
    /// - `src_x`: 源 X 坐标
    /// - `src_y`: 源 Y 坐标
    /// - `src_w`: 源宽度
    /// - `src_h`: 源高度
    pub fn copy_resized(dst: &mut GdImage, src: &GdImage, dst_x: i32, dst_y: i32, dst_w: i32, dst_h: i32, src_x: i32, src_y: i32, src_w: i32, src_h: i32) {
        if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
            return;
        }
        let x_ratio = src_w as f64 / dst_w as f64;
        let y_ratio = src_h as f64 / dst_h as f64;
        for y in 0..dst_h {
            for x in 0..dst_w {
                let sx = src_x + (x as f64 * x_ratio) as i32;
                let sy = src_y + (y as f64 * y_ratio) as i32;
                let dx = dst_x + x;
                let dy = dst_y + y;
                if sx >= 0 && sx < src.width() as i32 && sy >= 0 && sy < src.height() as i32 {
                    if dx >= 0 && dx < dst.width() as i32 && dy >= 0 && dy < dst.height() as i32 {
                        let color = src.get_pixel(sx, sy);
                        dst.set_pixel(dx, dy, color);
                    }
                }
            }
        }
    }

    /// 复制并合并图像区域（支持 Alpha 混合）
    /// 
    /// # 参数
    /// - `dst`: 目标图像
    /// - `src`: 源图像
    /// - `dst_x`: 目标 X 坐标
    /// - `dst_y`: 目标 Y 坐标
    /// - `src_x`: 源 X 坐标
    /// - `src_y`: 源 Y 坐标
    /// - `width`: 宽度
    /// - `height`: 高度
    /// - `pct`: 合并百分比 (0-100)
    pub fn copy_merge(dst: &mut GdImage, src: &GdImage, dst_x: i32, dst_y: i32, src_x: i32, src_y: i32, width: i32, height: i32, pct: i32) {
        let pct = pct.clamp(0, 100) as f64 / 100.0;
        for y in 0..height {
            for x in 0..width {
                let sx = src_x + x;
                let sy = src_y + y;
                let dx = dst_x + x;
                let dy = dst_y + y;
                if sx >= 0 && sx < src.width() as i32 && sy >= 0 && sy < src.height() as i32 {
                    if dx >= 0 && dx < dst.width() as i32 && dy >= 0 && dy < dst.height() as i32 {
                        let src_color = src.get_pixel_color(sx, sy);
                        let dst_color = dst.get_pixel_color(dx, dy);
                        let blended = src_color.blend(&dst_color, pct as f32);
                        dst.set_pixel(dx, dy, blended.to_int());
                    }
                }
            }
        }
    }

    /// 绘制抗锯齿线
    /// 
    /// 使用 Xiaolin Wu 直线算法绘制抗锯齿线
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `x1`: 起点 X 坐标
    /// - `y1`: 起点 Y 坐标
    /// - `x2`: 终点 X 坐标
    /// - `y2`: 终点 Y 坐标
    /// - `color`: 颜色
    pub fn aa_line(image: &mut GdImage, x1: i32, y1: i32, x2: i32, y2: i32, color: GdColor) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        if dx > dy {
            let mut x = x1;
            let mut y = y1 as f64;
            let gradient = dy as f64 / dx as f64;
            for _ in 0..=dx {
                let y_floor = y.floor() as i32;
                let y_frac = y - y.floor();
                let c1 = color.with_alpha((color.alpha as f64 * (1.0 - y_frac)) as u8);
                let c2 = color.with_alpha((color.alpha as f64 * y_frac) as u8);
                Self::blend_pixel(image, x, y_floor, c1);
                Self::blend_pixel(image, x, y_floor + 1, c2);
                x += sx;
                y += gradient * sx as f64;
            }
        } else {
            let mut x = x1 as f64;
            let mut y = y1;
            let gradient = dx as f64 / dy as f64;
            for _ in 0..=dy {
                let x_floor = x.floor() as i32;
                let x_frac = x - x.floor();
                let c1 = color.with_alpha((color.alpha as f64 * (1.0 - x_frac)) as u8);
                let c2 = color.with_alpha((color.alpha as f64 * x_frac) as u8);
                Self::blend_pixel(image, x_floor, y, c1);
                Self::blend_pixel(image, x_floor + 1, y, c2);
                y += sy;
                x += gradient * sy as f64;
            }
        }
    }

    /// 混合像素
    fn blend_pixel(image: &mut GdImage, x: i32, y: i32, color: GdColor) {
        if x < 0 || x >= image.width() as i32 || y < 0 || y >= image.height() as i32 {
            return;
        }
        let existing = image.get_pixel_color(x, y);
        let blended = color.blend(&existing, (1.0 - color.alpha as f32 / 127.0) as f32);
        image.set_pixel(x, y, blended.to_int());
    }

    /// 绘制贝塞尔曲线
    /// 
    /// # 参数
    /// - `image`: 目标图像
    /// - `points`: 控制点数组
    /// - `color`: 颜色索引或颜色值
    pub fn bezier(image: &mut GdImage, points: &[(i32, i32)], color: i32) {
        if points.len() < 4 {
            return;
        }
        let steps = 100;
        for i in 0..steps {
            let t1 = i as f64 / steps as f64;
            let t2 = (i + 1) as f64 / steps as f64;
            let (x1, y1) = Self::bezier_point(points, t1);
            let (x2, y2) = Self::bezier_point(points, t2);
            Self::line(image, x1 as i32, y1 as i32, x2 as i32, y2 as i32, color);
        }
    }

    /// 计算贝塞尔曲线上的点
    fn bezier_point(points: &[(i32, i32)], t: f64) -> (f64, f64) {
        let n = points.len() - 1;
        let mut x = 0.0;
        let mut y = 0.0;
        for i in 0..=n {
            let b = Self::bernstein(n, i, t);
            x += points[i].0 as f64 * b;
            y += points[i].1 as f64 * b;
        }
        (x, y)
    }

    /// 计算 Bernstein 基函数
    fn bernstein(n: usize, i: usize, t: f64) -> f64 {
        Self::binomial(n, i) as f64 * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
    }

    /// 计算二项式系数
    fn binomial(n: usize, k: usize) -> u64 {
        if k > n {
            return 0;
        }
        let mut result = 1u64;
        for i in 0..k.min(n - k) {
            result = result * (n - i) as u64 / (i + 1) as u64;
        }
        result
    }
}

bitflags::bitflags! {
    /// 弧样式标志
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ArcStyle: u32 {
        /// 绘制弧
        const ARC = 0x01;
        /// 绘制弦（连接弧的端点）
        const CHORD = 0x02;
        /// 绘制扇形（连接弧的端点到圆心）
        const PIE = 0x04;
        /// 不填充
        const NOFILL = 0x08;
        /// 填充扇形
        const FILLED = 0x10;
    }
}

impl Default for ArcStyle {
    fn default() -> Self {
        ArcStyle::ARC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_line() {
        let mut image = GdImage::new_true_color(100, 100);
        let color = image.color_allocate(255, 0, 0);
        GdDraw::line(&mut image, 0, 0, 99, 99, color);
        assert_ne!(image.get_pixel(50, 50), 0);
    }

    #[test]
    fn test_draw_rectangle() {
        let mut image = GdImage::new_true_color(100, 100);
        let color = image.color_allocate(0, 255, 0);
        GdDraw::rectangle(&mut image, 10, 10, 90, 90, color);
        assert_ne!(image.get_pixel(10, 10), 0);
    }

    #[test]
    fn test_draw_filled_rectangle() {
        let mut image = GdImage::new_true_color(100, 100);
        let color = image.color_allocate(0, 0, 255);
        GdDraw::filled_rectangle(&mut image, 10, 10, 90, 90, color);
        assert_ne!(image.get_pixel(50, 50), 0);
    }

    #[test]
    fn test_draw_circle() {
        let mut image = GdImage::new_true_color(100, 100);
        let color = image.color_allocate(255, 255, 0);
        GdDraw::circle(&mut image, 50, 50, 25, color);
        assert_ne!(image.get_pixel(75, 50), 0);
    }

    #[test]
    fn test_fill() {
        let mut image = GdImage::new_true_color(100, 100);
        let border = image.color_allocate(255, 0, 0);
        let fill_color = image.color_allocate(0, 255, 0);
        GdDraw::rectangle(&mut image, 10, 10, 90, 90, border);
        GdDraw::fill(&mut image, 50, 50, fill_color);
        assert_eq!(image.get_pixel(50, 50), fill_color);
    }
}
