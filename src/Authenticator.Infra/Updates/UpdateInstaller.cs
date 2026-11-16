using System.Diagnostics;

namespace Authenticator.Infra.Updates;

/// <summary>
/// Swaps in a downloaded exe. Windows lets a running exe be renamed (not deleted), so the swap is:
/// Authenticator.exe → Authenticator.exe.old, Authenticator.exe.update → Authenticator.exe.
/// The .old file is deleted on the next start.
/// </summary>
public static class UpdateInstaller
{
    public const string WaitForPidArgument = "--wait-pid";

    private static string? ExePath => Environment.ProcessPath;
    public static string? PendingPath => ExePath is { } exe ? exe + ".update" : null;
    private static string? OldPath => ExePath is { } exe ? exe + ".old" : null;

    /// <summary>
    /// Self-update only applies to the published single-file exe (that's what releases ship). In a
    /// normal build output the assembly has a file location and updates are skipped.
    /// </summary>
    public static bool IsSupported => Supported.Value;

    private static readonly Lazy<bool> Supported = new(() =>
        string.IsNullOrEmpty(typeof(UpdateInstaller).Assembly.Location) && ExePath != null && CanWriteNextToExe());

    public static bool HasPending => PendingPath is { } pending && File.Exists(pending);

    /// <summary>Removes the previous version's exe left behind by the last update.</summary>
    public static void CleanupPreviousVersion()
    {
        if (OldPath is not { } old || !File.Exists(old))
            return;
        for (var attempt = 0; attempt < 10; attempt++)
        {
            try
            {
                File.Delete(old);
                return;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                Thread.Sleep(300); // the old process may still be exiting
            }
        }
    }

    public static void DiscardPending()
    {
        if (PendingPath is { } pending && File.Exists(pending))
            File.Delete(pending);
    }

    /// <summary>Puts the pending exe in place. The running process keeps working from the renamed file.</summary>
    public static bool ApplyPending()
    {
        if (ExePath is not { } exe || PendingPath is not { } pending || OldPath is not { } old || !File.Exists(pending))
            return false;

        File.Move(exe, old, overwrite: true);
        try
        {
            File.Move(pending, exe);
            return true;
        }
        catch
        {
            File.Move(old, exe, overwrite: true); // roll back so the app still starts
            throw;
        }
    }

    /// <summary>Starts the (new) exe, which waits for this process to exit before taking over.</summary>
    public static void LaunchReplacement()
    {
        if (ExePath is not { } exe)
            return;
        Process.Start(new ProcessStartInfo(exe)
        {
            UseShellExecute = false,
            ArgumentList = { WaitForPidArgument, Environment.ProcessId.ToString() },
        });
    }

    /// <summary>Called first thing in Main: if launched by an update, wait for the old instance to exit.</summary>
    public static void WaitForPreviousInstance(string[] args)
    {
        var index = Array.IndexOf(args, WaitForPidArgument);
        if (index < 0 || index + 1 >= args.Length || !int.TryParse(args[index + 1], out var pid))
            return;
        try
        {
            using var previous = Process.GetProcessById(pid);
            previous.WaitForExit(TimeSpan.FromSeconds(15));
        }
        catch (ArgumentException)
        {
            // Already gone.
        }
    }

    private static bool CanWriteNextToExe()
    {
        try
        {
            var probe = Path.Combine(Path.GetDirectoryName(ExePath)!, $".write-test-{Guid.NewGuid():N}");
            File.WriteAllBytes(probe, []);
            File.Delete(probe);
            return true;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            return false;
        }
    }
}
