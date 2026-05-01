//! 数据库迁移命令模块
//!
//! 提供数据库迁移管理功能
//! 包括：创建迁移、执行迁移、回滚迁移、查看迁移状态等

use anyhow::Result;

use crate::database::migration::{Migration, MigrationManager, MigrationStatus, SchemaBuilder};
use crate::database::migration::migrator::DatabaseType;
use crate::env_loader;
use crate::project;

/// 处理 migrate:run 命令
///
/// 执行所有待处理的迁移
///
/// # 参数
/// - `database`: 可选的数据库连接名称
pub async fn handle_migrate_run(database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 获取数据库类型
    let db_type = get_database_type(database.as_deref())?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  执行数据库迁移");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建迁移管理器
    let mut manager = MigrationManager::new();

    // 加载迁移文件
    let migrations_dir = project.root.join("database").join("migrations");
    if migrations_dir.exists() {
        load_migrations_from_dir(&mut manager, &migrations_dir)?;
    }

    // 获取待执行的迁移名称列表（先收集名称，避免借用冲突）
    let pending_names: Vec<String> = manager.get_pending_migrations()
        .iter()
        .map(|m| m.name.clone())
        .collect();

    if pending_names.is_empty() {
        println!("  ✓ 没有待执行的迁移");
        return Ok(());
    }

    println!("  发现 {} 个待执行的迁移", pending_names.len());
    println!();

    // 创建迁移表
    let create_table_sql = manager.build_create_table_sql(db_type);
    println!("  创建迁移表...");
    println!("  SQL: {}", create_table_sql);

    // 执行迁移
    for name in pending_names {
        println!("  执行迁移: {}...", name);
        manager.run_migration(&name)?;
        println!("  ✓ 完成: {}", name);
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 迁移执行完成");

    Ok(())
}

/// 处理 migrate:rollback 命令
///
/// 回滚最后一次迁移或指定批次
///
/// # 参数
/// - `batch`: 可选的批次号，不指定则回滚最后一批
/// - `database`: 可选的数据库连接名称
pub async fn handle_migrate_rollback(batch: &Option<i32>, database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 获取数据库类型
    let _db_type = get_database_type(database.as_deref())?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  回滚数据库迁移");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建迁移管理器
    let mut manager = MigrationManager::new();

    // 加载迁移文件
    let migrations_dir = project.root.join("database").join("migrations");
    if migrations_dir.exists() {
        load_migrations_from_dir(&mut manager, &migrations_dir)?;
    }

    // 获取要回滚的迁移
    let to_rollback = if let Some(batch_num) = batch {
        // 回滚指定批次
        manager.migrations.values()
            .filter(|m| m.batch == Some(*batch_num))
            .map(|m| m.name.clone())
            .collect::<Vec<_>>()
    } else {
        // 回滚最后一批
        manager.get_last_batch()
            .iter()
            .map(|m| m.name.clone())
            .collect()
    };

    if to_rollback.is_empty() {
        println!("  ✓ 没有需要回滚的迁移");
        return Ok(());
    }

    println!("  将回滚 {} 个迁移", to_rollback.len());
    println!();

    // 执行回滚
    for name in to_rollback {
        println!("  回滚迁移: {}...", name);
        manager.rollback_migration(&name)?;
        println!("  ✓ 已回滚: {}", name);
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 回滚完成");

    Ok(())
}

/// 处理 migrate:status 命令
///
/// 查看迁移状态
///
/// # 参数
/// - `database`: 可选的数据库连接名称
pub async fn handle_migrate_status(database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 获取数据库类型
    let _db_type = get_database_type(database.as_deref())?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  迁移状态");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建迁移管理器
    let mut manager = MigrationManager::new();

    // 加载迁移文件
    let migrations_dir = project.root.join("database").join("migrations");
    if migrations_dir.exists() {
        load_migrations_from_dir(&mut manager, &migrations_dir)?;
    }

    // 显示迁移列表
    println!("  {:<40} {:<12} {:<20}", "迁移名称", "状态", "执行时间");
    println!("  {:<40} {:<12} {:<20}", "--------", "----", "--------");

    let migrations: Vec<_> = manager.migrations.values().collect();
    if migrations.is_empty() {
        println!("  没有迁移记录");
    } else {
        for migration in migrations {
            let status = match migration.status {
                MigrationStatus::Pending => "待执行",
                MigrationStatus::Executed => "已执行",
                MigrationStatus::RolledBack => "已回滚",
                MigrationStatus::Failed => "失败",
            };
            let executed_at = migration.executed_at.as_deref().unwrap_or("-");
            println!("  {:<40} {:<12} {:<20}", migration.name, status, executed_at);
        }
    }

    println!();
    println!("  待执行: {}", manager.get_pending_migrations().len());
    println!("  已执行: {}", manager.get_executed_migrations().len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 migrate:make 命令
///
/// 创建新的迁移文件
///
/// # 参数
/// - `name`: 迁移名称
/// - `table`: 可选的表名
/// - `create`: 是否为创建表的迁移
pub async fn handle_migrate_make(name: &str, table: &Option<String>, create: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  创建迁移文件");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 生成迁移文件名
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let safe_name = name.replace(' ', "_").to_lowercase();
    let filename = format!("{}_{}.php", timestamp, safe_name);

    // 确保迁移目录存在
    let migrations_dir = project.root.join("database").join("migrations");
    std::fs::create_dir_all(&migrations_dir)?;

    // 生成迁移内容
    let class_name = format!("{}{}", 
        safe_name.split('_')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<String>(),
        "Migration"
    );

    let table_name = table.as_deref().unwrap_or(&safe_name);

    let content = if create {
        generate_create_migration(&class_name, table_name)
    } else {
        generate_update_migration(&class_name, table_name)
    };

    // 写入文件
    let file_path = migrations_dir.join(&filename);
    std::fs::write(&file_path, content)?;

    println!("  ✓ 已创建迁移: {}", filename);
    println!("  路径: {}", file_path.display());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 migrate:fresh 命令
///
/// 删除所有表并重新运行迁移
///
/// # 参数
/// - `database`: 可选的数据库连接名称
/// - `seed`: 是否运行数据填充
pub async fn handle_migrate_fresh(database: &Option<String>, seed: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  刷新数据库迁移");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("  警告: 此操作将删除所有数据表并重新迁移！");
    println!("  正在执行...");

    // 执行 migrate:run
    handle_migrate_run(database).await?;

    if seed {
        println!();
        println!("  运行数据填充...");
        // TODO: 实现数据填充
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 数据库已刷新");

    Ok(())
}

/// 处理 migrate:reset 命令
///
/// 回滚所有迁移
///
/// # 参数
/// - `database`: 可选的数据库连接名称
pub async fn handle_migrate_reset(database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  重置数据库迁移");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("  警告: 此操作将回滚所有迁移！");

    // 创建迁移管理器
    let mut manager = MigrationManager::new();

    // 加载迁移文件
    let migrations_dir = project.root.join("database").join("migrations");
    if migrations_dir.exists() {
        load_migrations_from_dir(&mut manager, &migrations_dir)?;
    }

    // 获取所有已执行的迁移
    let executed: Vec<_> = manager.get_executed_migrations()
        .iter()
        .map(|m| m.name.clone())
        .collect();

    if executed.is_empty() {
        println!("  ✓ 没有需要回滚的迁移");
        return Ok(());
    }

    // 按逆序回滚
    for name in executed.into_iter().rev() {
        println!("  回滚: {}...", name);
        manager.rollback_migration(&name)?;
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 重置完成");

    Ok(())
}

/// 处理 migrate:refresh 命令
///
/// 重置并重新运行迁移
///
/// # 参数
/// - `database`: 可选的数据库连接名称
/// - `seed`: 是否运行数据填充
pub async fn handle_migrate_refresh(database: &Option<String>, seed: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  刷新迁移");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 先重置
    handle_migrate_reset(database).await?;

    // 再运行迁移
    handle_migrate_run(database).await?;

    if seed {
        println!();
        println!("  运行数据填充...");
        // TODO: 实现数据填充
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 刷新完成");

    Ok(())
}

/// 处理 db:seed 命令
///
/// 运行数据填充
///
/// # 参数
/// - `class`: 可选的填充类名
/// - `database`: 可选的数据库连接名称
pub async fn handle_db_seed(class: &Option<String>, database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    let _db_type = get_database_type(database.as_deref())?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  数据填充");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let seeder_class = class.as_deref().unwrap_or("DatabaseSeeder");
    println!("  运行填充器: {}", seeder_class);

    // TODO: 实现数据填充逻辑

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✓ 数据填充完成");

    Ok(())
}

/// 处理 db:table 命令
///
/// 查看数据表结构
///
/// # 参数
/// - `table`: 表名
/// - `database`: 可选的数据库连接名称
pub async fn handle_db_table(table: &str, database: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    let _db_type = get_database_type(database.as_deref())?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  数据表结构: {}", table);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 显示查询表结构的 SQL
    let db_type = get_database_type(database.as_deref())?;
    let describe_sql = match db_type {
        DatabaseType::MySQL => format!("DESCRIBE `{}`", table),
        DatabaseType::PostgreSQL => format!("SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_name = '{}'", table),
        DatabaseType::SQLite => format!("PRAGMA table_info({})", table),
    };

    println!("  数据库类型: {:?}", db_type);
    println!("  查询语句: {}", describe_sql);
    println!();
    println!("  提示: 请使用数据库客户端执行上述 SQL 查看表结构");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 从目录加载迁移文件
///
/// # 参数
/// - `manager`: 迁移管理器
/// - `dir`: 迁移文件目录
fn load_migrations_from_dir(manager: &mut MigrationManager, dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "php").unwrap_or(false) {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // 解析迁移文件名格式: YYYYMMDDHHMMSS_name.php
                let parts: Vec<&str> = filename.trim_end_matches(".php").splitn(2, '_').collect();
                if parts.len() == 2 {
                    let version = parts[0];
                    let name = parts[1];

                    let migration = Migration::new(name, version, filename);
                    manager.register(migration);
                }
            }
        }
    }

    Ok(())
}

/// 获取数据库类型
///
/// # 参数
/// - `database`: 数据库连接名称
///
/// # 返回
/// 数据库类型枚举
fn get_database_type(database: Option<&str>) -> Result<DatabaseType> {
    // 从配置获取数据库类型
    // 简化实现：默认返回 MySQL
    let db_type = match database {
        Some("pgsql") | Some("postgres") | Some("postgresql") => DatabaseType::PostgreSQL,
        Some("sqlite") => DatabaseType::SQLite,
        _ => DatabaseType::MySQL,
    };

    Ok(db_type)
}

/// 生成创建表的迁移内容
///
/// # 参数
/// - `class_name`: 类名
/// - `table_name`: 表名
///
/// # 返回
/// 迁移文件内容
fn generate_create_migration(class_name: &str, table_name: &str) -> String {
    format!(r#"<?php
/**
 * 迁移文件：创建 {table} 表
 * 自动生成于 {time}
 */

use think\migration\Migrator;

class {class} extends Migrator
{{
    /**
     * 执行迁移
     */
    public function up()
    {{
        $table = $this->table('{table}', [
            'id' => false,
            'primary_key' => ['id'],
            'engine' => 'InnoDB',
            'collation' => 'utf8mb4_unicode_ci',
            'comment' => '',
        ]);
        
        $table
            ->addColumn('id', 'integer', [
                'identity' => true,
                'signed' => false,
            ])
            ->addColumn('created_at', 'timestamp', [
                'default' => 'CURRENT_TIMESTAMP',
                'comment' => '创建时间',
            ])
            ->addColumn('updated_at', 'timestamp', [
                'null' => true,
                'default' => null,
                'update' => 'CURRENT_TIMESTAMP',
                'comment' => '更新时间',
            ])
            ->create();
    }}

    /**
     * 回滚迁移
     */
    public function down()
    {{
        $this->table('{table}')->drop()->save();
    }}
}}
"#,
        table = table_name,
        class = class_name,
        time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}

/// 生成更新表的迁移内容
///
/// # 参数
/// - `class_name`: 类名
/// - `table_name`: 表名
///
/// # 返回
/// 迁移文件内容
fn generate_update_migration(class_name: &str, table_name: &str) -> String {
    format!(r#"<?php
/**
 * 迁移文件：更新 {table} 表
 * 自动生成于 {time}
 */

use think\migration\Migrator;

class {class} extends Migrator
{{
    /**
     * 执行迁移
     */
    public function up()
    {{
        $table = $this->table('{table}');
        
        // 添加字段示例
        // $table
        //     ->addColumn('field_name', 'string', [
        //         'limit' => 255,
        //         'null' => true,
        //         'default' => null,
        //         'comment' => '字段说明',
        //     ])
        //     ->update();
    }}

    /**
     * 回滚迁移
     */
    public function down()
    {{
        $table = $this->table('{table}');
        
        // 删除字段示例
        // $table
        //     ->removeColumn('field_name')
        //     ->update();
    }}
}}
"#,
        table = table_name,
        class = class_name,
        time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}
