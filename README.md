# Orbit 🛰️

<p align="center">
  <strong>Lightweight, high-performance native desktop Kubernetes dashboard.</strong>
</p>

<p align="center">
  <a href="https://github.com/vantoan1511/orbit/releases"><img src="https://img.shields.io/github/v/release/vantoan1511/orbit?style=flat-square&color=4f8cff" alt="Latest Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-emerald?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Backend-Rust-orange?style=flat-square&logo=rust" alt="Rust"></a>
  <a href="https://neutralino.js.org/"><img src="https://img.shields.io/badge/Runtime-Neutralinojs-8b5cf6?style=flat-square" alt="Neutralinojs"></a>
  <a href="https://vuejs.org/"><img src="https://img.shields.io/badge/Frontend-Vue%203-emerald?style=flat-square&logo=vue.js" alt="Vue 3"></a>
  <a href="https://primevue.org/"><img src="https://img.shields.io/badge/UI-PrimeVue%20v4-blue?style=flat-square" alt="PrimeVue v4"></a>
</p>

---
   
Orbit is a fast, native-feeling desktop dashboard crafted for exploring, monitoring, and managing Kubernetes clusters. Built on top of a compiled **Rust engine** and the lightweight **Neutralinojs** desktop runtime, Orbit delivers instant startup, low memory footprint, and low-latency cluster API communication — without the bloat of traditional Electron applications.

---

## ⚡ Why Orbit?

- **Native Rust Engine**: Low-latency communication with Kubernetes APIs, stream parsing, and credentials management handled by compiled native Rust binaries.
- **Zero Electron Overhead**: Powered by [Neutralinojs](https://neutralino.js.org/), utilizing native operating system webview capabilities rather than bundling a duplicate Chromium browser.
- **Security-First Architecture**: Sensitive kubeconfigs, tokens, and certificates are isolated inside the local Rust backend and never directly exposed to the frontend presentation layer.
- **Compact & Technical Design**: Inspired by modern developer IDEs and the PrimeVue Nora theme — dense, monochrome/noir, and designed for efficient daily cluster operations.

---

## ✨ Features

### ☸️ Cluster & Context Management

- **Instant Context Switching**: Seamlessly toggle between local (k3s, Minikube, Kind) and remote cloud Kubernetes clusters.
- **Namespace Filtering**: Global namespace scope switcher with immediate resource updates across all views.
- **Offline Cluster Awareness**: Graceful handling and clear diagnostics when clusters or nodes are unreachable.

### 📦 Complete Resource Explorer

- **Workloads**: Inspect Deployments, Pods, StatefulSets, DaemonSets, Jobs, CronJobs, and ReplicaSets with live status indicators.
- **Configuration & Storage**: Inspect ConfigMaps, Secrets, HPAs, PersistentVolumes, PersistentVolumeClaims, and StorageClasses.
- **Network & Security**: Explore Services, Ingresses, NetworkPolicies, ResourceQuotas, and LimitRanges.
- **Cluster Infrastructure**: Deep-dive into Nodes, Namespaces, and system-level events.

### 📝 Live YAML & In-App Apply

- **Integrated Monaco Editor**: Syntax-highlighted YAML editor with real-time Kubernetes schema awareness.
- **Direct Apply & Diff**: Edit and apply manifests directly to the cluster with structured error feedback.

### 📊 Real-Time Logs & Events

- **Pod Log Streaming**: Follow live container logs, switch between multi-container pods, and filter log output.
- **Cluster Events**: Filter and monitor recent cluster warnings, errors, and lifecycle events.

### 🔄 Seamless Background Updates

- **Automatic Version Checks**: Integrated lightweight Rust updater keeps your installation secure and up to date.

---

## 🏗️ Architecture

Orbit enforces a strict single-responsibility boundary between the frontend and backend:

```
┌─────────────────────────────────────────────────────────┐
│                    Vue 3 Frontend                       │
│  - Composition API + TypeScript                         │
│  - PrimeVue v4 (Nora Theme) + Tailwind CSS v4           │
│  - View State, Monaco YAML Editor, UI Interactions      │
└────────────────────────────┬────────────────────────────┘
                             │ Neutralino IPC
┌────────────────────────────▼────────────────────────────┐
│                     Rust Backend                        │
│  - `core/engine`: Kubernetes API, kubeconfig, Cache     │
│  - `core/updater`: Dedicated auto-update mechanism      │
│  - Privileged OS, Network & Filesystem operations       │
└─────────────────────────────────────────────────────────┘
```

- **Frontend (`src/`)**: Pure presentation layer communicating only through structured IPC calls.
- **Backend (`core/engine`, `core/updater`)**: Handles all Kubernetes client logic, kubeconfig discovery, and system interactions.
- **IPC Boundary**: Strongly typed request/response contracts ensuring stability and strict separation of concerns.

---

## 🛠️ Tech Stack

| Layer                     | Technology                                                                                  |
| ------------------------- | ------------------------------------------------------------------------------------------- |
| **Desktop Runtime**       | [Neutralinojs](https://neutralino.js.org/)                                                  |
| **Backend**               | [Rust](https://www.rust-lang.org/) (`kube-rs`, `tokio`)                                     |
| **Frontend Framework**    | [Vue 3](https://vuejs.org/) (Composition API, TypeScript)                                   |
| **UI Components & Theme** | [PrimeVue v4](https://primevue.org/) (Nora Preset)                                          |
| **Styling**               | [Tailwind CSS v4](https://tailwindcss.com/)                                                 |
| **State Management**      | [Pinia](https://pinia.vuejs.org/)                                                           |
| **Editor**                | [Monaco Editor](https://microsoft.github.io/monaco-editor/) via `@guolao/vue-monaco-editor` |
| **Icons**                 | [Lucide Vue](https://lucide.dev/)                                                           |
| **Build Tool**            | [Vite](https://vite.dev/)                                                                   |

---

## 💾 Installation

### Windows

1. Navigate to the [Releases](https://github.com/vantoan1511/orbit/releases) page.
2. Download the latest `Orbit-Setup-x.y.z.exe` installer.
3. Run the installer to complete the setup.

> _Note: macOS and Linux support is planned for upcoming releases._

---

## 💻 Local Development

### Prerequisites

- [Node.js](https://nodejs.org/) (`>= 20.19.0` or `>= 22.12.0`)
- [Rust](https://www.rust-lang.org/) (Stable toolchain)
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) (Only required for building the Windows installer package)

### Getting Started

1. **Clone the repository:**

   ```bash
   git clone https://github.com/vantoan1511/orbit.git
   cd orbit
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Update Neutralino binaries:**

   ```bash
   npm run neu:update
   ```

4. **Run in development mode:**
   ```bash
   npm run neu:run
   ```

### Available Scripts

| Command              | Description                                                                     |
| -------------------- | ------------------------------------------------------------------------------- |
| `npm run dev`        | Starts the Vite development server                                              |
| `npm run neu:run`    | Launches the Neutralino desktop application in dev mode                         |
| `npm run build`      | Runs type checks and bundles the frontend production assets                     |
| `npm run package`    | Builds backend binaries, compiles resources, and packages the Windows installer |
| `npm run type-check` | Runs `vue-tsc` to validate TypeScript types across Vue components               |
| `npm run lint`       | Lints and fixes source code using ESLint                                        |
| `npm run format`     | Formats the codebase using Prettier                                             |

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/vantoan1511/orbit/issues).

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feat/amazing-feature`)
3. Commit your Changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the Branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

---

## ⚖️ License

Distributed under the [MIT License](LICENSE). Copyright &copy; 2026 Toan Nguyen.
