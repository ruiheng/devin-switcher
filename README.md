# Devin Switch

[中文说明](README.zh.md)

A tiny multi-account switcher for Devin: stores each account's
`credentials.toml` snapshot in a vault, and switching atomically swaps it
in + updates `devin.org_id` in `config.json`. The CLI and Desktop share
the same credentials, so one switch covers both (running CLI/Desktop
processes need a restart to pick it up).

Two frontends, same backend, same vault:

- **`devin-switch`** — Tauri GUI (macOS / Windows / Linux)
- **`dsw`** — pure CLI, zero GUI deps, builds fine on remote/SSH boxes

## Features

- Account cards: email, plan, quota bars (daily/weekly remaining %, ACU,
  overage balance), live reset countdowns
- Three ways to add: browser sign-in (reuses the devin account already
  signed into your browser — sign out first to use a different one),
  save the current sign-in, or paste a code/token (works remotely)
- Re-adding the same account refreshes it in place — no `-2` copies;
  auto-named by email, renamable afterwards
- "Run in parallel": launches a devin session for another account under
  an isolated `XDG_DATA_HOME`
- GUI is bilingual (auto-detects system language, toggle top-right)

## Build

Only a Rust toolchain is needed (the frontend is static HTML/JS under
`ui/` — no npm build step).

### GUI dev run

```bash
cd app
cargo run          # builds and opens the Devin Switch window
```

### GUI packaging

```powershell
powershell .\bundle.ps1   # release build + bundle, installers copied to dist\
```

(or `cd app && npx @tauri-apps/cli@2 build` directly)

Windows output lands in `dist/` (NSIS `-setup.exe` + MSI); raw bundles
are under `app/target/release/bundle/` (macOS produces
`.app`/`.dmg`, Linux `.AppImage`/`.deb`).

Tauri system deps: macOS needs only Xcode CLT; Linux needs `webkit2gtk`
etc. (see the tauri.app prerequisites docs); Windows needs WebView2
(preinstalled on most Win10/11).

### CLI (dsw) — no GUI deps

```bash
cd app
cargo install --path . --no-default-features --bin dsw --root ~/.local
# → ~/.local/bin/dsw
```

`--no-default-features` disables the `gui` feature so tauri/webview are
skipped — a remote Linux box only needs that one command after cloning.

Or run without installing:

```bash
cargo run --no-default-features --bin dsw -- list
```

### Tests

```bash
cd app && cargo test
```

## dsw usage

```
dsw status                  current sign-in + quota + running devin processes
dsw list                    all saved accounts (* = currently active)
dsw save [name]             save the current sign-in
dsw use <name>              switch to an account
dsw rename <from> <to>      rename an account
dsw delete <name>           delete an account
dsw refresh <name>          re-fetch that account's quota
dsw login [name]            interactive login: prints a PKCE link, paste the code back
dsw add-token <tok> [name]  same, non-interactive (token as an argument)
dsw parallel <name> [cwd]   print/launch a devin session with isolated data
```

`dsw login` prints `app.devin.ai/auth/cli/continue?…` (with PKCE
params) — open it in any browser, sign in, paste the shown code back
(echoes `*`). Pasting a `devin-session-token$…` works too.

## Data layout

```
vault:  ~/.local/share/devin-switch/   (Windows: %APPDATA%\devin-switch)
        profiles/<name>/
            credentials.toml           # verbatim snapshot
            meta.json                  # email/plan/org_id/note/quota cache

switch target: ~/.local/share/devin/credentials.toml   (Windows: %APPDATA%\devin\…)
               devin.org_id in ~/.config/devin/config.json follows along
```

Active detection compares credential bytes — no separate state file, so
hand-edited creds, restarts, and renames never get out of sync.

## Notes

- Quota comes from Devin's internal `GetUserStatus` Connect-RPC (same
  source the CLI itself uses); API changes only affect the quota
  display, not switching.
- Running devin CLI/Desktop processes must be restarted after a switch
  to pick up the new account.
