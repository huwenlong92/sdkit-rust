# sdkit-rust

Rust 工具项目的共享底座，公开 GitHub 仓库：`huwenlong92/sdkit-rust`。

- `crates/sdcore`：配置、日志、命令执行、进程、HTTP、认证、SQLite/PostgreSQL、SSE 与任务能力。
- `examples/admin-tool`：最小 Web 消费示例。
- 业务模型、业务流程和页面留在消费项目，共享机制在底座维护。

## 固定版本使用

```toml
sdcore = { git = "https://github.com/huwenlong92/sdkit-rust.git", tag = "v0.1.0", default-features = false, features = ["sqlite", "auth", "static-embed"] }
```

纯 CLI 可使用 `default-features = false`，不启用数据库和认证。默认 features 为 sqlite、auth、sse、static-embed、task。

**已发布 tag 不允许移动、删除或覆盖。** 修改底座后发布新版本，消费项目显式更新 tag 和 Cargo.lock，分别测试后升级；不依赖 main，不自动追随最新版本。

公开HTTPS依赖不需要GitHub账号、SSH key或CI secret。直接运行Cargo即可；Cargo.lock固定实际commit。这里使用Git tag分发crate，尚未发布到crates.io。安装消费项目已编译的二进制不需要底座源码。

## 开发与验证

```sh
cargo fmt --check
cargo test --workspace --locked
cargo test -p sdcore --no-default-features --locked
cargo check -p sdcore --all-features --locked
cargo run -p admin-tool
# http://localhost:9001/health
```

工具链固定在 rust-toolchain.toml。首次导入自旧 sdkit-rs 源码；旧 Gitee 仓库及现有消费项目保持原状，后续逐个迁移。

[能力说明](docs/sdcore-capabilities.md) · [版本与依赖](docs/release-and-consume.md)
