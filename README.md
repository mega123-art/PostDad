# Postdad 👟

> "He's not mad at your slow API, just disappointed."

**Postdad** is a high-speed, local-first Terminal UI (TUI) for testing APIs. It’s built in **Rust** 🦀 because Electron apps shouldn't eat 1GB of RAM just to send a GET request.

<pre>
   ____           _      _           _ 
  |  _ \ ___  ___| |_ __| | __ _  __| |
  | |_) / _ \/ __| __/ _` |/ _` |/ _` |
  |  __/ (_) \__ \ || (_| | (_| | (_| |
  |_|   \___/|___/\__\__,_|\__,_|\__,_|
                                       
</pre>

## Why Postdad?

Modern dev tools are bloated. Postman takes 5-10 seconds to launch. Postdad takes **<100ms**.

| Feature | Postman/Insomnia | CURL/HTTPie | **Postdad** |
| :--- | :--- | :--- | :--- |
| **Speed** | 🐢 Slow (Electron) | ⚡ Fast | 🚀 **Blazing Fast (Rust)** |
| **RAM Usage** | 500MB+ | ~5MB | **~15MB** |
| **Interface** | Mouse Clicky | CLI Args | **Vim-Style TUI** |
| **Storage** | Cloud Sync (Forced) | History File | **Local .hcl Files** |

## Features

- **Vim-Motion Navigation**: Use `j`, `k`, and `/` to fly through your request history.
- **Three-Pane Layout**: Collections on the left, Request on top, Response at bottom.
- **JSON Explorer**: Interactive tree view for massive JSON responses. Expansion/Collapse nodes with arrow keys.
- **Dad's Garage**: Local-first collection storage. No login required.
- **Async & Non-Blocking**: The UI never freezes, even if the API times out.
- **Latency Heartbeat**: Real-time graph monitoring your API's pulse.
- **Zen Mode**: Press `Ctrl+z` to focus purely on the response data.

## Installation

```bash
cargo install Postdad
```

## Usage

```bash
Postdad
```

- **Ctrl+z**: Toggle Zen Mode (Focus)
- **f**: Toggle Fullscreen Response
- **Enter**: Send Request
- **e**: Edit URL (Press **Tab** to cycle method)
- **Tab**: Switch Request Tabs (Params, Headers, Body)
- **q**: Quit (Dad needs a nap)

## Roadmap to Recognition

- [x] Basic TUI Engine
- [x] Async Request Worker
- [x] JSON Response Explorer (Interactive)
- [x] Collection Management (`.hcl` support)
- [x] "Dad's Directions" (Copy as Curl) (`c` key)
- [x] Response Timing (ms precision)
- [x] Latency Heartbeat (Sparkline Graph)
- [x] Zen Mode (`Ctrl+z`)
- [x] Search / Filter JSON (`/` key)
- [x] Fullscreen Response View (`f` key)
- [x] Quick Method Cycle (Tab in Edit Mode)
- [x] Environment Variables
- [x] Request Body Editor (`b` key -> `$EDITOR`)
   - *Pro Tip*: For VS Code integration, run `export EDITOR="code --wait"` (Mac/Linux) or set `$env:EDITOR="code --wait"` (PowerShell).
- [x] Method Cycling (`m` key)
- [x] Status Codes (Color-coded)
- [x] Help Screen (`?` key)
- [x] Header Editing (`H` key -> `$EDITOR` as JSON)
- [x] Persistence (`s` key -> `saved.hcl`)

## License

MIT
