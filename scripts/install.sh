#!/bin/bash
# install.sh - 安装脚本

set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.cops}"
DATA_DIR="$INSTALL_DIR/data"
CONFIG_FILE="$INSTALL_DIR/cops.toml"

echo "=== COPS 安装脚本 ==="
echo "安装目录: $INSTALL_DIR"

# 创建目录
mkdir -p "$INSTALL_DIR"
mkdir -p "$DATA_DIR"

# 复制可执行文件
echo "复制可执行文件..."
cp "$(dirname "$0")/cops" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/cops"

# 复制或创建配置文件
if [ ! -f "$CONFIG_FILE" ]; then
    echo "创建默认配置..."
    cp "$(dirname "$0")/cops.toml.example" "$CONFIG_FILE"
    # 更新数据库路径
    sed -i '' "s|./data/cops.db|$DATA_DIR/cops.db|g" "$CONFIG_FILE"
    echo "配置文件: $CONFIG_FILE"
    echo "请根据需要编辑配置文件"
else
    echo "配置文件已存在: $CONFIG_FILE"
fi

# 配置 PATH
SHELL_RC=""
case "$(basename "$SHELL")" in
    bash) SHELL_RC="$HOME/.bashrc" ;;
    zsh)  SHELL_RC="$HOME/.zshrc" ;;
    *)    SHELL_RC="$HOME/.profile" ;;
esac

# 添加 PATH 配置
if ! grep -q "$INSTALL_DIR" "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo "# COPS" >> "$SHELL_RC"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
    echo "已添加 PATH 到 $SHELL_RC"
    echo "请运行: source $SHELL_RC"
else
    echo "PATH 已配置"
fi

# 初始化数据库
echo "初始化数据库..."
cd "$INSTALL_DIR"
./cops db migrate

echo ""
echo "=== 安装完成 ==="
echo "运行以下命令启动:"
echo "  $INSTALL_DIR/cops web"
echo ""
echo "或添加到 PATH:"
echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "数据目录: $DATA_DIR"