# OYTAPHP Windows 安装脚本
#
# 使用方法:
#   irm https://oyta.dev/install.ps1 | iex
#   或
#   irm https://oyta.dev/install.ps1 | iex -Version 1.0.0
#

param(
    [string]$Version = "latest",
    [string]$InstallDir = "",
    [switch]$Help,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# 默认安装目录
if ([string]::IsNullOrEmpty($InstallDir)) {
    $InstallDir = "$env:LOCALAPPDATA\oyta"
}

$RepoUrl = "https://github.com/oytaphp/oyta/releases"

function Write-Info($message) {
    Write-Host "[INFO] " -ForegroundColor Blue -NoNewline
    Write-Host $message
}

function Write-Success($message) {
    Write-Host "[SUCCESS] " -ForegroundColor Green -NoNewline
    Write-Host $message
}

function Write-Warn($message) {
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $message
}

function Write-Err($message) {
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $message
    exit 1
}

function Show-Help {
    Write-Host "OYTAPHP Windows 安装脚本"
    Write-Host ""
    Write-Host "使用方法: irm https://oyta.dev/install.ps1 | iex"
    Write-Host ""
    Write-Host "参数:"
    Write-Host "  -Version VERSION   指定版本 (默认: latest)"
    Write-Host "  -InstallDir DIR    安装目录 (默认: `$env:LOCALAPPDATA\oyta)"
    Write-Host "  -Uninstall         卸载 oyta"
    Write-Host "  -Help              显示帮助信息"
    exit 0
}

function Get-Platform {
    $os = "windows"
    $arch = $env:PROCESSOR_ARCHITECTURE
    
    switch ($arch) {
        "AMD64" { $arch = "x64" }
        "ARM64" { $arch = "arm64" }
        default { $arch = "x64" }
    }
    
    return "$os-$arch"
}

function Test-OytaInstalled {
    $oytaPath = Join-Path $InstallDir "oyta.exe"
    return Test-Path $oytaPath
}

function Uninstall-Oyta {
    $oytaPath = Join-Path $InstallDir "oyta.exe"
    
    if (Test-Path $oytaPath) {
        Write-Info "卸载 oyta..."
        Remove-Item $oytaPath -Force
        Write-Success "oyta 已卸载"
    } else {
        Write-Warn "oyta 未安装在 $InstallDir"
    }
    
    # 从 PATH 中移除
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -like "*$InstallDir*") {
        $newPath = $userPath -replace [regex]::Escape(";$InstallDir"), ""
        $newPath = $newPath -replace [regex]::Escape("$InstallDir;"), ""
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Info "已从 PATH 中移除 $InstallDir"
    }
}

function Install-Oyta {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host "  OYTAPHP 安装程序"
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    
    Write-Info "安装版本: $Version"
    Write-Info "安装目录: $InstallDir"
    
    # 检测平台
    $platform = Get-Platform
    Write-Info "检测到平台: $platform"
    
    # 构建下载 URL
    $downloadUrl = if ($Version -eq "latest") {
        "$RepoUrl/latest/download/oyta-$platform.exe"
    } else {
        "$RepoUrl/download/v$Version/oyta-$platform.exe"
    }
    
    Write-Info "下载地址: $downloadUrl"
    
    # 创建安装目录
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # 下载文件
    $targetPath = Join-Path $InstallDir "oyta.exe"
    
    try {
        Write-Info "正在下载..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath -UseBasicParsing
    } catch {
        Write-Err "下载失败: $_"
    }
    
    # 添加到 PATH
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "添加到 PATH..."
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$InstallDir", "User")
    }
    
    # 验证安装
    Write-Success "oyta 已安装到: $targetPath"
    
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host "  OYTAPHP 已安装成功!"
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    Write-Host "  请重新打开终端窗口，然后运行:"
    Write-Host ""
    Write-Host "    oyta --version"
    Write-Host ""
    Write-Host "  快速开始:"
    Write-Host ""
    Write-Host "    oyta new myapp"
    Write-Host "    cd myapp"
    Write-Host "    oyta run"
    Write-Host ""
}

# 主逻辑
if ($Help) {
    Show-Help
}

if ($Uninstall) {
    Uninstall-Oyta
    exit 0
}

Install-Oyta
