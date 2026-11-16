using Authenticator.App.Native;
using Authenticator.Infra.Updates;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;

namespace Authenticator.App;

public static class Program
{
    private const string InstanceKey = "Authenticator.Main";

    [STAThread]
    private static int Main(string[] args)
    {
        WinRT.ComWrappersSupport.InitializeComWrappers();

        // After a self-update the new exe is started by the old one; let it exit first, then
        // remove its renamed file.
        UpdateInstaller.WaitForPreviousInstance(args);
        _ = Task.Run(UpdateInstaller.CleanupPreviousVersion);

        // Single instance: a second launch (e.g. Start menu while running in the tray) just shows the existing window.
        if (RedirectToRunningInstance())
            return 0;

        Application.Start(_ =>
        {
            var context = new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread());
            SynchronizationContext.SetSynchronizationContext(context);
            new App();
        });
        return 0;
    }

    private static bool RedirectToRunningInstance()
    {
        var instance = AppInstance.FindOrRegisterForKey(InstanceKey);
        if (instance.IsCurrent)
        {
            instance.Activated += (_, _) => App.Current?.OnRedirectedActivation();
            return false;
        }

        Win32.AllowSetForegroundWindow(instance.ProcessId);
        var activation = AppInstance.GetCurrent().GetActivatedEventArgs();
        Task.Run(() => instance.RedirectActivationToAsync(activation).AsTask()).Wait();
        return true;
    }
}
