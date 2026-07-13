
[Setup]
AppId={{vantoan1511.orbit}}
AppName=Orbit
AppVersion=1.0.0
AppPublisher=Toan Nguyen
DefaultDirName={autopf}\Orbit
DisableProgramGroupPage=yes
; We output to dist folder
OutputDir=dist
OutputBaseFilename=Orbit-Setup-1.0.0
SetupIconFile=dist\icon.ico
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
Source: "dist\orbit\orbit-win_x64.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\orbit\resources.neu"; DestDir: "{app}"; Flags: ignoreversion
; Extension binary
Source: "dist\orbit\bin\orbit-engine.exe"; DestDir: "{app}\bin"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Orbit"; Filename: "{app}\orbit-win_x64.exe"
Name: "{autodesktop}\Orbit"; Filename: "{app}\orbit-win_x64.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\orbit-win_x64.exe"; Description: "{cm:LaunchProgram,Orbit}"; Flags: nowait postinstall skipifsilent
