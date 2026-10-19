# sdcore 能力说明

`sdcore` 是 Rust 工具型 Web 应用的共享 core crate。它提供一组偏基础设施的能力，帮助业务项目减少重复代码。

当前第一个主要消费方是 `sdhook`。

## 能力总览

| 模块 | 能力 | 说明 |
| --- | --- | --- |
| `app` | Web 服务启动 | Axum/Tokio 启动、信号退出与可等待的产品清理回调。 |
| `config` | TOML 配置读取 | 反序列化产品自有配置形状，错误不包含配置原文。 |
| `paths` | 原生平台路径 | 用户应用数据目录、显式路径/PATH 程序查找及 Windows .exe 处理。 |
| `http` | API 响应结构 | 提供 `ApiError`、`ApiResult`、`ApiMessage` 等统一返回辅助。 |
| `db` | 数据库连接 | 提供 SQLite 和 PostgreSQL 的连接抽象。 |
| `auth` | 密码与会话认证 | Argon2id 密码哈希、OS 随机会话令牌、Cookie/Header 解析与内存 Session。 |
| `sse` | SSE 推送 | topic 订阅、发布；可选有界队列、丢失通知与会话撤销检测。 |
| `exec` | Shell 执行 | 支持指定工作目录、环境变量、超时、stdout/stderr 捕获和事件流。 |
| `process` | 常驻子进程 | 异步启动、状态查询、输出订阅、优雅停止和 Unix 进程组清理。 |
| `reporter` | 事件上报 | 提供可组合的 reporter，用于日志、进度、记录更新等场景。 |
| `logger` | 本地日志 | 支持控制台和文件日志输出。 |
| `static_files` | 静态资源响应 | 提供静态资源响应和 JSON 404 辅助。 |
| `task` | 简单任务执行 | 提供同步 shell 命令执行辅助。 |

## Web 服务

`sdcore::app::serve` 封装了 axum 的监听和启动逻辑：

```rust
sdcore::app::serve("127.0.0.1:9000", router).await
```

`serve_with_shutdown(addr, router, cleanup_future)` 在收到退出信号后先执行并等待产品清理，再让 Axum 排空活动请求。产品清理需停止后台任务、关闭 SSE；否则长连接可能阻止优雅退出。数据库通常在 HTTP 请求排空之后关闭。业务项目负责给清理设定总预算。

## 配置读取

`sdcore::config::load_toml::<T>(&path)` 读取 TOML 并反序列化到调用方的 `Deserialize` 类型。配置字段、默认值、路径解析和产品校验由消费方定义；读取或解析失败的错误只包含文件路径，不回显配置内容。

业务项目仍然负责：

- 定义 router。
- 组织路由分组。
- 注入业务 state。
- 配置 middleware。

## 平台路径

`sdcore::paths::user_data_dir(application)` 返回用户应用数据目录，不创建目录：Linux 使用绝对 XDG_DATA_HOME 或 HOME/.local/share，macOS 使用 HOME/Library/Application Support，Windows 使用 LOCALAPPDATA 或 USERPROFILE/AppData/Local。缺少绝对用户目录时报错，不回退到工作目录。产品负责显式路径覆盖与数据迁移策略。

`resolve_executable(program, search_path)` 接受独立传入的 PATH，优先处理显式路径；Unix 核对可执行位，Windows 支持无后缀程序的 .exe。不隐式运行 shell/.cmd/.bat。随包程序选择顺序属于产品层。

## HTTP API

`sdcore::http` 提供统一的错误响应：

```rust
use sdcore::http::{ApiError, ApiResult};

async fn handler() -> ApiResult<&'static str> {
    Err(ApiError::bad_request("invalid request"))
}
```

错误响应格式：

```json
{
  "error": "invalid request"
}
```

当前提供的错误类型：

- `internal`
- `bad_request`
- `unauthorized`
- `forbidden`
- `not_found`
- `conflict`
- `too_many_requests`

## 数据库

`sdcore::db` 提供数据库配置结构和运行时连接抽象：

```rust
use sdcore::db::{Database, DatabaseConfig};

let db = Database::connect(&config, sqlite_path).await?;
```

### SQLite

SQLite 是默认数据库能力，适合本地工具、单实例后台和轻量部署。

当前连接配置：

- 自动创建数据库目录。
- `create_if_missing(true)`。
- `foreign_keys(true)`。
- `journal_mode(WAL)`。
- `synchronous(NORMAL)`。
- `busy_timeout(5s)`。
- 连接池 `max_connections(5)`。

这套配置适合读多写少、单主程序写入的工具型应用。它不适合高频多写、大规模集群或多租户 cloud 场景。

### PostgreSQL

`postgres` feature 提供 PostgreSQL 连接抽象：

```toml
sdcore = { git = "https://github.com/huwenlong92/sdkit-rust.git", tag = "v0.1.0", features = ["postgres"] }
```

注意：这只是底层连接能力。业务项目是否能切到 PostgreSQL，取决于业务 repository、SQL 方言、迁移和事务逻辑是否已经适配。

## 认证

`sdcore::auth` 提供简单 session 能力：

- 用 OS CSPRNG 生成 32 字节随机 session token（编码为 64 字符十六进制）。
- `hash_password` 使用 Argon2id 与随机 salt，`verify_password` 校验编码哈希；格式错误返回 false。异步服务应通过 `spawn_blocking` 调用耗时密码操作。
- 从 `Authorization: Bearer` 读取 token。
- 从指定 header 读取 token。
- 从 cookie 读取 token。
- 创建、续期、查询、删除内存 session。
- 生成和清除 HttpOnly session cookie。

适合内部工具和单实例后台。

不适合直接作为 cloud 权限系统使用。cloud 场景还需要：

- 用户组织关系。
- 角色和权限。
- 多租户隔离。
- 审计日志。
- 登录风控。
- 订阅和套餐权限。

## SSE

`sdcore::sse` 提供 topic 级别的发布订阅：

```rust
let hub = sdcore::sse::SseHub::default();
let receiver = hub.subscribe("deploy");
hub.publish("deploy", sdcore::sse::SseMessage::new("output", "started"));
```

可通过 `receiver_to_sse` 转成 axum SSE response。原有 `SseHub` 保留无界队列以兼容已有消费方；持续进程输出等新场景优先使用 `BoundedSseHub::new(capacity)`。

有界 Hub 的 `subscribe(topic)` 返回 `BoundedSseReceiver`；慢订阅的 `recv()` 先返回 `event="gap"`、JSON `dropped` 数量，再继续接收保留事件。业务层在发布前限制消息大小并脱敏。`remove_topic` 关闭单个 topic，`close` 关闭现有所有 topic；二者会排空已保留事件。

`bounded_receiver_to_sse(receiver, authorized_predicate)` 在事件前和至少每秒检查授权谓词；会话撤销或过期时结束流。传入纯内存、快速且无阻塞的谓词；不要依赖初次建立连接时的一次鉴权。授权边界和历史日志仍属于消费方。

适合：

- 部署进度。
- 回滚进度。
- 长任务输出。
- 页面实时状态更新。

当前不是持久化消息队列。服务重启后，历史事件需要业务项目从数据库或日志文件恢复。

## Shell 执行

`sdcore::exec::ShellExecutor` 支持：

- 指定工作目录。
- 注入环境变量。
- 使用 `sh -lc` 执行命令。
- 通过 `run_capture()` 捕获 stdout/stderr。
- 按行发出执行事件。
- 设置超时时间。
- Unix 下按进程组终止超时命令。

典型用途：

- 部署脚本。
- 回滚脚本。
- 构建脚本。
- 本地维护命令。

业务项目应自己决定：

- 哪些命令允许执行。
- 运行目录是否合法。
- 超时时间。
- 日志如何存储。
- 执行结果如何落库。
- 输出事件如何接入 reporter、SSE 或业务日志。

## 常驻子进程

`sdcore::process` 用于管理需要持续运行的子进程，与一次性 Shell 命令执行分开。必须在 Tokio runtime 内调用。

```rust
use sdcore::process::{ProcessEvent, ProcessHandle, ProcessRequest};

let (process, mut events) = ProcessHandle::spawn(
    ProcessRequest::new("cloudflared")
        .with_arg("tunnel")
        .with_arg("run")
        .with_arg("--token-file")
        .with_arg("/data/secrets/tunnel.token"),
)?;

// 返回的订阅包含启动事件和启动后的输出；事件不会携带参数或环境值。
let log_task = tokio::spawn(async move {
    loop {
        match events.recv().await {
            Ok(ProcessEvent::Exited(_) | ProcessEvent::Failed(_)) => break,
            Ok(event) => {
                // 根据产品要求组合 reporter、SSE 或日志存储。
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                // 提示丢失了 count 个事件，按需补读业务日志，然后继续订阅。
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
});

let state = process.state();
let exit = process.stop().await; // 退出码表示实际进程结果，主动停止也可能是非零退出。
log_task.await?;
```

能力与边界：

- 可执行文件和参数独立传递，不隐式执行 Shell。环境值通过 `with_env` 注入，工作目录通过 `with_working_dir` 指定；标准输入关闭。`with_clear_env()` 可清空继承环境，再通过 `with_env` 注入所需 PATH/HOME 等允许项，避免继承宿主凭据或其他产品配置。
- `with_isolated_home(directory)` 清空继承环境并保留 PATH；Windows 额外保留 SystemRoot/WINDIR，统一隔离 HOME/USERPROFILE/APPDATA/LOCALAPPDATA/TEMP/TMP。调用方负责创建目录。再次调用会一起替换这些用户目录变量，避免多授权进程混用旧目录；不继承代理或凭据。
- `pid()` 提供 PID；`state()` / `subscribe_state()` 提供运行、停止中、退出或监管失败状态。进程运行状态不代表应用协议已经就绪。
- `wait()` 等待退出；`stop()` 可重复或并发调用。调用后的等待 future 被取消，不会撤销已发送的停止请求。
- Unix 下先向整个进程组发送 SIGTERM，默认等待 5 秒，超时后 SIGKILL；可通过 `with_stop_timeout` 调整。父进程自然退出后也清理组内后代。不支持让组内子进程脱离监管长期运行；主动脱离进程组的后代不在清理范围内。
- 非 Unix 平台停止时直接终止主进程，暂不提供进程树清理。
- handle 可以克隆。最后一个 handle 被释放时触发停止，单独释放事件订阅不影响进程。正常应用退出应显式调用 `stop().await`；runtime 被直接终止时，不能保证异步进程组清理完成。
- 输出按原始字节块发送，每块最多 8 KiB，可能跨行或跨 UTF-8 字符。消费者负责解码、拼接、业务解析和脱敏；子进程自己输出的敏感内容不会被底座自动过滤。
- 事件订阅使用容量为 256 的有界 broadcast，慢消费者可能收到 `RecvError::Lagged`，应处理丢失提示并继续接收或补读业务日志。慢订阅不会阻塞子进程，底座不保存历史输出。
- `spawn` 返回的 receiver 包含启动时事件；`subscribe()` 只接收订阅后的事件。退出事件在输出排空后发送，异常情况下排空每条输出流最多等待 1 秒。
- 自动重启、退避、连接就绪检测、日志持久化和应用专属配置留在消费项目，按实际复用需求再扩展。

## Reporter

`sdcore::reporter` 提供统一事件模型：

- `Started`
- `ArtifactBefore`
- `ArtifactAfter`
- `Output`
- `Succeeded`
- `Failed`
- `Rejected`

业务项目可以把同一个事件同时分发到多个 reporter，例如：

- 写日志文件。
- 推 SSE。
- 更新数据库状态。
- 记录产物版本。

## 日志

`sdcore::logger` 提供简单文件日志：

```rust
sdcore::logger::init(sdcore::logger::LogConfig::new("logs"))?;
sdcore::logger::info("server started");
```

日志格式：

```text
[2026-07-01 18:00:00] INFO  server started
```

文件路径：

```text
<root>/<name>/<name>.log
```

例如默认 name 是 `app`，root 是 `logs`：

```text
logs/app/app.log
```

生产环境仍建议通过 systemd journal、logrotate 或外部日志系统管理长期日志。

## 静态资源

`sdcore::static_files` 提供静态资源响应辅助：

- `asset_response`
- `not_found_json`

它不负责构建前端资源。业务项目仍然负责自己的 Vite、pnpm、打包和静态资源嵌入策略。

## 能力边界

`sdcore` 不应该包含：

- 业务表结构。
- 业务 repository。
- 业务 handler。
- 业务前端页面。
- 项目、节点、部署、用户这类具体产品概念。
- SaaS 计费、订阅、套餐、租户模型。
- 复杂工作流编排。

Rust 工具项目里的公共运行时和基础设施能力默认优先进入 `sdcore`，包括但不限于命令执行、SSE、reporter、认证、数据库连接、静态资源、日志、配置、后台任务、长任务进度、文件/系统操作辅助等。

如果一个能力只服务一个业务项目，应先留在业务项目中。只要确认它是跨项目公共能力，或现有 `sdcore` API 已经无法覆盖新的通用场景，就优先扩展 `sdcore` 再由业务项目消费，不要在业务项目里复制一套局部 core。

公共能力的行为测试默认放在 crate 根目录的 `tests/` 下，例如 `tests/exec_shell.rs`。实现文件里的 `#[cfg(test)]` 只用于少量必须贴近私有函数的单元测试。

## 演进方向

短期优先级：

- 稳定 SQLite、日志、SSE、Shell 执行这些工具型基础能力。
- 保持 API 小而明确。
- 让 `sdhook` 继续作为实际消费方验证。

后续可以考虑：

- 通用分页参数和响应结构。
- 更系统的配置加载与校验。
- 标准化 middleware 组装。
- 更完整的日志轮转或日志接口抽象。
- 按需增强 PostgreSQL 支持。

cloud、多租户、权限、订阅付费等能力应作为未来更高层产品能力规划，不直接放进当前 `sdcore` 基础层。

## 2026-09-30 SDTunnel 消费验证

本次增加密码哈希、随机会话令牌、TOML 读取、有界 SSE、授权流退出、可等待清理回调与进程环境隔离。公共行为测试放在 `tests/auth_password.rs`、`config_load.rs`、`sse_bounded.rs`、`process_managed.rs`。

SDTunnel 直接使用这些 API；其保存记录、手动启停、cloudflared 输出解析与就绪检测留在产品包。SDSub 在隔离的源码副本中通过编译检查，未改动原消费方 lockfile、配置或运行数据。
