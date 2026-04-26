//! PSR-4 自动加载生成器
//!
//! 生成 Composer 兼容的自动加载文件
//! 支持 PSR-4、PSR-0、classmap、files

use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use super::LockedPackage;

/// 自动加载生成器
pub struct AutoloadGenerator {
    /// 项目路径
    project_path: String,
    /// 类映射
    class_map: HashMap<String, String>,
}

impl AutoloadGenerator {
    /// 创建新的自动加载生成器
    pub fn new(project_path: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            class_map: HashMap::new(),
        }
    }

    /// 生成自动加载文件
    ///
    /// # 参数
    /// - `packages`: 已安装的包列表
    pub async fn generate(&mut self, packages: &[LockedPackage]) -> Result<()> {
        tracing::info!("开始生成自动加载文件...");

        // 扫描所有包的类
        for package in packages {
            self.scan_package_classes(package).await?;
        }

        // 生成 autoload.php
        self.generate_autoload_file().await?;

        // 生成 autoload_classmap.php
        self.generate_classmap_file().await?;

        // 生成 autoload_psr4.php
        self.generate_psr4_file(packages).await?;

        // 生成 autoload_static.php
        self.generate_static_file().await?;

        tracing::info!("自动加载文件生成完成，共 {} 个类", self.class_map.len());

        Ok(())
    }

    /// 扫描包中的类
    async fn scan_package_classes(&mut self, package: &LockedPackage) -> Result<()> {
        let package_path = Path::new(&self.project_path).join(&package.install_path);

        if !package_path.exists() {
            return Ok(());
        }

        // 扫描 PSR-4 目录
        for (namespace, dirs) in &package.autoload.psr_4 {
            for dir in dirs {
                let full_dir = package_path.join(dir);
                if full_dir.exists() {
                    self.scan_directory(&full_dir, namespace).await?;
                }
            }
        }

        // 扫描 classmap
        for path in &package.autoload.classmap {
            let full_path = package_path.join(path);
            if full_path.exists() {
                if full_path.is_dir() {
                    self.scan_directory(&full_path, "").await?;
                } else if path.ends_with(".php") {
                    self.scan_php_file(&full_path, "").await?;
                }
            }
        }

        Ok(())
    }

    /// 扫描目录中的 PHP 文件（使用 Box::pin 处理递归）
    fn scan_directory<'a>(
        &'a mut self,
        dir: &'a Path,
        namespace: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    let sub_namespace = if namespace.is_empty() {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string()
                    } else {
                        format!("{}\\{}",
                            namespace,
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                        )
                    };
                    self.scan_directory(&path, &sub_namespace).await?;
                } else if path.extension().map(|e| e == "php").unwrap_or(false) {
                    self.scan_php_file(&path, namespace).await?;
                }
            }

            Ok(())
        })
    }

    /// 扫描单个 PHP 文件
    async fn scan_php_file(&mut self, file_path: &Path, namespace: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;

        // 简单的类名提取（不使用完整解析器）
        let class_name = extract_class_name(&content, namespace);

        if let Some(full_class_name) = class_name {
            let relative_path = file_path.strip_prefix(&self.project_path)
                .unwrap_or(file_path);

            self.class_map.insert(
                full_class_name,
                relative_path.to_string_lossy().to_string(),
            );
        }

        Ok(())
    }

    /// 生成 autoload.php
    async fn generate_autoload_file(&self) -> Result<()> {
        let content = r#"<?php
// 自动加载文件 - 由 OYTAPHP 生成

// 加载类映射
$vendorDir = dirname(__DIR__);
$baseDir = dirname($vendorDir);

// 加载类映射文件
include_once __DIR__ . '/autoload_classmap.php';

// 注册自动加载器
spl_autoload_register(function ($class) use ($vendorDir, $baseDir) {
    global $autoload_classmap;

    if (isset($autoload_classmap[$class])) {
        $file = $vendorDir . '/' . $autoload_classmap[$class];
        if (file_exists($file)) {
            require $file;
            return true;
        }
    }

    return false;
});

// 加载文件列表
$includeFiles = include_once __DIR__ . '/autoload_files.php';
foreach ($includeFiles as $file) {
    require $vendorDir . '/' . $file;
}

return true;
"#;

        let file_path = Path::new(&self.project_path)
            .join("vendor")
            .join("autoload.php");

        tokio::fs::write(&file_path, content).await?;

        Ok(())
    }

    /// 生成 autoload_classmap.php
    async fn generate_classmap_file(&self) -> Result<()> {
        let mut content = String::from("<?php\n// 类映射 - 由 OYTAPHP 生成\n\n$autoload_classmap = array(\n");

        for (class, path) in &self.class_map {
            content.push_str(&format!("    '{}' => '{}',\n", class, path));
        }

        content.push_str(");\n");

        let file_path = Path::new(&self.project_path)
            .join("vendor")
            .join("composer")
            .join("autoload_classmap.php");

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&file_path, content).await?;

        Ok(())
    }

    /// 生成 autoload_psr4.php
    async fn generate_psr4_file(&self, packages: &[LockedPackage]) -> Result<()> {
        let mut content = String::from("<?php\n// PSR-4 自动加载映射 - 由 OYTAPHP 生成\n\n$autoload_psr4 = array(\n");

        for package in packages {
            for (namespace, dirs) in &package.autoload.psr_4 {
                let paths: Vec<String> = dirs.iter()
                    .map(|d| format!("vendor/{}/{}", package.name, d))
                    .collect();
                content.push_str(&format!(
                    "    '{}' => array('{}'),\n",
                    namespace.trim_end_matches('\\'),
                    paths.join("', '")
                ));
            }
        }

        content.push_str(");\n");

        let file_path = Path::new(&self.project_path)
            .join("vendor")
            .join("composer")
            .join("autoload_psr4.php");

        tokio::fs::write(&file_path, content).await?;

        Ok(())
    }

    /// 生成 autoload_static.php
    async fn generate_static_file(&self) -> Result<()> {
        let mut content = String::from("<?php\n// 静态自动加载 - 由 OYTAPHP 生成\n\n");

        content.push_str("class ComposerAutoloaderInit {\n");
        content.push_str("    public static $classMap = array(\n");

        for (class, path) in &self.class_map {
            content.push_str(&format!("        '{}' => '{}',\n", class, path));
        }

        content.push_str("    );\n");
        content.push_str("}\n");

        let file_path = Path::new(&self.project_path)
            .join("vendor")
            .join("composer")
            .join("autoload_static.php");

        tokio::fs::write(&file_path, content).await?;

        Ok(())
    }
}

/// 从 PHP 文件内容中提取类名
fn extract_class_name(content: &str, namespace: &str) -> Option<String> {
    let mut class_name = None;
    let mut found_namespace = None;

    for line in content.lines() {
        let line = line.trim();

        // 提取命名空间
        if line.starts_with("namespace ") {
            found_namespace = line
                .trim_start_matches("namespace ")
                .trim_end_matches(';')
                .trim()
                .to_string()
                .into();
        }

        // 提取类名
        if line.starts_with("class ") || line.starts_with("interface ") || line.starts_with("trait ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                class_name = parts[1].to_string().into();
                break;
            }
        }
    }

    class_name.map(|name| {
        if let Some(ns) = found_namespace {
            format!("{}\\{}", ns, name)
        } else if !namespace.is_empty() {
            format!("{}\\{}", namespace, name)
        } else {
            name
        }
    })
}

/// 生成自动加载文件（入口函数）
pub async fn generate(project_path: &str, packages: &[LockedPackage]) -> Result<()> {
    let mut generator = AutoloadGenerator::new(project_path);
    generator.generate(packages).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_class_name() {
        let content = r#"
<?php
namespace App\Model;

class User {
    // ...
}
"#;

        let class_name = extract_class_name(content, "");
        assert_eq!(class_name, Some("App\\Model\\User".to_string()));
    }

    #[test]
    fn test_extract_class_name_no_namespace() {
        let content = r#"
<?php
class Helper {
    // ...
}
"#;

        let class_name = extract_class_name(content, "App");
        assert_eq!(class_name, Some("App\\Helper".to_string()));
    }
}
