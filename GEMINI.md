# GEMINI.md

# Project

Orbit is a lightweight, native desktop Kubernetes dashboard.

Technology stack:

- Rust (backend)
- Neutralinojs (desktop runtime)
- Vue 3
- Tailwindcss v4
- PrimeVue v4
- TypeScript
- Vite

The frontend communicates with the Rust backend exclusively through Neutralinojs IPC. Rust is responsible for all privileged operations, while Vue is responsible for presenting data and handling user interactions.

---

# Coding rules

Before generating code:

1. Inspect the existing architecture.
2. Follow established project conventions.
3. Reuse existing modules whenever possible.
4. Keep changes focused and minimal.
5. Avoid unrelated refactoring.
6. Explain trade-offs when multiple implementations are reasonable.
7. Always check-out to another branch for features implementing or bugs fixing, never work on main/master.

Never invent APIs that do not exist.

If a required API is missing, propose adding it rather than assuming it already exists.

---

# Mission

Orbit should be:

- Lightweight
- Fast
- Native-feeling
- Reliable
- Secure
- Cross-platform
- Easy to maintain
- Production ready

Every implementation should favor simplicity, predictability, and long-term maintainability.

---

# Architecture

Orbit is divided into two layers.

## Backend (Rust)

The Rust backend owns:

- Kubernetes API communication
- File system access
- Network access
- Authentication
- kubeconfig management
- Caching
- Background tasks
- Business logic
- Performance-critical operations
- System integration

The backend should not contain UI concerns.

---

## Frontend (Vue)

The frontend owns:

- Rendering
- User interactions
- View state
- Routing
- Local UI state
- Animations

The frontend should avoid implementing business logic.

Whenever logic requires Kubernetes knowledge or system access, it belongs in Rust.

---

# IPC

The IPC boundary is the contract between frontend and backend.

Always:

- keep commands small
- use strongly typed request/response models
- return structured errors
- avoid sending unnecessary data
- keep payloads versionable

Do not expose internal backend implementation details through IPC.

Treat IPC as a stable public API.

---

# Single Source of Truth

Business rules must exist in one place only.

Avoid duplicating logic between Rust and TypeScript.

If validation or calculations are required, prefer implementing them in Rust and exposing the results through IPC.

---

# Rust Guidelines

Prefer:

- ownership over unnecessary cloning
- explicit error handling
- Result<T, E>
- idiomatic Rust
- modular crates
- strong typing

Avoid:

- unwrap()
- expect() outside tests
- panic! for recoverable errors
- unnecessary Arc<Mutex<T>>
- global mutable state

Prefer immutable data structures whenever practical.

---

# Vue Guidelines

Use:

- Composition API
- script setup
- TypeScript
- composables
- reusable components

Avoid:

- Options API
- large components
- business logic inside views
- direct IPC calls scattered across components

Components should remain focused on rendering.

---

# Services

Frontend services should only wrap IPC calls.

Example flow:

View

↓

Composable

↓

Frontend Service

↓

Neutralino IPC

↓

Rust Backend

↓

Kubernetes

Components should never call IPC directly.

---

# State Management

Keep state local whenever possible.

Shared state should only exist when genuinely shared across multiple views.

Do not duplicate backend state inside multiple frontend stores.

---

# Kubernetes

The Rust backend owns every interaction with Kubernetes.

Never access Kubernetes directly from the frontend.

Support:

- multiple clusters
- multiple contexts
- namespace switching
- watch APIs
- reconnect behavior
- authentication providers

Never assume:

- default namespace
- cloud provider
- Kubernetes version

---

# Security

Sensitive information never belongs in the frontend.

Do not expose:

- kubeconfig contents
- private keys
- bearer tokens
- certificates
- credentials

The frontend should only receive the minimum information necessary to render the UI.

---

# Error Handling

Errors returned through IPC should:

- include machine-readable error codes
- contain user-friendly messages
- avoid leaking implementation details

Unexpected failures should be logged by the backend.

---

# Performance

Prefer:

- streaming or incremental updates
- background workers
- caching
- lazy loading
- batching IPC requests

Avoid polling when Kubernetes watch APIs are available.

Avoid unnecessary serialization across IPC.

---

# Dependencies

Before adding dependencies:

Evaluate:

- maintenance
- security
- compile time
- binary size
- community adoption

Prefer standard library functionality whenever practical.

Avoid introducing dependencies for small utilities.

---

# Code Style

Write code that is easy to understand.

Prefer descriptive names.

Avoid abbreviations.

Good:

- clusterConnection
- activeNamespace
- workloadSummary

Avoid:

- ctx
- obj
- tmp
- data1

Variable Naming in CSS

Good:

- border-(--border)
- min-w-35
- shrink-0

Avoid

- border-[var(--border)]
- min-w-[140px]
- flex-shrink-0

---

# Comments

Comments should explain why.

Avoid comments that simply describe what the code is doing.

---

# Logging

Backend:

- log diagnostics
- log failures
- avoid sensitive data

Frontend:

- avoid excessive console logging
- never log secrets

---

# Documentation

Whenever a public IPC command changes:

Update:

- IPC documentation
- request schema
- response schema
- examples

Documentation should reflect implementation.

---

# Goal

Every contribution should move Orbit closer to being a professional-grade native Kubernetes desktop application with a clear separation between the Rust backend and the Vue frontend, connected through a stable, well-defined IPC interface.
