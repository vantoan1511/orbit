import { readFileSync, writeFileSync } from 'fs'

const packageJson = JSON.parse(readFileSync('package.json', 'utf8'))

const manifest = {
  version: packageJson.version,
  url: `https://github.com/vantoan1511/orbit/releases/download/v${packageJson.version}/orbit-update.zip`
}

writeFileSync('update-manifest.json', JSON.stringify(manifest, null, 2))
console.log('Successfully generated update-manifest.json')
