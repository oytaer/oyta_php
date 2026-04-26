//! 信息命令模块
//!
//! 包含版本信息和命令列表相关命令

use anyhow::Result;

/// 处理 version 命令
///
/// 显示 OYTAPHP 版本信息
pub async fn handle_version() -> Result<()> {
    println!("OYTAPHP v{}", env!("CARGO_PKG_VERSION"));
    println!("高性能 PHP 运行时框架");
    println!();
    // CARGO_PKG_RUST_VERSION 在编译时确定，直接使用
    // 如果未设置则显示 unknown
    let rust_version = option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown");
    println!("Rust 版本: {}", rust_version);
    println!("项目主页: https://github.com/oyta-php/oyta");

    Ok(())
}

/// 处理 list 命令
///
/// 列出所有可用的命令
pub async fn handle_list() -> Result<()> {
    println!("OYTAPHP 命令列表 v{}", env!("CARGO_PKG_VERSION"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("服务器命令:");
    println!("  run              启动 HTTP 服务器");
    println!("  fastcgi          启动 FastCGI 服务器");
    println!();
    println!("构建命令:");
    println!("  build <name>     创建应用目录结构");
    println!();
    println!("生成命令 (make:*):");
    println!("  make:controller <name> [--plain] [--api]  创建控制器");
    println!("  make:model <name>                        创建模型");
    println!("  make:middleware <name>                   创建中间件");
    println!("  make:validate <name>                     创建验证器");
    println!("  make:event <name>                        创建事件类");
    println!("  make:listener <name>                     创建事件监听器");
    println!("  make:subscribe <name>                    创建事件订阅者");
    println!("  make:service <name>                      创建服务类");
    println!("  make:command <name>                      创建自定义命令");
    println!();
    println!("路由命令:");
    println!("  route:list       查看路由列表");
    println!("  route:cache      缓存路由配置");
    println!();
    println!("优化命令:");
    println!("  optimize         执行所有优化");
    println!("  optimize:route   生成路由缓存");
    println!("  optimize:schema  生成数据表字段缓存");
    println!("  clear            清除运行时缓存");
    println!("  config:cache     缓存配置文件");
    println!();
    println!("Composer 命令:");
    println!("  composer:install [--no-dev] [--optimize]  安装依赖");
    println!("  composer:update [packages] [--no-dev]     更新依赖");
    println!("  composer:require <packages> [--dev]       添加依赖");
    println!("  composer:remove <packages>                移除依赖");
    println!("  composer:dump-autoload [--optimize]       重新生成自动加载");
    println!("  composer:show [package] [--direct]        查看已安装的包");
    println!("  composer:outdated                         检查过时的包");
    println!("  composer:validate                         验证 composer.json");
    println!("  composer:config [args] [--list]           配置管理");
    println!("  composer:clear-cache                      清除缓存");
    println!("  composer:diagnose                         环境诊断");
    println!();
    println!("服务命令:");
    println!("  service:discover        自动注册扩展包服务");
    println!("  vendor:publish [package] [--force]  发布扩展资源");
    println!();
    println!("其他命令:");
    println!("  version          显示版本信息");
    println!("  list             显示命令列表");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
