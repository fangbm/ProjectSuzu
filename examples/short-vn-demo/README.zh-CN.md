# Project Suzu 短篇 VN Demo

这个示例是 `v0.1.6` 之后开发方向中规划的完整短篇视觉小说 demo 的第一版。

从仓库根目录运行：

```powershell
cargo run -p suzu-short-vn-demo
```

当前 demo 覆盖：

- 标题界面启动
- 背景转场
- 角色显示、更新和隐藏
- 对白与点击等待
- 选择项
- 变量和 `@if/@else/@endif`
- BGM 与 voice 命令占位
- 自动存档和预置 load slot
- 通过共享运行时 UI 使用历史记录、系统菜单和自动模式
- 可通过 `suzu-packer` 打包的脚本和资源

当前视觉纹理由代码生成 fallback 颜色。把它当成打磨后的样例之前，请替换为原创或开放许可证素材。
