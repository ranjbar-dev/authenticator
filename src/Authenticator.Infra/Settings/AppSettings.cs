using System.Text.Json;
using System.Text.Json.Serialization;

namespace Authenticator.Infra.Settings;

public enum AppTheme
{
    System,
    Light,
    Dark,
}

public enum CloseAction
{
    MinimizeToTray,
    Exit,
}

/// <summary>A global hotkey: Win32 MOD_* flags plus a virtual-key code.</summary>
public sealed class HotkeySetting
{
    public const uint ModAlt = 0x1, ModControl = 0x2, ModShift = 0x4, ModWin = 0x8;

    public uint Modifiers { get; set; } = ModControl | ModShift;
    public uint VirtualKey { get; set; } = 'Q';

    public override string ToString()
    {
        var parts = new List<string>();
        if ((Modifiers & ModControl) != 0) parts.Add("Ctrl");
        if ((Modifiers & ModAlt) != 0) parts.Add("Alt");
        if ((Modifiers & ModShift) != 0) parts.Add("Shift");
        if ((Modifiers & ModWin) != 0) parts.Add("Win");
        parts.Add(KeyName(VirtualKey));
        return string.Join(" + ", parts);
    }

    private static string KeyName(uint vk) => vk switch
    {
        >= 'A' and <= 'Z' or >= '0' and <= '9' => ((char)vk).ToString(),
        >= 0x70 and <= 0x87 => $"F{vk - 0x6F}",
        0x20 => "Space",
        _ => $"Key {vk}",
    };
}

/// <summary>Non-secret preferences, stored as plain JSON next to the vault.</summary>
public sealed class AppSettings
{
    public AppTheme Theme { get; set; } = AppTheme.System;

    /// <summary>0 = never.</summary>
    public int AutoLockMinutes { get; set; } = 5;

    /// <summary>0 = never.</summary>
    public int ClipboardClearSeconds { get; set; } = 30;

    public bool ShowNextCode { get; set; }
    public HotkeySetting ScanHotkey { get; set; } = new();
    public bool ScreenCaptureProtection { get; set; } = true;
    public bool StartWithWindows { get; set; }
    public CloseAction CloseAction { get; set; } = CloseAction.MinimizeToTray;
    public bool CheckClockDrift { get; set; }

    /// <summary>Check GitHub releases and download new versions in the background.</summary>
    public bool AutoUpdate { get; set; } = true;

    /// <summary>Version of the verified exe waiting next to the app (Authenticator.exe.update), if any.</summary>
    public string? PendingUpdateVersion { get; set; }

    public bool TrayNoticeShown { get; set; }
    public DateTimeOffset? LastBackupAt { get; set; }
    public DateTimeOffset? BackupReminderDismissedAt { get; set; }

    public static AppSettings Load()
    {
        try
        {
            if (File.Exists(AppPaths.Settings))
                return JsonSerializer.Deserialize(File.ReadAllBytes(AppPaths.Settings), SettingsJsonContext.Default.AppSettings) ?? new();
        }
        catch (Exception ex) when (ex is IOException or JsonException or UnauthorizedAccessException)
        {
            // Corrupt or unreadable settings are not fatal; fall back to defaults.
        }
        return new();
    }

    public void Save() =>
        AppPaths.WriteAtomic(AppPaths.Settings, JsonSerializer.SerializeToUtf8Bytes(this, SettingsJsonContext.Default.AppSettings));
}

[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    UseStringEnumConverter = true)]
[JsonSerializable(typeof(AppSettings))]
internal sealed partial class SettingsJsonContext : JsonSerializerContext;
