# GD 图形处理模块 (GD)

## 模块概述

GD 图形处理模块提供完整的 PHP GD 库兼容功能，包括图像创建、绘图操作、图像滤镜、文字渲染、图像变换等。

## 模块结构

```
gd/
├── mod.rs          # 模块入口
├── color.rs        # 颜色操作
├── draw.rs         # 绘图操作
├── filter.rs       # 图像滤镜
├── image.rs        # 图像处理
├── text.rs         # 文字渲染
└── transform.rs    # 图像变换
```

## 主要类型

| 类型 | 说明 |
|------|------|
| GdImage | 图像对象 |
| GdColor | 颜色操作 |
| GdDraw | 绘图操作 |
| GdFilter | 图像滤镜 |
| GdFont | 字体处理 |
| GdText | 文字渲染 |
| GdTransform | 图像变换 |

## 使用示例

### 创建图像

```php
<?php
// 创建空白图像
$image = imagecreatetruecolor(200, 100);

// 分配颜色
$white = imagecolorallocate($image, 255, 255, 255);
$black = imagecolorallocate($image, 0, 0, 0);

// 填充背景
imagefill($image, 0, 0, $white);

// 输出图像
header('Content-Type: image/png');
imagepng($image);

// 销毁图像
imagedestroy($image);
```

### 绘图操作

```php
<?php
$image = imagecreatetruecolor(400, 300);
$white = imagecolorallocate($image, 255, 255, 255);
$red = imagecolorallocate($image, 255, 0, 0);
$blue = imagecolorallocate($image, 0, 0, 255);

// 填充背景
imagefill($image, 0, 0, $white);

// 画线
imageline($image, 0, 0, 400, 300, $red);

// 画矩形
imagerectangle($image, 50, 50, 150, 150, $blue);

// 画椭圆
imageellipse($image, 300, 150, 100, 80, $blue);

// 画圆弧
imagearc($image, 300, 150, 100, 80, 0, 180, $red);

// 输出
header('Content-Type: image/png');
imagepng($image);
imagedestroy($image);
```

### 文字渲染

```php
<?php
$image = imagecreatetruecolor(400, 100);
$white = imagecolorallocate($image, 255, 255, 255);
$black = imagecolorallocate($image, 0, 0, 0);

imagefill($image, 0, 0, $white);

// 使用内置字体
imagestring($image, 5, 10, 10, 'Hello World', $black);

// 使用 TrueType 字体
imagettftext($image, 20, 0, 10, 50, $black, '/path/to/font.ttf', '中文文字');

header('Content-Type: image/png');
imagepng($image);
imagedestroy($image);
```

### 图像操作

```php
<?php
// 从文件加载
$image = imagecreatefromjpeg('photo.jpg');

// 获取尺寸
$width = imagesx($image);
$height = imagesy($image);

// 缩放
$new_image = imagescale($image, 200, 150);

// 裁剪
$cropped = imagecrop($image, ['x' => 0, 'y' => 0, 'width' => 100, 'height' => 100]);

// 旋转
$rotated = imagerotate($image, 90, 0);

// 翻转
imageflip($image, IMG_FLIP_HORIZONTAL);

// 保存
imagejpeg($image, 'output.jpg', 90);
imagepng($image, 'output.png');
```

### 图像滤镜

```php
<?php
$image = imagecreatefromjpeg('photo.jpg');

// 灰度
imagefilter($image, IMG_FILTER_GRAYSCALE);

// 模糊
imagefilter($image, IMG_FILTER_GAUSSIAN_BLUR);

// 锐化
imagefilter($image, IMG_FILTER_MEAN_REMOVAL);

// 亮度
imagefilter($image, IMG_FILTER_BRIGHTNESS, 30);

// 对比度
imagefilter($image, IMG_FILTER_CONTRAST, -20);

header('Content-Type: image/jpeg');
imagejpeg($image);
imagedestroy($image);
```

## 支持的图像格式

| 格式 | 创建函数 | 输出函数 |
|------|----------|----------|
| JPEG | imagecreatefromjpeg | imagejpeg |
| PNG | imagecreatefrompng | imagepng |
| GIF | imagecreatefromgif | imagegif |
| WebP | imagecreatefromwebp | imagewebp |
| WBMP | - | imagewbmp |

## 技术实现

- 基于 image crate 实现
- 支持多种图像格式
- 支持图像滤镜
- 支持 TrueType 字体
