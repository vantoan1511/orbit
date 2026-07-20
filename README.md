# Orbit 🛰️

**Navigate Kubernetes with confidence.**

Orbit is a fast, lightweight, and native desktop dashboard designed to explore, monitor, and manage your Kubernetes clusters. Built to feel native and stay resource-efficient, Orbit gives you a powerful visual overview of your clusters without the heavy memory footprint and sluggish startup times of traditional Electron-based tools.

---

## ⚡ Why Orbit?

* **Native Performance**: Leverages a compiled Rust engine for low-latency cluster API communication and high-speed data processing.
* **Minimal Resource Footprint**: Built on [Neutralinojs](https://neutralino.js.org/), bypassing the heavy Chromium runtime package. It runs on a fraction of the RAM used by typical desktop dashboards.
* **Production-Ready Security**: Sensitive credentials, tokens, and certificates are handled strictly by the local Rust backend and never exposed to the frontend presentation layer.

---

## ✨ Features

- **Multi-Cluster Context Switching**: Easily toggle between different Kubernetes clusters and contexts.
- **Resource Explorer**: Inspect and manage pods, deployments, services, config maps, namespaces, and custom resource definitions (CRDs).
- **Real-Time Logs & Events**: Instantly stream logs and view cluster events directly in a clean, filterable interface.
- **Global Namespace Filtering**: Filter resource views globally or drill down into specific namespaces.
- **Automatic Updates**: Always run the latest version with built-in, secure background updates managed by a dedicated rust updater.
- **Modern UI/UX**: Responsive interface built using PrimeVue v4 and Tailwind CSS v4, supporting rich animations and transitions.

---

## 💾 Installation

Orbit is currently available for Windows.

### Download the Installer
1. Go to the [Releases](https://github.com/vantoan1511/orbit/releases) page.
2. Download the latest `Orbit-Setup-x.y.z.exe` installer.
3. Run the installer to install Orbit on your system.

*Support for macOS and Linux builds is coming soon.*

---

## 🏗️ Architecture

Orbit is structured to maintain a clean separation of concerns:

```
┌──────────────────────────────────────┐
│            Vue 3 Frontend            │  <-- Rendering, View State, UI Animations
└──────────────────┬───────────────────┘
                   │ Neutralino IPC
┌──────────────────▼───────────────────┐
│             Rust Backend             │  <-- K8s API, Credentials, File System, Updater
└──────────────────────────────────────┘
```

- **Rust Backend (`core/engine` & `core/updater`)**: Handles all privileged operations including Kubernetes API communication, config parsing, file system writes, and background update tasks.
- **Vue Frontend (`src/`)**: Built using Vue 3 (Composition API), Tailwind CSS v4, and PrimeVue v4. It manages the presentation layer and user interactions.
- **Neutralinojs IPC**: The frontend wrapper and the Rust engine communicate securely via Neutralino's lightweight IPC bridge.

---

## 🛠️ Tech Stack

* **Desktop Runtime**: [Neutralinojs](https://neutralino.js.org/)
* **Backend Engine**: [Rust](https://www.rust-lang.org/)
* **Frontend Framework**: [Vue 3](https://vuejs.org/) (Composition API, TypeScript)
* **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) & [PrimeVue v4](https://primevue.org/)
* **Build System**: [Vite](https://vite.dev/)

---

## 💻 Local Development

### Prerequisites
- [Node.js](https://nodejs.org/) (v20 or newer)
- [Rust](https://www.rust-lang.org/) (stable)

### Setup & Run
1. Clone the repository and install npm packages:
   ```bash
   git clone https://github.com/vantoan1511/orbit.git
   cd orbit
   npm install
   ```

2. Download Neutralino client binaries:
   ```bash
   npx neu update
   ```

3. Run the app in development mode:
   ```bash
   npx neu run
   ```

### Building & Packaging
To build both the frontend assets and compile the Rust binaries locally:
- **Build assets**: `npm run build`
- **Compile installer**: `npm run package` (compiles backend binaries and packages the Windows installer using Inno Setup under the `dist/` directory)

---

## ⚖️ License

Orbit is licensed under the [MIT License](LICENSE).
