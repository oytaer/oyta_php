# Logging 日志模块

## 模块结构

```
logging/
├── mod.rs        # 模块入口
├── setup.rs      # 日志配置
├── file_writer.rs # 文件写入器
├── channel.rs    # 日志通道
└── facade.rs     # 日志门面
```

## 基本使用

```php
<?php
// 记录日志
Log::info('用户登录', ['user_id' => 1]);
Log::error('系统错误', ['error' => $e->getMessage()]);
Log::warning('警告信息');
Log::debug('调试信息');

// 指定通道
Log::channel('daily')->info('每日日志');

// 写入到文件
Log::write('custom.log', '自定义日志内容');
```

## 日志级别

```php
<?php
Log::emergency($message);  // 紧急
Log::alert($message);      // 警报
Log::critical($message);   // 严重
Log::error($message);      // 错误
Log::warning($message);    // 警告
Log::notice($message);     // 注意
Log::info($message);       // 信息
Log::debug($message);      // 调试
```

## 配置

```php
<?php
// config/log.php
return [
    'default' => 'file',
    
    'channels' => [
        'file' => [
            'type' => 'file',
            'path' => runtime_path('log/app.log'),
            'level' => 'debug',
        ],
        
        'daily' => [
            'type' => 'daily',
            'path' => runtime_path('log/app.log'),
            'days' => 7,
        ],
        
        'syslog' => [
            'type' => 'syslog',
            'facility' => LOG_USER,
        ],
    ],
];
```
