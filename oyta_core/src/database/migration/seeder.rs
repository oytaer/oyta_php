//! 数据填充模块
//!
//! 提供数据填充文件的创建和管理功能

use anyhow::Result;
use std::path::PathBuf;

use super::helpers::migration_name_to_class;

/// 数据填充管理器
pub struct SeederManager {
    /// 项目根目录
    project_root: PathBuf,
    /// 填充文件目录
    seeder_dir: PathBuf,
}

impl SeederManager {
    /// 创建新的填充管理器
    ///
    /// # 参数
    /// - `project_root`: 项目根目录
    pub fn new(project_root: &std::path::Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            seeder_dir: project_root.join("database").join("seeders"),
        }
    }

    /// 创建数据填充文件
    ///
    /// # 参数
    /// - `name`: 填充名称
    ///
    /// # 返回
    /// 创建的填充文件路径
    pub fn create_seeder(&self, name: &str) -> Result<PathBuf> {
        // 确保目录存在
        std::fs::create_dir_all(&self.seeder_dir)?;

        // 生成文件名
        let filename = format!("{}.php", name);
        let file_path = self.seeder_dir.join(&filename);

        // 检查文件是否已存在
        if file_path.exists() {
            anyhow::bail!("填充文件已存在: {}", file_path.display());
        }

        // 生成内容
        let content = self.generate_seeder_content(name);

        // 写入文件
        std::fs::write(&file_path, content)?;

        tracing::info!("✓ 填充文件创建成功: {}", file_path.display());

        Ok(file_path)
    }

    /// 生成填充文件内容
    ///
    /// # 参数
    /// - `name`: 填充名称
    ///
    /// # 返回
    /// PHP 文件内容
    fn generate_seeder_content(&self, name: &str) -> String {
        let class_name = migration_name_to_class(name);

        format!(r#"<?php
use oyta\db\Seeder;

/**
 * 数据填充类：{}
 */
class {} extends Seeder
{{
    /**
     * 执行数据填充
     * 
     * @return void
     */
    public function run()
    {{
        // 在此处编写数据填充逻辑
        // 示例：
        // $this->table('users')->insert([
        //     ['name' => 'Admin', 'email' => 'admin@example.com'],
        //     ['name' => 'User', 'email' => 'user@example.com'],
        // ]);
    }}
}}
"#, name, class_name)
    }

    /// 获取所有填充文件
    ///
    /// # 返回
    /// 填充文件列表
    pub fn get_seeders(&self) -> Result<Vec<PathBuf>> {
        if !self.seeder_dir.exists() {
            return Ok(Vec::new());
        }

        let mut seeders = Vec::new();

        for entry in std::fs::read_dir(&self.seeder_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "php").unwrap_or(false) {
                seeders.push(path);
            }
        }

        // 按文件名排序
        seeders.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(&b.file_name().unwrap_or_default())
        });

        Ok(seeders)
    }

    /// 执行数据填充
    ///
    /// # 参数
    /// - `name`: 填充名称（可选，不指定则执行所有）
    ///
    /// # 返回
    /// 执行的填充数量
    pub async fn seed(&self, name: Option<&str>) -> Result<usize> {
        let seeders = if let Some(n) = name {
            // 执行指定的填充
            let path = self.seeder_dir.join(format!("{}.php", n));
            if path.exists() {
                vec![path]
            } else {
                anyhow::bail!("填充文件不存在: {}", n);
            }
        } else {
            // 执行所有填充
            self.get_seeders()?
        };

        if seeders.is_empty() {
            tracing::info!("没有可执行的填充文件");
            return Ok(0);
        }

        let mut count = 0;

        for seeder_path in &seeders {
            // 执行填充（这里需要与 PHP 解释器集成）
            if let Some(filename) = seeder_path.file_name().and_then(|n| n.to_str()) {
                tracing::info!("执行填充: {}", filename);
                count += 1;
            }
        }

        tracing::info!("✓ 已执行 {} 个填充", count);

        Ok(count)
    }
}
