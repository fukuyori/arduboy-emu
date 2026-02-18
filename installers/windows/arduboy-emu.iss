; Arduboy Emulator - Inno Setup Script
; Build: iscc arduboy-emu.iss
; Requires: Inno Setup 6.x (https://jrsoftware.org/isinfo.php)

#define MyAppName "Arduboy Emulator"
#define MyAppVersion "0.8.1"
#define MyAppPublisher "arduboy-emu"
#define MyAppURL "https://github.com/example/arduboy-emu"
#define MyAppExeName "arduboy-emu.exe"

[Setup]
AppId={{B8A3F2E1-4D5C-6E7F-8A9B-0C1D2E3F4A5B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE-MIT
OutputDir=..\..\dist\windows
OutputBaseFilename=arduboy-emu-{#MyAppVersion}-setup-x64
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
; SetupIconFile=..\..\assets\arduboy-emu.ico  ; Uncomment when icon exists
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "fileassoc_hex"; Description: "Associate .hex files"; GroupDescription: "File associations:"
Name: "fileassoc_arduboy"; Description: "Associate .arduboy files"; GroupDescription: "File associations:"

[Files]
; Main executable (built with: cargo build --release)
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; Licenses and docs
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "..\..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion


; VC++ runtime (if statically linked, not needed; include if dynamically linked)
; Source: "vcruntime140.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; .hex file association
Root: HKA; Subkey: "Software\Classes\.hex\OpenWithProgids"; ValueType: string; ValueName: "ArduboyEmu.hex"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc_hex
Root: HKA; Subkey: "Software\Classes\ArduboyEmu.hex"; ValueType: string; ValueName: ""; ValueData: "Arduboy HEX ROM"; Flags: uninsdeletekey; Tasks: fileassoc_hex
Root: HKA; Subkey: "Software\Classes\ArduboyEmu.hex\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc_hex

; .arduboy file association
Root: HKA; Subkey: "Software\Classes\.arduboy\OpenWithProgids"; ValueType: string; ValueName: "ArduboyEmu.arduboy"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc_arduboy
Root: HKA; Subkey: "Software\Classes\ArduboyEmu.arduboy"; ValueType: string; ValueName: ""; ValueData: "Arduboy Game Archive"; Flags: uninsdeletekey; Tasks: fileassoc_arduboy
Root: HKA; Subkey: "Software\Classes\ArduboyEmu.arduboy\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc_arduboy

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
