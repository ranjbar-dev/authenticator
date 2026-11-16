using System.Runtime.InteropServices;
using Authenticator.App.Native;
using Authenticator.App.Services;
using Authenticator.App.Views;
using Authenticator.Infra.Platform;
using Authenticator.Infra.Security;
using Authenticator.Infra.Settings;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Input;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media.Animation;
using Microsoft.UI.Xaml.Navigation;
using Windows.Graphics;
using WinRT.Interop;

namespace Authenticator.App;

public sealed partial class MainWindow : Window
{
    private const int ScanHotkeyId = 1;
    private const int DefaultWidth = 420, DefaultHeight = 640, MinWidth = 360, MinHeight = 480;

    private readonly nint _hwnd;
    private readonly WindowMessageHook _hook;
    private readonly TrayIcon _tray;
    private readonly DispatcherQueueTimer _idleTimer;
    private DateTimeOffset _lastActivity = DateTimeOffset.UtcNow;
    private bool _quitting;
    private bool _forceCaptureProtection;
    private Func<Task>? _afterUnlock;

    private static AppSettings Settings => App.Current!.Settings;
    private static VaultSession Vault => App.Current!.Vault;

    /// <summary>Raised when the window is brought up from hidden/minimized (e.g. from the tray).</summary>
    public event EventHandler? Shown;

    /// <summary>Set after unlock when the opt-in clock check finds drift; shown on the main page.</summary>
    public string? ClockWarning { get; set; }

    public string? HotkeyError { get; private set; }

    public MainWindow()
    {
        InitializeComponent();
        _hwnd = WindowNative.GetWindowHandle(this);

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.SetIcon(Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico"));
        AppWindow.Title = "Authenticator";
        SizeAndCenter();

        _hook = new WindowMessageHook(_hwnd, OnWindowMessage);
        _tray = new TrayIcon(_hwnd);
        _tray.CommandInvoked += OnTrayCommand;
        Win32.WTSRegisterSessionNotification(_hwnd, 0);

        AppWindow.Closing += OnClosing;
        Closed += OnClosed;

        // Any input inside the window counts as activity for auto-lock.
        RootGrid.AddHandler(UIElement.PointerMovedEvent, new PointerEventHandler((_, _) => MarkActivity()), true);
        RootGrid.AddHandler(UIElement.KeyDownEvent, new KeyEventHandler((_, _) => MarkActivity()), true);
        Activated += (_, _) => MarkActivity();

        _idleTimer = DispatcherQueue.CreateTimer();
        _idleTimer.Interval = TimeSpan.FromSeconds(5);
        _idleTimer.Tick += (_, _) => CheckIdle();
        _idleTimer.Start();

        Vault.Locked += (_, _) => OnVaultLocked();

        ApplySettings();
    }

    /// <summary>Shows the setup or unlock page. Called once App.Window is assigned, since pages use it.</summary>
    public void Start() => NavigateToLockScreen();

    public XamlRoot XamlRoot => RootGrid.XamlRoot;
    public ElementTheme ActualTheme => RootGrid.ActualTheme;

    // ---------------------------------------------------------------- Show / hide / quit

    public void ShowAndActivate()
    {
        var wasHidden = !AppWindow.IsVisible;
        AppWindow.Show();
        if (AppWindow.Presenter is OverlappedPresenter { State: OverlappedPresenterState.Minimized } presenter)
            presenter.Restore();
        Activate();
        Win32.SetForegroundWindow(_hwnd);
        if (wasHidden)
            Shown?.Invoke(this, EventArgs.Empty);
    }

    /// <summary>Start-with-Windows launch: tray icon only, window stays hidden.</summary>
    public void StartHidden() => AppWindow.Hide();

    public void HideToTray()
    {
        AppWindow.Hide();
        if (!Settings.TrayNoticeShown)
        {
            Settings.TrayNoticeShown = true;
            Settings.Save();
            _tray.ShowNotification("Authenticator is still running", "It's in the system tray, so the scan shortcut keeps working. Right-click the icon to quit.");
        }
    }

    public void Quit()
    {
        _quitting = true;
        Close();
    }

    private void OnClosing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        if (_quitting || Settings.CloseAction == CloseAction.Exit)
            return;
        args.Cancel = true;
        HideToTray();
    }

    private void OnClosed(object sender, WindowEventArgs args)
    {
        _idleTimer.Stop();
        Win32.UnregisterHotKey(_hwnd, ScanHotkeyId);
        Win32.WTSUnRegisterSessionNotification(_hwnd);
        _tray.Dispose();
        _hook.Dispose();
        ClipboardService.ClearIfOurs();
        Vault.Flush();
        Vault.Lock();
        App.Current!.Updates.ApplyOnExit();
    }

    private void OnTrayCommand(object? sender, TrayCommand command)
    {
        switch (command)
        {
            case TrayCommand.Open: ShowAndActivate(); break;
            case TrayCommand.Scan: _ = StartScanAsync(); break;
            case TrayCommand.Lock: Lock(); break;
            case TrayCommand.Quit: Quit(); break;
        }
    }

    // ---------------------------------------------------------------- Navigation & locking

    private void NavigateToLockScreen()
    {
        if (!VaultSession.Exists)
            RootFrame.Navigate(typeof(SetupPage));
        else
        {
            Vault.Load();
            RootFrame.Navigate(typeof(UnlockPage));
        }
        RootFrame.BackStack.Clear();
    }

    /// <summary>Called by the setup and unlock pages once the vault is open.</summary>
    public async void OnUnlocked()
    {
        MarkActivity();
        RootFrame.Navigate(typeof(MainPage), null, new DrillInNavigationTransitionInfo());
        RootFrame.BackStack.Clear();

        if (Settings.CheckClockDrift)
            _ = CheckClockAsync();

        var pending = _afterUnlock;
        _afterUnlock = null;
        if (pending != null)
            await pending();
    }

    private async Task CheckClockAsync()
    {
        try
        {
            var drift = await ClockCheck.MeasureDriftAsync();
            if (Math.Abs(drift.TotalSeconds) < ClockCheck.WarningThresholdSeconds)
                return;
            ClockWarning = $"Your PC's clock is {Math.Abs(drift.TotalSeconds):0} seconds {(drift > TimeSpan.Zero ? "fast" : "slow")}, so codes may be rejected. " +
                           "Turn on \"Set time automatically\" in Windows Settings → Time & language.";
            if (RootFrame.Content is MainPage page)
                page.ShowClockWarning(ClockWarning);
        }
        catch (Exception)
        {
            // Offline or blocked: the check is best-effort.
        }
    }

    public void Lock() => Vault.Lock();

    private void OnVaultLocked()
    {
        if (XamlRoot != null)
            DialogService.CloseAll(XamlRoot);
        if (!_quitting)
            NavigateToLockScreen();
    }

    public void NavigateToSettings() => RootFrame.Navigate(typeof(SettingsPage), null, new DrillInNavigationTransitionInfo());

    private void OnBackRequested(TitleBar sender, object args)
    {
        if (RootFrame.CanGoBack)
            RootFrame.GoBack();
    }

    // Settings is the only page you navigate "into"; setup/unlock/main replace each other.
    private void OnNavigated(object sender, NavigationEventArgs e) =>
        AppTitleBar.IsBackButtonVisible = e.SourcePageType == typeof(SettingsPage);

    private void MarkActivity() => _lastActivity = DateTimeOffset.UtcNow;

    private void CheckIdle()
    {
        if (!Vault.IsUnlocked || Settings.AutoLockMinutes <= 0)
            return;
        if (DateTimeOffset.UtcNow - _lastActivity >= TimeSpan.FromMinutes(Settings.AutoLockMinutes))
            Lock();
    }

    // ---------------------------------------------------------------- Scan

    /// <summary>Global hotkey / tray / button entry point. If locked, unlocks first and then scans.</summary>
    public async Task StartScanAsync()
    {
        if (!Vault.IsUnlocked)
        {
            _afterUnlock = StartScanAsync;
            ShowAndActivate();
            return;
        }

        var wasVisible = AppWindow.IsVisible;
        if (wasVisible)
        {
            AppWindow.Hide();
            await Task.Delay(250); // let the hide animation finish so we don't capture ourselves
        }

        var image = await SnipWindow.PickAsync();

        ShowAndActivate();
        if (RootFrame.Content is not MainPage)
        {
            RootFrame.Navigate(typeof(MainPage));
        }
        if (image != null && RootFrame.Content is MainPage page)
            await page.ImportFromImageAsync(image);
    }

    // ---------------------------------------------------------------- Settings

    public void ApplySettings()
    {
        RootGrid.RequestedTheme = Settings.Theme switch
        {
            AppTheme.Light => ElementTheme.Light,
            AppTheme.Dark => ElementTheme.Dark,
            _ => ElementTheme.Default,
        };
        AppWindow.TitleBar.PreferredTheme = Settings.Theme switch
        {
            AppTheme.Light => TitleBarTheme.Light,
            AppTheme.Dark => TitleBarTheme.Dark,
            _ => TitleBarTheme.UseDefaultAppMode,
        };
        ApplyCaptureProtection();
        RegisterScanHotkey();
    }

    /// <summary>Temporarily forces capture protection on (e.g. while transfer QR codes are visible).</summary>
    public IDisposable ForceCaptureProtection()
    {
        _forceCaptureProtection = true;
        ApplyCaptureProtection();
        return new Releaser(() =>
        {
            _forceCaptureProtection = false;
            ApplyCaptureProtection();
        });
    }

    private void ApplyCaptureProtection()
    {
        var protect = Settings.ScreenCaptureProtection || _forceCaptureProtection;
        if (!protect)
            Win32.SetWindowDisplayAffinity(_hwnd, Win32.WDA_NONE);
        else if (!Win32.SetWindowDisplayAffinity(_hwnd, Win32.WDA_EXCLUDEFROMCAPTURE))
            Win32.SetWindowDisplayAffinity(_hwnd, Win32.WDA_MONITOR); // pre-2004 Windows 10
    }

    public bool RegisterScanHotkey()
    {
        Win32.UnregisterHotKey(_hwnd, ScanHotkeyId);
        var hotkey = Settings.ScanHotkey;
        if (Win32.RegisterHotKey(_hwnd, ScanHotkeyId, hotkey.Modifiers | Win32.MOD_NOREPEAT, hotkey.VirtualKey))
        {
            HotkeyError = null;
            return true;
        }
        HotkeyError = $"{hotkey} is already used by another app. Choose a different shortcut.";
        return false;
    }

    // ---------------------------------------------------------------- Win32

    private void SizeAndCenter()
    {
        var scale = Scale;
        var size = new SizeInt32((int)(DefaultWidth * scale), (int)(DefaultHeight * scale));
        var area = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        AppWindow.MoveAndResize(new RectInt32(
            area.X + (area.Width - size.Width) / 2,
            area.Y + (area.Height - size.Height) / 2,
            size.Width, size.Height));
    }

    private double Scale => Win32.GetDpiForWindow(_hwnd) is var dpi and > 0 ? dpi / 96.0 : 1.0;

    private bool OnWindowMessage(uint msg, nuint wParam, nint lParam, out nint result)
    {
        result = 0;
        if (_tray.HandleMessage(msg, lParam))
            return true;

        switch (msg)
        {
            case Win32.WM_HOTKEY when (int)wParam == ScanHotkeyId:
                _ = StartScanAsync();
                return true;

            case Win32.WM_WTSSESSION_CHANGE when wParam == Win32.WTS_SESSION_LOCK:
            case Win32.WM_POWERBROADCAST when wParam == Win32.PBT_APMSUSPEND:
                Lock();
                return false;

            case Win32.WM_GETMINMAXINFO:
                var info = Marshal.PtrToStructure<Win32.MINMAXINFO>(lParam);
                info.ptMinTrackSize.X = (int)(MinWidth * Scale);
                info.ptMinTrackSize.Y = (int)(MinHeight * Scale);
                Marshal.StructureToPtr(info, lParam, false);
                return true;
        }
        return false;
    }

    private sealed class Releaser(Action release) : IDisposable
    {
        private Action? _release = release;

        public void Dispose()
        {
            _release?.Invoke();
            _release = null;
        }
    }
}
