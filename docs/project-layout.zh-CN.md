# 工程目录布局

Project Suzu 现在支持脚本优先的工程文件夹。作者可以只维护脚本和资源，不必先写 Rust binary。

## 标准目录

```text
my-game/
  game.suzu.toml
  scenario/main.szs
  assets/
    bg/
    chara/
    bgm/
    voice/
  saves/
```

`scenario/main.szs` 是默认入口。如果没有 `game.suzu.toml`，`suzu-player` 和 `suzu-launcher` 会先尝试 `scenario/main.szs`，再兼容旧模板的 `script/main.szs`。

## game.suzu.toml

```toml
title = "My Game"
subtitle = "A Project Suzu visual novel"
entry = "scenario/main.szs"

[title_screen]
enabled = true
background = "title_bg"

[window]
title = "My Game"
width = 1280
height = 720
resizable = true

[assets]
roots = ["assets"]

[package]
files = ["data.suzupack"]
```

`package.files` 是可选项。生成 `.suzupack` 后再填写；直接使用散文件开发时保持空列表即可。

## 资源 ID

每个资源目录会按相对路径注册资源：

- `assets/bg/school.png` 变成 `bg/school`。
- `assets/chara/suzu_normal.png` 变成 `chara/suzu_normal`。
- 图片资源还会注册文件名短别名，例如 `school` 或 `suzu_normal`，方便快速原型脚本使用。

大型项目建议使用路径式 ID，避免文件名冲突。短别名主要服务小项目和脚本优先的快速试作。

## 运行和检查

```powershell
suzu-player my-game
suzu-player --check my-game
suzu-launcher --check --project-root my-game
suzu-editor --check --project-root my-game
```

当 `suzu-player` 没有收到工程目录，并且当前目录也不是 Suzu 工程时，它会尝试打开可执行文件旁边或当前目录下的 `templates/starter-vn`。这样在 release 包根目录双击 `suzu-player` 会进入内置 starter project，而不是因为缺少 `.\scenario\main.szs` 直接报错。

新脚本推荐使用 `syntax=indent`：

```text
@script version=1 syntax=indent
bg school
Suzu: Hello.
choice "Library" goto=library
label library:
Suzu: Let us begin.
```

这个目录布局不会把 Project Suzu 变成完整 KRKR/TJS/KAG runtime。KRKR 包扫描和 KAG 转换仍然是迁移工具；新项目应直接面向 `.szs` 编写。
