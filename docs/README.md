# OYTAPHP 项目文档

## 项目简介

OYTAPHP 是一个用 Rust 重写的 ThinkPHP 8.0 运行时，提供**同等能力，更简更高效**的 PHP 开发体验。

### 核心特性

- **100% 兼容**：用户项目目录结构与 ThinkPHP 8.0 完全一致
- **零依赖**：无需安装 PHP、Nginx、Composer，单二进制运行
- **高性能**：比 PHP-FPM 响应快 10 倍+，内存低 5 倍+
- **热重载**：修改 PHP 代码即时生效，无需重启
- **内嵌服务**：WebSocket、定时器、队列等服务内置支持

## 文档目录

### 架构文档

| 文档 | 说明 |
|------|------|
| [系统架构](./architecture/overview.md) | 整体系统架构设计 |
| [目录结构](./architecture/directory.md) | 项目目录结构说明 |
| [技术栈](./architecture/tech-stack.md) | 依赖库和技术选型 |

### 模块文档

| 文档 | 说明 |
|------|------|
| [解释器模块](./modules/interpreter.md) | PHP 解释器实现 |
| [定时器模块](./modules/timer.md) | 多级时间轮定时器 |
| [监控模块](./modules/monitor.md) | 实时性能监控 |
| [缓存模块](./modules/cache.md) | 多级缓存系统 |
| [协程模块](./modules/coroutine.md) | 原生协程调度器 |
| [搜索模块](./modules/search.md) | 全文搜索引擎 |
| [SIMD 模块](./modules/simd.md) | SIMD 向量化加速 |
| [序列化模块](./modules/serialize.md) | 零拷贝序列化引擎 |
| [沙箱模块](./modules/sandbox.md) | 内存安全沙箱隔离 |
| [微服务模块](./modules/microservice.md) | 内置微服务框架 |
| [HTTP/3 模块](./modules/http3.md) | HTTP/3 和 QUIC 支持 |

### API 文档

| 文档 | 说明 |
|------|------|
| [Facade 门面](./api/facade.md) | Facade 门面系统 API |
| [助手函数](./api/helpers.md) | 内置助手函数 |

### 使用指南

| 文档 | 说明 |
|------|------|
| [快速开始](./guides/quick-start.md) | 快速入门指南 |
| [命令行工具](./guides/cli.md) | CLI 命令使用说明 |
| [配置说明](./guides/configuration.md) | 配置系统详解 |

## 版本信息

- **当前版本**：0.1.0
- **Rust 版本**：2024 Edition
- **兼容 PHP 版本**：8.0+

## 许可证

MIT License
