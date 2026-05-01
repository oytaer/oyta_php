# Storage 文件存储模块

## 模块结构

```
storage/
├── mod.rs       # 模块入口
├── traits.rs    # 存储驱动接口
├── local.rs     # 本地存储
├── ftp.rs       # FTP 存储
├── s3.rs        # AWS S3 存储
├── oss.rs       # 阿里云 OSS 存储
├── cos.rs       # 腾讯云 COS 存储
├── manager.rs   # 存储管理器
└── facade.rs    # 存储门面
```

## 支持的存储驱动

- **local**: 本地文件系统
- **ftp**: FTP 服务器
- **s3**: AWS S3
- **oss**: 阿里云 OSS
- **cos**: 腾讯云 COS

## 基本使用

```php
<?php
// 上传文件
$path = Storage::put('uploads', $file);
$path = Storage::putFile('uploads', $file, 'public');

// 获取文件内容
$content = Storage::get('uploads/file.txt');

// 写入文件
Storage::put('uploads/file.txt', $content);

// 删除文件
Storage::delete('uploads/file.txt');

// 检查文件是否存在
if (Storage::exists('uploads/file.txt')) {
    // ...
}

// 获取文件 URL
$url = Storage::url('uploads/file.txt');

// 获取临时访问 URL
$url = Storage::temporaryUrl('uploads/file.txt', now()->addHour());

// 复制文件
Storage::copy('old/path', 'new/path');

// 移动文件
Storage::move('old/path', 'new/path');

// 列出目录文件
$files = Storage::files('uploads');
$allFiles = Storage::allFiles('uploads');

// 创建目录
Storage::makeDirectory('uploads/new');

// 删除目录
Storage::deleteDirectory('uploads/old');
```

## 配置示例

```php
<?php
// config/filesystem.php
return [
    'default' => 'local',
    
    'disks' => [
        'local' => [
            'type' => 'local',
            'root' => runtime_path('storage'),
        ],
        
        'public' => [
            'type' => 'local',
            'root' => public_path('uploads'),
            'url' => '/uploads',
        ],
        
        's3' => [
            'type' => 's3',
            'key' => env('AWS_ACCESS_KEY_ID'),
            'secret' => env('AWS_SECRET_ACCESS_KEY'),
            'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
            'bucket' => env('AWS_BUCKET'),
            'url' => env('AWS_URL'),
        ],
        
        'oss' => [
            'type' => 'oss',
            'access_id' => env('OSS_ACCESS_KEY_ID'),
            'access_key' => env('OSS_ACCESS_KEY_SECRET'),
            'bucket' => env('OSS_BUCKET'),
            'endpoint' => env('OSS_ENDPOINT'),
            'cdnDomain' => env('OSS_CDN_DOMAIN'),
        ],
    ],
];
```
