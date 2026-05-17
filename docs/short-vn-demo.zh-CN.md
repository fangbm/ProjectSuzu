# 短篇 VN Demo 计划

`examples/short-vn-demo` 用来证明 Project Suzu 可以发布一个小而完整的视觉小说循环，并且不依赖私有或游戏专用素材。

## 当前切片

首个提交进仓库的切片刻意保持轻量：

- workspace 示例 binary：`suzu-short-vn-demo`
- 标题界面和预置存档槽
- 一个 `.szs` 脚本，覆盖背景切换、角色显示、对白、选择、变量、条件、BGM/voice 占位、自动存档、效果、等待和角色隐藏
- Rust 代码生成的运行时 fallback 纹理
- 可通过 `suzu-packer` 打包的脚本和资源

这样既能保持 CI 快速，也让 demo 成为真实可执行程序，而不是单纯设计说明。

## 下一步完成目标

在 `v0.2.0` 之后，demo 应继续发展为适合教程使用的短篇 VN：

- 3 到 5 分钟游玩时间
- 至少两条有意义路线和一个汇合结局
- 原创或 CC0 背景图
- 原创或 CC0 角色立绘
- 可选的开放许可证 BGM 和短语音占位
- README 说明如何编辑脚本、替换素材、打包资源和发布示例
- 发布包包含 demo binary 和打包后的 demo 资源

## 素材规则

允许：

- 为 Project Suzu 原创制作的素材
- CC0 素材
- 明确允许在本仓库和 release archive 中再分发的素材

不允许：

- 从商业游戏中提取的素材
- 解密 XP3 的输出
- 特定游戏的 plugin 配置
- 再分发条款不清晰的素材

## 验收清单

- `cargo run -p suzu-short-vn-demo` 从标题界面启动。
- `cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs` 通过。
- `cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack` 成功。
- `cargo test --workspace` 编译示例并 smoke-test 脚本。
- Release 包包含 `suzu-short-vn-demo` 和打包后的 short-demo 资源。
