# 旧项目迁移清单

2026-10-19扫描所得。用户要求先完成新底座和模板，之后逐个切换；初次模板交付未修改下列项目；后续逐项目进展在表中单独记录，不能将计划视作已完成。

| 项目 | 已观察引用 | 迁移注意事项 |
| --- | --- | --- |
| sdhook | 旧sdkit-rs path | 审核Linux构建与发布依赖获取 |
| codex-webui | 旧sdkit-rs path | 仓库尚无提交，保留全部未跟踪工作，不自动纳入 |
| sdgate-nginx | 旧sdkit-rs path | Makefile和Dockerfile有既有修改，保留并单独审核 |
| sdtunnel | 旧sdkit-rs path与Docker上下文 | Cargo/构建/业务均有既有修改，不批量覆盖 |
| sdsub | 旧sdkit-rs path与Docker上下文 | 核对私有Git依赖和SSH构建权限 |
| sdacme | 已迁移公开HTTPS固定v0.1.0，提交9d4aec2 | 85项Rust测试、Linux两包、非root运行、真实双节点升级与3项来源保护通过；原39个制品文件不变，线上0.3.0不动 |

未迁移项目必须重新检查真实Git状态与项目规则，再将sdcore改为私有sdkit-rust固定tag，更新锁文件、审计私有依赖在本地/CI/Docker中的读取方式，编译测试并单独交付。不得把此清单视作已迁移或已验证兼容。
