# WebX Browser 🌐

**WebX** is the official system browser for **Ledokoz OS**, built in **pure Rust** with a focus on speed, safety, and tight OS integration.

WebX is designed to be lightweight, modern, and hackable — a browser that feels *native* to the operating system, not bolted on.

---

## ✨ Goals

* Native Rust browser for Ledokoz OS
* Minimal memory usage
* Secure by default
* Clean, modern UI
* Strong foundation for future features (tabs, extensions, devtools)

---

## 🧱 Tech Stack

### Core Language

* **Rust** — memory-safe, fast, and ideal for OS-level software

### GUI Framework

* **Native System Integration**

  * Tao for window management and event handling
  * WRY for WebView integration
  * Cross-platform system-native appearance
  * Direct OS API access for optimal performance

### Web Engine

* **System WebView** (via platform integration)

  * WebKit (Linux/macOS)
  * WebView2 (Windows)
  * Allows fast rendering without shipping a full engine

### Window & Events

* **Tao** — windowing and event loop abstraction
* **WRY** — WebView and web content rendering

---

## 🎨 Design Philosophy

* Flat, clean UI
* Dark-mode first
* No unnecessary animations
* Fast startup time

---

## 🔒 Security Principles

* No telemetry by default
* Sandboxed web content
* Minimal attack surface
* OS-level permission control

---

## Development

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run
```

---

## 📜 License

WebX is licensed under the **GNU GERENERAL PUBLIC LICENSE V3**. for more information, see [LICENSE](./LICENSE)

---

## 🧠 Part of the Ledokoz Ecosystem

WebX is a core application of **Ledokoz OS**, alongside:

* WebX (Browser)

> Built with long-term vision. Designed for the future.
