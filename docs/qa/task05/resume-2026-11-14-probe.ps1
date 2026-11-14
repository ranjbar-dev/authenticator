# Read-only, bounded environment capability inventory. Does not change host state.
$ErrorActionPreference = 'Stop'
$report = [ordered]@{
    TimeUtc = [DateTime]::UtcNow.ToString('o')
    Source = (rtk git rev-parse HEAD)
}
$report.Services = @(Get-Service vmcompute, vmms, hns -ErrorAction SilentlyContinue |
    Select-Object Name, @{Name='Status'; Expression={$_.Status.ToString()}})
$report.Features = @(foreach ($feature in @(
    'Containers-DisposableClientVM', 'Microsoft-Hyper-V-All', 'VirtualMachinePlatform'
)) {
    try {
        $value = Get-WindowsOptionalFeature -Online -FeatureName $feature
        [pscustomobject]@{Name=$feature; State=$value.State.ToString()}
    } catch {
        [pscustomobject]@{Name=$feature; Error=$_.Exception.Message}
    }
})
try {
    $vms = @(Get-VM)
    $report.RegisteredVmCount = $vms.Count
    $report.VmStates = @($vms | Group-Object State | Select-Object Name, Count)
    $hostInfo = Get-VMHost
    $report.Host = $hostInfo | Select-Object VirtualHardDiskPath, VirtualMachinePath,
        LogicalProcessorCount, MemoryCapacity
    $roots = @($hostInfo.VirtualHardDiskPath, $hostInfo.VirtualMachinePath,
        'D:/VMs', 'D:/ISO', 'D:/ISOs', 'C:/Users/root/Downloads')
    $report.Media = @(foreach ($scanRoot in $roots | Select-Object -Unique) {
        $exists = Test-Path -LiteralPath $scanRoot
        $files = @(if ($exists) {
            Get-ChildItem -LiteralPath $scanRoot -File -ErrorAction SilentlyContinue |
                Where-Object {$_.Extension -in @('.iso', '.vhd', '.vhdx', '.ova', '.ovf')}
        })
        [pscustomobject]@{Root=$scanRoot; Exists=$exists; TopLevelMediaCount=$files.Count}
    })
} catch {
    $report.HyperVError = $_.Exception.Message
}
$report.AlternativeTools = @(foreach ($candidate in @(
    'C:/Program Files/Oracle/VirtualBox/VBoxManage.exe',
    'C:/Program Files (x86)/VMware/VMware Workstation/vmrun.exe',
    'C:/Program Files/qemu/qemu-system-x86_64.exe'
)) {
    [pscustomobject]@{Candidate=$candidate; Exists=(Test-Path -LiteralPath $candidate)}
})
$report.Disks = @(Get-PSDrive -PSProvider FileSystem | Select-Object Name, Free, Used)
$report.FirewallProfiles = @(Get-NetFirewallProfile -PolicyStore ActiveStore |
    Select-Object Name, Enabled)
$report.SoundDeviceStatuses = @(Get-CimInstance Win32_SoundDevice | Select-Object Status)
$report.NarratorRunning = @(Get-Process Narrator -ErrorAction SilentlyContinue).Count
$report.OS = Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, ProductType
$report | ConvertTo-Json -Depth 5
