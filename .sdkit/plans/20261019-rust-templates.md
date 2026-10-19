# Rust 底座与工具模板

任务ID 20261019-rust-templates；revision 2；初始Git基线 unborn。

已确认：两个私有GitHub仓库 sdkit-rust、sdbase-rust；共享crate仍叫sdcore；固定tag、提交锁文件、已发布tag不移动。旧sdkit-rs已经完整复制源码到新目录，原Gitee及旧项目依赖保持不动，后续逐个切换。

范围：新副本底座的特性边界、文档、CI、首版tag；单个模板中的CLI和可选Web管理服务骨架、示例、测试、构建发布入口、私有依赖认证说明。Primary sdkit-rust-core，使用repository/core/routing/workflow/testing约束与Git协作规范。

禁止：修改原sdkit-rs、既有消费项目、正式服务、正式数据库或未知文件；不复制产品证书、账号、DNS与自动升级领域逻辑。

阶段：底座默认/最小/全部特性验证与示例消费 → 私有仓库与v0.1.0 → 统一模板CLI验证 → 可选Web与既有admin-lite固定版本真实联调 → GitHub模板标识与远端核验。

状态 verified。发布前审查源码、敏感信息和精确暂存路径；每阶段以实际测试与Git状态为证据。

修订2：用户指定模板位于sdbase工作区下sdbase-rust；web为可选前端，复用既有admin-lite而非另造Vue模板。旧的双模板方案取消。

底座阶段 verified：24项默认特性测试、20项最小特性测试、全部特性cargo check、fmt与Clippy通过；admin-tool消费示例编译通过。只修复特性门禁和两处Clippy等价写法；旧sdkit-rs未修改。统一模板不带Vue源码，可选Web后端配套独立admin-lite。

最终范围：sdbase-rust为单仓模板，默认CLI，可选server、sse、embed-web，不带Vue源码。现有独立admin-lite兼容账户契约，配套说明保留在其工作区；未知功能改动未提交。旧项目未切换。
Git交付：两个私有GitHub仓库均已推送main与v0.1.0；底座tag对应f9e8c883e0d238298a44c0668e442a88fd7459f8，模板tag对应6d9fbdb37464c82a4fda3bfbf82228d0f2ea66c5。底座CI成功，模板CI 39268554160已成功（Linux make check和make verify-runtime）。只读底座deploy key与模板secret已配置，临时私钥已删除。
模板验证：CLI/server/sse格式、Clippy与测试；真实本地登录与SSE推送/断线重连/会话撤销/进程关闭/重启；admin-lite固定版本13项测试、开发代理与实际嵌入页面；生成项目改名与CLI打包均通过。

修订3（公开底座）：用户认为构建认证不便，明确将sdkit-rust公开；历史私有方案不再适用。已审查全部58个可达历史blob，无私钥/访问令牌特征；仓库改为public且匿名API访问通过，固定v0.1.0 tag不变。消费方改为公开HTTPS，sdbase-rust模板仍私有，准备v0.1.1去除底座认证步骤；专用只读密钥待新CI验证后撤销。
