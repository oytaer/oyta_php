# 沙箱模块

## 模块概述

沙箱模块实现了内存安全沙箱隔离，提供资源限制、文件系统隔离、网络隔离和监控统计功能，用于安全执行不受信任的代码。

## 文件结构

```
sandbox/
├── mod.rs          # 模块入口
├── executor.rs     # 沙箱执行器
├── resource.rs     # 资源管理
├── filesystem.rs   # 文件系统隔离
├── network.rs      # 网络隔离
└── monitor.rs      # 监控统计
```

## 核心设计

### 沙箱架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        沙箱执行环境                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    SandboxExecutor                        │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │                  代码执行区                       │   │   │
│   │   │   ┌─────────────────────────────────────────┐   │   │   │
│   │   │   │              PHP 代码                     │   │   │   │
│   │   │   └─────────────────────────────────────────┘   │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   │                           │                              │   │
│   │   ┌───────────────────────┼───────────────────────┐    │   │
│   │   │                       │                       │    │   │
│   │   ▼                       ▼                       ▼    │   │
│   │ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │   │
│   │ │ ResourceManager│ │FileSystemIso│ │ NetworkIso  │    │   │
│   │ │  内存/CPU/时间 │ │  文件系统隔离│ │  网络隔离   │    │   │
│   │ └──────────────┘ └──────────────┘ └──────────────┘    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### SandboxExecutor

沙箱执行器：

```rust
pub struct SandboxExecutor {
    /// 资源管理器
    resource_manager: ResourceManager,
    /// 文件系统隔离
    filesystem: FileSystemIsolation,
    /// 网络隔离
    network: NetworkIsolation,
    /// 监控器
    monitor: SandboxMonitor,
    /// 沙箱配置
    config: SandboxConfig,
}

pub struct SandboxConfig {
    /// 最大内存（字节）
    pub max_memory: usize,
    /// 最大 CPU 时间（毫秒）
    pub max_cpu_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 允许的文件路径
    pub allowed_paths: Vec<String>,
    /// 允许的网络地址
    pub allowed_hosts: Vec<String>,
    /// 允许的系统调用
    pub allowed_syscalls: Vec<String>,
}
```

### ResourceManager

资源管理器：

```rust
pub struct ResourceManager {
    /// 内存限制
    memory_limit: usize,
    /// 当前内存使用
    current_memory: AtomicUsize,
    /// CPU 时间限制
    cpu_time_limit_ms: u64,
    /// 已用 CPU 时间
    cpu_time_used_ms: AtomicU64,
    /// 执行时间限制
    execution_time_limit_ms: u64,
    /// 开始时间
    start_time: Option<Instant>,
}

impl ResourceManager {
    /// 分配内存
    pub fn allocate_memory(&self, size: usize) -> Result<(), SandboxError>;
    
    /// 释放内存
    pub fn free_memory(&self, size: usize);
    
    /// 检查资源限制
    pub fn check_limits(&self) -> Result<(), SandboxError>;
    
    /// 获取资源使用情况
    pub fn usage(&self) -> ResourceUsage;
}

pub struct ResourceUsage {
    /// 内存使用（字节）
    pub memory_used: usize,
    /// CPU 时间（毫秒）
    pub cpu_time_ms: u64,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}
```

### FileSystemIsolation

文件系统隔离：

```rust
pub struct FileSystemIsolation {
    /// 根目录映射
    root_map: HashMap<String, PathBuf>,
    /// 允许读取的路径
    allowed_read_paths: Vec<PathPattern>,
    /// 允许写入的路径
    allowed_write_paths: Vec<PathPattern>,
    /// 禁止访问的路径
    denied_paths: Vec<PathPattern>,
}

impl FileSystemIsolation {
    /// 检查读权限
    pub fn can_read(&self, path: &Path) -> bool;
    
    /// 检查写权限
    pub fn can_write(&self, path: &Path) -> bool;
    
    /// 映射路径
    pub fn map_path(&self, path: &Path) -> Result<PathBuf, SandboxError>;
    
    /// 添加允许路径
    pub fn allow_read(&mut self, pattern: &str);
    pub fn allow_write(&mut self, pattern: &str);
    
    /// 添加禁止路径
    pub fn deny(&mut self, pattern: &str);
}
```

### NetworkIsolation

网络隔离：

```rust
pub struct NetworkIsolation {
    /// 允许的主机
    allowed_hosts: Vec<HostPattern>,
    /// 允许的端口
    allowed_ports: Vec<u16>,
    /// 禁止的主机
    denied_hosts: Vec<HostPattern>,
    /// 网络模式
    mode: NetworkMode,
}

pub enum NetworkMode {
    /// 完全禁止网络
    Disabled,
    /// 仅允许出站
    OutboundOnly,
    /// 允许指定主机
    Whitelist,
    /// 允许所有（不推荐）
    Full,
}

pub enum HostPattern {
    /// 精确匹配
    Exact(String),
    /// 通配符匹配
    Wildcard(String),
    /// 正则匹配
    Regex(Regex),
}

impl NetworkIsolation {
    /// 检查是否允许连接
    pub fn can_connect(&self, host: &str, port: u16) -> bool;
    
    /// 添加允许主机
    pub fn allow_host(&mut self, pattern: &str);
    
    /// 添加禁止主机
    pub fn deny_host(&mut self, pattern: &str);
    
    /// 添加允许端口
    pub fn allow_port(&mut self, port: u16);
}
```

### SandboxMonitor

监控统计：

```rust
pub struct SandboxMonitor {
    /// 执行次数
    execution_count: AtomicU64,
    /// 成功次数
    success_count: AtomicU64,
    /// 失败次数
    failure_count: AtomicU64,
    /// 超时次数
    timeout_count: AtomicU64,
    /// 内存超限次数
    memory_exceeded_count: AtomicU64,
    /// 总执行时间
    total_execution_time_ms: AtomicU64,
    /// 峰值内存使用
    peak_memory_usage: AtomicUsize,
}

impl SandboxMonitor {
    /// 记录执行
    pub fn record_execution(&self, result: &ExecutionResult);
    
    /// 获取统计信息
    pub fn stats(&self) -> SandboxStats;
}

pub struct SandboxStats {
    /// 执行次数
    pub execution_count: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均执行时间
    pub avg_execution_time_ms: f64,
    /// 峰值内存
    pub peak_memory_usage: usize,
    /// 超时率
    pub timeout_rate: f64,
}
```

## 执行结果

```rust
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 返回值
    pub return_value: Option<Value>,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 内存使用峰值
    pub peak_memory: usize,
    /// 错误信息
    pub error: Option<SandboxError>,
}

pub enum SandboxError {
    /// 内存超限
    MemoryExceeded { limit: usize, used: usize },
    /// 执行超时
    Timeout { limit_ms: u64, actual_ms: u64 },
    /// CPU 时间超限
    CpuTimeExceeded { limit_ms: u64, actual_ms: u64 },
    /// 文件访问被拒绝
    FileAccessDenied { path: String },
    /// 网络访问被拒绝
    NetworkAccessDenied { host: String, port: u16 },
    /// 系统调用被禁止
    SyscallDenied { syscall: String },
    /// 代码执行错误
    ExecutionError { message: String },
}
```

## PHP API

### 创建沙箱

```php
// 创建沙箱配置
$config = new \oyta\Sandbox\Config();
$config->setMaxMemory(64 * 1024 * 1024); // 64MB
$config->setMaxExecutionTime(5000); // 5 秒
$config->setMaxCpuTime(3000); // 3 秒

// 创建沙箱
$sandbox = new \oyta\Sandbox($config);
```

### 执行代码

```php
// 执行代码字符串
$result = $sandbox->execute('<?php return 1 + 1;');

if ($result->isSuccess()) {
    echo "Result: " . $result->getReturnValue() . "\n";
} else {
    echo "Error: " . $result->getError() . "\n";
}

// 执行文件
$result = $sandbox->executeFile('/path/to/script.php');
```

### 配置隔离

```php
// 文件系统隔离
$sandbox->allowRead('/data/input/*');
$sandbox->allowWrite('/data/output/*');
$sandbox->deny('/etc/*');

// 网络隔离
$sandbox->allowHost('api.example.com');
$sandbox->allowPort(443);
$sandbox->denyHost('internal.company.com');
```

### 获取统计

```php
// 获取沙箱统计
$stats = $sandbox->getStats();
echo "Executions: " . $stats['execution_count'] . "\n";
echo "Success rate: " . ($stats['success_rate'] * 100) . "%\n";
echo "Avg time: " . $stats['avg_execution_time_ms'] . " ms\n";
```

## 使用场景

### 用户代码执行

```php
// 在线代码执行平台
$sandbox = new \oyta\Sandbox([
    'max_memory' => 32 * 1024 * 1024, // 32MB
    'max_execution_time' => 10000, // 10 秒
]);

$result = $sandbox->execute($userCode);
```

### 插件系统

```php
// 安全执行插件代码
$sandbox = new \oyta\Sandbox([
    'max_memory' => 128 * 1024 * 1024, // 128MB
    'max_execution_time' => 30000, // 30 秒
]);

$sandbox->allowRead('/app/plugins/*');
$sandbox->allowWrite('/app/data/*');

$result = $sandbox->executeFile($pluginFile);
```

### 函数即服务 (FaaS)

```php
// 执行无服务器函数
$sandbox = new \oyta\Sandbox([
    'max_memory' => 256 * 1024 * 1024, // 256MB
    'max_execution_time' => 60000, // 60 秒
]);

$sandbox->allowHost('database.internal');
$sandbox->allowPort(5432);

$result = $sandbox->execute($functionCode, $event);
```

## 安全建议

1. **最小权限原则**：只授予必要的权限
2. **资源限制**：始终设置内存和时间限制
3. **网络隔离**：禁止不必要的网络访问
4. **文件隔离**：限制文件系统访问范围
5. **监控告警**：监控异常执行行为
