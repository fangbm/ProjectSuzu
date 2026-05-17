把图片、音频、字体和数据资源放在这里。

推荐目录：

- `bg/`：背景图。
- `chara/`：角色立绘。
- `bgm/`：音乐。
- `voice/`：语音。

Project Suzu 会用去掉扩展名的相对路径注册资源 ID，例如 `assets/bg/title_bg.png` 对应 `bg/title_bg`。图片资源也会额外注册文件名短别名，例如 `title_bg`，方便短命令脚本使用。
