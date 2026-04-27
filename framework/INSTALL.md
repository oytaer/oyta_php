# OYTAPHP 安装说明

## 方式一：通过 Composer 创建项目（推荐）

```bash
# 创建新项目
composer create-project oyta/framework myapp

# 进入项目目录
cd myapp

# 此时 oyta 和 oyta-cli 已经可以通过 vendor/bin 访问
vendor/bin/oyta --version
vendor/bin/oyta-cli --version

# 或者将 vendor/bin 添加到 PATH
export PATH="$PWD/vendor/bin:$PATH"

# 现在可以直接使用
oyta --version
oyta-cli --version
```

## 方式二：全局安装（一次性安装，所有项目可用）

```bash
# 全局安装 OYTAPHP
composer global require oyta/framework

# 将 Composer 全局 bin 目录添加到 PATH
# Linux/Mac: 添加到 ~/.bashrc 或 ~/.zshrc
export PATH="$HOME/.composer/vendor/bin:$PATH"

# Windows: 添加到系统环境变量
# %USERPROFILE%\AppData\Roaming\Composer\vendor\bin

# 现在可以在任何地方使用
oyta --version
oyta-cli --version
```

## 方式三：手动添加到 PATH

```bash
# 将框架目录添加到 PATH
export PATH="/root/oyta_php/framework:$PATH"

# 添加到 ~/.bashrc 永久生效
echo 'export PATH="/root/oyta_php/framework:$PATH"' >> ~/.bashrc
source ~/.bashrc

# 现在可以直接使用
oyta --version
```

## 验证安装

```bash
# 查看版本
oyta --version
oyta-cli --version

# 启动开发服务器
oyta run

# 创建新控制器
oyta-cli make:controller User
```

## 目录结构说明

```
myapp/
├── oyta              # 启动脚本（Linux/Mac）
├── oyta-cli          # CLI 工具（Linux/Mac）
├── oyta.bat          # 启动脚本（Windows）
├── oyta-cli.bat      # CLI 工具（Windows）
├── vendor/
│   └── oyta_php/
│       └── bin/      # 多平台二进制文件
│           ├── linux-x86_64/
│           ├── linux-aarch64/
│           └── windows-x86_64/
└── vendor/bin/       # Composer 全局可执行文件目录
    ├── oyta -> ../oyta_php/bin/linux-x86_64/oyta
    └── oyta-cli -> ../oyta_php/bin/linux-x86_64/oyta-cli
```

## 注意事项

1. **Linux/Mac 用户**：使用 `oyta` 和 `oyta-cli` 脚本
2. **Windows 用户**：使用 `oyta.bat` 和 `oyta-cli.bat` 脚本
3. **权限问题**：如果遇到权限错误，请执行 `chmod +x oyta oyta-cli`
