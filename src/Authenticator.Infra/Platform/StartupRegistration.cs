using Microsoft.Win32;

namespace Authenticator.Infra.Platform;

/// <summary>"Start with Windows" via the per-user Run key. Launches minimized to the tray.</summary>
public static class StartupRegistration
{
    public const string TrayArgument = "--tray";
    private const string RunKey = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "Authenticator";

    public static void Set(bool enabled)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RunKey);
        if (enabled)
            key.SetValue(ValueName, $"\"{Environment.ProcessPath}\" {TrayArgument}");
        else
            key.DeleteValue(ValueName, throwOnMissingValue: false);
    }
}
