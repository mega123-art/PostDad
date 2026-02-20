# 🚀 PostDad Roadmap & Feature Ideas

This document outlines potential future features and upgrades to existing functionalities for PostDad.

## 🎨 UI & UX Enhancements
### 1. Advanced JSON Explorer
*   **Current**: Sequential list view.
*   **Upgrade**: Fully interactive, navigable tree with keyboard-driven folding/unfolding.
*   **Feature**: "Copy Path as JSONPath" or "Copy Path as Rhai Script" directly from the selection.

### 2. Layout Customization
*   **Feature**: Toggle between Vertical (default) and Horizontal layouts for Request/Response views.
*   **Feature**: Zen Mode 2.0 – Focus only on the active input or the response output.

### 3. Cheat Sheet Overlay
*   **Feature**: A searchable command palette (like VS Code) for every keyboard shortcut, triggered by `Ctrl+P`.

---

## 🛠️ Protocol Support
### 1. MQTT Client
*   **Feature**: A dedicated mode for subscribing and publishing to MQTT topics. Great for IoT developers.

### 2. Headless CLI Runner
*   **Feature**: Run collections directly from the terminal without the TUI (`PostDad run my_collection.json`) for CI/CD integration.

### 3. OpenAPI / Swagger Import
*   **Feature**: Direct import of Swagger (OAS 3.0) URLs or files to instantly generate collections.

---

## ⚡ Sentinel Mode Upgrades
### 1. Visual Latency Graphs
*   **Feature**: Real-time Sparklines showing the last 50-100 request latencies to spot performance spikes visually.

### 2. Alert Webhooks
*   **Feature**: Configure Discord/Slack or Custom Webhooks to trigger on Sentinel failures.

---

## 🧠 Scripting & Automation
### 1. Script Libraries
*   **Upgrade**: Add built-in libraries for Rhai scripts:
    *   `crypto.sha256()` / `crypto.hmac()`
    *   `jwt.sign()` / `jwt.verify()` (useful for auth testing)
    *   `faker` style random data generation.

### 2. Request Chaining (Workflows)
*   **Feature**: A dedicated "Flow" mode where you can drag/order requests and automatically map the output of Request A to the body/params of Request B.

---

## 🧪 Testing & Stress Upgrades
### 1. Dynamic Stress Testing
*   **Upgrade**: Allow Stress Tests to vary payloads (e.g., using random variables from a CSV file).

### 2. Detailed Performance Reports
*   **Feature**: Export stress test results as high-quality PDF or CSV reports with P50, P90, and P99 metrics.

---

## ☁️ Integrations
### 1. Gist Sync
*   **Feature**: Backup/Sync your environments and collections to a private GitHub Gist.

### 2. Postman / Insomnia Bridge
*   **Feature**: Seamless export of PostDad collections into native formats for other clients.

---

## 🏗️ Technical Upgrades
*   **Plugin System**: Allow users to write their own TUI widgets or data importers in Rust or via Rhai.
*   **Performance**: Use streaming parsers for multi-megabyte JSON responses to keep the UI at 60fps.
