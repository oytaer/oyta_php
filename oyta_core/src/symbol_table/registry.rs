//! 符号表注册表模块
//!
//! 使用 dashmap 实现并发安全的符号表
//! 存储解析后的所有类、函数、常量、命名空间等符号信息
//! 支持多线程并发读写，用于热重载场景下的符号表更新

use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::types::{
    ClassDef, ConstDef, EnumDef, FileParseResult, FuncDef, InterfaceDef, NamespaceDef, TraitDef,
};

/// 符号表注册表
/// 使用 Arc<DashMap> 实现并发安全，支持多线程同时读写
/// 所有查询操作都是 O(1) 级别（基于 HashMap）
#[derive(Debug, Clone)]
pub struct SymbolRegistry {
    /// 类定义表
    /// 键：完整类名（含命名空间），如 "app\\controller\\Index"
    /// 值：类定义结构体
    classes: Arc<DashMap<String, ClassDef>>,

    /// 函数定义表
    /// 键：完整函数名（含命名空间），如 "app\\common\\json_success"
    /// 值：函数定义结构体
    functions: Arc<DashMap<String, FuncDef>>,

    /// 常量定义表
    /// 键：完整常量名（含命名空间），如 "app\\common\\APP_VERSION"
    /// 值：常量定义结构体
    constants: Arc<DashMap<String, ConstDef>>,

    /// 命名空间定义表
    /// 键：命名空间名称，如 "app\\controller"
    /// 值：命名空间定义结构体
    namespaces: Arc<DashMap<String, NamespaceDef>>,

    /// 接口定义表
    /// 键：完整接口名（含命名空间），如 "app\\contracts\\Renderable"
    /// 值：接口定义结构体
    interfaces: Arc<DashMap<String, InterfaceDef>>,

    /// Trait 定义表
    /// 键：完整 Trait 名（含命名空间），如 "app\\traits\\SoftDelete"
    /// 值：Trait 定义结构体
    traits: Arc<DashMap<String, TraitDef>>,

    /// 枚举定义表
    /// 键：完整枚举名（含命名空间），如 "app\\enums\\Status"
    /// 值：枚举定义结构体
    enums: Arc<DashMap<String, EnumDef>>,

    /// 文件到符号的映射
    /// 键：源文件路径
    /// 值：该文件解析后产生的所有符号名称
    file_symbols: Arc<DashMap<String, FileSymbols>>,

    /// 统计信息
    stats: Arc<SymbolStats>,
}

/// 文件中包含的符号名称集合
/// 用于热重载时快速清除旧符号
#[derive(Debug, Clone, Default)]
pub struct FileSymbols {
    /// 该文件中定义的类名列表
    pub class_names: Vec<String>,
    /// 该文件中定义的接口名列表
    pub interface_names: Vec<String>,
    /// 该文件中定义的 trait 名列表
    pub trait_names: Vec<String>,
    /// 该文件中定义的枚举名列表
    pub enum_names: Vec<String>,
    /// 该文件中定义的函数名列表
    pub function_names: Vec<String>,
    /// 该文件中定义的常量名列表
    pub constant_names: Vec<String>,
    /// 该文件的命名空间
    pub namespace: Option<String>,
}

/// 符号表统计信息
#[derive(Debug, Default)]
struct SymbolStats {
    /// 已解析的文件数量
    file_count: AtomicUsize,
    /// 已注册的类数量
    class_count: AtomicUsize,
    /// 已注册的函数数量
    function_count: AtomicUsize,
    /// 已注册的常量数量
    constant_count: AtomicUsize,
    /// 已注册的命名空间数量
    namespace_count: AtomicUsize,
}

/// 符号表快照
/// 用于获取符号表的当前状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSnapshot {
    /// 类数量
    pub class_count: usize,
    /// 函数数量
    pub function_count: usize,
    /// 常量数量
    pub constant_count: usize,
    /// 命名空间数量
    pub namespace_count: usize,
    /// 已解析文件数量
    pub file_count: usize,
    /// 所有控制器类名列表
    pub controllers: Vec<String>,
    /// 所有模型类名列表
    pub models: Vec<String>,
    /// 所有中间件类名列表
    pub middlewares: Vec<String>,
}

use serde::{Deserialize, Serialize};

impl SymbolRegistry {
    /// 创建新的空符号表注册表
    pub fn new() -> Self {
        Self {
            classes: Arc::new(DashMap::new()),
            functions: Arc::new(DashMap::new()),
            constants: Arc::new(DashMap::new()),
            namespaces: Arc::new(DashMap::new()),
            interfaces: Arc::new(DashMap::new()),
            traits: Arc::new(DashMap::new()),
            enums: Arc::new(DashMap::new()),
            file_symbols: Arc::new(DashMap::new()),
            stats: Arc::new(SymbolStats::default()),
        }
    }

    /// 注册一个文件解析结果到符号表
    /// 如果该文件之前已注册，会先清除旧符号再注册新符号
    pub fn register_file(&self, result: &FileParseResult) {
        // 先检查是否已有该文件的旧符号，如果有则清除
        self.unregister_file(&result.file_path);

        // 注册类定义
        for class_def in &result.classes {
            self.classes.insert(class_def.full_name.clone(), class_def.clone());
            self.stats.class_count.fetch_add(1, Ordering::Relaxed);
        }

        // 注册接口定义
        for iface_def in &result.interfaces {
            self.interfaces.insert(iface_def.full_name.clone(), iface_def.clone());
        }

        // 注册 trait 定义
        for trait_def in &result.traits {
            self.traits.insert(trait_def.full_name.clone(), trait_def.clone());
        }

        // 注册枚举定义
        for enum_def in &result.enums {
            self.enums.insert(enum_def.full_name.clone(), enum_def.clone());
        }

        // 注册函数定义
        for func_def in &result.functions {
            self.functions.insert(func_def.full_name.clone(), func_def.clone());
            self.stats.function_count.fetch_add(1, Ordering::Relaxed);
        }

        // 注册常量定义
        for const_def in &result.constants {
            let key = match &const_def.namespace {
                Some(ns) => format!("{}\\{}", ns, const_def.name),
                None => const_def.name.clone(),
            };
            self.constants.insert(key, const_def.clone());
            self.stats.constant_count.fetch_add(1, Ordering::Relaxed);
        }

        // 注册命名空间
        if let Some(ns_name) = &result.namespace {
            let ns_def = NamespaceDef {
                name: Some(ns_name.clone()),
                use_items: result.use_items.clone(),
                file_path: result.file_path.clone(),
            };
            self.namespaces.insert(ns_name.clone(), ns_def);
            self.stats.namespace_count.fetch_add(1, Ordering::Relaxed);
        }

        // 记录文件到符号的映射
        let file_sym = FileSymbols {
            class_names: result.classes.iter().map(|c| c.full_name.clone()).collect(),
            interface_names: result.interfaces.iter().map(|i| i.full_name.clone()).collect(),
            trait_names: result.traits.iter().map(|t| t.full_name.clone()).collect(),
            enum_names: result.enums.iter().map(|e| e.full_name.clone()).collect(),
            function_names: result.functions.iter().map(|f| f.full_name.clone()).collect(),
            constant_names: result.constants.iter().map(|c| {
                match &c.namespace {
                    Some(ns) => format!("{}\\{}", ns, c.name),
                    None => c.name.clone(),
                }
            }).collect(),
            namespace: result.namespace.clone(),
        };
        self.file_symbols.insert(result.file_path.clone(), file_sym);
        self.stats.file_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 注销一个文件的所有符号
    /// 用于热重载时清除旧文件中的符号
    pub fn unregister_file(&self, file_path: &str) {
        if let Some((_, file_sym)) = self.file_symbols.remove(file_path) {
            // 清除该文件中的类
            for class_name in &file_sym.class_names {
                self.classes.remove(class_name);
                self.stats.class_count.fetch_sub(1, Ordering::Relaxed);
            }
            // 清除该文件中的接口
            for iface_name in &file_sym.interface_names {
                self.interfaces.remove(iface_name);
            }
            // 清除该文件中的 trait
            for trait_name in &file_sym.trait_names {
                self.traits.remove(trait_name);
            }
            // 清除该文件中的枚举
            for enum_name in &file_sym.enum_names {
                self.enums.remove(enum_name);
            }
            // 清除该文件中的函数
            for func_name in &file_sym.function_names {
                self.functions.remove(func_name);
                self.stats.function_count.fetch_sub(1, Ordering::Relaxed);
            }
            // 清除该文件中的常量
            for const_name in &file_sym.constant_names {
                self.constants.remove(const_name);
                self.stats.constant_count.fetch_sub(1, Ordering::Relaxed);
            }
            // 清除命名空间
            if let Some(ns_name) = &file_sym.namespace {
                self.namespaces.remove(ns_name);
                self.stats.namespace_count.fetch_sub(1, Ordering::Relaxed);
            }
            self.stats.file_count.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// 查找类定义
    /// 支持完整类名查询（含命名空间）
    pub fn find_class(&self, full_name: &str) -> Option<ClassDef> {
        self.classes.get(full_name).map(|r| r.value().clone())
    }

    /// 模糊查找类定义
    /// 支持短类名查询，自动在所有命名空间中搜索
    /// 如查找 "Index" 会匹配 "app\\controller\\Index"
    pub fn find_class_fuzzy(&self, short_name: &str) -> Vec<ClassDef> {
        let mut results = Vec::new();
        for entry in self.classes.iter() {
            let class = entry.value();
            if class.name == short_name || class.full_name.ends_with(&format!("\\{}", short_name)) {
                results.push(class.clone());
            }
        }
        results
    }

    /// 查找函数定义
    pub fn find_function(&self, full_name: &str) -> Option<FuncDef> {
        self.functions.get(full_name).map(|r| r.value().clone())
    }

    /// 查找常量定义
    pub fn find_constant(&self, full_name: &str) -> Option<ConstDef> {
        self.constants.get(full_name).map(|r| r.value().clone())
    }

    /// 查找命名空间定义
    pub fn find_namespace(&self, name: &str) -> Option<NamespaceDef> {
        self.namespaces.get(name).map(|r| r.value().clone())
    }

    /// 查找接口定义
    /// 支持完整接口名查询（含命名空间）
    pub fn find_interface(&self, full_name: &str) -> Option<InterfaceDef> {
        self.interfaces.get(full_name).map(|r| r.value().clone())
    }

    /// 模糊查找接口定义
    /// 支持短接口名查询，自动在所有命名空间中搜索
    pub fn find_interface_fuzzy(&self, short_name: &str) -> Vec<InterfaceDef> {
        let mut results = Vec::new();
        for entry in self.interfaces.iter() {
            let iface = entry.value();
            if iface.name == short_name || iface.full_name.ends_with(&format!("\\{}", short_name)) {
                results.push(iface.clone());
            }
        }
        results
    }

    /// 查找 trait 定义
    /// 支持完整 trait 名查询（含命名空间）
    pub fn find_trait(&self, full_name: &str) -> Option<TraitDef> {
        self.traits.get(full_name).map(|r| r.value().clone())
    }

    /// 模糊查找 trait 定义
    /// 支持短 trait 名查询
    pub fn find_trait_fuzzy(&self, short_name: &str) -> Vec<TraitDef> {
        let mut results = Vec::new();
        for entry in self.traits.iter() {
            let tr = entry.value();
            if tr.name == short_name || tr.full_name.ends_with(&format!("\\{}", short_name)) {
                results.push(tr.clone());
            }
        }
        results
    }

    /// 查找枚举定义
    /// 支持完整枚举名查询（含命名空间）
    pub fn find_enum(&self, full_name: &str) -> Option<EnumDef> {
        self.enums.get(full_name).map(|r| r.value().clone())
    }

    /// 模糊查找枚举定义
    /// 支持短枚举名查询
    pub fn find_enum_fuzzy(&self, short_name: &str) -> Vec<EnumDef> {
        let mut results = Vec::new();
        for entry in self.enums.iter() {
            let en = entry.value();
            if en.name == short_name || en.full_name.ends_with(&format!("\\{}", short_name)) {
                results.push(en.clone());
            }
        }
        results
    }

    /// 获取所有控制器类
    /// 返回命名空间中包含 "controller" 的所有类
    pub fn get_controllers(&self) -> Vec<ClassDef> {
        self.classes
            .iter()
            .filter(|e| e.value().is_controller())
            .map(|e| e.value().clone())
            .collect()
    }

    /// 获取所有模型类
    /// 返回命名空间中包含 "model" 的所有类
    pub fn get_models(&self) -> Vec<ClassDef> {
        self.classes
            .iter()
            .filter(|e| e.value().is_model())
            .map(|e| e.value().clone())
            .collect()
    }

    /// 获取所有中间件类
    /// 返回命名空间中包含 "middleware" 的所有类
    pub fn get_middlewares(&self) -> Vec<ClassDef> {
        self.classes
            .iter()
            .filter(|e| e.value().is_middleware())
            .map(|e| e.value().clone())
            .collect()
    }

    /// 获取所有监听器类
    pub fn get_listeners(&self) -> Vec<ClassDef> {
        self.classes
            .iter()
            .filter(|e| e.value().is_listener())
            .map(|e| e.value().clone())
            .collect()
    }

    /// 获取所有类名列表
    pub fn all_class_names(&self) -> Vec<String> {
        self.classes.iter().map(|e| e.key().clone()).collect()
    }

    /// 获取所有函数名列表
    pub fn all_function_names(&self) -> Vec<String> {
        self.functions.iter().map(|e| e.key().clone()).collect()
    }

    /// 获取符号表快照
    /// 用于显示当前符号表状态
    pub fn snapshot(&self) -> SymbolSnapshot {
        let controllers = self.get_controllers();
        let models = self.get_models();
        let middlewares = self.get_middlewares();

        SymbolSnapshot {
            class_count: self.stats.class_count.load(Ordering::Relaxed),
            function_count: self.stats.function_count.load(Ordering::Relaxed),
            constant_count: self.stats.constant_count.load(Ordering::Relaxed),
            namespace_count: self.stats.namespace_count.load(Ordering::Relaxed),
            file_count: self.stats.file_count.load(Ordering::Relaxed),
            controllers: controllers.iter().map(|c| c.full_name.clone()).collect(),
            models: models.iter().map(|m| m.full_name.clone()).collect(),
            middlewares: middlewares.iter().map(|m| m.full_name.clone()).collect(),
        }
    }

    /// 清空符号表
    /// 用于全量热重载场景
    pub fn clear(&self) {
        self.classes.clear();
        self.functions.clear();
        self.constants.clear();
        self.namespaces.clear();
        self.interfaces.clear();
        self.traits.clear();
        self.enums.clear();
        self.file_symbols.clear();
        self.stats.file_count.store(0, Ordering::Relaxed);
        self.stats.class_count.store(0, Ordering::Relaxed);
        self.stats.function_count.store(0, Ordering::Relaxed);
        self.stats.constant_count.store(0, Ordering::Relaxed);
        self.stats.namespace_count.store(0, Ordering::Relaxed);
    }

    /// 获取已注册的类数量
    pub fn class_count(&self) -> usize {
        self.stats.class_count.load(Ordering::Relaxed)
    }

    /// 获取已注册的函数数量
    pub fn function_count(&self) -> usize {
        self.stats.function_count.load(Ordering::Relaxed)
    }

    /// 获取已解析的文件数量
    pub fn file_count(&self) -> usize {
        self.stats.file_count.load(Ordering::Relaxed)
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
