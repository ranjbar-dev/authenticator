namespace Authenticator.Infra;

public static class AppPaths
{
    /// <summary>%LOCALAPPDATA%\Authenticator, or AUTHENTICATOR_DATA_DIR when set (development and testing).</summary>
    public static string DataDirectory { get; } =
        Environment.GetEnvironmentVariable("AUTHENTICATOR_DATA_DIR") is { Length: > 0 } overridden
            ? overridden
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Authenticator");

    public static string Vault => Path.Combine(DataDirectory, "vault.dat");
    public static string VaultBackup => Path.Combine(DataDirectory, "vault.bak");
    public static string Settings => Path.Combine(DataDirectory, "settings.json");

    public static void EnsureDataDirectory() => Directory.CreateDirectory(DataDirectory);

    /// <summary>Write-then-replace so a crash never leaves a half-written file. Keeps the previous copy as <paramref name="backupPath"/>.</summary>
    public static void WriteAtomic(string path, ReadOnlySpan<byte> bytes, string? backupPath = null)
    {
        EnsureDataDirectory();
        var temp = path + ".tmp";
        using (var stream = new FileStream(temp, FileMode.Create, FileAccess.Write, FileShare.None))
        {
            stream.Write(bytes);
            stream.Flush(flushToDisk: true);
        }

        if (File.Exists(path))
            File.Replace(temp, path, backupPath, ignoreMetadataErrors: true);
        else
            File.Move(temp, path);
    }
}
