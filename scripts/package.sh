#!/bin/bash
# package.sh - 打包发布

set -e

DIST_DIR="$(dirname "$0")/dist"
VERSION=$(grep -m1 'version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
PACKAGE_NAME="cops-$VERSION-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"

echo "=== 打包 COPS $VERSION ==="

# 清理并创建 dist 目录
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/$PACKAGE_NAME"

# 1. 复制可执行文件
echo "复制可执行文件..."
if [ ! -f "target/release/cops" ]; then
    echo "错误: 未找到可执行文件，请先运行 ./scripts/build_release.sh"
    exit 1
fi
cp target/release/cops "$DIST_DIR/$PACKAGE_NAME/"

# 2. 复制配置文件模板
echo "复制配置文件..."
cp cops.toml.example "$DIST_DIR/$PACKAGE_NAME/"

# 3. 复制安装脚本
echo "复制安装脚本..."
cp scripts/install.sh "$DIST_DIR/$PACKAGE_NAME/"

# 4. 复制数据库初始化 SQL
echo "复制数据库脚本..."
mkdir -p "$DIST_DIR/$PACKAGE_NAME/sql"
cp src/db/migrations/001_initial.sql "$DIST_DIR/$PACKAGE_NAME/sql/"

# 5. 创建 README
cat > "$DIST_DIR/$PACKAGE_NAME/README.md" << 'EOF'
# COPS - Company Operations Task System

## 快速开始

### 1. 安装

```bash
# 解压并运行安装脚本
tar -xzf cops-*.tar.gz
cd cops-*/
./install.sh
```

### 2. 启动

```bash
# 默认启动 web 界面 (http://127.0.0.1:9090)
~/.cops/cops web

# 或使用 CLI
~/.cops/cops --help
```

### 3. 配置

配置文件位于: `~/.cops/cops.toml`

主要配置项:
- `database.sqlite_path` - 数据库路径
- `server.host` / `server.port` - 服务地址

## 升级

重新运行安装脚本即可升级（数据库会自动迁移）:
```bash
./install.sh
```

## 数据备份

只需备份数据目录:
```bash
cp ~/.cops/cops.db backup-cops-$(date +%Y%m%d).db
```
EOF

# 6. 创建 tar.gz
echo "创建压缩包..."
cd "$DIST_DIR"
tar -czf "${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME"

# 显示结果
echo ""
echo "=== 打包完成 ==="
ls -lh "${PACKAGE_NAME}.tar.gz"
echo ""
echo "分发文件: $DIST_DIR/${PACKAGE_NAME}.tar.gz"