//! GD 图像处理模块
//! 
//! 本模块提供完整的 PHP GD 库兼容功能，包括：
//! - 图像创建、加载和保存
//! - 绘图操作（点、线、矩形、多边形、弧、椭圆等）
//! - 图像滤镜（模糊、锐化、灰度、反色等）
//! - 文字渲染（TrueType 字体、内置字体）
//! - 图像变换（缩放、旋转、裁剪、翻转等）
//! - 颜色操作（颜色分配、透明度、调色板等）

pub mod color;
pub mod draw;
pub mod filter;
pub mod image;
pub mod text;
pub mod transform;

pub use color::{GdColor, GdColorAllocateResult, GdPalette};
pub use draw::GdDraw;
pub use filter::GdFilter;
pub use image::{GdImage, GdImageType, GdImageInfo};
pub use text::{GdFont, GdText, GdTextAlignment};
pub use transform::GdTransform;
