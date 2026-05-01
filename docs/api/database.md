# Database 数据库模块

## 模块概述

Database 模块提供完整的数据库操作能力，包括连接池管理、查询构建器、ORM 模型、数据库迁移、读写分离、集群管理等。对应 ThinkPHP 8.0 的数据库层。

## 模块结构

```
database/
├── mod.rs              # 模块入口
├── connection.rs       # 连接管理
├── pool_trait.rs       # 连接池接口
├── postgres_pool.rs    # PostgreSQL 连接池
├── sqlite_pool.rs      # SQLite 连接池
├── executor.rs         # 执行器
├── query_builder.rs    # 查询构建器
├── facade.rs           # Db 门面
├── cluster.rs          # 数据库集群
├── read_write_split.rs # 读写分离
├── relations.rs        # 关系映射
│
├── model/              # 模型系统
│   ├── entity.rs       # 实体定义
│   ├── instance.rs     # 模型实例
│   ├── collection.rs   # 模型集合
│   ├── static_methods.rs # 静态方法
│   ├── preload.rs      # 预加载
│   ├── relation_ops.rs # 关系操作
│   ├── soft_delete_ext.rs # 软删除
│   ├── scope.rs        # 查询作用域
│   ├── searcher.rs     # 搜索器
│   ├── mutator.rs      # 突变器
│   ├── accessor.rs     # 访问器
│   ├── field_map.rs    # 字段映射
│   ├── query_ext.rs    # 查询扩展
│   ├── events.rs       # 模型事件
│   ├── paginate.rs     # 分页
│   ├── lazy_write.rs   # 延迟写入
│   ├── virtual_model.rs # 虚拟模型
│   ├── relation_stats.rs # 关系统计
│   ├── output.rs       # 输出控制
│   ├── types.rs        # 类型定义
│   └── helpers.rs      # 辅助函数
│
├── query/              # 查询构建
│   ├── field.rs        # 字段查询
│   ├── where_ext.rs    # WHERE 扩展
│   ├── order_ext.rs    # ORDER 扩展
│   ├── aggregate.rs    # 聚合查询
│   ├── paging.rs       # 分页查询
│   ├── lock.rs         # 查询锁
│   ├── join_ext.rs     # JOIN 扩展
│   ├── having.rs       # HAVING 子句
│   ├── subquery.rs     # 子查询
│   └── union_query.rs  # UNION 查询
│
├── crud/               # CRUD 操作
│   ├── select.rs       # 查询
│   ├── insert.rs       # 插入
│   ├── update.rs       # 更新
│   └── delete.rs       # 删除
│
├── orm/                # ORM 功能
│   ├── timestamp.rs    # 时间戳
│   ├── soft_delete.rs  # 软删除
│   ├── json_field.rs   # JSON 字段
│   └── events.rs       # ORM 事件
│
├── migration/          # 数据库迁移
│   ├── manager.rs      # 迁移管理
│   ├── schema.rs       # 模式定义
│   ├── types.rs        # 迁移类型
│   └── helpers.rs      # 迁移辅助
│
└── advanced/           # 高级功能
    ├── cache.rs        # 查询缓存
    ├── logger.rs       # 查询日志
    ├── listener.rs     # 查询监听
    └── reconnect.rs    # 断线重连
```

## 支持的数据库

- **MySQL/MariaDB**: 使用 mysql crate
- **PostgreSQL**: 使用 postgres crate
- **SQLite**: 使用 rusqlite crate

## 连接池接口 (PoolTrait)

```rust
pub trait PoolTrait: Send + Sync {
    async fn get_connection(&self) -> Result<Box<dyn ConnectionTrait>>;
    async fn release_connection(&self, conn: Box<dyn ConnectionTrait>);
    fn pool_size(&self) -> (u32, u32); // (active, idle)
    fn health_check(&self) -> bool;
}
```

## 查询构建器

### 基本查询

```php
<?php
// 查询所有
$users = Db::table('users')->select();

// 条件查询
$users = Db::table('users')
    ->where('status', 1)
    ->where('age', '>', 18)
    ->whereIn('id', [1, 2, 3])
    ->whereLike('name', '%John%')
    ->whereBetween('created_at', ['2024-01-01', '2024-12-31'])
    ->whereNull('deleted_at')
    ->select();

// 指定字段
$users = Db::table('users')
    ->field('id, name, email')
    ->field('COUNT(*) as count', true) // 排除
    ->select();

// 排序和分页
$users = Db::table('users')
    ->order('created_at', 'desc')
    ->order('id', 'asc')
    ->limit(10)
    ->page(1)
    ->select();
```

### JOIN 查询

```php
<?php
// INNER JOIN
$users = Db::table('users')
    ->join('profiles', 'users.id', '=', 'profiles.user_id')
    ->select();

// LEFT JOIN
$users = Db::table('users')
    ->leftJoin('profiles', 'users.id', '=', 'profiles.user_id')
    ->select();

// RIGHT JOIN
$users = Db::table('users')
    ->rightJoin('profiles', 'users.id', '=', 'profiles.user_id')
    ->select();

// 复杂 JOIN 条件
$users = Db::table('users')
    ->join('profiles', function($join) {
        $join->on('users.id', '=', 'profiles.user_id')
             ->where('profiles.status', 1);
    })
    ->select();
```

### 聚合查询

```php
<?php
// 计数
$count = Db::table('users')->count();
$count = Db::table('users')->count('DISTINCT email');

// 求和
$total = Db::table('orders')->sum('amount');

// 平均值
$avg = Db::table('products')->avg('price');

// 最大值/最小值
$max = Db::table('products')->max('price');
$min = Db::table('products')->min('price');
```

### 子查询

```php
<?php
// 子查询 WHERE
$users = Db::table('users')
    ->whereExists(function($query) {
        $query->table('orders')
              ->whereRaw('orders.user_id = users.id')
              ->where('amount', '>', 100);
    })
    ->select();

// 子查询 JOIN
$users = Db::table('users')
    ->joinSub(function($query) {
        $query->table('orders')
              ->field('user_id, SUM(amount) as total')
              ->group('user_id');
    }, 'order_totals', 'users.id', '=', 'order_totals.user_id')
    ->select();

// UNION
$result = Db::table('users')
    ->union(function($query) {
        $query->table('admins')->field('id, name, email');
    })
    ->field('id, name, email')
    ->select();
```

### 插入数据

```php
<?php
// 插入单条
$id = Db::table('users')->insert([
    'name' => 'John',
    'email' => 'john@example.com',
    'created_at' => date('Y-m-d H:i:s'),
]);

// 批量插入
Db::table('users')->insertAll([
    ['name' => 'John', 'email' => 'john@example.com'],
    ['name' => 'Jane', 'email' => 'jane@example.com'],
]);

// 插入并获取 ID
$id = Db::table('users')->insertGetId($data);

// 插入忽略
Db::table('users')->insertIgnore($data);

// 冲突时更新
Db::table('users')
    ->onDuplicateKeyUpdate(['name' => 'New Name'])
    ->insert($data);
```

### 更新数据

```php
<?php
// 更新
$affected = Db::table('users')
    ->where('id', 1)
    ->update(['name' => 'New Name']);

// 自增
Db::table('users')
    ->where('id', 1)
    ->inc('score', 10)
    ->update();

// 自减
Db::table('users')
    ->where('id', 1)
    ->dec('score', 5)
    ->update();

// JSON 字段更新
Db::table('users')
    ->where('id', 1)
    ->update(['settings->theme' => 'dark']);

// 条件更新
Db::table('users')
    ->where('id', 1)
    ->update([
        'login_count' => Db::raw('login_count + 1'),
        'last_login' => Db::raw('NOW()'),
    ]);
```

### 删除数据

```php
<?php
// 删除
$affected = Db::table('users')
    ->where('id', 1)
    ->delete();

// 软删除（需要模型支持）
User::destroy(1);
User::destroy([1, 2, 3]);

// 强制删除
User::withTrashed()->where('id', 1)->forceDelete();
```

## 模型 (Model)

### 定义模型

```php
<?php
namespace app\model;

class User extends Model
{
    // 表名
    protected $name = 'user';
    
    // 主键
    protected $pk = 'id';
    
    // 自动时间戳
    protected $autoWriteTimestamp = true;
    
    // 时间戳字段
    protected $createTime = 'created_at';
    protected $updateTime = 'updated_at';
    
    // 软删除
    use SoftDelete;
    protected $deleteTime = 'deleted_at';
    protected $defaultSoftDelete = 0;
    
    // 类型转换
    protected $type = [
        'status' => 'integer',
        'config' => 'json',
        'is_admin' => 'boolean',
    ];
    
    // 隐藏字段
    protected $hidden = ['password', 'token'];
    
    // 只读字段
    protected $readonly = ['id', 'created_at'];
    
    // JSON 字段
    protected $json = ['settings', 'preferences'];
    protected $jsonAssoc = true;
}
```

### 模型操作

```php
<?php
// 查询所有
$users = User::select();
$users = User::all();

// 查询单条
$user = User::find(1);
$user = User::findOrFail(1); // 不存在抛出异常
$user = User::where('email', 'john@example.com')->find();
$user = User::first();
$user = User::firstOrFail();

// 创建
$user = new User();
$user->name = 'John';
$user->email = 'john@example.com';
$user->save();

// 或使用静态方法
$user = User::create([
    'name' => 'John',
    'email' => 'john@example.com',
]);

// 更新
$user = User::find(1);
$user->name = 'New Name';
$user->save();

// 批量更新
User::where('status', 0)->update(['status' => 1]);

// 删除
User::destroy(1);
$user = User::find(1);
$user->delete();

// 软删除恢复
User::withTrashed()->find(1)->restore();
```

### 查询作用域

```php
<?php
class User extends Model
{
    // 定义作用域
    public function scopeActive($query)
    {
        return $query->where('status', 1);
    }
    
    public function scopeAdmin($query)
    {
        return $query->where('is_admin', 1);
    }
    
    public function scopeRecent($query, $days = 7)
    {
        return $query->where('created_at', '>=', date('Y-m-d', strtotime("-{$days} days")));
    }
}

// 使用作用域
$users = User::active()->admin()->recent(30)->select();
```

### 访问器和突变器

```php
<?php
class User extends Model
{
    // 访问器
    public function getNameAttr($value)
    {
        return ucfirst($value);
    }
    
    public function getFullNameAttr($value, $data)
    {
        return $data['first_name'] . ' ' . $data['last_name'];
    }
    
    // 突变器
    public function setPasswordAttr($value)
    {
        return password_hash($value, PASSWORD_DEFAULT);
    }
    
    public function setEmailAttr($value)
    {
        return strtolower($value);
    }
}

// 使用
$user = User::find(1);
echo $user->name; // 自动调用 getNameAttr
echo $user->full_name; // 虚拟属性

$user->password = '123456'; // 自动加密
$user->email = 'JOHN@EXAMPLE.COM'; // 自动转小写
```

### 关联关系

```php
<?php
class User extends Model
{
    // 一对多
    public function posts()
    {
        return $this->hasMany(Post::class, 'user_id', 'id');
    }
    
    // 一对一
    public function profile()
    {
        return $this->hasOne(Profile::class, 'user_id', 'id');
    }
    
    // 属于
    public function role()
    {
        return $this->belongsTo(Role::class, 'role_id', 'id');
    }
    
    // 多对多
    public function tags()
    {
        return $this->belongsToMany(Tag::class, 'user_tags', 'user_id', 'tag_id');
    }
    
    // 远程一对多
    public function comments()
    {
        return $this->hasManyThrough(Comment::class, Post::class);
    }
    
    // 多态关联
    public function images()
    {
        return $this->morphMany(Image::class, 'imageable');
    }
}

// 使用关联
$user = User::with(['posts', 'profile'])->find(1);
foreach ($user->posts as $post) {
    echo $post->title;
}

// 预加载约束
$users = User::with(['posts' => function($query) {
    $query->where('status', 1)->field('id,title');
}])->select();

// 延迟加载
$users = User::select();
$users->load('posts');

// 关联统计
$users = User::withCount('posts')->select();
foreach ($users as $user) {
    echo $user->posts_count;
}

// 关联是否存在
$users = User::has('posts')->select();
$users = User::has('posts', '>=', 3)->select();
$users = User::whereHas('posts', function($query) {
    $query->where('status', 1);
})->select();
```

### 模型事件

```php
<?php
class User extends Model
{
    protected static function booted()
    {
        // 创建前
        static::creating(function($user) {
            $user->uuid = Str::uuid();
        });
        
        // 创建后
        static::created(function($user) {
            // 发送欢迎邮件
        });
        
        // 更新前
        static::updating(function($user) {
            $user->updated_by = auth()->id();
        });
        
        // 删除前
        static::deleting(function($user) {
            $user->posts()->delete();
        });
    }
}
```

## 读写分离

```php
<?php
// 配置
'connections' => [
    'mysql' => [
        'type' => 'mysql',
        'write' => [
            'host' => '192.168.1.100',
        ],
        'read' => [
            'host' => ['192.168.1.101', '192.168.1.102'],
        ],
    ],
],

// 使用
Db::name('user')->select(); // 自动走从库
Db::name('user')->insert($data); // 自动走主库
Db::name('user')->master()->find(1); // 强制走主库
```

## 数据库迁移

```php
<?php
// 创建迁移
php oyta migrate:create create_users_table

// 迁移文件示例
class CreateUsersTable
{
    public function up()
    {
        Schema::create('users', function($table) {
            $table->id();
            $table->string('name', 100);
            $table->string('email', 100)->unique();
            $table->string('password', 255);
            $table->tinyInteger('status')->default(1);
            $table->timestamp('created_at')->nullable();
            $table->timestamp('updated_at')->nullable();
            $table->softDeletes();
            
            $table->index(['status', 'created_at']);
        });
    }
    
    public function down()
    {
        Schema::drop('users');
    }
}

// 执行迁移
php oyta migrate:run

// 回滚迁移
php oyta migrate:rollback

// 重置迁移
php oyta migrate:reset

// 刷新迁移
php oyta migrate:refresh

// 查看状态
php oyta migrate:status
```

## 事务处理

```php
<?php
// 自动事务
Db::transaction(function() {
    $user = User::create(['name' => 'John']);
    Profile::create(['user_id' => $user->id]);
});

// 手动控制
Db::startTrans();
try {
    // 操作
    Db::commit();
} catch (\Exception $e) {
    Db::rollback();
    throw $e;
}

// 事务隔离级别
Db::transaction(function() {
    // ...
}, \PDO::ATTR_TRANSACTION_ISOLATION, \PDO::ISOLATION_SERIALIZABLE);
```

## 查询缓存

```php
<?php
// 开启查询缓存
$users = Db::table('users')
    ->cache(true, 3600)
    ->select();

// 指定缓存键
$users = Db::table('users')
    ->cache('user_list', 3600)
    ->select();

// 清除缓存
Db::clearCache('user_list');
```

## 配置示例

```php
<?php
// config/database.php
return [
    'default' => 'mysql',
    
    'connections' => [
        'mysql' => [
            'type' => 'mysql',
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 3306),
            'database' => env('DB_NAME', 'oytaphp'),
            'username' => env('DB_USER', 'root'),
            'password' => env('DB_PASS', ''),
            'charset' => 'utf8mb4',
            'collation' => 'utf8mb4_unicode_ci',
            'prefix' => '',
            'pool' => [
                'max_connections' => 100,
                'min_connections' => 10,
                'connect_timeout' => 5,
                'idle_timeout' => 300,
            ],
        ],
        
        'pgsql' => [
            'type' => 'pgsql',
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 5432),
            'database' => env('DB_NAME', 'oytaphp'),
            'username' => env('DB_USER', 'postgres'),
            'password' => env('DB_PASS', ''),
        ],
        
        'sqlite' => [
            'type' => 'sqlite',
            'database' => runtime_path('database.db'),
        ],
    ],
    
    // 读写分离
    'read_write_split' => [
        'enabled' => env('DB_RW_SPLIT', false),
        'strategy' => 'random', // random, round_robin, weight
    ],
    
    // 查询缓存
    'cache' => [
        'enabled' => env('DB_CACHE', false),
        'ttl' => 3600,
    ],
    
    // 日志
    'log' => [
        'enabled' => env('DB_LOG', env('APP_DEBUG', false)),
        'slow_query' => 1000, // 慢查询阈值（毫秒）
    ],
];
```
