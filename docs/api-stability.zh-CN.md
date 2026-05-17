# API 稳定性

Project Suzu 仍处于 `1.0` 之前。项目会尽量保持公共 API 稳定，但运行时、工具和 crate 边界仍会随着工程收敛继续调整。

## 当前稳定性承诺

- `suzu-app::SuzuApp`、`TitleMenuAction` 和 `SystemMenuAction` 是主要运行时 facade 类型。
- 在 `0.1.x` 版本线中，已有 public `SuzuApp` 方法不应在没有 changelog 说明的情况下改名或删除。
- 脚本格式 `@script version=1` 是当前稳定脚本格式。没有 `syntax` 字段时，默认使用 classic 语法。
- `syntax=classic`、`syntax=indent`、`syntax=braces` 和 `syntax=markup` 都会编译到同一套命令模型。`0.1.x` 阶段非 classic 风格还比较新，可能继续获得增量解析改进，但已文档化的示例应保持可编译，除非 changelog 明确说明破坏性变化。
- `.suzupack` 格式版本 `1` 会继续由 asset crate 读取。
- GUI `--check` 接口用于 CI smoke 检查，应保持可脚本化。

## 实验性区域

- 渲染器内部实现和 frame 构建细节。
- 桌面 GUI 布局和编辑器数据模型。
- `suzu.xp3-plugin.v1` 之外的 XP3 plugin schema；当前接口见 `docs/xp3-plugin-interface.zh-CN.md`。
- KRKR/KAG 转换启发式规则。
- 可选 `lua` feature 下的 Lua 扩展注册。

## 兼容性建议

应用侧优先使用 `SuzuApp` facade 方法和 asset-manager 注册方法，不要直接依赖 crate 内部实现。工具自动化优先使用 `--check`；在项目进入 `0.2.x` 前，把可视化 GUI 行为视为预览质量。
