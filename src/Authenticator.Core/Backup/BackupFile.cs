using System.Security.Cryptography;
using System.Text.Json;
using Authenticator.Core.Crypto;
using Authenticator.Core.Models;
using Authenticator.Core.Storage;

namespace Authenticator.Core.Backup;

/// <summary>Encrypted, password-protected <c>.authbackup</c> file (AES-256-GCM, key from Argon2id).</summary>
public sealed class BackupFile
{
    public const string Extension = ".authbackup";
    public const string FormatName = "authenticator-backup";

    public string Format { get; set; } = FormatName;
    public int Version { get; set; } = 1;
    public KdfParameters Kdf { get; set; } = new();
    public string Cipher { get; set; } = "aes-256-gcm";
    public byte[] Data { get; set; } = [];

    private static readonly byte[] Aad = "authenticator-backup-v1"u8.ToArray();

    public static byte[] Create(IEnumerable<Account> accounts, string password, DateTimeOffset now, KdfParameters? kdf = null)
    {
        kdf ??= KdfParameters.CreateDefault();
        var contents = new BackupContents { ExportedAt = now, Accounts = accounts.ToList() };
        var plaintext = JsonSerializer.SerializeToUtf8Bytes(contents, CoreJsonContext.Default.BackupContents);
        var key = PasswordKdf.DeriveKey(password, kdf);
        try
        {
            var file = new BackupFile { Kdf = kdf, Data = Aead.Seal(key, plaintext, Aad) };
            return JsonSerializer.SerializeToUtf8Bytes(file, CoreJsonContext.Default.BackupFile);
        }
        finally
        {
            CryptographicOperations.ZeroMemory(plaintext);
            CryptographicOperations.ZeroMemory(key);
        }
    }

    /// <exception cref="InvalidDataException">Not a backup file.</exception>
    /// <exception cref="CryptographicException">Wrong password or corrupted file.</exception>
    public static BackupContents Open(ReadOnlySpan<byte> fileBytes, string password)
    {
        BackupFile? file;
        try
        {
            file = JsonSerializer.Deserialize(fileBytes, CoreJsonContext.Default.BackupFile);
        }
        catch (JsonException)
        {
            file = null;
        }
        if (file == null || file.Format != FormatName)
            throw new InvalidDataException("This file is not an Authenticator backup.");
        if (file.Version != 1 || file.Cipher != "aes-256-gcm")
            throw new InvalidDataException("This backup was made by a newer version of Authenticator.");

        var key = PasswordKdf.DeriveKey(password, file.Kdf);
        byte[]? plaintext = null;
        try
        {
            plaintext = Aead.Open(key, file.Data, Aad);
            return JsonSerializer.Deserialize(plaintext, CoreJsonContext.Default.BackupContents)
                   ?? throw new InvalidDataException("The backup is empty.");
        }
        finally
        {
            CryptographicOperations.ZeroMemory(key);
            if (plaintext != null)
                CryptographicOperations.ZeroMemory(plaintext);
        }
    }
}

public sealed class BackupContents
{
    public DateTimeOffset ExportedAt { get; set; }
    public List<Account> Accounts { get; set; } = [];
}
