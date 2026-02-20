# PostDad

A fast API client for your terminal. Written in Rust.

```
   ____           _      _           _ 
  |  _ \ ___  ___| |_ __| | __ _  __| |
  | |_) / _ \/ __| __/ _` |/ _` |/ _` |
  |  __/ (_) \__ \ || (_| | (_| | (_| |
  |_|   \___/|___/\__\__,_|\__,_|\__,_|
```

I got tired of waiting for Postman to load so I built this.

## Install

```bash
cargo install PostDad
```

Then run `PostDad` to start.

## What it does

Three-pane TUI: collections on left, request builder on top, response at bottom.

- **Vim keys** - `j`/`k` to move, `e` to edit URL, `/` to search
- **JSON explorer** - expand/collapse nodes in large responses
- **Local storage** - everything saved as `.hcl` files, no account needed
- **Image Rendering** - View images directly in terminal (High-res via Sixel/Kitty, fallback to Ascii/Blocks)
- **Non-blocking** - UI stays responsive even when requests hang

## Quick reference

### General
| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Toggle help overlay |
| `Ctrl+h` | Switch focus: Sidebar ↔ Main |
| `Ctrl+e` | Switch environment |
| `Ctrl+t` | Cycle themes |
| `Ctrl+z` | Toggle Zen mode |

### Command Palette
| Key | Action |
|-----|--------|
| `Ctrl+p` | Open Command Palette (search commands) |
| `:` | Enter Command Mode (type commands like `quit`, `new`, `theme`) |

### Tabs & Navigation
| Key | Action |
|-----|--------|
| `Ctrl+n` | New request tab |
| `Ctrl+x` | Close current tab |
| `[ / ]` | Cycle between open tabs |
| `Tab` | Cycle: Params → Headers → Body → Auth → Chain |
| `j / k` | Move up/down in lists |
| `h / l` | Collapse/expand JSON nodes |
| `/` | Search/filter JSON response |

### Request Building
| Key | Action |
|-----|--------|
| `e` | Edit URL |
| `m` | Cycle HTTP method |
| `t` | Switch Body Type (Body Tab) / Auth Type (Auth Tab) |
| `H` | Edit headers (external editor) |
| `b` | Edit body (external editor) |
| `Enter` | Send request |
| `s` | Save request to collection |
| `f` | Toggle fullscreen response |

### Response
| Key | Action |
|-----|--------|
| `C` | Copy response to clipboard |
| `D` | Download response (detects binary/images, saves to file) |
| `Shift+D` | Force download binary content |
| `P` | Preview Response (or open in external viewer) |
| `D` | **Diff View**: Press `D` on a history item (side bar) to select Base, then `D` on another to Compare. |
| `y` | Copy JSON path of selected node |

### Body modes

Press `t` in the Body tab to cycle through: Raw JSON, Multipart (for file uploads), GraphQL, gRPC.

### Auth

Press `t` in the Auth tab to switch between: None, Bearer token, Basic auth, OAuth 2.0.

For OAuth, hit `Enter` to start the browser flow.

### WebSocket

`Ctrl+w` toggles WebSocket mode. Connect to a WS endpoint, send messages, see responses in real-time.

### gRPC

Needs [grpcurl](https://github.com/fullstorydev/grpcurl) installed. Set your URL to the gRPC server, switch body mode to gRPC, and go.

### Mock server

`Ctrl+k` opens the mock server manager. You can spin up endpoints on localhost for testing.

### Scripts

- `P` - Edit pre-request script (runs before sending)
- `Shift+T` - Edit test script (runs after response)

Scripts are written in [Rhai](https://rhai.rs/). You get functions like `set_header()`, `json_path()`, `timestamp()`, etc.

Example:
```rhai
set_header("X-Request-ID", uuid());
test("Status OK", status_code() == 200);
```

### Chaining Requests

You can extract values from a response to use in future requests (like an Auth Token).

1. Go to the **Chain** tab (Tab 5).
2. Add a rule: `auth_token` <- `data.token`.
3. This saves `data.token` from the JSON response into the `{{auth_token}}` variable.
4. Use `{{auth_token}}` in your next request header/body.

### Import

```bash
PostDad --import collection.json
```

Supports both **Postman** and **OpenAPI v3** formats (auto-detected):

- **Postman**: Import your existing Postman collections
- **OpenAPI**: Import `openapi.json` specs to auto-generate request collections

Example with OpenAPI:
```bash
PostDad --import openapi.json
# → Detected OpenAPI v3 format
# → Successfully imported 'Pet Store API' v1.0.0 to 'collections/pet_store_api.hcl'
# → 15 requests created
```

### cURL Import

You can also import single requests from cURL commands while the app is running:

1. Press `I` (Shift+i) to open the import modal
2. Paste your cURL command
3. Press `Enter` to populate the current request tab

Supported features:
- Method (`-X`, `--request`)
- Headers (`-H`, `--header`)
- Body (`-d`, `--data`)
- Form Data (`-F`, `--form`)
- Basic Auth (`-u`, `--user`)
- Auto-handles quotes and line continuations

### Stress Testing

PostDad includes a built-in load testing tool (similar to k6 but simpler).

1. Press `%` (`Shift+5`) to open the Stress Test modal.
2. Enter **Virtual Users (VUs)** (concurrency) and **Duration** (seconds).
3. Hit `Enter` to start the attack.

**Metrics:**
- Real-time progress bar
- Requests per second (RPS)
- Latency (Avg, P95, Max)
- Error rate / Status codes

**Note**: This runs from your local machine, so you're limited by your own CPU/Network.

### Sentinel Mode 🛡️

A live TUI monitoring dashboard for your API endpoints. 

1. Press `Shift+S` (or `S` in command mode) to **Start/Stop** Sentinel Mode.
2. The dashboard replaces the main view with:
   - **Real-time Latency Sparkline**: Visualizing performance trends.
   - **Status History**: Recent HTTP status codes.
   - **Key Metrics**: Total checks, failed checks, and last latency.
3. **Controls**:
   - `S`: Start/Stop monitoring.
   - `L`: Save history to CSV log.
   - `Esc`: Exit dashboard.

### Sentinel Failure Conditions

By default, Sentinel alerts on non-200 status codes. You can also fail on specific response content using the `X-Fail-If` header configuration (hidden from actual request).

1. In Header tab, add `X-Fail-If` with a keyword (e.g., `error_code":"500`).
2. If that keyword appears in the response body, Sentinel marks it as a failure (Status 500).

### Documentation Generator

Generate offline documentation for your collections in one keystroke.

1. Press `M` (or use Command Palette: `Export HTML Docs`).
2. Generates:
   - `API_DOCS.md`: Markdown file for Git/Wiki.
   - `API_DOCS.html`: Single-page, beautiful HTML site with sidebar navigation and search.
3. Both files are saved to your current directory.

### Environments

Separate your logic (Dev/Staging/Prod) using `environments.hcl`.

```hcl
env "production" {
  base_url = "https://api.myapp.com"
  token = "prod_secret_123"
}

env "local" {
  base_url = "http://localhost:3000"
  token = "dev_token"
}
```

Use variables in your requests like syntax: `{{base_url}}/users`.
Switch environments with `Ctrl+e`.

## CLI mode

Run collections without the TUI - useful for CI/CD pipelines.

```bash
# Run a collection
PostDad run api_tests.hcl

# Start Mock Server
PostDad mock --port 3000 --routes routes.json

# With environment variables
PostDad run api_tests.hcl -e production.hcl

# JSON output for scripting
PostDad run api_tests.hcl --json > results.json

# Verbose mode (shows URLs)
PostDad run api_tests.hcl -v
```

Exit codes: 0 if all requests pass, 1 if any fail.

## Storage

Everything lives in `.hcl` files. Press `s` to save your current request.

```hcl
request "Get users" {
  method = "GET"
  url = "https://api.example.com/users"
  expected_status = 200
  timeout_ms = 5000
}
```

Chain rules and environment variables are persisted too.

## Why not just use curl?

Curl is great for one-offs. This is for when you're actively developing against an API and want to:
- Keep a collection of requests around
- See formatted JSON responses
- Chain requests together (extract a token from response A, use it in request B)
- Not retype the same headers over and over

## SSL Certificates

PostDad supports custom SSL certificates for enterprise environments:

### Configuration via Environment Variables

```bash
# Custom CA certificate (for self-signed/internal CAs)
export POSTDAD_CA_CERT=/path/to/ca.pem

# Client certificate for mTLS
export POSTDAD_CLIENT_CERT=/path/to/client.pem
export POSTDAD_CLIENT_KEY=/path/to/client.key

# Disable SSL verification (development only!)
export POSTDAD_SSL_VERIFY=false
```

### What's Supported

- **Custom CA Certificates**: Trust internal/self-signed certificates
- **Client Certificates (mTLS)**: Authenticate with client certificates
- **Skip Verification**: For development with self-signed certs (not recommended for production)

## Proxy Support

PostDad supports HTTP/HTTPS proxies for corporate networks:

### Configuration via Environment Variables

```bash
# Standard proxy variables (auto-detected)
export HTTPS_PROXY=http://proxy.company.com:8080
export HTTP_PROXY=http://proxy.company.com:8080

# Hosts to bypass proxy (comma-separated)
export NO_PROXY=localhost,127.0.0.1,.internal.company.com

# Proxy authentication (if required)
export POSTDAD_PROXY_USER=username
export POSTDAD_PROXY_PASS=password
```

### What's Supported

- **HTTP/HTTPS Proxies**: Route all traffic through corporate proxy
- **Proxy Authentication**: Basic auth for authenticated proxies
- **NO_PROXY Bypass**: Skip proxy for specific hosts/domains

## Code Generators

Instantly generate code snippets for your current request in multiple languages:

| Key | Language | Output |
|-----|----------|--------|
| `c` | cURL | Shell command |
| `G` | Python | `requests` library |
| `J` | JavaScript | `fetch` API |
| `O` | Go | `net/http` package |
| `R` | Rust | `reqwest` crate |
| `B` | Ruby | `Net::HTTP` library |
| `E` | PHP | `curl_*` functions |
| `S` | C# | `HttpClient` class |

The generated code is copied directly to your clipboard.

## Themes

Customize your look with `Ctrl+t`.
- **Default**: Classic dark mode
- **Matrix**: Green on black
- **Cyberpunk**: Neon pink/cyan
- **Dracula**: Vampire contrast

## License

MIT
