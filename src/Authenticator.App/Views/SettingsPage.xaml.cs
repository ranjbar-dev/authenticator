using System.Diagnostics;
using System.Reflection;
using Authenticator.App.Dialogs;
using Authenticator.App.Services;
using Authenticator.Core.Vault;
using Authenticator.Infra;
using Authenticator.Infra.Platform;
using Authenticator.Infra.Security;
using Authenticator.Infra.Settings;
using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Navigation;
using Windows.System;
using Windows.UI.Core;

namespace Authenticator.App.Views;

public sealed partial class SettingsPage : Page
{
    private static readonly (string Label, int Minutes)[] AutoLockOptions =
        [("1 minute", 1), ("5 minutes", 5), ("15 minutes", 15), ("30 minutes", 30), ("Never", 0)];

    private static readonly (string Label, int Seconds)[] ClipboardOptions =
        [("After 15 seconds", 15), ("After 30 seconds", 30), ("After 1 minute", 60), ("Never", 0)];

    private bool _loading;
    private bool _recordingHotkey;

    public SettingsPage()
    {
        InitializeComponent();
        foreach (var (label, _) in AutoLockOptions)
            AutoLockBox.Items.Add(label);
        foreach (var (label, _) in ClipboardOptions)
            ClipboardBox.Items.Add(label);
    }

    private static AppSettings Settings => App.Current!.Settings;
    private static MainWindow Window => App.Current!.Window;

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        _loading = true;
        ThemeBox.SelectedIndex = (int)Settings.Theme;
        AutoLockBox.SelectedIndex = Math.Max(0, Array.FindIndex(AutoLockOptions, o => o.Minutes == Settings.AutoLockMinutes));
        ClipboardBox.SelectedIndex = Math.Max(0, Array.FindIndex(ClipboardOptions, o => o.Seconds == Settings.ClipboardClearSeconds));
        CaptureToggle.IsOn = Settings.ScreenCaptureProtection;
        NextCodeToggle.IsOn = Settings.ShowNextCode;
        ClockToggle.IsOn = Settings.CheckClockDrift;
        StartupToggle.IsOn = Settings.StartWithWindows;
        CloseBox.SelectedIndex = (int)Settings.CloseAction;
        AutoUpdateToggle.IsOn = Settings.AutoUpdate;
        _loading = false;

        UpdateHotkeyDisplay();
        UpdateUnlockCard();
        UpdateBackupCard();
        UpdateSummary();
        App.Current!.Updates.Changed += OnUpdatesChanged;
        UpdateUpdatesCard();

        var version = Assembly.GetExecutingAssembly().GetName().Version;
        AboutExpander.Description = $"Version {version?.ToString(3)}";
        DataFolderCard.Description = AppPaths.DataDirectory;
    }

    private static void Save() => Settings.Save();

    protected override void OnNavigatedFrom(NavigationEventArgs e) => App.Current!.Updates.Changed -= OnUpdatesChanged;

    // ---------------------------------------------------------------- Updates

    private void OnUpdatesChanged(object? sender, EventArgs e) => UpdateUpdatesCard();

    private void UpdateUpdatesCard()
    {
        var updates = App.Current!.Updates;
        var current = $"Version {updates.CurrentVersion.ToString(3)}";
        UpdatesExpander.Description = updates.State switch
        {
            UpdateState.Unsupported => $"{current} · Self-update works in the released Authenticator.exe, not in development builds.",
            UpdateState.Checking => $"{current} · Checking for updates…",
            UpdateState.Downloading => $"{current} · Downloading {updates.Latest?.Version.ToString(3)}…",
            UpdateState.Ready => $"{current} · Version {updates.ReadyVersion} is ready to install.",
            UpdateState.UpToDate => $"{current} · You're up to date.",
            UpdateState.Failed => $"{current} · {updates.Error}",
            _ => current,
        };
        var ready = updates.State == UpdateState.Ready;
        RestartUpdateButton.Visibility = ready ? Visibility.Visible : Visibility.Collapsed;
        CheckUpdatesButton.Visibility = ready ? Visibility.Collapsed : Visibility.Visible;
        CheckUpdatesButton.IsEnabled = updates.State is not (UpdateState.Unsupported or UpdateState.Checking or UpdateState.Downloading);
        AutoUpdateToggle.IsEnabled = updates.State != UpdateState.Unsupported;
    }

    private async void OnCheckUpdatesClick(object sender, RoutedEventArgs e) => await App.Current!.Updates.CheckAsync();

    private void OnRestartToUpdateClick(object sender, RoutedEventArgs e) => App.Current!.Updates.RestartToUpdate();

    private void OnAutoUpdateToggled(object sender, RoutedEventArgs e)
    {
        if (_loading)
            return;
        Settings.AutoUpdate = AutoUpdateToggle.IsOn;
        Save();
    }

    private void OnReleaseNotesClick(object sender, RoutedEventArgs e)
    {
        var url = App.Current!.Updates.Latest?.PageUrl ?? Infra.Updates.UpdateClient.ReleasesPage;
        Process.Start(new ProcessStartInfo(url.ToString()) { UseShellExecute = true });
    }

    private void UpdateSummary()
    {
        var hello = App.Current!.Vault.Protection == VaultProtection.WindowsHello;
        SummaryUnlock.Text = hello ? "Windows Hello" : "App password";

        var noBackup = Settings.LastBackupAt is not { } last;
        var stale = !noBackup && DateTimeOffset.UtcNow - Settings.LastBackupAt!.Value > TimeSpan.FromDays(90);
        SummaryBackup.Text = noBackup ? "No backup yet"
            : stale ? $"Last backup {Settings.LastBackupAt!.Value.LocalDateTime:d MMM yyyy}, over 90 days ago"
            : $"Last backup {Settings.LastBackupAt!.Value.LocalDateTime:d MMM yyyy}";
        SummaryBackupWarning.Visibility = noBackup || stale ? Visibility.Visible : Visibility.Collapsed;
        SummaryBackupButton.Visibility = noBackup || stale ? Visibility.Visible : Visibility.Collapsed;

        SummaryLock.Text = Settings.AutoLockMinutes switch
        {
            0 => "Only when Windows locks or sleeps",
            1 => "After 1 minute idle",
            var m => $"After {m} minutes idle",
        };
    }

    // ---------------------------------------------------------------- Appearance

    private void OnThemeChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_loading)
            return;
        Settings.Theme = (AppTheme)ThemeBox.SelectedIndex;
        Save();
        Window.ApplySettings();
    }

    // ---------------------------------------------------------------- Security

    private async void UpdateUnlockCard()
    {
        var hello = App.Current!.Vault.Protection == VaultProtection.WindowsHello;
        UnlockCard.Description = hello ? "Windows Hello (face, fingerprint or PIN)" : "App password";
        ChangePasswordButton.Visibility = hello ? Visibility.Collapsed : Visibility.Visible;
        SwitchUnlockButton.Content = hello ? "Use a password instead" : "Use Windows Hello";
        SwitchUnlockButton.Visibility = hello || await WindowsHello.IsAvailableAsync() ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void OnSwitchUnlockClick(object sender, RoutedEventArgs e)
    {
        var vault = App.Current!.Vault;
        if (!await DialogService.ReauthenticateAsync(XamlRoot, "Enter your current app password."))
            return;
        try
        {
            if (vault.Protection == VaultProtection.WindowsHello)
            {
                var password = await PasswordDialog.CreateAsync(XamlRoot, "Choose an app password",
                    "You'll type this password to unlock Authenticator. It can't be recovered if you forget it — keep a backup file too.", "Use password");
                if (password == null)
                    return;
                await vault.SwitchToPasswordAsync(password);
            }
            else
            {
                await vault.SwitchToHelloAsync();
            }
        }
        catch (OperationCanceledException)
        {
            return;
        }
        catch (WindowsHelloException ex)
        {
            await DialogService.ShowMessageAsync(XamlRoot, "Couldn't switch", ex.Message);
        }
        UpdateUnlockCard();
        UpdateSummary();
    }

    private async void OnChangePasswordClick(object sender, RoutedEventArgs e)
    {
        if (!await DialogService.ReauthenticateAsync(XamlRoot, "Enter your current app password."))
            return;
        var password = await PasswordDialog.CreateAsync(XamlRoot, "New app password", "Choose a new password for unlocking Authenticator.", "Change password");
        if (password == null)
            return;
        await App.Current!.Vault.SwitchToPasswordAsync(password);
        await DialogService.ShowMessageAsync(XamlRoot, "Password changed", "Use your new password next time you unlock.");
    }

    private void OnAutoLockChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_loading || AutoLockBox.SelectedIndex < 0)
            return;
        Settings.AutoLockMinutes = AutoLockOptions[AutoLockBox.SelectedIndex].Minutes;
        Save();
        UpdateSummary();
    }

    private void OnClipboardChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_loading || ClipboardBox.SelectedIndex < 0)
            return;
        Settings.ClipboardClearSeconds = ClipboardOptions[ClipboardBox.SelectedIndex].Seconds;
        Save();
    }

    private void OnCaptureToggled(object sender, RoutedEventArgs e)
    {
        if (_loading)
            return;
        Settings.ScreenCaptureProtection = CaptureToggle.IsOn;
        Save();
        Window.ApplySettings();
    }

    // ---------------------------------------------------------------- Codes

    private void OnNextCodeToggled(object sender, RoutedEventArgs e)
    {
        if (_loading)
            return;
        Settings.ShowNextCode = NextCodeToggle.IsOn;
        Save();
    }

    private void OnClockToggled(object sender, RoutedEventArgs e)
    {
        if (_loading)
            return;
        Settings.CheckClockDrift = ClockToggle.IsOn;
        Save();
    }

    private async void OnCheckClockClick(object sender, RoutedEventArgs e)
    {
        CheckClockButton.IsEnabled = false;
        ClockResultCard.Description = "Checking…";
        try
        {
            var drift = await ClockCheck.MeasureDriftAsync();
            ClockResultCard.Description = Math.Abs(drift.TotalSeconds) < ClockCheck.WarningThresholdSeconds
                ? $"Your clock is correct (off by {Math.Abs(drift.TotalSeconds):0.0} s)."
                : $"Your clock is {Math.Abs(drift.TotalSeconds):0} seconds {(drift > TimeSpan.Zero ? "fast" : "slow")}. Turn on \"Set time automatically\" in Windows Settings → Time & language.";
        }
        catch (Exception ex) when (ex is OperationCanceledException or System.Net.Sockets.SocketException or InvalidDataException)
        {
            ClockResultCard.Description = "Couldn't reach the time server. Check your internet connection.";
        }
        finally
        {
            CheckClockButton.IsEnabled = true;
        }
    }

    // ---------------------------------------------------------------- Shortcut and startup

    private void UpdateHotkeyDisplay()
    {
        HotkeyButton.Content = _recordingHotkey ? "Press new shortcut…" : Settings.ScanHotkey.ToString();
        HotkeyCard.Description = Window.HotkeyError ?? "Works anywhere in Windows while Authenticator is running. Click to change.";
    }

    private void OnHotkeyClick(object sender, RoutedEventArgs e)
    {
        _recordingHotkey = !_recordingHotkey;
        UpdateHotkeyDisplay();
    }

    private void OnHotkeyLosingFocus(UIElement sender, LosingFocusEventArgs args)
    {
        _recordingHotkey = false;
        UpdateHotkeyDisplay();
    }

    private void OnHotkeyKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (!_recordingHotkey)
            return;
        e.Handled = true;

        if (e.Key == VirtualKey.Escape)
        {
            _recordingHotkey = false;
            UpdateHotkeyDisplay();
            return;
        }
        if (e.Key is VirtualKey.Control or VirtualKey.Shift or VirtualKey.Menu or VirtualKey.LeftWindows or VirtualKey.RightWindows
            or VirtualKey.LeftControl or VirtualKey.RightControl or VirtualKey.LeftShift or VirtualKey.RightShift or VirtualKey.LeftMenu or VirtualKey.RightMenu)
            return; // wait for the actual key

        uint modifiers = 0;
        if (IsDown(VirtualKey.Control)) modifiers |= HotkeySetting.ModControl;
        if (IsDown(VirtualKey.Menu)) modifiers |= HotkeySetting.ModAlt;
        if (IsDown(VirtualKey.Shift)) modifiers |= HotkeySetting.ModShift;
        if (IsDown(VirtualKey.LeftWindows) || IsDown(VirtualKey.RightWindows)) modifiers |= HotkeySetting.ModWin;

        if ((modifiers & (HotkeySetting.ModControl | HotkeySetting.ModAlt | HotkeySetting.ModWin)) == 0)
        {
            HotkeyCard.Description = "Include Ctrl, Alt or Win in the shortcut.";
            return;
        }

        var previous = Settings.ScanHotkey;
        Settings.ScanHotkey = new HotkeySetting { Modifiers = modifiers, VirtualKey = (uint)e.Key };
        if (!Window.RegisterScanHotkey())
        {
            var error = Window.HotkeyError;
            Settings.ScanHotkey = previous;
            Window.RegisterScanHotkey();
            _recordingHotkey = false;
            UpdateHotkeyDisplay();
            HotkeyCard.Description = error ?? "";
            return;
        }
        Save();
        _recordingHotkey = false;
        UpdateHotkeyDisplay();
    }

    private static bool IsDown(VirtualKey key) =>
        InputKeyboardSource.GetKeyStateForCurrentThread(key).HasFlag(CoreVirtualKeyStates.Down);

    private async void OnStartupToggled(object sender, RoutedEventArgs e)
    {
        if (_loading)
            return;
        try
        {
            StartupRegistration.Set(StartupToggle.IsOn);
            Settings.StartWithWindows = StartupToggle.IsOn;
            Save();
        }
        catch (Exception ex) when (ex is UnauthorizedAccessException or System.Security.SecurityException or IOException)
        {
            _loading = true;
            StartupToggle.IsOn = Settings.StartWithWindows;
            _loading = false;
            await DialogService.ShowMessageAsync(XamlRoot, "Couldn't change startup", ex.Message);
        }
    }

    private void OnCloseActionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_loading)
            return;
        Settings.CloseAction = (CloseAction)CloseBox.SelectedIndex;
        Save();
    }

    // ---------------------------------------------------------------- Backup

    private void UpdateBackupCard() =>
        BackupCard.Description = Settings.LastBackupAt is { } last
            ? $"Password-protected file. Last backup: {last.LocalDateTime:d}."
            : "Password-protected file. You haven't made a backup yet.";

    private async void OnExportBackupClick(object sender, RoutedEventArgs e)
    {
        await TransferActions.ExportBackupAsync(XamlRoot);
        UpdateBackupCard();
        UpdateSummary();
    }

    private async void OnImportBackupClick(object sender, RoutedEventArgs e)
    {
        var added = await TransferActions.ImportBackupAsync(XamlRoot);
        if (added > 0)
            await DialogService.ShowMessageAsync(XamlRoot, "Accounts imported", $"{added} account{(added == 1 ? "" : "s")} added.");
    }

    private async void OnTransferClick(object sender, RoutedEventArgs e) => await TransferActions.ShowTransferQrAsync(XamlRoot);

    private void OnOpenDataFolderClick(object sender, RoutedEventArgs e)
    {
        AppPaths.EnsureDataDirectory();
        Process.Start(new ProcessStartInfo("explorer.exe", $"\"{AppPaths.DataDirectory}\"") { UseShellExecute = true });
    }
}
