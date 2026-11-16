using Authenticator.App.Services;
using Authenticator.Infra;
using Authenticator.Infra.Platform;
using Authenticator.Infra.Security;
using Authenticator.Infra.Settings;
using Microsoft.UI.Xaml;

namespace Authenticator.App;

public partial class App : Application
{
    public static new App? Current => Application.Current as App;

    public AppSettings Settings { get; } = AppSettings.Load();
    public VaultSession Vault { get; } = new();
    public MainWindow Window { get; private set; } = null!;
    internal UpdateService Updates { get; private set; } = null!;

    public App()
    {
        InitializeComponent();
        UnhandledException += (_, e) => WriteCrashLog(e.Exception);
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        var startInTray = Environment.GetCommandLineArgs().Contains(StartupRegistration.TrayArgument, StringComparer.OrdinalIgnoreCase);
        Window = new MainWindow();
        Updates = new UpdateService(Settings, Window.DispatcherQueue);
        Window.Start();
        if (startInTray)
            Window.StartHidden();
        else
            Window.ShowAndActivate();
        Updates.Start();
    }

    /// <summary>Another launch was redirected to this instance (runs on a background thread).</summary>
    internal void OnRedirectedActivation() => Window?.DispatcherQueue.TryEnqueue(() => Window.ShowAndActivate());

    /// <summary>Exception type and stack only, never messages that could contain account data.</summary>
    private static void WriteCrashLog(Exception exception)
    {
        try
        {
            AppPaths.EnsureDataDirectory();
            File.AppendAllText(Path.Combine(AppPaths.DataDirectory, "crash.log"),
                $"[{DateTimeOffset.Now:O}] {exception.GetType().FullName}{Environment.NewLine}{exception.StackTrace}{Environment.NewLine}{Environment.NewLine}");
        }
        catch (IOException)
        {
            // Nothing else we can do while crashing.
        }
    }
}
