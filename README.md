# COPS - Company Operations Task System

任务管理系统，基于 Rust + SQLite 构建。

## 功能特性

- **任务管理**: 创建、分配、跟踪任务状态
- **看板视图**: 拖拽式任务看板 (NEW → ASSIGNED → IN_PROGRESS → REVIEW → DONE)
- **问答系统**: 任务相关的问答投票
- **评论系统**: 任务评论和讨论
- **实时更新**: WebSocket 实时推送
- **CLI 工具**: 完整的命令行界面
- **REST API**: 完整的 REST API

## 快速开始

### 安装

```bash
# 方式 1: 使用发布包
tar -xzf cops-*.tar.gz
cd cops-*/
./install.sh

# 方式 2: 源码编译
cargo build --release
./target/release/cops --help
```

### 启动

```bash
# 启动 Web 服务 (http://127.0.0.1:9090)
cops web

# 或使用 CLI
cops task list
cops board show
```

## 配置

配置文件: `~/.cops/cops.toml`

```toml
[database]
backend = "sqlite"
sqlite_path = "./data/cops.db"

[server]
host = "127.0.0.1"
port = 9090

[board]
default_columns = ["NEW", "ASSIGNED", "IN_PROGRESS", "BLOCKED", "REVIEW", "DONE"]
```

## CLI 命令

| 命令 | 描述 |
|------|------|
| `cops task` | 任务管理 |
| `cops question` | 问答管理 |
| `cops comment` | 评论管理 |
| `cops status` | 状态管理 |
| `cops board` | 看板视图 |
| `cops config` | 配置管理 |
| `cops db` | 数据库操作 |
| `cops web` | 启动 Web 服务 |

## API

- `GET /api/tasks` - 获取任务列表
- `POST /api/tasks` - 创建任务
- `GET /api/tasks/:id` - 获取任务详情
- `PUT /api/tasks/:id` - 更新任务
- `DELETE /api/tasks/:id` - 删除任务
- `WS /ws` - WebSocket 实时更新

## 架构

```
src/
├── main.rs          # 入口点
├── cli/             # CLI 命令
├── api/             # REST API
├── db/              # 数据库层
├── core/            # 核心模型
├── ws/              # WebSocket
└── frontend/       # 嵌入式前端
```

## 许可证

MIT