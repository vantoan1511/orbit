import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { spawnSync } from 'node:child_process'

describe('Shift-Left PR CI Workflow & verify_shift_left Contracts', () => {
  it('verify_shift_left.py auto-detects GITHUB_ACTIONS=true to bypass branch safety and scan all repo files', () => {
    const res = spawnSync('python', ['.agents/scripts/verify_shift_left.py'], {
      encoding: 'utf8',
      env: {
        ...process.env,
        GITHUB_ACTIONS: 'true'
      }
    })

    assert.equal(res.status, 0, `verify_shift_left.py failed with: ${res.stderr || res.stdout}`)
    assert.match(
      res.stdout,
      /CI environment detected/i,
      'verify_shift_left.py should log CI environment detection'
    )

    const inspectMatch = res.stdout.match(/Files to inspect:\s*(\d+)/i)
    assert.ok(inspectMatch, 'verify_shift_left.py must report count of files to inspect')
    const count = parseInt(inspectMatch[1], 10)
    assert.ok(count > 50, `Expected more than 50 files inspected in CI mode, but found: ${count}`)
  })

  it('ci.yml triggers on pull_request and push to main and runs on windows-latest', () => {
    const ciPath = path.resolve('.github/workflows/ci.yml')
    assert.ok(fs.existsSync(ciPath), '.github/workflows/ci.yml must exist')

    const content = fs.readFileSync(ciPath, 'utf8')
    assert.match(content, /pull_request:\s*\n\s*branches:\s*\n\s*-\s*main/)
    assert.match(content, /push:\s*\n\s*branches:\s*\n\s*-\s*main/)
    assert.match(content, /runs-on:\s*windows-latest/)
  })

  it('ci.yml configures Node 22 with npm cache, stable Rust toolchain, and swatinem/rust-cache@v2', () => {
    const ciPath = path.resolve('.github/workflows/ci.yml')
    assert.ok(fs.existsSync(ciPath), '.github/workflows/ci.yml must exist')

    const content = fs.readFileSync(ciPath, 'utf8')
    assert.match(content, /actions\/setup-node@v4[\s\S]*?node-version:\s*['"]22['"]/)
    assert.match(content, /actions\/setup-node@v4[\s\S]*?cache:\s*['"]npm['"]/)
    assert.match(content, /dtolnay\/rust-toolchain@stable/)
    assert.match(content, /swatinem\/rust-cache@v2/)
    assert.match(content, /run:\s*npm\s+ci\b/)
  })

  it('ci.yml executes all five required Shift-Left verification gates in order', () => {
    const ciPath = path.resolve('.github/workflows/ci.yml')
    assert.ok(fs.existsSync(ciPath), '.github/workflows/ci.yml must exist')

    const content = fs.readFileSync(ciPath, 'utf8')
    const gates = [
      'python .agents/scripts/verify_shift_left.py',
      'npm run type-check',
      'npm run lint',
      'npm test',
      'cargo test --manifest-path src-tauri/Cargo.toml'
    ]

    let lastIndex = -1
    for (const gate of gates) {
      const idx = content.indexOf(gate)
      assert.ok(idx !== -1, `ci.yml must include verification gate: "${gate}"`)
      assert.ok(idx > lastIndex, `ci.yml must execute gate "${gate}" in expected sequence`)
      lastIndex = idx
    }
  })
})
