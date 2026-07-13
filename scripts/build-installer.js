import { readFileSync, copyFileSync, writeFileSync } from 'fs'
import { join } from 'path'
import innosetupCompiler from 'innosetup-compiler'

console.log('Preparing to build installer...')

// Read neutralino.config.json
const configStr = readFileSync('neutralino.config.json', 'utf8')
const config = JSON.parse(configStr)

const appName = config.applicationName || 'Orbit'
const appVersion = config.version || '1.0.0'
const appId = config.applicationId || 'vantoan1511.orbit'

// Copy icon
const iconSrc = join('public', 'favicon.ico')
const iconDest = join('dist', 'icon.ico')
console.log(`Copying icon from ${iconSrc} to ${iconDest}...`)
copyFileSync(iconSrc, iconDest)

// Generate ISS script
const issContent = `
[Setup]
AppId={{${appId}}}
AppName=${appName}
AppVersion=${appVersion}
AppPublisher=${config.author || 'Toan Nguyen'}
DefaultDirName={autopf}\\${appName}
DisableProgramGroupPage=yes
; We output to dist folder
OutputDir=dist
OutputBaseFilename=${appName}-Setup-${appVersion}
SetupIconFile=dist\\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main Neutralino files
Source: "dist\\orbit\\orbit-win_x64.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\\orbit\\resources.neu"; DestDir: "{app}"; Flags: ignoreversion
; Extension binary
Source: "dist\\orbit\\bin\\orbit-engine.exe"; DestDir: "{app}\\bin"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\\${appName}"; Filename: "{app}\\orbit-win_x64.exe"
Name: "{autodesktop}\\${appName}"; Filename: "{app}\\orbit-win_x64.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\\orbit-win_x64.exe"; Description: "{cm:LaunchProgram,${appName}}"; Flags: nowait postinstall skipifsilent
`

const issPath = 'installer.iss'
writeFileSync(issPath, issContent)

console.log('Compiling installer...')
innosetupCompiler(issPath, { gui: false }, (error) => {
  if (error) {
    console.error('Failed to compile installer:', error)
    process.exit(1)
  }
  console.log('Installer built successfully!')
})
