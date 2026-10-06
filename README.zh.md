# Devin Switch

[English](README.md)

在多个 Devin 账号之间一键切换的小工具：把 `credentials.toml` 按账号存入
vault，切换时原子替换 + 更新 `config.json` 的 `org_id`，CLI 和 Desktop
共用同一份凭据所以一次切换两头生效（正在运行的 CLI/Desktop 进程需重启）。

两个入口，同一套后端、同一个 vault：

- **`devin-switch`** — Tauri GUI（macOS / Windows / Linux）
- **`dsw`** — 纯命令行，零 GUI 依赖，可以编到远程/SSH 机器上

## 功能

- 账号卡片列表：邮箱、plan、额度进度条（当日/本周剩余百分比、ACU、超额余额）
- 三种添加方式：浏览器登录（复用浏览器里已登录的 devin 账号；换号先登出）、
  保存当前登录、手动 code/token 粘贴（可远程用）
- 同账号重复添加自动刷新原条目，不产生 `-2` 副本；自动按邮箱命名，可改名
- "并行运行"：给另一个账号起隔离的 `XDG_DATA_HOME` 开并行 devin 会话
- GUI 中英双语（按系统语言自动检测，右上角可切换）

## 构建

只需要 Rust 工具链（前端是 `ui/` 下的静态 HTML/JS，没有 npm 构建步骤）。

### GUI 开发运行

```bash
cd src-tauri
cargo run          # 编译并打开 Devin Switch 窗口
```

### GUI 打包发布

```bash
powershell .undle.ps1   # release 构建 + 打包，产物拷到 dist/
```

Windows 产物在 `dist/`（NSIS `-setup.exe` + MSI）；
原始输出在 `src-tauri/target/release/bundle/`（macOS 出 `.app`/`.dmg`，
Linux 出 `.AppImage`/`.deb`）。

Tauri 的系统依赖：macOS 只需 Xcode CLT；Linux 需要 `webkit2gtk` 等
（见 tauri.app 的 prerequisite 文档）；Windows 需要 WebView2
（Win10/11 一般自带）。

### CLI（dsw）安装——可无 GUI 依赖

```bash
cd src-tauri
cargo install --path . --no-default-features --bin dsw --root ~/.local
# → ~/.local/bin/dsw
```

`--no-default-features` 关掉 `gui` feature，不编 tauri/webview——
远程 Linux 机器 clone 仓库后这一条命令即可。

也可以不装，直接跑：

```bash
cargo run --no-default-features --bin dsw -- list
```

### 测试

```bash
cd src-tauri && cargo test
```

## dsw 用法

```
dsw status                  当前登录 + 额度 + 运行中的 devin 进程
dsw list                    所有已保存账号（* = 当前激活）
dsw save [name]             保存当前登录
dsw use <name>              切换到某账号
dsw rename <from> <to>      改名
dsw delete <name>           删除
dsw refresh <name>          重新拉取该账号额度
dsw login [name]            交互式登录：打印 PKCE 链接，粘贴页面给的 code
dsw add-token <tok> [name]  同上，非交互（token 作参数）
dsw parallel <name> [cwd]   打印/启动一个隔离数据的并行 devin 会话
```

`dsw login` 流程：打印 `app.devin.ai/auth/cli/continue?…`（带 PKCE 参数），
在任何浏览器打开登录 → 页面显示 code → 粘回终端（逐字符回显 `*`）。
也可直接粘贴 `devin-session-token$…`。

## 数据布局

```
vault:  ~/.local/share/devin-switch/   (Windows: %APPDATA%\devin-switch)
        profiles/<name>/
            credentials.toml           # 原样保存
            meta.json                  # email/plan/org_id/note/额度快照

切换目标: ~/.local/share/devin/credentials.toml   (Windows: %APPDATA%\devin\…)
         ~/.config/devin/config.json 的 devin.org_id 随之更新
```

激活检测靠凭据字节比对，没有单独的状态文件——手动换凭据、重启、rename
都不会失步。

## 注意

- 额度来自 Devin 内部 `GetUserStatus` Connect-RPC（与 CLI 自身同源）；
  接口若变动只会影响额度显示，不影响切换。
- 切换后正在运行的 devin CLI/Desktop 需要重启才会用新账号。
