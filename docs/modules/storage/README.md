# 存储模块 (Storage)

## 模块概述

存储模块提供 OYTAPHP 的文件存储功能，对应 ThinkPHP 8.0 的 Storage 门面。支持多种存储驱动：本地、S3、OSS、COS、FTP 等。

## 模块结构

```
storage/
├── mod.rs          # 模块入口
├── facade.rs       # Storage 门面
├── traits.rs       # 存储 trait
├── manager.rs      # 存储管理器
├── local.rs        # 本地存储
├── s3.rs           # S3 存储
├── oss.rs          # 阿里云 OSS
├── cos.rs          # 腾讯云 COS
└── ftp.rs          # FTP 存储
```

## Storage 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `exists` | path: &str | bool | 检查文件是否存在 |
| `get` | path: &str | Result<String> | 获取文件内容 |
| `put` | path: &str, content: &str | Result<()> | 写入文件 |
| `delete` | path: &str | Result<()> | 删除文件 |
| `copy` | from: &str, to: &str | Result<()> | 复制文件 |
| `move` | from: &str, to: &str | Result<()> | 移动文件 |
| `size` | path: &str | Result<u64> | 获取文件大小 |
| `url` | path: &str | String | 获取文件 URL |
| `files` | directory: &str | Result<Vec<String>> | 获取目录文件列表 |
| `makeDirectory` | path: &str | Result<()> | 创建目录 |
| `deleteDirectory` | path: &str | Result<()> | 删除目录 |

## 使用示例

### 基础操作

```php
<?php
use oyta\facade\Storage;

// 检查文件是否存在
if (Storage::exists('uploads/avatar.jpg')) {
    echo "文件存在";
}

// 获取文件内容
$content = Storage::get('uploads/avatar.jpg');

// 写入文件
Storage::put('uploads/test.txt', '文件内容');

// 删除文件
Storage::delete('uploads/test.txt');

// 复制文件
Storage::copy('uploads/a.txt', 'uploads/b.txt');

// 移动文件
Storage::move('uploads/old.txt', 'uploads/new.txt');

// 获取文件大小
$size = Storage::size('uploads/avatar.jpg');

// 获取文件 URL
$url = Storage::url('uploads/avatar.jpg');
```

### 目录操作

```php
<?php
use oyta\facade\Storage;

// 创建目录
Storage::makeDirectory('uploads/2024');

// 删除目录
Storage::deleteDirectory('uploads/2024');

// 获取目录文件列表
$files = Storage::files('uploads');
```

## 配置示例

```php
<?php
// config/filesystem.php
return [
    'default' => 'local',
    'disks' => [
        'local' => [
            'driver' => 'local',
            'root' => public_path('uploads'),
            'url' => '/uploads',
        ],
        's3' => [
            'driver' => 's3',
            'key' => env('AWS_ACCESS_KEY_ID'),
            'secret' => env('AWS_SECRET_ACCESS_KEY'),
            'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
            'bucket' => env('AWS_BUCKET'),
            'url' => env('AWS_URL'),
        ],
        'oss' => [
            'driver' => 'oss',
            'access_id' => env('OSS_ACCESS_ID'),
            'access_key' => env('OSS_ACCESS_KEY'),
            'bucket' => env('OSS_BUCKET'),
            'endpoint' => env('OSS_ENDPOINT'),
        ],
        'cos' => [
            'driver' => 'cos',
            'secret_id' => env('COS_SECRET_ID'),
            'secret_key' => env('COS_SECRET_KEY'),
            'region' => env('COS_REGION'),
            'bucket' => env('COS_BUCKET'),
        ],
    ],
];
```

## 支持的存储驱动

| 驱动 | 说明 | 适用场景 |
|------|------|----------|
| local | 本地文件系统 | 开发环境、小规模应用 |
| s3 | AWS S3 / MinIO / Cloudflare R2 | 大规模存储、CDN 加速 |
| oss | 阿里云 OSS | 国内用户、CDN 加速 |
| cos | 腾讯云 COS | 国内用户、CDN 加速 |
| ftp | FTP/FTPS | 传统服务器存储 |

## 技术实现

- 统一的存储接口
- 支持多种云存储
- 异步文件操作
- 自动 MIME 类型检测
