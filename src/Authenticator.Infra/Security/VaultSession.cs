using System.Security.Cryptography;
using Authenticator.Core.Crypto;
using Authenticator.Core.Models;
using Authenticator.Core.Vault;

namespace Authenticator.Infra.Security;

public sealed class WrongPasswordException() : Exception("That password is incorrect.");

/// <summary>
/// Owns the encrypted vault on disk and, while unlocked, the vault key and decrypted accounts in memory.
/// <para>Hello mode: wrappedKey = DPAPI(AES-GCM(SHA-256(HelloSignature(challenge)), vaultKey)).</para>
/// <para>Password mode: wrappedKey = AES-GCM(Argon2id(password), vaultKey).</para>
/// </summary>
public sealed class VaultSession
{
    private static readonly byte[] WrapAad = "authenticator-vault-key-v1"u8.ToArray();

    private VaultFile? _file;
    private byte[]? _vaultKey;
    private Task _pendingWrite = Task.CompletedTask;

    public List<Account> Accounts { get; private set; } = [];
    public bool IsUnlocked => _vaultKey != null;
    public VaultProtection? Protection => _file?.Protection;

    public event EventHandler? Locked;

    public static bool Exists => File.Exists(AppPaths.Vault);

    /// <summary>Loads the encrypted file header so <see cref="Protection"/> is known before unlocking.</summary>
    public void Load()
    {
        try
        {
            _file = VaultFile.Deserialize(File.ReadAllBytes(AppPaths.Vault));
        }
        catch (Exception ex) when (ex is IOException or System.Text.Json.JsonException or InvalidDataException && File.Exists(AppPaths.VaultBackup))
        {
            _file = VaultFile.Deserialize(File.ReadAllBytes(AppPaths.VaultBackup));
        }
    }

    public async Task CreateWithHelloAsync()
    {
        var challenge = RandomNumberGenerator.GetBytes(32);
        var signature = await WindowsHello.CreateAndSignAsync(challenge);
        var vaultKey = Aead.NewKey();
        _file = new VaultFile
        {
            Protection = VaultProtection.WindowsHello,
            HelloChallenge = challenge,
            WrappedKey = WrapForHello(signature, vaultKey),
        };
        Start(vaultKey, []);
        await SaveAsync();
    }

    public async Task CreateWithPasswordAsync(string password)
    {
        var kdf = KdfParameters.CreateDefault();
        var vaultKey = Aead.NewKey();
        var wrapped = await Task.Run(() => WrapForPassword(password, kdf, vaultKey));
        _file = new VaultFile { Protection = VaultProtection.Password, Kdf = kdf, WrappedKey = wrapped };
        Start(vaultKey, []);
        await SaveAsync();
    }

    public async Task UnlockWithHelloAsync()
    {
        var file = RequireFile(VaultProtection.WindowsHello);
        var signature = await WindowsHello.SignAsync(file.HelloChallenge!);
        var vaultKey = UnwrapForHello(signature, file.WrappedKey);
        Start(vaultKey, file.OpenContents(vaultKey));
    }

    public async Task UnlockWithPasswordAsync(string password)
    {
        var file = RequireFile(VaultProtection.Password);
        var vaultKey = await Task.Run(() => UnwrapForPassword(password, file));
        Start(vaultKey, file.OpenContents(vaultKey));
    }

    /// <summary>Re-authenticates the user while unlocked (before revealing secrets or exporting).</summary>
    public async Task VerifyAsync(string? password = null)
    {
        var file = _file ?? throw new InvalidOperationException("No vault loaded.");
        var key = file.Protection == VaultProtection.WindowsHello
            ? UnwrapForHello(await WindowsHello.SignAsync(file.HelloChallenge!), file.WrappedKey)
            : await Task.Run(() => UnwrapForPassword(password ?? "", file));
        CryptographicOperations.ZeroMemory(key);
    }

    public async Task SwitchToPasswordAsync(string password)
    {
        var vaultKey = RequireKey();
        var kdf = KdfParameters.CreateDefault();
        var wrapped = await Task.Run(() => WrapForPassword(password, kdf, vaultKey));
        _file = new VaultFile { Protection = VaultProtection.Password, Kdf = kdf, WrappedKey = wrapped };
        await SaveAsync();
        await WindowsHello.DeleteAsync();
    }

    public async Task SwitchToHelloAsync()
    {
        var vaultKey = RequireKey();
        var challenge = RandomNumberGenerator.GetBytes(32);
        var signature = await WindowsHello.CreateAndSignAsync(challenge);
        _file = new VaultFile
        {
            Protection = VaultProtection.WindowsHello,
            HelloChallenge = challenge,
            WrappedKey = WrapForHello(signature, vaultKey),
        };
        await SaveAsync();
    }

    /// <summary>Encrypts the current <see cref="Accounts"/> and queues an atomic write. Writes complete in call order.</summary>
    public Task SaveAsync()
    {
        var vaultKey = RequireKey();
        var file = _file!;
        file.SealContents(vaultKey, Accounts);
        var bytes = VaultFile.Serialize(file);
        _pendingWrite = _pendingWrite.ContinueWith(
            _ => AppPaths.WriteAtomic(AppPaths.Vault, bytes, AppPaths.VaultBackup),
            CancellationToken.None, TaskContinuationOptions.None, TaskScheduler.Default);
        return _pendingWrite;
    }

    public void Lock()
    {
        if (_vaultKey == null)
            return;
        CryptographicOperations.ZeroMemory(_vaultKey);
        _vaultKey = null;
        foreach (var account in Accounts)
            account.Wipe();
        Accounts = [];
        Locked?.Invoke(this, EventArgs.Empty);
    }

    /// <summary>Waits for queued vault writes, e.g. before the process exits.</summary>
    public void Flush() => _pendingWrite.Wait(TimeSpan.FromSeconds(5));

    private void Start(byte[] vaultKey, List<Account> accounts)
    {
        _vaultKey = vaultKey;
        Accounts = accounts;
    }

    private byte[] RequireKey() => _vaultKey ?? throw new InvalidOperationException("The vault is locked.");

    private VaultFile RequireFile(VaultProtection expected)
    {
        if (_file == null)
            Load();
        if (_file!.Protection != expected)
            throw new InvalidOperationException($"Vault is protected by {_file.Protection}.");
        return _file;
    }

    private static byte[] HelloWrappingKey(byte[] signature)
    {
        var key = SHA256.HashData(signature);
        CryptographicOperations.ZeroMemory(signature);
        return key;
    }

    private static byte[] WrapForHello(byte[] signature, byte[] vaultKey)
    {
        var wrappingKey = HelloWrappingKey(signature);
        try
        {
            return Dpapi.Protect(Aead.Seal(wrappingKey, vaultKey, WrapAad));
        }
        finally
        {
            CryptographicOperations.ZeroMemory(wrappingKey);
        }
    }

    private static byte[] UnwrapForHello(byte[] signature, byte[] wrapped)
    {
        var wrappingKey = HelloWrappingKey(signature);
        try
        {
            return Aead.Open(wrappingKey, Dpapi.Unprotect(wrapped), WrapAad);
        }
        catch (CryptographicException)
        {
            throw new WindowsHelloException("Windows Hello could not unlock the vault. If Hello was reset, restore from a backup.");
        }
        finally
        {
            CryptographicOperations.ZeroMemory(wrappingKey);
        }
    }

    private static byte[] WrapForPassword(string password, KdfParameters kdf, byte[] vaultKey)
    {
        var wrappingKey = PasswordKdf.DeriveKey(password, kdf);
        try
        {
            return Aead.Seal(wrappingKey, vaultKey, WrapAad);
        }
        finally
        {
            CryptographicOperations.ZeroMemory(wrappingKey);
        }
    }

    private static byte[] UnwrapForPassword(string password, VaultFile file)
    {
        var wrappingKey = PasswordKdf.DeriveKey(password, file.Kdf!);
        try
        {
            return Aead.Open(wrappingKey, file.WrappedKey, WrapAad);
        }
        catch (CryptographicException)
        {
            throw new WrongPasswordException();
        }
        finally
        {
            CryptographicOperations.ZeroMemory(wrappingKey);
        }
    }
}
