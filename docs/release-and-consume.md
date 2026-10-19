# 版本发布与消费

权威源码：私有仓库 `huwenlong92/sdkit-rust`。crate 名仍为 `sdcore`。

## 版本约束

- 首个版本 v0.1.0。底座修改后按语义发布新的 tag。
- 已发布 tag 永久对应同一 commit，不移动、不覆盖、不重新打包替换。
- 应用必须使用固定 tag 并提交 Cargo.lock；日常构建执行 `cargo build --locked`。
- 不使用 branch=main，不自动 cargo update，不把底座源码复制进业务项目。
- 升级通过单独变更更新 tag/锁文件并运行该项目测试；历史发行制品不重建或覆盖。

```toml
sdcore = { git = "ssh://git@github.com/huwenlong92/sdkit-rust.git", tag = "v0.1.0", default-features = false, features = ["sqlite", "auth"] }
```

## 私有依赖认证

本地通过已有 SSH key 访问 GitHub；在用户 Cargo 配置设置：

```toml
[net]
git-fetch-with-cli = true
```

CI 的 GITHUB_TOKEN 默认不能读取另一个私有仓库。给 sdkit-rust 配置只读 deploy key，把私钥保存在消费仓库的 `SDKIT_RUST_DEPLOY_KEY` Actions secret，通过 SSH agent 加载；需要预先核实 GitHub SSH host key。不要把私钥或 token 写入 Cargo.toml、镜像层或日志。

模板已提供对应 CI 入口；新建项目需要单独配置该 secret。底座自己的 CI 不需要跨仓库 key。

本地联调可在用户级临时 Cargo 配置中添加 patch，勿提交本机绝对路径；发布前移除 patch，确认锁文件确实记录 Git tag 对应 commit。

## 逐个迁移旧项目

先审计工作区，再单独修改 Cargo.toml、更新锁文件和构建取依赖方式。私有依赖在 Docker 中需要 BuildKit SSH 转发或从已校验版本导出源码构建；不能继续悄悄绑定旧 sdkit-rs 目录。每个项目独立验收，禁止批量覆盖未知改动。
