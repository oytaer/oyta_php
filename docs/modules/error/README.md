# 错误处理模块 (Error)

## 模块概述

错误处理模块提供完整的 PHP 错误处理功能，包括错误处理器、异常处理器、关闭函数、错误级别等。

## 模块结构

```
error/
├── mod.rs          # 模块入口
├── handler.rs      # 错误处理器
├── level.rs        # 错误级别
├── reporter.rs     # 错误报告器
├── shutdown.rs     # 关闭函数
└── types.rs        # 错误类型
```

## 主要类型

| 类型 | 说明 |
|------|------|
| ErrorHandler | 错误处理器 |
| ErrorLevel | 错误级别 |
| ErrorReporter | 错误报告器 |
| ShutdownManager | 关闭函数管理 |
| ErrorInfo | 错误信息 |
| ExceptionInfo | 异常信息 |

## 错误级别

| 级别 | 说明 |
|------|------|
| E_ERROR | 致命错误 |
| E_WARNING | 警告 |
| E_PARSE | 解析错误 |
| E_NOTICE | 通知 |
| E_DEPRECATED | 弃用警告 |

## 使用示例

### 错误处理

```php
<?php
// 设置错误处理器
set_error_handler(function($errno, $errstr, $errfile, $errline) {
    throw new ErrorException($errstr, 0, $errno, $errfile, $errline);
});

// 设置异常处理器
set_exception_handler(function($exception) {
    error_log($exception->getMessage());
});
```

### 异常捕获

```php
<?php
try {
    // 可能抛出异常的代码
} catch (Exception $e) {
    echo $e->getMessage();
} finally {
    // 清理代码
}
```

## 技术实现

- 错误处理器
- 异常处理器
- 关闭函数
- 错误级别定义
