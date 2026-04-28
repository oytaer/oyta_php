# 数据库模块 (Database)

## 模块概述

数据库模块提供 OYTAPHP 的数据库操作功能，对应 ThinkPHP 8.0 的 Db 门面。支持多种数据库驱动，提供查询构建器和原生 SQL 查询能力。

## 模块结构

```
database/
├── mod.rs          # 模块入口
├── facade.rs       # Db 门面
├── connection.rs   # 数据库连接管理
├── executor.rs     # SQL 执行器
├── query_builder.rs # 查询构建器
├── transaction.rs  # 事务管理
└── model/          # ORM 模型
```

## 支持的数据库驱动

| 数据库 | 驱动 | 说明 |
|--------|------|------|
| MySQL | sqlx + SeaORM | MySQL 5.6+ |
| PostgreSQL | sqlx + SeaORM | PostgreSQL 12+ |
| SQLite | sqlx + SeaORM | SQLite 3 |

## Db 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `init` | config: &ConfigStore | - | 初始化数据库连接 |
| `query` | sql: &str | Result<QueryResult> | 执行查询 SQL |
| `query_with_params` | sql: &str, params: &[Value] | Result<QueryResult> | 执行参数化查询 |
| `execute` | sql: &str | Result<u64> | 执行更新 SQL |
| `execute_with_params` | sql: &str, params: &[Value] | Result<u64> | 执行参数化更新 |
| `insert_get_id` | sql: &str | Result<i64> | 插入并返回自增 ID |
| `find` | sql: &str | Result<Option<HashMap>> | 查询单行 |
| `value` | sql: &str | Result<Value> | 查询单个值 |
| `table` | table: &str | QueryBuilder | 创建查询构建器 |
| `name` | table: &str | QueryBuilder | 创建查询构建器（不添加前缀） |
| `close` | - | - | 关闭所有连接 |

## 使用示例

### 原生 SQL 查询

```php
<?php
use oyta\facade\Db;

// 查询所有数据
$users = Db::query("SELECT * FROM users");

// 参数化查询
$user = Db::query("SELECT * FROM users WHERE id = ?", [1]);

// 执行更新
$affected = Db::execute("UPDATE users SET name = ? WHERE id = ?", ['张三', 1]);

// 插入并获取 ID
$id = Db::insertGetId("INSERT INTO users (name) VALUES (?)", ['李四']);

// 查询单行
$user = Db::find("SELECT * FROM users WHERE id = 1");

// 查询单个值
$count = Db::value("SELECT COUNT(*) FROM users");
```

### 查询构建器

```php
<?php
use oyta\facade\Db;

// 基础查询
$users = Db::table('users')
    ->where('status', 1)
    ->order('id', 'desc')
    ->limit(10)
    ->select();

// 条件查询
$users = Db::table('users')
    ->where('age', '>', 18)
    ->where('status', 'in', [1, 2, 3])
    ->select();

// 关联查询
$list = Db::table('orders')
    ->alias('o')
    ->join('users u', 'o.user_id = u.id')
    ->field('o.*, u.name')
    ->select();

// 聚合查询
$count = Db::table('users')->count();
$sum = Db::table('orders')->sum('amount');
$avg = Db::table('products')->avg('price');

// 插入数据
Db::table('users')->insert([
    'name' => '张三',
    'email' => 'zhangsan@example.com',
]);

// 批量插入
Db::table('users')->insertAll([
    ['name' => '张三', 'email' => 'zhangsan@example.com'],
    ['name' => '李四', 'email' => 'lisi@example.com'],
]);

// 更新数据
Db::table('users')
    ->where('id', 1)
    ->update(['name' => '王五']);

// 删除数据
Db::table('users')
    ->where('id', 1)
    ->delete();
```

### 事务操作

```php
<?php
use oyta\facade\Db;

Db::transaction(function() {
    Db::table('orders')->insert(['user_id' => 1, 'amount' => 100]);
    Db::table('users')->where('id', 1)->dec('balance', 100)->update();
});
```

## 配置示例

```php
<?php
// config/database.php
return [
    'default' => 'mysql',
    'connections' => [
        'mysql' => [
            'driver' => 'mysql',
            'host' => '127.0.0.1',
            'port' => 3306,
            'database' => 'test',
            'username' => 'root',
            'password' => '',
            'charset' => 'utf8mb4',
            'collation' => 'utf8mb4_unicode_ci',
            'prefix' => '',
        ],
        'pgsql' => [
            'driver' => 'pgsql',
            'host' => '127.0.0.1',
            'port' => 5432,
            'database' => 'test',
            'username' => 'postgres',
            'password' => '',
            'charset' => 'utf8',
            'prefix' => '',
        ],
        'sqlite' => [
            'driver' => 'sqlite',
            'database' => runtime_path('database.db'),
        ],
    ],
];
```

## 技术实现

- 使用 sqlx 进行异步数据库操作
- 使用 SeaORM 作为 ORM 层
- 支持连接池管理
- 自动参数绑定防止 SQL 注入
