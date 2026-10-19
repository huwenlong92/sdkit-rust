# Rust 底座与工具模板

任务ID 20261019-rust-templates；revision 2；初始Git基线 unborn。

已确认：两个私有GitHub仓库 sdkit-rust、sdbase-rust；共享crate仍叫sdcore；固定tag、提交锁文件、已发布tag不移动。旧sdkit-rs已经完整复制源码到新目录，原Gitee及旧项目依赖保持不动，后续逐个切换。

范围：新副本底座的特性边界、文档、CI、首版tag；单个模板中的CLI和可选Web管理服务骨架、示例、测试、构建发布入口、私有依赖认证说明。Primary sdkit-rust-core，使用repository/core/routing/workflow/testing约束与Git协作规范。

禁止：修改原sdkit-rs、既有消费项目、正式服务、正式数据库或未知文件；不复制产品证书、账号、DNS与自动升级领域逻辑。

阶段：底座默认/最小/全部特性验证与示例消费 → 私有仓库与v0.1.0 → 统一模板CLI验证 → 可选Web与既有admin-lite固定版本真实联调 → GitHub模板标识与远端核验。

状态 in_progress。发布前审查源码、敏感信息和精确暂存路径；每阶段以实际测试与Git状态为证据。

修订2：用户指定模板位于sdbase工作区下sdbase-rust；web为可选前端，复用既有admin-lite而非另造Vue模板。旧的双模板方案取消。

底座阶段 verified：26项默认特性测试、20项最小特性测试、全部特性cargo check、fmt与Clippy通过；admin-tool消费示例编译通过。只修复特性门禁和两处Clippy等价写法；旧sdkit-rs未修改。统一模板不带Vue源码，可选Web后端配套独立admin-lite。
