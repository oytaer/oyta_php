//! eval() 动态执行模块
//! 
//! 本模块实现 PHP 的 eval() 动态代码执行功能，包括：
//! - 代码解析和执行
//! - 执行上下文管理
//! - 沙箱安全机制
//! - 变量作用域处理

pub mod context;
pub mod evaluator;
pub mod sandbox;

pub use context::{EvalContext, EvalVariable, EvalScope};
pub use evaluator::{Eval, EvalResult, EvalError};
pub use sandbox::{EvalSandbox, SandboxPolicy, SandboxPermission};
