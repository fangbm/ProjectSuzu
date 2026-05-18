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

在 `0.2.x` 版本线中，demo 应继续发展为适合教程使用的短篇 VN：

- 3 到 5 分钟游玩时间
- 至少两条有意义路线和一个汇合结局
- 原创或 CC0 背景图
- 原创或 CC0 角色立绘
- 可选的开放许可证 BGM 和短语音占位
- README 说明如何编辑脚本、替换素材、打包资源和发布示例
- 发布包包含 demo binary 和打包后的 demo 资源

## 完成版剧本轮廓

完成版 demo 应保持足够轻量，方便 CI 和 release 包维护，同时又能证明完整作者闭环：

1. 标题界面进入车站开场。
2. 第一次选择改变说明路线并设置变量。
3. 中段场景演示自动存档、等待、视觉效果和角色动画。
4. 第二次选择改变后续一句文本或 route note，而不是让剧本指数膨胀。
5. 汇合结局引导作者替换素材并打包项目。

公开仓库中的剧本应保持 synthetic 和教程定位。不要引用私有项目、商业游戏或本地兼容性实验。

## 素材来源方案

- 背景：使用原创简易插图、带再分发说明的生成占位图，或能在 `examples/short-vn-demo/assets/README.md` 记录来源与许可证的 CC0 图片。
- 角色：使用原创占位立绘或可再分发的 CC0/开放许可证素材。优先采用简单静态图，用来覆盖 `@char`、`@anim` 和 `@hidechar`。
- 音频：使用静音占位、原创短音效，或附带 license 文本的开放许可证 BGM/voice。最终教程打磨前，音频可以保持可选。
- 打包：通过 `suzu-packer` 保持源素材和 `.suzupack` 输出可复现；除非 release packaging 明确需要，不提交生成的 archive。

## 素材规则

允许：

- 为 Project Suzu 原创制作的素材
- CC0 素材
- 明确允许在本仓库和 release archive 中再分发的素材
- 带明确再分发说明的生成占位素材

不允许：

- 从商业游戏中提取的素材
- 来自第三方 archive 的处理后 XP3 输出
- 特定游戏的 plugin 配置
- 再分发条款不清晰的素材

## 验收清单

- `cargo run -p suzu-short-vn-demo` 从标题界面启动。
- `cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs` 通过。
- `cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack` 成功。
- `cargo test --workspace` 编译示例并 smoke-test 脚本。
- Release 包包含 `suzu-short-vn-demo` 和打包后的 short-demo 资源。
