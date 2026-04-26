//! Make 命令模块
//!
//! 包含所有 make:* 命令的实现
//! 用于创建控制器、模型、中间件、验证器等

use anyhow::{bail, Result};

use crate::env_loader;
use crate::project;

use super::helpers::{camel_to_snake, parse_name_with_namespace};

/// 处理 make:controller 命令：创建控制器
/// 在 app/controller/ 目录下生成控制器 PHP 文件
/// 支持 plain（空控制器）和 api（API控制器）模式
///
/// # 参数
/// - `name`: 控制器名称
/// - `plain`: 是否为空控制器
/// - `api`: 是否为 API 控制器
pub async fn handle_make_controller(name: &str, plain: bool, api: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定控制器文件路径
    let controller_dir = project.controller_dir(None);
    let file_path = controller_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("控制器文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 根据命名空间规则生成类名
    // 如 admin/Index → namespace app\controller\admin; class Index
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\controller");

    // 根据模式生成不同的控制器内容
    let content = if plain {
        // 空控制器：不包含任何方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n}}\n",
            namespace, class_name
        )
    } else if api {
        // API 控制器：包含 RESTful 风格方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n    public function index()\n    {{\n        return json([{{'status' => 'ok'}}]);\n    }}\n\n    public function show($id)\n    {{\n        return json([{{'id' => $id}}]);\n    }}\n\n    public function save()\n    {{\n        return json([{{'status' => 'created'}}], 201);\n    }}\n\n    public function update($id)\n    {{\n        return json([{{'id' => $id, 'status' => 'updated'}}]);\n    }}\n\n    public function delete($id)\n    {{\n        return json([{{'id' => $id, 'status' => 'deleted'}}]);\n    }}\n}}\n",
            namespace, class_name
        )
    } else {
        // 标准控制器：包含 index 和 hello 方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n    public function index()\n    {{\n        return 'Hello, OYTAPHP!';\n    }}\n\n    public function hello(string $name = 'think')\n    {{\n        return 'Hello,' . $name;\n    }}\n}}\n",
            namespace, class_name
        )
    };

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 控制器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:model 命令：创建模型
/// 在 app/model/ 目录下生成模型 PHP 文件
/// 模型默认继承 oyta\Model
///
/// # 参数
/// - `name`: 模型名称
pub async fn handle_make_model(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定模型文件路径
    let model_dir = project.model_dir(None);
    let file_path = model_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("模型文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\model");

    // 生成模型内容，继承 oyta\Model
    let content = format!(
        "<?php\nnamespace {};\n\nuse oyta\\Model;\n\nclass {} extends Model\n{{\n    // 模型名称（对应数据表名，默认为类名的下划线形式）\n    protected $name = '{}';\n\n    // 主键字段名，默认为 id\n    protected $pk = 'id';\n\n    // 自动写入时间戳\n    protected $autoWriteTimestamp = true;\n}}\n",
        namespace,
        class_name,
        // 将类名转换为下划线形式作为默认表名
        camel_to_snake(&class_name).as_str()
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 模型创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:middleware 命令：创建中间件
/// 在 app/middleware/ 目录下生成中间件 PHP 文件
///
/// # 参数
/// - `name`: 中间件名称
pub async fn handle_make_middleware(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定中间件文件路径
    let middleware_dir = project.middleware_dir(None);
    let file_path = middleware_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("中间件文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\middleware");

    // 生成中间件内容，包含 handle 和 end 方法
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 中间件处理\n     * @param mixed  $request 请求对象\n     * @param \\Closure $next  下一个中间件\n     * @return mixed\n     */\n    public function handle($request, \\Closure $next)\n    {{\n        // 前置中间件逻辑\n\n        // 执行下一个中间件\n        $response = $next($request);\n\n        // 后置中间件逻辑\n\n        return $response;\n    }}\n\n    /**\n     * 中间件结束回调\n     * @param mixed $response 响应对象\n     */\n    public function end($response)\n    {{\n        // 请求结束后的回调逻辑\n    }}\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 中间件创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:validate 命令：创建验证器
/// 在 app/validate/ 目录下生成验证器 PHP 文件
///
/// # 参数
/// - `name`: 验证器名称
pub async fn handle_make_validate(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定验证器文件路径
    let validate_dir = project.app_dir.join("validate");
    let file_path = validate_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("验证器文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&validate_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\validate");

    // 生成验证器内容，继承 oyta\Validate
    let content = format!(
        "<?php\nnamespace {};\n\nuse oyta\\Validate;\n\nclass {} extends Validate\n{{\n    // 验证规则\n    protected $rule = [\n        // 'name'  => 'require|max:25',\n        // 'email' => 'require|email',\n    ];\n\n    // 验证消息\n    protected $message = [\n        // 'name.require' => '名称必须',\n        // 'name.max'     => '名称最多25个字符',\n        // 'email.require'=> '邮箱必须',\n        // 'email.email'  => '邮箱格式错误',\n    ];\n\n    // 验证场景\n    protected $scene = [\n        // 'login'   => ['email'],\n        // 'register'=> ['name', 'email'],\n    ];\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 验证器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:event 命令：创建事件类
/// 在 app/event/ 目录下生成事件 PHP 文件
///
/// # 参数
/// - `name`: 事件名称
pub async fn handle_make_event(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定事件文件路径
    let event_dir = project.app_dir.join("event");
    let file_path = event_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("事件文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&event_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\event");

    // 生成事件内容
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    // 事件数据\n    public $data;\n\n    public function __construct($data = null)\n    {{\n        $this->data = $data;\n    }}\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 事件创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:listener 命令：创建事件监听器
/// 在 app/listener/ 目录下生成监听器 PHP 文件
///
/// # 参数
/// - `name`: 监听器名称
pub async fn handle_make_listener(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定监听器文件路径
    let listener_dir = project.app_dir.join("listener");
    let file_path = listener_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("监听器文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&listener_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\listener");

    // 生成监听器内容
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 事件监听处理\n     * @param mixed $event 事件数据\n     */\n    public function handle($event)\n    {{\n        // 监听器逻辑\n    }}\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 监听器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:subscribe 命令：创建事件订阅者
/// 在 app/subscribe/ 目录下生成订阅者 PHP 文件
///
/// # 参数
/// - `name`: 订阅者名称
pub async fn handle_make_subscribe(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定订阅者文件路径
    let subscribe_dir = project.app_dir.join("subscribe");
    let file_path = subscribe_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("订阅者文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&subscribe_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\subscribe");

    // 生成订阅者内容
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 订阅的事件列表\n     * @return array\n     */\n    public function subscribe()\n    {{\n        return [\n            // 'UserLogin' => 'onUserLogin',\n            // 'UserLogout'=> 'onUserLogout',\n        ];\n    }}\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 订阅者创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:service 命令：创建服务类
/// 在 app/service/ 目录下生成服务类 PHP 文件
///
/// # 参数
/// - `name`: 服务类名称
pub async fn handle_make_service(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定服务类文件路径
    let service_dir = project.app_dir.join("service");
    let file_path = service_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("服务类文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&service_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\service");

    // 生成服务类内容
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    // 服务类逻辑\n}}\n",
        namespace, class_name
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 服务类创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:command 命令：创建自定义命令
/// 在 app/command/ 目录下生成命令类 PHP 文件
///
/// # 参数
/// - `name`: 命令名称
pub async fn handle_make_command(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确定命令文件路径
    let command_dir = project.app_dir.join("command");
    let file_path = command_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("命令文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    std::fs::create_dir_all(&command_dir)?;

    // 解析命名空间和类名
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\command");

    // 生成命令内容
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 命令名称\n     * @var string\n     */\n    protected $name = '{}';\n\n    /**\n     * 命令描述\n     * @var string\n     */\n    protected $description = '';\n\n    /**\n     * 执行命令\n     */\n    public function execute()\n    {{\n        // 命令逻辑\n    }}\n}}\n",
        namespace, class_name, camel_to_snake(&class_name).as_str()
    );

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 命令创建成功: {}", file_path.display());

    Ok(())
}
