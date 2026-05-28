Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)
ps1 = scriptDir & "\scripts\open_correct_terminal.ps1"

cmd = "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File " & Chr(34) & ps1 & Chr(34) & " -ProjectRoot " & Chr(34) & scriptDir & Chr(34) & " -LauncherName DOGRU_TERMINAL_AC.vbs"

shell.Run cmd, 0, False
