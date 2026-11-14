# Read-only servicing prerequisite snapshot. Does not enable, disable or launch
# Sandbox, restart Windows, inspect vaults, or alter networking.
$ErrorActionPreference = 'Stop'
$featureStates = @(foreach ($featureName in @(
    'Containers-DisposableClientVM', 'Microsoft-Hyper-V-All',
    'Microsoft-Hyper-V-Hypervisor', 'VirtualMachinePlatform'
)) {
    $feature = Get-WindowsOptionalFeature -Online -FeatureName $featureName
    [pscustomobject]@{
        Name = $featureName
        State = $feature.State.ToString()
        RestartRequired = $feature.RestartRequired.ToString()
    }
})
[ordered]@{
    TimeUtc = [DateTime]::UtcNow.ToString('o')
    DiagnosticOnly = $true
    Features = $featureStates
    CbsRebootPending = Test-Path -LiteralPath 'HKLM:/SOFTWARE/Microsoft/Windows/CurrentVersion/Component Based Servicing/RebootPending'
    WindowsUpdateRebootRequired = Test-Path -LiteralPath 'HKLM:/SOFTWARE/Microsoft/Windows/CurrentVersion/WindowsUpdate/Auto Update/RebootRequired'
    PendingXml = Test-Path -LiteralPath 'C:/Windows/WinSxS/pending.xml'
    SandboxCommandAvailable = [bool](Get-Command WindowsSandbox.exe -ErrorAction SilentlyContinue)
    WsbCommandAvailable = [bool](Get-Command wsb.exe -ErrorAction SilentlyContinue)
    SandboxProcessCount = @(Get-Process WindowsSandbox* -ErrorAction SilentlyContinue).Count
} | ConvertTo-Json -Depth 4
