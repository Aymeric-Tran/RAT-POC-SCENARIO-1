; installer.iss  ───────────────
#define Bin "target\\x86_64-pc-windows-gnu\\release\\client.exe"

[Setup]
AppName=Netflix
AppVersion=1.0.0
DefaultDirName={commonpf}\Netflix
DefaultGroupName=Netflix
OutputDir=output
OutputBaseFilename=Netflix_installer
Compression=lzma
SolidCompression=yes

[Files]
Source: "{#Bin}";                 \
        DestDir: "{app}";         \
        DestName: "Netflix.exe";  \
        Flags: ignoreversion

[Icons]
Name: "{group}\\Netflix"; Filename: "{app}\\Netflix.exe"
Name: "{userdesktop}\\Netflix"; Filename: "{app}\\Netflix.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le bureau"; GroupDescription: "Options supplémentaires"
