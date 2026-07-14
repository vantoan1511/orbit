import { execSync } from 'child_process'
import { copyFileSync, mkdirSync, existsSync } from 'fs'
import { join } from 'path'
import { platform } from 'os'

console.log('Building Orbit Binaries (Engine & Updater)...')

// Building Orbit Workspace in core directory
execSync('cargo build --release', { cwd: 'core', stdio: 'inherit' })

// Create the bin directory
const binDir = 'bin'
if (!existsSync(binDir)) {
  mkdirSync(binDir)
}

// Copy the binaries
const ext = platform() === 'win32' ? '.exe' : ''

const binaries = ['orbit-engine', 'orbit-updater']

for (const binName of binaries) {
  const binaryName = `${binName}${ext}`
  const src = join('core', 'target', 'release', binaryName)
  const dest = join(binDir, binaryName)

  console.log(`Copying binary from ${src} to ${dest}...`)
  copyFileSync(src, dest)
}

console.log('Orbit Binaries built successfully!')
