# Project Suzu 短篇 VN Demo

这个示例是 `0.2.x` 开发方向中规划的完整短篇视觉小说 demo 的第一版。

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

## 编辑脚本

剧情源文件位于 `script/main.szs`。它刻意保持小型，并覆盖编辑器 MVP 应支持的核心命令：背景切换、角色显示、对白、选择、变量、条件、等待、自动存档、voice 占位、动画和效果。

编辑后可从仓库根目录检查脚本：

```powershell
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
```

## 替换素材

把可再分发素材放在 `assets/` 下。素材应为原创、CC0，或明确允许在本仓库和 release archive 中再分发。请在 `assets/README.md` 记录来源和许可证说明。

不要加入商业游戏资源、第三方 archive 处理输出、私有 plugin 配置或再分发条款不清晰的文件。

## 打包和运行

```powershell
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
cargo run -p suzu-short-vn-demo
```

当示例从 fallback 纹理推进到可再分发美术/音频后，release 包应包含 demo binary 和打包后的 demo 资源。
