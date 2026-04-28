# 沙箱模块 (Sandbox)

## 模块概述

沙箱模块提供安全的沙箱环境，用于执行不可信代码、插件系统、用户自定义逻辑等场景。

## 模块结构

```
sandbox/
├── mod.rs          # 模块入口
├── executor.rs     # 沙箱执行器
├── resource.rs     # 资源管理
├── filesystem.rs   # 文件系统隔离
├── network.rs      # 网络隔离
└── monitor.rs      # 沙箱监控
```

## 主要类型

| 类型 | 说明 |
|------|------|
| SandboxExecutor | 沙箱执行器 |
| SandboxConfig | 沙箱配置 |
| ResourceLimits | 资源限制 |
| FilesystemIsolation | 文件系统隔离 |
| NetworkIsolation | 网络隔离 |

## 资源限制

| 限制 | 说明 |
|------|------|
| memory_limit | 内存限制 |
| time_limit | 执行时间限制 |
| cpu_limit | CPU 限制 |
| file_access | 文件访问权限 |
| network_access | 网络访问权限 |

## 使用示例

### 创建沙箱

```php
<?php
use oyta\sandbox\SandboxExecutor;
use oyta\sandbox\SandboxConfig;

// 创建沙箱配置
$config = new SandboxConfig([
    'memory_limit' => 128 * 1024 * 1024,  // 128MB
    'time_limit' => 5,                      // 5秒
    'cpu_limit' => 50,                      // 50%
    'file_access' => ['read' => ['/data']], // 只读 /data
    'network_access' => false,              // 禁止网络
]);

// 创建沙箱执行器
$executor = new SandboxExecutor($config);

// 执行代码
$result = $executor->execute('<?php return 1 + 1;');

echo $result; // 输出: 2
```

### 文件系统隔离

```php
<?php
use oyta\sandbox\FilesystemIsolation;

$fs = new FilesystemIsolation([
    'read_only' => ['/app/data'],
    'read_write' => ['/app/temp'],
    'denied' => ['/etc', '/var'],
]);
```

### 网络隔离

```php
<?php
use oyta\sandbox\NetworkIsolation;

$network = new NetworkIsolation([
    'allow_outbound' => ['api.example.com:443'],
    'allow_inbound' => false,
]);
```

## 技术实现

- 内存安全隔离
- 时间限制强制终止
- 文件系统隔离
- 网络隔离
- 违规行为监控
