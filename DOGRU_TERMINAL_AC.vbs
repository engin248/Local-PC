Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)
projectRoot = scriptDir
sessionScript = scriptDir & "\scripts\project_terminal_session.ps1"
psScript = scriptDir & "\scripts\open_correct_terminal.ps1"

Set wmi = GetObject("winmgmts:\\.\root\cimv2")
Set processes = wmi.ExecQuery("SELECT ProcessId, CommandLine FROM Win32_Process WHERE Name = 'powershell.exe'")

For Each process In processes
  If Not IsNull(process.CommandLine) Then
    If InStr(1, process.CommandLine, sessionScript, vbTextCompare) > 0 And InStr(1, process.CommandLine, projectRoot, vbTextCompare) > 0 Then
      WScript.Quit 0
    End If
  End If
Next

command = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File " & Chr(34) & psScript & Chr(34)
shell.Run command, 0, False
