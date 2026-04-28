//! 沙箱安全模块
//! 
//! 本模块实现 eval() 的沙箱安全机制，包括：
//! - 权限控制
//! - 函数白名单/黑名单
//! - 资源限制
//! - 安全策略

use std::collections::{HashMap, HashSet};

/// 沙箱权限
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxPermission {
    /// 允许文件读取
    FileRead,
    /// 允许文件写入
    FileWrite,
    /// 允许文件删除
    FileDelete,
    /// 允许执行外部命令
    ExecuteCommand,
    /// 允许网络访问
    NetworkAccess,
    /// 允许访问环境变量
    AccessEnv,
    /// 允许访问超全局变量
    AccessSuperGlobals,
    /// 允许定义函数
    DefineFunction,
    /// 允许定义类
    DefineClass,
    /// 允许使用 eval
    UseEval,
    /// 允许使用 include/require
    UseInclude,
    /// 允许访问数据库
    DatabaseAccess,
    /// 允许创建进程
    CreateProcess,
    /// 允许修改配置
    ModifyConfig,
    /// 允许动态加载扩展
    LoadExtension,
    /// 允许访问系统信息
    SystemInfo,
}

/// 沙箱策略
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    /// 允许的权限
    allowed_permissions: HashSet<SandboxPermission>,
    /// 禁止的权限（优先级高于允许）
    denied_permissions: HashSet<SandboxPermission>,
    /// 允许的函数白名单
    allowed_functions: Option<HashSet<String>>,
    /// 禁止的函数黑名单
    denied_functions: HashSet<String>,
    /// 允许的类白名单
    allowed_classes: Option<HashSet<String>>,
    /// 禁止的类黑名单
    denied_classes: HashSet<String>,
    /// 允许访问的目录
    allowed_directories: HashSet<String>,
    /// 禁止访问的目录
    denied_directories: HashSet<String>,
    /// 允许访问的 URL 模式
    allowed_url_patterns: Vec<String>,
    /// 最大执行时间（毫秒）
    max_execution_time: Option<u64>,
    /// 最大内存使用（字节）
    max_memory_usage: Option<usize>,
    /// 最大递归深度
    max_recursion_depth: usize,
    /// 最大函数调用次数
    max_function_calls: Option<usize>,
    /// 最大字符串长度
    max_string_length: Option<usize>,
    /// 最大数组大小
    max_array_size: Option<usize>,
    /// 是否启用严格模式
    strict_mode: bool,
    /// 是否允许错误抑制符
    allow_error_suppression: bool,
}

impl SandboxPolicy {
    /// 创建默认的安全策略
    pub fn safe() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(SandboxPermission::DefineFunction);
        allowed.insert(SandboxPermission::DefineClass);
        let mut denied = HashSet::new();
        denied.insert(SandboxPermission::FileWrite);
        denied.insert(SandboxPermission::FileDelete);
        denied.insert(SandboxPermission::ExecuteCommand);
        denied.insert(SandboxPermission::NetworkAccess);
        denied.insert(SandboxPermission::CreateProcess);
        denied.insert(SandboxPermission::LoadExtension);
        denied.insert(SandboxPermission::ModifyConfig);
        let mut denied_functions = HashSet::new();
        denied_functions.insert("exec".to_string());
        denied_functions.insert("shell_exec".to_string());
        denied_functions.insert("system".to_string());
        denied_functions.insert("passthru".to_string());
        denied_functions.insert("popen".to_string());
        denied_functions.insert("proc_open".to_string());
        denied_functions.insert("pcntl_exec".to_string());
        denied_functions.insert("file_put_contents".to_string());
        denied_functions.insert("fwrite".to_string());
        denied_functions.insert("unlink".to_string());
        denied_functions.insert("rmdir".to_string());
        denied_functions.insert("rename".to_string());
        denied_functions.insert("chmod".to_string());
        denied_functions.insert("chown".to_string());
        denied_functions.insert("eval".to_string());
        denied_functions.insert("assert".to_string());
        denied_functions.insert("create_function".to_string());
        denied_functions.insert("call_user_func".to_string());
        denied_functions.insert("call_user_func_array".to_string());
        denied_functions.insert("preg_replace".to_string()); // /e 修饰符
        SandboxPolicy {
            allowed_permissions: allowed,
            denied_permissions: denied,
            allowed_functions: None,
            denied_functions,
            allowed_classes: None,
            denied_classes: HashSet::new(),
            allowed_directories: HashSet::new(),
            denied_directories: HashSet::new(),
            allowed_url_patterns: Vec::new(),
            max_execution_time: Some(30000), // 30 秒
            max_memory_usage: Some(32 * 1024 * 1024), // 32MB
            max_recursion_depth: 100,
            max_function_calls: Some(10000),
            max_string_length: Some(1024 * 1024), // 1MB
            max_array_size: Some(10000),
            strict_mode: true,
            allow_error_suppression: false,
        }
    }

    /// 创建宽松的策略
    pub fn permissive() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(SandboxPermission::FileRead);
        allowed.insert(SandboxPermission::FileWrite);
        allowed.insert(SandboxPermission::DefineFunction);
        allowed.insert(SandboxPermission::DefineClass);
        allowed.insert(SandboxPermission::UseEval);
        allowed.insert(SandboxPermission::UseInclude);
        allowed.insert(SandboxPermission::AccessSuperGlobals);
        SandboxPolicy {
            allowed_permissions: allowed,
            denied_permissions: HashSet::new(),
            allowed_functions: None,
            denied_functions: HashSet::new(),
            allowed_classes: None,
            denied_classes: HashSet::new(),
            allowed_directories: HashSet::new(),
            denied_directories: HashSet::new(),
            allowed_url_patterns: Vec::new(),
            max_execution_time: Some(60000),
            max_memory_usage: Some(128 * 1024 * 1024),
            max_recursion_depth: 256,
            max_function_calls: Some(100000),
            max_string_length: Some(10 * 1024 * 1024),
            max_array_size: Some(100000),
            strict_mode: false,
            allow_error_suppression: true,
        }
    }

    /// 创建最严格的策略
    pub fn strict() -> Self {
        let denied_functions: HashSet<String> = [
            "exec", "shell_exec", "system", "passthru", "popen", "proc_open",
            "pcntl_exec", "file_put_contents", "fwrite", "fputs", "fputcsv",
            "unlink", "rmdir", "rename", "copy", "move_uploaded_file",
            "chmod", "chown", "chgrp", "touch", "mkdir", "fopen", "fclose",
            "file", "file_get_contents", "file_exists", "is_file", "is_dir",
            "is_readable", "is_writable", "is_executable", "glob", "scandir",
            "opendir", "readdir", "closedir", "readfile", "fpassthru",
            "eval", "assert", "create_function", "call_user_func",
            "call_user_func_array", "call_user_method", "call_user_method_array",
            "preg_replace", "mb_ereg_replace", "mb_eregi_replace",
            "filter_input", "filter_input_array", "filter_var", "filter_var_array",
            "getenv", "putenv", "get_cfg_var", "get_current_user", "getmygid",
            "getmyinode", "getmypid", "getmyuid", "getopt", "ini_get", "ini_set",
            "ini_restore", "ini_get_all", "phpinfo", "phpversion", "php_uname",
            "php_sapi_name", "php_ini_loaded_file", "php_ini_scanned_files",
            "set_time_limit", "set_include_path", "restore_include_path",
            "dl", "extension_loaded", "get_extension_funcs",
            "mail", "header", "header_remove", "setcookie", "setrawcookie",
            "http_response_code", "http_build_query",
            "curl_init", "curl_exec", "curl_multi_exec",
            "fsockopen", "pfsockopen", "socket_create", "stream_socket_client",
            "ftp_connect", "ftp_ssl_connect", "ftp_login",
            "imap_open", "ldap_connect", "mysqli_connect", "pg_connect",
            "pdo_construct", "sqlite_open",
        ].iter().map(|s| s.to_string()).collect();
        let mut denied = HashSet::new();
        denied.insert(SandboxPermission::FileRead);
        denied.insert(SandboxPermission::FileWrite);
        denied.insert(SandboxPermission::FileDelete);
        denied.insert(SandboxPermission::ExecuteCommand);
        denied.insert(SandboxPermission::NetworkAccess);
        denied.insert(SandboxPermission::AccessEnv);
        denied.insert(SandboxPermission::AccessSuperGlobals);
        denied.insert(SandboxPermission::UseEval);
        denied.insert(SandboxPermission::UseInclude);
        denied.insert(SandboxPermission::DatabaseAccess);
        denied.insert(SandboxPermission::CreateProcess);
        denied.insert(SandboxPermission::ModifyConfig);
        denied.insert(SandboxPermission::LoadExtension);
        denied.insert(SandboxPermission::SystemInfo);
        SandboxPolicy {
            allowed_permissions: HashSet::new(),
            denied_permissions: denied,
            allowed_functions: None,
            denied_functions,
            allowed_classes: None,
            denied_classes: HashSet::new(),
            allowed_directories: HashSet::new(),
            denied_directories: HashSet::new(),
            allowed_url_patterns: Vec::new(),
            max_execution_time: Some(5000), // 5 秒
            max_memory_usage: Some(8 * 1024 * 1024), // 8MB
            max_recursion_depth: 50,
            max_function_calls: Some(1000),
            max_string_length: Some(64 * 1024), // 64KB
            max_array_size: Some(1000),
            strict_mode: true,
            allow_error_suppression: false,
        }
    }

    /// 检查权限
    pub fn has_permission(&self, permission: SandboxPermission) -> bool {
        if self.denied_permissions.contains(&permission) {
            return false;
        }
        self.allowed_permissions.contains(&permission)
    }

    /// 授予权限
    pub fn grant(&mut self, permission: SandboxPermission) {
        self.denied_permissions.remove(&permission);
        self.allowed_permissions.insert(permission);
    }

    /// 撤销权限
    pub fn revoke(&mut self, permission: SandboxPermission) {
        self.allowed_permissions.remove(&permission);
        self.denied_permissions.insert(permission);
    }

    /// 检查函数是否允许
    pub fn is_function_allowed(&self, name: &str) -> bool {
        if self.denied_functions.contains(name) {
            return false;
        }
        if let Some(ref allowed) = self.allowed_functions {
            allowed.contains(name)
        } else {
            true
        }
    }

    /// 允许函数
    pub fn allow_function(&mut self, name: &str) {
        self.denied_functions.remove(name);
        if let Some(ref mut allowed) = self.allowed_functions {
            allowed.insert(name.to_string());
        }
    }

    /// 禁止函数
    pub fn deny_function(&mut self, name: &str) {
        self.denied_functions.insert(name.to_string());
    }

    /// 检查类是否允许
    pub fn is_class_allowed(&self, name: &str) -> bool {
        if self.denied_classes.contains(name) {
            return false;
        }
        if let Some(ref allowed) = self.allowed_classes {
            allowed.contains(name)
        } else {
            true
        }
    }

    /// 允许类
    pub fn allow_class(&mut self, name: &str) {
        self.denied_classes.remove(name);
        if let Some(ref mut allowed) = self.allowed_classes {
            allowed.insert(name.to_string());
        }
    }

    /// 禁止类
    pub fn deny_class(&mut self, name: &str) {
        self.denied_classes.insert(name.to_string());
    }

    /// 检查目录是否允许访问
    pub fn is_directory_allowed(&self, path: &str) -> bool {
        // 检查禁止目录
        for denied in &self.denied_directories {
            if path.starts_with(denied) {
                return false;
            }
        }
        // 如果有允许目录列表，检查是否在允许范围内
        if !self.allowed_directories.is_empty() {
            for allowed in &self.allowed_directories {
                if path.starts_with(allowed) {
                    return true;
                }
            }
            return false;
        }
        true
    }

    /// 允许目录访问
    pub fn allow_directory(&mut self, path: &str) {
        self.denied_directories.remove(path);
        self.allowed_directories.insert(path.to_string());
    }

    /// 禁止目录访问
    pub fn deny_directory(&mut self, path: &str) {
        self.denied_directories.insert(path.to_string());
    }

    /// 获取最大执行时间
    pub fn max_execution_time(&self) -> Option<u64> {
        self.max_execution_time
    }

    /// 设置最大执行时间
    pub fn set_max_execution_time(&mut self, ms: u64) {
        self.max_execution_time = Some(ms);
    }

    /// 获取最大内存使用
    pub fn max_memory_usage(&self) -> Option<usize> {
        self.max_memory_usage
    }

    /// 设置最大内存使用
    pub fn set_max_memory_usage(&mut self, bytes: usize) {
        self.max_memory_usage = Some(bytes);
    }

    /// 获取最大递归深度
    pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// 设置最大递归深度
    pub fn set_max_recursion_depth(&mut self, depth: usize) {
        self.max_recursion_depth = depth;
    }

    /// 检查是否启用严格模式
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// 启用严格模式
    pub fn enable_strict_mode(&mut self) {
        self.strict_mode = true;
    }

    /// 禁用严格模式
    pub fn disable_strict_mode(&mut self) {
        self.strict_mode = false;
    }
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self::safe()
    }
}

/// eval 沙箱
#[derive(Debug)]
pub struct EvalSandbox {
    /// 沙箱策略
    policy: SandboxPolicy,
    /// 当前执行时间
    execution_time: u64,
    /// 当前内存使用
    memory_usage: usize,
    /// 当前递归深度
    recursion_depth: usize,
    /// 当前函数调用次数
    function_call_count: usize,
    /// 是否已超时
    timed_out: bool,
    /// 是否内存不足
    out_of_memory: bool,
}

impl EvalSandbox {
    /// 创建新的沙箱
    pub fn new() -> Self {
        EvalSandbox {
            policy: SandboxPolicy::safe(),
            execution_time: 0,
            memory_usage: 0,
            recursion_depth: 0,
            function_call_count: 0,
            timed_out: false,
            out_of_memory: false,
        }
    }

    /// 使用指定策略创建沙箱
    pub fn with_policy(policy: SandboxPolicy) -> Self {
        EvalSandbox {
            policy,
            execution_time: 0,
            memory_usage: 0,
            recursion_depth: 0,
            function_call_count: 0,
            timed_out: false,
            out_of_memory: false,
        }
    }

    /// 获取策略引用
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// 获取策略可变引用
    pub fn policy_mut(&mut self) -> &mut SandboxPolicy {
        &mut self.policy
    }

    /// 检查权限
    pub fn check_permission(&self, permission: SandboxPermission) -> Result<(), String> {
        if !self.policy.has_permission(permission) {
            return Err(format!("权限被拒绝: {:?}", permission));
        }
        Ok(())
    }

    /// 检查函数是否允许
    pub fn check_function(&self, name: &str) -> Result<(), String> {
        if !self.policy.is_function_allowed(name) {
            return Err(format!("函数 '{}' 被禁止", name));
        }
        Ok(())
    }

    /// 检查类是否允许
    pub fn check_class(&self, name: &str) -> Result<(), String> {
        if !self.policy.is_class_allowed(name) {
            return Err(format!("类 '{}' 被禁止", name));
        }
        Ok(())
    }

    /// 检查目录访问
    pub fn check_directory(&self, path: &str) -> Result<(), String> {
        if !self.policy.is_directory_allowed(path) {
            return Err(format!("目录 '{}' 访问被拒绝", path));
        }
        Ok(())
    }

    /// 进入递归
    pub fn enter_recursion(&mut self) -> Result<(), String> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.policy.max_recursion_depth {
            return Err(format!("超过最大递归深度: {}", self.policy.max_recursion_depth));
        }
        Ok(())
    }

    /// 退出递归
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// 记录函数调用
    pub fn record_function_call(&mut self) -> Result<(), String> {
        self.function_call_count += 1;
        if let Some(max) = self.policy.max_function_calls {
            if self.function_call_count > max {
                return Err(format!("超过最大函数调用次数: {}", max));
            }
        }
        Ok(())
    }

    /// 更新执行时间
    pub fn update_execution_time(&mut self, ms: u64) -> Result<(), String> {
        self.execution_time = ms;
        if let Some(max) = self.policy.max_execution_time {
            if ms > max {
                self.timed_out = true;
                return Err(format!("执行超时: {}ms > {}ms", ms, max));
            }
        }
        Ok(())
    }

    /// 更新内存使用
    pub fn update_memory_usage(&mut self, bytes: usize) -> Result<(), String> {
        self.memory_usage = bytes;
        if let Some(max) = self.policy.max_memory_usage {
            if bytes > max {
                self.out_of_memory = true;
                return Err(format!("内存不足: {}bytes > {}bytes", bytes, max));
            }
        }
        Ok(())
    }

    /// 检查字符串长度
    pub fn check_string_length(&self, len: usize) -> Result<(), String> {
        if let Some(max) = self.policy.max_string_length {
            if len > max {
                return Err(format!("字符串长度超过限制: {} > {}", len, max));
            }
        }
        Ok(())
    }

    /// 检查数组大小
    pub fn check_array_size(&self, size: usize) -> Result<(), String> {
        if let Some(max) = self.policy.max_array_size {
            if size > max {
                return Err(format!("数组大小超过限制: {} > {}", size, max));
            }
        }
        Ok(())
    }

    /// 重置计数器
    pub fn reset(&mut self) {
        self.execution_time = 0;
        self.memory_usage = 0;
        self.recursion_depth = 0;
        self.function_call_count = 0;
        self.timed_out = false;
        self.out_of_memory = false;
    }

    /// 检查是否超时
    pub fn is_timed_out(&self) -> bool {
        self.timed_out
    }

    /// 检查是否内存不足
    pub fn is_out_of_memory(&self) -> bool {
        self.out_of_memory
    }

    /// 获取当前执行时间
    pub fn execution_time(&self) -> u64 {
        self.execution_time
    }

    /// 获取当前内存使用
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// 获取当前递归深度
    pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// 获取当前函数调用次数
    pub fn function_call_count(&self) -> usize {
        self.function_call_count
    }
}

impl Default for EvalSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_policy_safe() {
        let policy = SandboxPolicy::safe();
        assert!(policy.has_permission(SandboxPermission::DefineFunction));
        assert!(!policy.has_permission(SandboxPermission::ExecuteCommand));
    }

    #[test]
    fn test_sandbox_policy_strict() {
        let policy = SandboxPolicy::strict();
        assert!(!policy.has_permission(SandboxPermission::FileRead));
        assert!(!policy.has_permission(SandboxPermission::NetworkAccess));
    }

    #[test]
    fn test_sandbox_policy_function() {
        let policy = SandboxPolicy::safe();
        assert!(!policy.is_function_allowed("exec"));
        assert!(policy.is_function_allowed("strlen"));
    }

    #[test]
    fn test_eval_sandbox() {
        let mut sandbox = EvalSandbox::new();
        assert!(sandbox.check_permission(SandboxPermission::DefineFunction).is_ok());
        assert!(sandbox.check_permission(SandboxPermission::ExecuteCommand).is_err());
    }

    #[test]
    fn test_eval_sandbox_recursion() {
        let mut sandbox = EvalSandbox::new();
        // 默认的 max_recursion_depth 是 100
        // 调用 100 次应该成功
        for _ in 0..100 {
            assert!(sandbox.enter_recursion().is_ok());
        }
        // 第 101 次应该失败
        assert!(sandbox.enter_recursion().is_err());
    }
}
