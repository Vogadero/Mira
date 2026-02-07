#!/bin/bash
# Mira 发布构建脚本 (macOS/Linux)
# 用于创建优化的发布版本并验证大小要求

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 参数解析
CLEAN=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "未知参数: $1"
            echo "用法: $0 [--clean] [--verbose]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Mira 发布构建脚本${NC}"
echo -e "${GREEN}===================${NC}"

# 获取项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${CYAN}项目根目录: $PROJECT_ROOT${NC}"

# 清理之前的构建（如果需要）
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}清理之前的构建...${NC}"
    if [ -d "target" ]; then
        rm -rf target
    fi
fi

# 检查 Rust 工具链
echo -e "${CYAN}检查 Rust 工具链...${NC}"
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}错误: Rust 工具链未安装${NC}"
    exit 1
fi

RUST_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version)
echo -e "${GREEN}Rust 版本: $RUST_VERSION${NC}"
echo -e "${GREEN}Cargo 版本: $CARGO_VERSION${NC}"

# 检测操作系统
OS=$(uname -s)
case $OS in
    Darwin)
        PLATFORM="macos"
        BINARY_NAME="mira"
        MAX_SIZE_MB=25
        ;;
    Linux)
        PLATFORM="linux"
        BINARY_NAME="mira"
        MAX_SIZE_MB=25
        ;;
    *)
        echo -e "${RED}不支持的操作系统: $OS${NC}"
        exit 1
        ;;
esac

echo -e "${CYAN}目标平台: $PLATFORM${NC}"

# 构建发布版本
echo -e "${CYAN}开始构建发布版本...${NC}"
echo -e "${YELLOW}注意: macOS/Linux 平台不需要控制台隐藏配置${NC}"
echo -e "${YELLOW}      控制台隐藏功能仅适用于 Windows 平台${NC}"
BUILD_START=$(date +%s)

if [ "$VERBOSE" = true ]; then
    cargo build --release --verbose
else
    cargo build --release
fi

BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))
echo -e "${GREEN}构建完成，耗时: ${BUILD_TIME} 秒${NC}"

# 检查二进制文件
BINARY_PATH="target/release/$BINARY_NAME"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}错误: 未找到构建的二进制文件: $BINARY_PATH${NC}"
    exit 1
fi

# 获取文件大小
FILE_SIZE_BYTES=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE_BYTES / 1024 / 1024" | bc)

echo -e "${CYAN}二进制文件信息:${NC}"
echo -e "  路径: $BINARY_PATH"
echo -e "  大小: ${FILE_SIZE_MB} MB"
echo -e "  创建时间: $(stat -f%Sm "$BINARY_PATH" 2>/dev/null || stat -c%y "$BINARY_PATH" 2>/dev/null)"

# 验证大小要求
if (( $(echo "$FILE_SIZE_MB <= $MAX_SIZE_MB" | bc -l) )); then
    echo -e "${GREEN}✓ 二进制文件大小符合要求 (${FILE_SIZE_MB} MB <= $MAX_SIZE_MB MB)${NC}"
else
    echo -e "${YELLOW}⚠ 二进制文件大小超过要求 (${FILE_SIZE_MB} MB > $MAX_SIZE_MB MB)${NC}"
fi

# 检查依赖项
echo -e "${CYAN}检查依赖项...${NC}"
if command -v otool &> /dev/null && [ "$PLATFORM" = "macos" ]; then
    echo -e "${YELLOW}依赖的动态库:${NC}"
    otool -L "$BINARY_PATH" | tail -n +2 | while read -r line; do
        echo -e "  $line"
    done
elif command -v ldd &> /dev/null && [ "$PLATFORM" = "linux" ]; then
    echo -e "${YELLOW}依赖的动态库:${NC}"
    ldd "$BINARY_PATH" | while read -r line; do
        echo -e "  $line"
    done
else
    echo -e "${YELLOW}无法检查依赖项${NC}"
fi

# 创建分发目录
DIST_DIR="dist"
if [ -d "$DIST_DIR" ]; then
    rm -rf "$DIST_DIR"
fi
mkdir -p "$DIST_DIR"

echo -e "${CYAN}创建分发包...${NC}"

# 复制二进制文件
cp "$BINARY_PATH" "$DIST_DIR/$BINARY_NAME"

# 复制文档文件
DOC_FILES=("README.md" "LICENSE")
for doc_file in "${DOC_FILES[@]}"; do
    if [ -f "$doc_file" ]; then
        cp "$doc_file" "$DIST_DIR/"
    fi
done

# 创建版本信息文件
VERSION_INFO="Mira - 桌面摄像精灵
版本: $(cargo pkgid | cut -d'#' -f2)
构建时间: $(date '+%Y-%m-%d %H:%M:%S')
构建机器: $(hostname)
操作系统: $OS
Rust 版本: $RUST_VERSION
二进制大小: ${FILE_SIZE_MB} MB

系统要求:
- $PLATFORM
- 支持的显卡驱动
- 至少一个摄像头设备

使用说明:
1. 运行 ./$BINARY_NAME 启动应用程序
2. 左键拖拽移动窗口
3. 鼠标滚轮缩放窗口
4. Ctrl + 鼠标滚轮旋转窗口
5. F1-F5 切换形状
6. Tab 切换摄像头设备"

echo "$VERSION_INFO" > "$DIST_DIR/VERSION.txt"

# 计算分发包总大小
DIST_SIZE_BYTES=$(du -sb "$DIST_DIR" | cut -f1)
DIST_SIZE_MB=$(echo "scale=2; $DIST_SIZE_BYTES / 1024 / 1024" | bc)
FILE_COUNT=$(find "$DIST_DIR" -type f | wc -l)

echo -e "${CYAN}分发包信息:${NC}"
echo -e "  目录: $DIST_DIR"
echo -e "  总大小: ${DIST_SIZE_MB} MB"
echo -e "  文件数量: $FILE_COUNT"

# 验证分发包大小
MAX_DIST_SIZE=50
if (( $(echo "$DIST_SIZE_MB <= $MAX_DIST_SIZE" | bc -l) )); then
    echo -e "${GREEN}✓ 分发包大小符合要求 (${DIST_SIZE_MB} MB <= $MAX_DIST_SIZE MB)${NC}"
else
    echo -e "${YELLOW}⚠ 分发包大小超过要求 (${DIST_SIZE_MB} MB > $MAX_DIST_SIZE MB)${NC}"
fi

# 创建 tar.gz 压缩包
ARCHIVE_NAME="mira-${PLATFORM}-x64.tar.gz"
if [ -f "$ARCHIVE_NAME" ]; then
    rm "$ARCHIVE_NAME"
fi

echo -e "${CYAN}创建 tar.gz 压缩包...${NC}"
tar -czf "$ARCHIVE_NAME" -C "$DIST_DIR" .

ARCHIVE_SIZE_BYTES=$(stat -f%z "$ARCHIVE_NAME" 2>/dev/null || stat -c%s "$ARCHIVE_NAME" 2>/dev/null)
ARCHIVE_SIZE_MB=$(echo "scale=2; $ARCHIVE_SIZE_BYTES / 1024 / 1024" | bc)

echo -e "${GREEN}✓ tar.gz 压缩包创建成功:${NC}"
echo -e "  文件: $ARCHIVE_NAME"
echo -e "  大小: ${ARCHIVE_SIZE_MB} MB"

# 验证压缩包大小
if (( $(echo "$ARCHIVE_SIZE_MB <= $MAX_SIZE_MB" | bc -l) )); then
    echo -e "${GREEN}✓ 压缩包大小符合下载要求 (${ARCHIVE_SIZE_MB} MB <= $MAX_SIZE_MB MB)${NC}"
else
    echo -e "${YELLOW}⚠ 压缩包大小可能影响下载体验 (${ARCHIVE_SIZE_MB} MB > $MAX_SIZE_MB MB)${NC}"
fi

# 运行基本测试（如果可能）
echo -e "${CYAN}运行基本验证...${NC}"
if "$BINARY_PATH" --version &>/dev/null; then
    VERSION_OUTPUT=$("$BINARY_PATH" --version 2>/dev/null || echo "无版本信息")
    echo -e "${GREEN}✓ 应用程序可以正常启动${NC}"
    echo -e "  版本信息: $VERSION_OUTPUT"
else
    echo -e "${YELLOW}⚠ 应用程序启动测试失败（可能需要图形环境）${NC}"
fi

echo ""
echo -e "${GREEN}构建摘要:${NC}"
echo -e "${GREEN}=========${NC}"
echo -e "${GREEN}✓ 发布构建完成${NC}"
echo -e "${GREEN}✓ 二进制文件: ${FILE_SIZE_MB} MB${NC}"
echo -e "${GREEN}✓ 分发包: ${DIST_SIZE_MB} MB${NC}"
echo -e "${GREEN}✓ 压缩包: ${ARCHIVE_SIZE_MB} MB${NC}"
echo ""
echo -e "${CYAN}文件位置:${NC}"
echo -e "  二进制: $BINARY_PATH"
echo -e "  分发目录: $DIST_DIR"
echo -e "  压缩包: $ARCHIVE_NAME"
echo ""
echo -e "${GREEN}发布构建完成！${NC}"