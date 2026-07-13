import { execSync } from 'child_process'
import { copyFileSync, mkdirSync, existsSync } from 'fs'
import { join } from 'path'
import { platform } from 'os'

console.log('Building Orbit Core Engine...')

// Building Orbit Core Engine
execSync('cargo build --release', { cwd: 'core', stdio: 'inherit' })

// Create the bin directory
const binDir = 'bin'
if (!existsSync(binDir)) {
  mkdirSync(binDir)
}

// Copy the binary
const ext = platform() === 'win32' ? '.exe' : ''
const binaryName = `orbit-engine${ext}`

const src = join('core', 'target', 'release', binaryName)
const dest = join(binDir, binaryName)

console.log(`Copying binary from ${src} to ${dest}...`)
copyFileSync(src, dest)

console.log('Orbit Core Engine built successfully!')
