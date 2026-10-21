$os = Get-CimInstance Win32_OperatingSystem
"Commit: {0}/{1} GB ({2}%)" -f [math]::Round(($os.TotalVirtualMemorySize-$os.FreeVirtualMemory)/1MB,1), [math]::Round($os.TotalVirtualMemorySize/1MB,1), [math]::Round(100*(($os.TotalVirtualMemorySize-$os.FreeVirtualMemory)/$os.TotalVirtualMemorySize))
Get-CimInstance Win32_Process -Filter "Name='devin.exe'" |
  Select-Object ProcessId,
    @{n='CommitGB';e={[math]::Round($_.PrivatePageCount/1GB,1)}},
    @{n='Cmd';e={ if ($_.CommandLine) { $_.CommandLine.Substring(0,[Math]::Min(80,$_.CommandLine.Length)) } }} |
  Sort-Object CommitGB -Descending | Format-Table -AutoSize
