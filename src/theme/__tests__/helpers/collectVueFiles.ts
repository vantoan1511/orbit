import fs from 'node:fs'
import path from 'node:path'

export function collectVueFiles(dir: string): string[] {
  const results: string[] = []
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      results.push(...collectVueFiles(fullPath))
    } else if (entry.isFile() && entry.name.endsWith('.vue')) {
      results.push(fullPath)
    }
  }
  return results
}
