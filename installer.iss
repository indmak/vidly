; Usage: ISCC /DVersion=x.y.z installer.iss
; Output: Output/Vidly-Setup-x.y.z.exe
;
; FFmpeg is bundled as separate files (LGPL build) and only installed when the
; target machine does not already provide ffmpeg + ffprobe on PATH.
; Version is the 4-part MS Store version, e.g. 0.1.0.0 (passed from CI as <Cargo version>.0).
#ifndef Version
  #define Version "0.0.0.0"
#endif

[Setup]
AppId={{3F9C1A54-7B2E-4D86-9A17-C05E4412B780}}   ; generate once, never change (upgrades key off it)
AppName=Vidly
AppVersion={#Version}
AppPublisher=indmak
AppPublisherURL=https://github.com/indmak/vidly
DefaultDirName={autopf}\Vidly
PrivilegesRequired=lowest          ; install to the user dir: no UAC, cleaner uninstall
OutputBaseFilename=Vidly-Setup-{#Version}
SetupIconFile=assets\vidly.ico
UninstallDisplayIcon={app}\vidly.exe
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible

[Files]
Source: "dist\vidly.exe";          DestDir: "{app}"; Flags: ignoreversion
Source: "dist\ffmpeg.exe";         DestDir: "{app}"; Flags: ignoreversion; Check: NeedsFfmpeg
Source: "dist\ffprobe.exe";        DestDir: "{app}"; Flags: ignoreversion; Check: NeedsFfmpeg
Source: "dist\LICENSE-ffmpeg.txt"; DestDir: "{app}"; Flags: ignoreversion; Check: NeedsFfmpeg

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; Flags: unchecked

[Icons]
Name: "{autoprograms}\Vidly"; Filename: "{app}\vidly.exe"
Name: "{autodesktop}\Vidly";  Filename: "{app}\vidly.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\vidly.exe"; Description: "立即运行"; Flags: nowait postinstall skipifsilent

[Code]
// Returns True when `exe` is resolvable on the target's PATH.
function OnPath(const Exe: String): Boolean;
var
  ResultCode: Integer;
begin
  Result := Exec('where.exe', Exe, '', SW_HIDE, ewWaitUntilTerminated, ResultCode)
    and (ResultCode = 0);
end;

// Install the bundled FFmpeg only when the system does not already have both
// ffmpeg and ffprobe on PATH. If it does, Vidly finds them via PATH and the
// bundled copy is skipped entirely.
function NeedsFfmpeg(): Boolean;
begin
  Result := not (OnPath('ffmpeg.exe') and OnPath('ffprobe.exe'));
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and (not NeedsFfmpeg()) and (not WizardSilent) then
    MsgBox('检测到系统已安装 FFmpeg，已跳过内置版本。', mbInformation, MB_OK);
end;
