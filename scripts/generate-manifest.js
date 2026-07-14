import { readFileSync, writeFileSync } from 'fs'

const packageJson = JSON.parse(readFileSync('package.json', 'utf8'))
const coreEngineCargo = readFileSync('core/engine/Cargo.toml', 'utf8')

// Parse version from Cargo.toml
const engineVersionMatch = coreEngineCargo.match(/version\s*=\s*"([^"]+)"/)
const engineVersion = engineVersionMatch ? engineVersionMatch[1] : '1.0.0'

const manifest = {
  engine: {
    version: engineVersion,
    url: `https://github.com/vantoan1511/orbit/releases/download/engine-v${engineVersion}/orbit-engine.zip`
  },
  resources: {
    version: packageJson.version,
    url: `https://github.com/vantoan1511/orbit/releases/download/resources-v${packageJson.version}/resources.neu`
  }
}

writeFileSync('update-manifest.json', JSON.stringify(manifest, null, 2))
console.log('Successfully generated update-manifest.json')
