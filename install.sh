#!/bin/bash
#
# OYTAPHP 安装脚本
# 
# 使用方法:
#   curl -fsSL https://oyta.dev/install.sh | bash
#   或
#   curl -fsSL https://oyta.dev/install.sh | bash -s -- --version 1.0.0
#
# 支持的平台:
#   - linux-x64
#   - linux-arm64
#   - darwin-x64 (macOS Intel)
#   - darwin-arm64 (macOS Apple Silicon)
#

set -e

# 默认配置
DEFAULT_VERSION="latest"
INSTALL_DIR="/usr/local/bin"
REPO_URL="https://github.com/oytaphp/oyta/releases"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# 检测操作系统
detect_os() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux)  echo "linux" ;;
        darwin) echo "darwin" ;;
        *)      error "不支持的操作系统: $os" ;;
    esac
}

# 检测 CPU 架构
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)  echo "x64" ;;
        aarch64|arm64) echo "arm64" ;;
        *)             error "不支持的 CPU 架构: $arch" ;;
    esac
}

# 解析命令行参数
parse_args() {
    VERSION="$DEFAULT_VERSION"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version|-v)
                VERSION="$2"
                shift 2
                ;;
            --dir|-d)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help|-h)
                echo "OYTAPHP 安装脚本"
                echo ""
                echo "使用方法: curl -fsSL https://oyta.dev/install.sh | bash"
                echo ""
                echo "选项:"
                echo "  --version, -v VERSION  指定版本 (默认: latest)"
                echo "  --dir, -d DIR          安装目录 (默认: /usr/local/bin)"
                echo "  --help, -h             显示帮助信息"
                exit 0
                ;;
            *)
                warn "未知参数: $1"
                shift
                ;;
        esac
    done
}

# 检查是否已安装
check_existing() {
    if command -v oyta &> /dev/null; then
        local current_version=$(oyta --version 2>/dev/null || echo "unknown")
        warn "检测到已安装的 oyta (版本: $current_version)"
        read -p "是否覆盖安装? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "安装已取消"
            exit 0
        fi
    fi
}

# 下载二进制文件
download_binary() {
    local os=$(detect_os)
    local arch=$(detect_arch)
    local platform="${os}-${arch}"
    
    info "检测到平台: $platform"
    
    # 确定下载 URL
    local download_url
    if [[ "$VERSION" == "latest" ]]; then
        download_url="${REPO_URL}/latest/download/oyta-${platform}"
    else
        download_url="${REPO_URL}/download/v${VERSION}/oyta-${platform}"
    fi
    
    info "下载地址: $download_url"
    
    # 创建临时文件
    local tmp_file=$(mktemp)
    
    # 下载
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "$tmp_file" || error "下载失败"
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "$tmp_file" || error "下载失败"
    else
        error "需要 curl 或 wget 来下载文件"
    fi
    
    # 设置可执行权限
    chmod +x "$tmp_file"
    
    echo "$tmp_file"
}

# 安装二进制文件
install_binary() {
    local tmp_file="$1"
    local target="${INSTALL_DIR}/oyta"
    
    info "安装目录: $INSTALL_DIR"
    
    # 检查安装目录是否存在
    if [[ ! -d "$INSTALL_DIR" ]]; then
        info "创建安装目录: $INSTALL_DIR"
        sudo mkdir -p "$INSTALL_DIR"
    fi
    
    # 检查是否有写入权限
    if [[ ! -w "$INSTALL_DIR" ]]; then
        info "需要管理员权限来安装到 $INSTALL_DIR"
        sudo mv "$tmp_file" "$target"
    else
        mv "$tmp_file" "$target"
    fi
    
    # 确保可执行
    chmod +x "$target"
    
    success "oyta 已安装到: $target"
}

# 验证安装
verify_installation() {
    if command -v oyta &> /dev/null; then
        local version=$(oyta --version 2>/dev/null || echo "unknown")
        success "安装成功! 版本: $version"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  OYTAPHP 已安装成功!"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "  快速开始:"
        echo ""
        echo "    # 创建新项目"
        echo "    oyta new myapp"
        echo "    cd myapp"
        echo ""
        echo "    # 启动服务器"
        echo "    oyta run"
        echo ""
        echo "  更多命令:"
        echo "    oyta --help"
        echo ""
    else
        warn "安装完成，但 oyta 未在 PATH 中找到"
        info "请确保 $INSTALL_DIR 在您的 PATH 中"
        info "可以运行: export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# 卸载函数
uninstall() {
    local target="${INSTALL_DIR}/oyta"
    
    if [[ -f "$target" ]]; then
        info "卸载 oyta..."
        if [[ ! -w "$INSTALL_DIR" ]]; then
            sudo rm -f "$target"
        else
            rm -f "$target"
        fi
        success "oyta 已卸载"
    else
        warn "oyta 未安装在 $INSTALL_DIR"
    fi
}

# 主函数
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  OYTAPHP 安装程序"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    parse_args "$@"
    
    info "安装版本: $VERSION"
    
    check_existing
    
    local tmp_file=$(download_binary)
    
    # 确保清理临时文件
    trap "rm -f $tmp_file" EXIT
    
    install_binary "$tmp_file"
    
    verify_installation
}

# 检查是否为卸载
if [[ "$1" == "--uninstall" ]] || [[ "$1" == "-u" ]]; then
    uninstall
    exit 0
fi

main "$@"
