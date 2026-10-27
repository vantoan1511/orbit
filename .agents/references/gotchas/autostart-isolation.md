# Autostart Isolation & OS Store Protection

## Context
Orbit supports automatically launching at user login via operating system autostart facilities:
- Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- macOS: `~/Library/LaunchAgents/{name}.plist`
- Linux: `~/.config/autostart/{name}.desktop`

## Gotcha: OS Autostart Pollution During Unit/Integration Tests
Previous versions of `AutostartManager` invoked `current_exe()` and `new_reg_command()` directly.
When running tests (`cargo test`), this caused:
1. Registration of ephemeral cargo test binaries (`target\debug\deps\orbit_app_lib-<hash>.exe`) under `"Orbit Dev"` in the real Windows Run registry key.
2. Machine pollution that survived reboots and test completion.
3. Inability to safely run `cargo test` in CI or developer machines without corrupting the host OS autostart configuration.

## Prevention & Architecture Rules
1. **Never Call Real OS Backends in Tests**:
   - The global provider `crate::autostart::service()` automatically defaults to `InMemoryBackend` under `#[cfg(test)]`.
   - If a test needs custom autostart behavior, use `AutostartTestGuard::install(...)`.
   - Do NOT construct `WindowsRunKeyBackend::production()` or `FileAutostartBackend::for_current_os()` in any test intended to mutate autostart state.
2. **Registry Sandbox Pattern (Windows)**:
   - When verifying Windows registry behavior, use `SandboxRunKey`.
   - `SandboxRunKey` registers entries under an isolated subkey `HKCU\Software\OrbitTest\<uuid>\Run`.
   - Its RAII `Drop` implementation deletes the entire subtree `HKCU\Software\OrbitTest\<uuid>` safely, even if a test panics (`catch_unwind` safe).
3. **Defense in Depth**:
   - `AutostartService` rejects any executable residing in a Cargo `deps/` directory with `AutostartError::UnsafeExecutable`.
   - Development mode (`RunMode::Dev`) automatically skips autostart mutations (`ApplyOutcome::SkippedDevMode`).
   - Stale test binaries and legacy `"Orbit Dev"` registrations are automatically cleaned up on application startup before reconciliation.
