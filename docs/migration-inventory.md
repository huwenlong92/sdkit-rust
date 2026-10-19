# 旧项目迁移清单

2026-10-19扫描所得。用户要求先完成新底座和模板，之后逐个切换；初次模板交付未修改下列项目；后续逐项目进展在表中单独记录，不能将计划视作已完成。

| 项目 | 已观察引用 | 迁移注意事项 |
| --- | --- | --- |
| sdhook | 公开HTTPS固定v0.1.0，已推送6bdf0fb | 43项Rust、Linux两包、双Agent任务和SSE通过；版本与线上不变 |
| codex-webui | 本地已迁移公开HTTPS固定v0.1.0 | 50项Rust、7项前端单测与Clippy/类型检查、隔离配对/鉴权/SSE通过；4项真实模型测试ignored。尚无首次提交和remote，按项目规则保留全部未跟踪工作，未自动提交推送 |
| sdgate-nginx | 底座迁移53b5c37，前端修复9bf0c84，均已推送 | 47项Rust、Linux amd64包/校验/版本启动通过（迁移时使用原静态资源副本）。后续统一规则选择空值为null，完整vue-tsc/Vite/gzip构建通过，前端阻塞已解除；ESLint未安装，未执行。原Makefile/Dockerfile链接改动保留 |
| sdtunnel | 公开HTTPS固定v0.1.0，已推送ffc5a55 | 当前工作区15项Rust与前端类型检查通过；精确提交副本编译及10项测试通过。现有arm64 Docker编译/CLI和Compose检查通过；原未提交CLI/模型/SQL/打包改动保留，未启用真实隧道 |
| sdsub | 公开HTTPS固定v0.1.0，已推送c95edbf | 51项Rust、前端类型、Docker amd64编译导出、临时非root只读断外网SQLite健康和退出通过；2项显式夹具测试ignored。未推镜像或部署 |
| sdacme | 已迁移公开HTTPS固定v0.1.0，提交9d4aec2 | 85项Rust测试、Linux两包、非root运行、真实双节点升级与3项来源保护通过；原39个制品文件不变，线上0.3.0不动 |

上列消费方的实际 Cargo 声明与锁文件均已使用公开 `https://github.com/huwenlong92/sdkit-rust.git` 的固定 tag `v0.1.0`，锁定提交 `f9e8c883e0d238298a44c0668e442a88fd7459f8`；未移动该 tag。2026-10-19 再次扫描 lab/sdkit 内排除 target/node_modules 后的 Cargo.toml，没有遗留 `sdkit-rs/crates/sdcore` path 引用。

迁移代码、测试、提交推送及真实部署必须分别核验；本轮未更新线上或GitHub Release。历史方案文档和未授权修改的项目规则不作为当前 Cargo 依赖来源，后续以实际 Cargo.toml/Cargo.lock 为准。
