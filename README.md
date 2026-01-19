# PostDad 👟

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
- **Multi-Tab Interface**: Create, switch, and close multiple request tabs (`Ctrl+n`).

## Installation

```bash
cargo install PostDad
```

### Update
```bash
cargo install --force PostDad
```

## Usage

```bash
Postdad
```

- **Ctrl+z**: Toggle Zen Mode (Focus)
- **Ctrl+w**: Toggle WebSocket Mode
- **f**: Toggle Fullscreen Response
- **Enter**: Send Request (or Sync Param Edit)
- **e**: Edit URL (Press **Tab** to cycle method)
- **Tab**: Switch Request Tabs (Params, Headers, Body, Auth)
- **c**: Copy as cURL command
- **G** (Shift+g): Copy as Python (requests) code
- **J** (Shift+j): Copy as JavaScript (fetch) code
- **M** (Shift+m): Generate API Documentation (Markdown)
- **Ctrl+k**: Open Mock Server Manager
- **Ctrl+t**: Cycle Themes (Default, Matrix, Cyberpunk, Dracula)
- **Ctrl+n**: New Request Tab
- **Ctrl+x**: Close Request Tab
- **[ / ]**: Cycle Request Tabs
- **q**: Quit (Dad needs a nap)

### Tab Context Actions
- **Params Tab**: 
  - `a`: Add Param | `d`: Delete Param | `e`: Edit Key/Value
- **Auth Tab**: 
  - `t`: Switch Type (None/Bearer/Basic/OAuth2) | `u`: Edit User | `p`: Edit Password
  - **OAuth 2.0**: `Enter` to Start Flow | `i`/`1`/`2` to Edit Config
- **Chain Tab**:
  - `a`: Add Rule | `d`: Delete Rule | `e`: Edit Key/Path
- **Body Tab**:
  - `m`: Switch Type (Raw/Multipart/GraphQL)
  - **Multipart**: `a` Add | `Space` Toggle File | `d` Delete
  - **GraphQL**: `Q` (Shift+q) Edit Query | `V` (Shift+v) Edit Variables
  - **GraphQL Introspection 🔍**: Auto-complete from schema (Press `Ctrl+I`)
## Roadmap to Recognition

- [x] Basic TUI Engine
- [x] Async Request Worker
- [x] JSON Response Explorer (Interactive)
- [x] Collection Management (`.hcl` support)
- [x] "Dad's Directions" (Copy as Curl) (`c` key)
- [x] Response Timing (ms precision)
- [x] Request Chaining (**Variable Extraction** from JSON Response using full JSONPath support via `jsonpath_lib`)
- [x] Latency Heartbeat (Real-time graph)
- [x] Zen Mode (Focus on Response)
- [x] Search / Filter JSON (`/` key)
- [x] Fullscreen Response View (`f` key)
- [x] Interactive Query Params (Table Editor)
- [x] Extended Auth (Basic, Bearer, **OAuth 2.0** with Browser Flow)
- [x] Time-Travel History (Restore full response state)
- [x] Environment Variables
- [x] Request Body Editor (`b` key -> `$EDITOR`)
   - *Pro Tip*: For VS Code integration, run `export EDITOR="code --wait"` (Mac/Linux) or set `$env:EDITOR="code --wait"` (PowerShell).
- [x] Method Cycling (`m` key)
- [x] Multipart Form Data Support (Form Data & File Uploads via `Space`)
- [x] GraphQL Support (Query & Variables editing)
- [x] Status Codes (Color-coded)
- [x] Help Screen (`?` key)
- [x] Header Editing (`H` key -> `$EDITOR` as JSON)
- [x] Persistence (`s` key -> `saved.hcl`)
- [x] **Persistence for Chain Rules & Multipart Data**
- [x] Refactor Collection Saving Logic
- [x] **gRPC Support 🚀**: Full gRPC client via `grpcurl` (Requires `grpcurl` installed)
- [x] **Cookie Jar**: Automatically stores and sends `Set-Cookie` headers for stateful sessions
- [x] **Code Generators**: Generate request code for Python (Requests) and JavaScript (Fetch) with `G` and `J` keys
- [x] **WebSocket Support**: Full WS/WSS client with `Ctrl+W` to toggle mode, real-time messaging, and connection management
- [x] **Pre-Request Scripts**: Rhai scripting engine for running hooks before requests (`P` to edit)
- [x] **Collection Runner**: Run all requests in a collection sequentially with status code assertions (`Ctrl+R`)
- [x] **Dynamic Themes**: Cycle between Matrix, Cyberpunk, Dracula, and Default themes (`Ctrl+T`)
- [x] **Splash Screen**: Awesome retro-terminal startup screen
- [x] **Import/Export**: Import Postman Collections (`.json`) via `--import` flag
- [x] **Test Scripts**: Post-request assertions (Rhai) and console logs (`Shift+T` to edit)
- [x] **Request Tabs 📑**: Work on multiple requests simultaneously
- [x] **Syntax Highlighting 🎨**: In-app JSON/code highlighting
- [x] **Response History 📜**: View previous responses for a request
- [x] **Request Diff 🔀**: Compare two responses side by side (Press 'D' in History)
- [x] **API Documentation Gen 📝**: Generate docs from collections (Press 'M')
- [x] **Mock Servers 🎭**: Create mock endpoints for testing (Press 'Ctrl+K')
- [x] **GraphQL Introspection 🔍**: Auto-complete from schema (Press 'Ctrl+I')

### WebSocket Mode (`Ctrl+W` to enter)
- **e**: Edit WebSocket URL
- **Enter**: Connect / Disconnect
- **i**: Start typing a message
- **Enter** (while typing): Send message
- **j/k**: Scroll through message history
- **x**: Clear message history
- **?**: WebSocket Help

### Collection Runner (`Ctrl+R` to enter)
Run all requests in a collection sequentially and see pass/fail results.

- **j/k**: Navigate collections (before run) or scroll results (after run)
- **Enter**: Run selected collection
- **x**: Clear results
- **Esc**: Exit runner mode
- **?**: Help

### Mock Server Manager (`Ctrl+K`)
Create and run local mock endpoints.

- **s**: Start / Stop Server (Port 3000 default)
- **a**: Add new mock route (template)
- **d**: Delete selected route
- **Esc**: Exit Manager

### gRPC Mode (Switch Body to 'gRPC' with `m` key)

Make gRPC calls using `grpcurl` as backend.

**Prerequisites:** Install [grpcurl](https://github.com/fullstorydev/grpcurl)

**Usage:**
1. Set URL to your gRPC server (e.g., `localhost:50051`)
2. Press `m` in Body tab until you get to `gRPC (Proto)` mode
3. Press `u` to set Service/Method (e.g., `grpc.health.v1.Health/Check`)
4. Press `p` to set Proto file path (optional, for servers without reflection)
5. Press `b` to edit JSON payload
6. Press `Enter` to send the request

**Keys:**
- **u**: Edit Service/Method
- **p**: Edit Proto file path
- **b**: Edit request body (JSON format)
- **L** (Shift+L): Discover services via server reflection
- **D** (in services list): Show service method signatures
- **Enter**: Send gRPC request / Select service


**Status Code Assertions:**
By default, expects HTTP 200. Add `expected_status = XXX` in your `.hcl` file to specify a different expected status:

```hcl
request "Create User" {
  method = "POST"
  url = "https://api.example.com/users"
  expected_status = 201
}
```

### Pre-Request Scripts (`P` to edit)
Press `P` to open your `$EDITOR` and write Rhai scripts that run before each request.

**Available Functions:**
| Function | Description |
|----------|-------------|
| `set_header(name, value)` | Add or modify a request header |
| `get_header(name)` | Get current header value |
| `set_var(name, value)` | Set an environment variable |
| `get_var(name)` | Get an environment variable |
| `set_body(body)` | Override the request body |
| `set_url(url)` | Override the request URL |
| `timestamp()` | Get Unix timestamp (seconds) |
| `timestamp_ms()` | Get Unix timestamp (milliseconds) |
| `uuid()` | Generate a random UUID v4 |
| `base64_encode(text)` | Encode text as Base64 |
| `base64_decode(text)` | Decode Base64 text |
| `print(msg)` | Debug log (shows in output) |

**Constants:** `METHOD`, `URL`, `BODY`

**Example Script:**
```rhai
// Add timestamp header to every request
let ts = timestamp();
set_header("X-Request-Time", ts.to_string());

// Add request ID
let id = uuid();
set_header("X-Request-ID", id);
```

### Post-Request Test Scripts (`Shift+T`)

Run assertions on the response. Results appear in the status bar and "Tests & Console" panel.

**Additional Functions:**
| Function | Description |
|----------|-------------|
| `test(name, bool)` | Record a test result |
| `status_code()` | Get response status code |
| `response_time()` | Get latency in ms |
| `response_body()` | Get raw response body |
| `json_path(query)` | Extract value using JSONPath (e.g. `$.data.id`) |

**Example:**
```rhai
test("Status is 200", status_code() == 200);
test("Fast response", response_time() < 500);

let token = json_path("$.token");
test("Token received", token != "");

if token != "" {
    print("Received token: " + token);
}
```

## Leveling Up to Postman (Future Ideas)

To truly rival Postman, we still need:

1. **More Code Generators 💻**: Add support for Go, Rust, Ruby, PHP, and C#
2. **Proxy Support 🔒**: HTTP/SOCKS proxy configuration for corporate environments
3. **SSL Certificate Config 🔐**: Custom CA certs, client certificates
4. **Request Timeout Settings ⏱️**: Per-request timeout configuration

## License

MIT
