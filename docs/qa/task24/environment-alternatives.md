# Additional isolated-host feasibility check

Parent read-only check on 13 November 2026, through hidden child processes while native QA owned the foreground:

    rtk proxy C:/Windows/System32/dism.exe /Online /Get-FeatureInfo /FeatureName:Containers-DisposableClientVM

Exit0. DISM 10.0.26100.9549, image 10.0.26100.9711 reported Windows Sandbox feature State: Disabled and Restart Required: Possible. C:/Windows/System32/WindowsSandbox.exe was absent. No feature was enabled, global policy changed or shared host restarted, and no Sandbox guest was launched. This supplements Task23's zero ready Hyper-V VMs/no available third-party VM executable/other ready Windows-user session evidence. It establishes no provisioned isolated test environment here, not that installing Windows features is universally impossible.

Microsoft documents [Sandbox installation](https://learn.microsoft.com/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-install) and [configuration](https://learn.microsoft.com/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file), including disabled guest networking and read-only mapped fixture folders. A separately provisioned compatible Sandbox/VM could test the unsigned package without Rust and with effective networking disabled, followed by independently keyed/profile backup recovery. Map only owned disposable inputs, preserve generated backup output deliberately and verify actual guest network/runtime state. Do not treat a proposed configuration as an executed pass.

The earlier exact-program firewall attempt had CategoryDisabled enforcement and was removed; it is not an offline result. Available native widths/app zoom are being tested separately. Physical DPI availability must be assessed independently rather than inferred from this isolated-host check.
