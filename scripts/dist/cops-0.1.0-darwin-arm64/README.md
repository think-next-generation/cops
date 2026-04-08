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
