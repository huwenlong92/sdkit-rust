# 版本发布与消费

权威源码：公开仓库 `huwenlong92/sdkit-rust`。crate 名仍为 `sdcore`。

## 版本约束

- 首个版本 v0.1.0。底座修改后按语义发布新的 tag。
- 已发布 tag 永久对应同一 commit，不移动、不覆盖、不重新打包替换。
- 应用必须使用固定 tag 并提交 Cargo.lock；日常构建执行 `cargo build --locked`。
- 不使用 branch=main，不自动 cargo update，不把底座源码复制进业务项目。
- 升级通过单独变更更新 tag/锁文件并运行该项目测试；历史发行制品不重建或覆盖。

```toml
sdcore = { git = "https://github.com/huwenlong92/sdkit-rust.git", tag = "v0.1.0", default-features = false, features = ["sqlite", "auth"] }
```

## 公开HTTPS消费

仓库已按用户选择公开，Cargo通过HTTPS拉取，不需要GitHub账号、SSH密钥、deploy key或跨仓库CI secret。旧v0.1.0 tag与代码不变，新的消费声明改用HTTPS地址。

Rust的crate可从crates.io或Git仓库读取。目前sdcore使用Git固定tag分发，无需另行发布到crates.io。Cargo.lock锁定实际commit；普通构建使用--locked，不自动更新依赖。项目维护者向自己的源码仓库推送时所用认证，与读取公开底座是两件事。

本地联调可使用临时Cargo patch，但不得提交个人绝对路径；发行前移除patch并验证实际锁定的Git来源。安装已编译好的程序不需要Cargo或底座源码。

## 逐个迁移旧项目

先审计工作区，再单独修改 Cargo.toml、更新锁文件和构建取依赖方式。Docker可通过公开HTTPS按锁文件读取，不挂载宿主SSH或Cargo凭据；不能继续绑定旧sdkit-rs目录。每个项目独立验收，禁止批量覆盖未知改动。
