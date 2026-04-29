//! 代码生成命令模块
//!
//! 提供创建控制器、模型、中间件、验证器等代码生成功能

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 build 命令
///
/// 自动生成应用目录和文件
/// 只创建核心目录，service/event/listener 等按需创建
///
/// # 参数
/// - `name`: 应用名称
pub async fn handle_build(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在构建应用: {}", name);

    // 创建应用目录结构（只创建核心目录）
    let app_dir = project.root.join("app").join(name);
    
    // 创建核心目录
    std::fs::create_dir_all(&app_dir)?;
    std::fs::create_dir_all(app_dir.join("controller"))?;
    std::fs::create_dir_all(app_dir.join("model"))?;
    std::fs::create_dir_all(app_dir.join("middleware"))?;
    std::fs::create_dir_all(app_dir.join("validate"))?;
    // view 目录可选，用于传统模板渲染
    std::fs::create_dir_all(app_dir.join("view"))?;

    // 创建默认控制器（传递应用名称）
    let controller_content = generate_controller_content("Index", false, false, Some(name));
    std::fs::write(
        app_dir.join("controller/Index.php"),
        controller_content,
    )?;

    println!("✓ 应用 '{}' 构建完成", name);
    println!("  目录结构:");
    println!("    app/{}/", name);
    println!("    ├── controller/    # 控制器目录");
    println!("    ├── model/         # 模型目录");
    println!("    ├── middleware/    # 中间件目录");
    println!("    ├── validate/      # 验证器目录");
    println!("    └── view/          # 视图目录（可选）");
    println!();
    println!("  按需创建其他目录:");
    println!("    oyta make:service <name>   → app/service/");
    println!("    oyta make:event <name>     → app/event/");
    println!("    oyta make:listener <name>  → app/listener/");
    println!("    oyta make:job <name>       → app/job/");
    println!("    oyta make:task <name>      → app/task/");

    Ok(())
}

/// 处理 make:controller 命令
///
/// 创建控制器
///
/// # 参数
/// - `name`: 控制器名称（支持格式：`Index` 或 `admin/Index`）
/// - `plain`: 是否创建空控制器
/// - `api`: 是否创建 API 控制器
pub async fn handle_make_controller(name: &str, plain: bool, api: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 解析控制器路径，支持多应用模式
    // 格式：`Index`（单应用）或 `admin/Index`（多应用）
    let (app_name, class_name) = if name.contains('/') || name.contains('\\') {
        let parts: Vec<&str> = name.split(['/', '\\']).collect();
        let app = parts[0].to_string();
        let class = parts.get(1).unwrap_or(&"Index").to_string();
        (Some(app), class)
    } else {
        (None, name.to_string())
    };
    
    // 创建目录
    let controller_dir = if let Some(ref app) = app_name {
        project.root.join("app").join(app).join("controller")
    } else {
        project.root.join("app").join("controller")
    };
    
    let file_path = controller_dir.join(format!("{}.php", class_name));
    
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 生成控制器内容
    let content = generate_controller_content(&class_name, plain, api, app_name.as_deref());
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 控制器已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:model 命令
///
/// 创建模型
///
/// # 参数
/// - `name`: 模型名称
pub async fn handle_make_model(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let model_dir = project.root.join("app").join("model");
    let file_path = model_dir.join(format!("{}.php", name.replace("\\", "/")));
    
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 生成模型内容
    let content = generate_model_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 模型已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:middleware 命令
///
/// 创建中间件
///
/// # 参数
/// - `name`: 中间件名称
pub async fn handle_make_middleware(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let middleware_dir = project.root.join("app").join("middleware");
    let file_path = middleware_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&middleware_dir)?;

    // 生成中间件内容
    let content = generate_middleware_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 中间件已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:validate 命令
///
/// 创建验证器
///
/// # 参数
/// - `name`: 验证器名称
pub async fn handle_make_validate(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let validate_dir = project.root.join("app").join("validate");
    let file_path = validate_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&validate_dir)?;

    // 生成验证器内容
    let content = generate_validate_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 验证器已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:event 命令
///
/// 创建事件类
///
/// # 参数
/// - `name`: 事件名称
pub async fn handle_make_event(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let event_dir = project.root.join("app").join("event");
    let file_path = event_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&event_dir)?;

    // 生成事件内容
    let content = generate_event_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 事件已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:listener 命令
///
/// 创建事件监听器
///
/// # 参数
/// - `name`: 监听器名称
pub async fn handle_make_listener(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let listener_dir = project.root.join("app").join("listener");
    let file_path = listener_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&listener_dir)?;

    // 生成监听器内容
    let content = generate_listener_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 监听器已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:subscribe 命令
///
/// 创建事件订阅者
///
/// # 参数
/// - `name`: 订阅者名称
pub async fn handle_make_subscribe(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let subscribe_dir = project.root.join("app").join("subscribe");
    let file_path = subscribe_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&subscribe_dir)?;

    // 生成订阅者内容
    let content = generate_subscribe_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 订阅者已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:service 命令
///
/// 创建服务类
///
/// # 参数
/// - `name`: 服务类名称
pub async fn handle_make_service(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let service_dir = project.root.join("app").join("service");
    let file_path = service_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&service_dir)?;

    // 生成服务类内容
    let content = generate_service_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 服务类已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:command 命令
///
/// 创建自定义命令
///
/// # 参数
/// - `name`: 命令名称
pub async fn handle_make_command(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let command_dir = project.root.join("app").join("command");
    let file_path = command_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&command_dir)?;

    // 生成命令内容
    let content = generate_command_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 命令已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:job 命令
///
/// 创建队列任务类
///
/// # 参数
/// - `name`: 任务名称
pub async fn handle_make_job(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let job_dir = project.root.join("app").join("job");
    let file_path = job_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&job_dir)?;

    // 生成队列任务内容
    let content = generate_job_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 队列任务已创建: {}", file_path.display());

    Ok(())
}

/// 处理 make:task 命令
///
/// 创建定时任务类
///
/// # 参数
/// - `name`: 任务名称
pub async fn handle_make_task(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建目录
    let task_dir = project.root.join("app").join("task");
    let file_path = task_dir.join(format!("{}.php", name));
    
    std::fs::create_dir_all(&task_dir)?;

    // 生成定时任务内容
    let content = generate_task_content(name);
    
    // 写入文件
    std::fs::write(&file_path, content)?;
    
    println!("✓ 定时任务已创建: {}", file_path.display());

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 解析类名，返回命名空间和类名
fn parse_class_name(name: &str, default_namespace: &str) -> (String, String) {
    if name.contains('\\') || name.contains('/') {
        let parts: Vec<&str> = name.split(['\\', '/']).collect();
        let class_name = parts.last().unwrap_or(&name).to_string();
        let namespace = parts[..parts.len() - 1].join("\\");
        (namespace, class_name)
    } else {
        (default_namespace.to_string(), name.to_string())
    }
}

/// 生成控制器内容
fn generate_controller_content(name: &str, plain: bool, api: bool, app_name: Option<&str>) -> String {
    let namespace = if let Some(app) = app_name {
        format!("app\\{}\\controller", app)
    } else {
        "app\\controller".to_string()
    };
    
    if api {
        format!(r#"<?php

namespace {};

use app\BaseController;

/**
 * {} 控制器
 */
class {} extends BaseController
{{
    /**
     * 默认方法
     */
    public function index()
    {{
        return json([
            'code' => 200,
            'msg' => 'success',
            'data' => []
        ]);
    }}
}}
"#, namespace, name, name)
    } else if plain {
        format!(r#"<?php

namespace {};

use app\BaseController;

/**
 * {} 控制器
 */
class {} extends BaseController
{{
    // 在此添加你的方法
}}
"#, namespace, name, name)
    } else {
        format!(r#"<?php

namespace {};

use app\BaseController;

/**
 * {} 控制器
 */
class {} extends BaseController
{{
    /**
     * 默认方法
     */
    public function index()
    {{
        return 'Hello, {}!';
    }}
}}
"#, namespace, name, name, name)
    }
}

/// 生成模型内容
fn generate_model_content(name: &str) -> String {
    format!(r#"<?php

namespace app\model;

use think\Model;

/**
 * {} 模型
 */
class {} extends Model
{{
    // 表名
    protected $name = '{}';
    
    // 主键
    protected $pk = 'id';
    
    // 自动时间戳
    protected $autoWriteTimestamp = true;
}}
"#, name, name, name.to_lowercase())
}

/// 生成中间件内容
fn generate_middleware_content(name: &str) -> String {
    format!(r#"<?php

namespace app\middleware;

use Closure;
use think\Request;
use think\Response;

/**
 * {} 中间件
 */
class {}
{{
    /**
     * 处理请求
     *
     * @param Request $request
     * @param Closure $next
     * @return Response
     */
    public function handle(Request $request, Closure $next): Response
    {{
        // 前置操作
        
        $response = $next($request);
        
        // 后置操作
        
        return $response;
    }}
}}
"#, name, name)
}

/// 生成验证器内容
fn generate_validate_content(name: &str) -> String {
    format!(r#"<?php

namespace app\validate;

use think\Validate;

/**
 * {} 验证器
 */
class {} extends Validate
{{
    /**
     * 验证规则
     */
    protected $rule = [
        'name' => 'require|max:255',
        'email' => 'require|email',
    ];
    
    /**
     * 验证消息
     */
    protected $message = [
        'name.require' => '名称不能为空',
        'name.max' => '名称最多255个字符',
        'email.require' => '邮箱不能为空',
        'email.email' => '邮箱格式不正确',
    ];
    
    /**
     * 验证场景
     */
    protected $scene = [
        'create' => ['name', 'email'],
        'update' => ['name', 'email'],
    ];
}}
"#, name, name)
}

/// 生成事件内容
fn generate_event_content(name: &str) -> String {
    format!(r#"<?php

namespace app\event;

/**
 * {} 事件
 */
class {}
{{
    /**
     * 事件数据
     */
    public $data;
    
    /**
     * 创建事件实例
     */
    public function __construct($data = null)
    {{
        $this->data = $data;
    }}
}}
"#, name, name)
}

/// 生成监听器内容
fn generate_listener_content(name: &str) -> String {
    format!(r#"<?php

namespace app\listener;

/**
 * {} 监听器
 */
class {}
{{
    /**
     * 处理事件
     *
     * @param object $event
     */
    public function handle($event)
    {{
        // 处理事件逻辑
    }}
}}
"#, name, name)
}

/// 生成订阅者内容
fn generate_subscribe_content(name: &str) -> String {
    format!(r#"<?php

namespace app\subscribe;

/**
 * {} 订阅者
 */
class {}
{{
    /**
     * 订阅事件
     */
    public function subscribe($events)
    {{
        // $events->listen('App\Event\Example', 'App\Listener\ExampleListener');
    }}
}}
"#, name, name)
}

/// 生成服务类内容
fn generate_service_content(name: &str) -> String {
    format!(r#"<?php

namespace app\service;

/**
 * {} 服务类
 */
class {}
{{
    /**
     * 服务实例
     */
    protected static $instance;
    
    /**
     * 获取服务实例
     */
    public static function getInstance(): self
    {{
        if (is_null(self::$instance)) {{
            self::$instance = new static();
        }}
        return self::$instance;
    }}
    
    /**
     * 执行服务方法
     */
    public function execute()
    {{
        // 实现服务逻辑
    }}
}}
"#, name, name)
}

/// 生成命令内容
fn generate_command_content(name: &str) -> String {
    format!(r#"<?php

namespace app\command;

use think\console\Command;
use think\console\Input;
use think\console\Output;

/**
 * {} 命令
 */
class {} extends Command
{{
    protected function configure()
    {{
        $this->setName('{}')
            ->setDescription('自定义命令');
    }}
    
    protected function execute(Input $input, Output $output)
    {{
        $output->writeln('Hello, {}!');
    }}
}}
"#, name, name, name.to_lowercase(), name)
}

/// 生成队列任务内容
fn generate_job_content(name: &str) -> String {
    format!(r#"<?php

namespace app\job;

use think\queue\Job;

/**
 * {} 队列任务
 */
class {}
{{
    /**
     * 处理任务
     *
     * @param Job $job
     * @param array $data
     */
    public function fire(Job $job, $data)
    {{
        // 处理任务逻辑
        
        // 任务成功，删除任务
        $job->delete();
        
        // 任务失败
        // $job->failed();
    }}
    
    /**
     * 任务失败回调
     *
     * @param array $data
     */
    public function failed($data)
    {{
        // 任务失败处理
    }}
}}
"#, name, name)
}

/// 生成定时任务内容
fn generate_task_content(name: &str) -> String {
    format!(r#"<?php

namespace app\task;

/**
 * {} 定时任务
 * 
 * @TimerTask(cron: "0 * * * *")
 */
class {}
{{
    /**
     * 执行任务
     */
    public function handle()
    {{
        // 实现定时任务逻辑
    }}
}}
"#, name, name)
}
