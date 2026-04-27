# OYTAPHP 快速开始指南

## 使用方法

### 方式一：直接运行（开发环境）

```bash
# 在项目根目录下
./oyta --version
./oyta-cli --version

# 启动开发服务器
./oyta run

# 创建控制器
./oyta-cli make:controller User
```

### 方式二：添加到 PATH（推荐）

```bash
# 临时添加到 PATH（当前终端会话有效）
export PATH="$PWD/vendor/bin:$PATH"

# 永久添加到 PATH（Linux/Mac）
echo 'export PATH="$HOME/your-project/vendor/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# 永久添加到 PATH（Windows PowerShell）
Add-Content -Path $PROFILE -Value '$env:PATH += ";$PSScriptRoot\vendor\bin"'

# 现在可以直接使用
oyta --version
oyta-cli --version
oyta run
```

### 方式三：全局安装（所有项目可用）

```bash
# 全局安装 OYTAPHP
composer global require oyta/framework

# 添加全局 bin 目录到 PATH
# Linux/Mac: ~/.composer/vendor/bin
# Windows: %USERPROFILE%\AppData\Roaming\Composer\vendor\bin

export PATH="$HOME/.composer/vendor/bin:$PATH"

# 现在可以在任何地方使用
oyta --version
```

## 命令说明

### oyta 命令

```bash
oyta --version      # 查看版本
oyta run            # 启动开发服务器（默认端口 8000）
oyta run -p 8080    # 指定端口
oyta build          # 构建生产版本
oyta --help         # 查看帮助
```

### oyta-cli 命令

```bash
oyta-cli --version              # 查看版本
oyta-cli new myapp              # 创建新项目
oyta-cli make:controller User   # 创建控制器
oyta-cli make:model User        # 创建模型
oyta-cli make:middleware Auth   # 创建中间件
oyta-cli --help                 # 查看帮助
```

## 目录结构

```
myapp/
├── app/                    # 应用目录
│   ├── controller/         # 控制器
│   ├── model/              # 模型
│   ├── middleware/         # 中间件
│   └── common.php          # 公共函数
├── config/                 # 配置文件
├── route/                  # 路由定义
├── public/                 # WEB 目录
├── view/                   # 视图模板
├── vendor/
│   ├── oyta_php/
│   │   └── bin/            # 多平台二进制
│   └── bin/                # Composer 全局可执行文件
│       ├── oyta            → ../oyta_php/bin/linux-x86_64/oyta
│       └── oyta-cli        → ../oyta_php/bin/linux-x86_64/oyta-cli
├── oyta                    # 启动脚本
├── oyta-cli                # CLI 脚本
└── composer.json
```

## 注意事项

1. **首次使用**：建议先将 `vendor/bin` 添加到 PATH
2. **Windows 用户**：使用 `.bat` 文件或确保 Git Bash 在 PATH 中
3. **权限问题**：执行 `chmod +x oyta oyta-cli` 添加执行权限
4. **多项目开发**：建议使用全局安装方式
