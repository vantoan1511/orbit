import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

const ROOT_DIR = path.resolve(import.meta.dirname, '..')
const TAURI_CONF_PATH = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json')
const LICENSE_PATH = path.join(ROOT_DIR, 'LICENSE')
const NSIS_DIR = path.join(ROOT_DIR, 'src-tauri', 'windows', 'nsis')
const HEADER_BMP_PATH = path.join(NSIS_DIR, 'header.bmp')
const SIDEBAR_BMP_PATH = path.join(NSIS_DIR, 'sidebar.bmp')
const INSTALLER_NSI_PATH = path.join(NSIS_DIR, 'installer.nsi')
const EN_LANG_PATH = path.join(NSIS_DIR, 'languages', 'English.custom.nsh')
const VI_LANG_PATH = path.join(NSIS_DIR, 'languages', 'Vietnamese.custom.nsh')

test('NSIS installer configuration in tauri.conf.json', () => {
  assert.ok(fs.existsSync(TAURI_CONF_PATH), 'tauri.conf.json must exist')
  const config = JSON.parse(fs.readFileSync(TAURI_CONF_PATH, 'utf-8'))

  assert.ok(config.bundle, 'bundle configuration must exist')
  assert.equal(config.bundle.licenseFile, '../LICENSE', 'licenseFile must point to root LICENSE')
  assert.ok(fs.existsSync(LICENSE_PATH), 'root LICENSE file must exist')

  const nsis = config.bundle?.windows?.nsis
  assert.ok(nsis, 'bundle.windows.nsis must be configured')
  assert.equal(nsis.template, 'windows/nsis/installer.nsi')
  assert.equal(nsis.headerImage, 'windows/nsis/header.bmp')
  assert.equal(nsis.sidebarImage, 'windows/nsis/sidebar.bmp')
  assert.equal(nsis.installerIcon, 'icons/icon.ico')
  assert.equal(nsis.uninstallerIcon, 'icons/icon.ico')
  assert.equal(nsis.installMode, 'both')
  assert.deepEqual(nsis.languages, ['English', 'Vietnamese'])
  assert.equal(nsis.displayLanguageSelector, true)
  assert.equal(nsis.compression, 'lzma')
  assert.deepEqual(nsis.customLanguageFiles, {
    English: 'windows/nsis/languages/English.custom.nsh',
    Vietnamese: 'windows/nsis/languages/Vietnamese.custom.nsh'
  })
})

function validateBmp(filePath: string, expectedWidth: number, expectedHeight: number) {
  assert.ok(fs.existsSync(filePath), `BMP file must exist: ${filePath}`)
  const buf = fs.readFileSync(filePath)

  assert.ok(buf.length >= 54, 'BMP file must be at least 54 bytes (header size)')
  assert.equal(buf.subarray(0, 2).toString('ascii'), 'BM', 'Must have BMP signature BM')

  const dibHeaderSize = buf.readUInt32LE(14)
  assert.equal(
    dibHeaderSize,
    40,
    'Must use standard BITMAPINFOHEADER (40 bytes), no V5 color profile'
  )

  const width = buf.readInt32LE(18)
  const height = buf.readInt32LE(22)
  const planes = buf.readUInt16LE(26)
  const bitCount = buf.readUInt16LE(28)
  const compression = buf.readUInt32LE(30)

  assert.equal(width, expectedWidth, `Width must be ${expectedWidth}`)
  assert.equal(Math.abs(height), expectedHeight, `Height must be ${expectedHeight}`)
  assert.equal(planes, 1, 'Planes must be 1')
  assert.equal(bitCount, 24, 'Color depth must be 24-bit RGB')
  assert.equal(compression, 0, 'Must be uncompressed BI_RGB (0)')
}

test('NSIS branding bitmap assets (header.bmp & sidebar.bmp)', () => {
  validateBmp(HEADER_BMP_PATH, 150, 57)
  validateBmp(SIDEBAR_BMP_PATH, 164, 314)
})

const REQUIRED_LANG_KEYS = [
  'tasksPageTitle',
  'tasksPageSubtitle',
  'tasksGroupShortcuts',
  'taskDesktopShortcut',
  'taskStartMenuShortcut',
  'tasksGroupStartup',
  'taskLaunchOnStartup',
  'purgeOrbitConfigData'
]

function validateLanguageFile(filePath: string, langMacro: string) {
  assert.ok(fs.existsSync(filePath), `Language file must exist: ${filePath}`)
  const buf = fs.readFileSync(filePath)

  // Validate UTF-8 without BOM (Tauri bundler automatically adds a BOM on copy; having a BOM in source causes duplicate BOM and makensis failure)
  assert.ok(
    !(buf[0] === 0xef && buf[1] === 0xbb && buf[2] === 0xbf),
    `File must NOT start with UTF-8 BOM to prevent duplicate BOM when bundled by Tauri: ${filePath}`
  )

  const content = buf.toString('utf-8')
  for (const key of REQUIRED_LANG_KEYS) {
    const pattern = new RegExp(`LangString\\s+${key}\\s+\\$\{${langMacro}\}`)
    assert.match(
      content,
      pattern,
      `Language file ${filePath} must define LangString ${key} for \${${langMacro}}`
    )
  }
}

test('NSIS localization files (English and Vietnamese UTF-8 without duplicate BOM)', () => {
  validateLanguageFile(EN_LANG_PATH, 'LANG_ENGLISH')
  validateLanguageFile(VI_LANG_PATH, 'LANG_VIETNAMESE')
})

test('Extended NSIS template (installer.nsi)', () => {
  assert.ok(fs.existsSync(INSTALLER_NSI_PATH), 'installer.nsi must exist')
  const content = fs.readFileSync(INSTALLER_NSI_PATH, 'utf-8')

  // Check custom page declaration
  assert.match(content, /Page\s+custom\s+PageTasks\s+PageLeaveTasks/)

  // Check task variables
  assert.match(content, /Var\s+CreateDesktopShortcut/)
  assert.match(content, /Var\s+CreateStartMenuShortcut/)
  assert.match(content, /Var\s+LaunchOnStartup/)

  // Check functions exist
  assert.match(content, /Function\s+PageTasks/)
  assert.match(content, /Function\s+PageLeaveTasks/)

  // Check Run key write for startup
  assert.match(content, /Software\\Microsoft\\Windows\\CurrentVersion\\Run/)

  // Check purge logic for ~/.orbit
  assert.match(content, /RmDir\s+\/r\s+"\$PROFILE\\\.orbit"/)
})
