; TikTok Cleaner Installation Script
; High-performance TikTok detection and organization tool

#define MyAppName "TikTok Cleaner"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Tegran Grigorian"
#define MyAppURL "https://github.com/yourusername/tiktok-cleaner"
#define MyAppExeName "tiktok-cleaner.exe"
#define MyAppDescription "High-performance TikTok detection and organization tool with multithreaded processing"

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
; Uncomment the following line to run in non administrative install mode (install for current user only.)
;PrivilegesRequired=lowest
OutputDir=.
OutputBaseFilename=TikTokCleaner-{#MyAppVersion}-Setup
SetupIconFile=
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
MinVersion=6.1sp1

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1

[Files]
; Main executable
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Documentation
Source: ".\README.md"; DestDir: "{app}"; Flags: ignoreversion
; Source: "..\WINDOWS_GUIDE.md"; DestDir: "{app}"; Flags: ignoreversion
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: quicklaunchicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent; Parameters: "--help"

[Code]
procedure InitializeWizard;
begin
  WizardForm.WelcomeLabel2.Caption := 
    'This will install {#MyAppName} {#MyAppVersion} on your computer.' + #13#10 + #13#10 +
    '{#MyAppDescription}' + #13#10 + #13#10 +
    'Features:' + #13#10 +
    '• Multithreaded Processing with all CPU cores' + #13#10 +
    '• Phone Filesystem Support (MTP/Android)' + #13#10 +
    '• Intelligent Caching for fast scans' + #13#10 +
    '• Confidence Scoring and Smart Organization' + #13#10 +
    '• Advanced TikTok Detection Algorithm' + #13#10 + #13#10 +
    'Click Next to continue, or Cancel to exit Setup.';
end;

[UninstallDelete]
Type: filesandordirs; Name: "{app}"
