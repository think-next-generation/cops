#!/bin/bash
# build_release.sh - 构建发布版本

set -e

echo "=== 构建 release 版本 ==="

cd "$(dirname "$0")"

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "错误: 未找到 Rust 环境，请先安装 Rust"
    exit 1
fi

# 构建 release 版本
echo "编译中..."
cargo build --release

# 检查是否成功
if [ ! -f "target/release/cops" ]; then
    echo "错误: 编译失败，未找到可执行文件"
    exit 1
fi

# 显示文件大小
size=$(du -h target/release/cops | cut -f1)
echo "编译完成: target/release/cops ($size)"

echo "=== 构建完成 ==="