using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;
using Authenticator.Core.Crypto;
using Authenticator.Core.Models;
using Authenticator.Core.Storage;

namespace Authenticator.Core.Vault;

public enum VaultProtection
{
    WindowsHello,
    Password,
}

/// <summary>
/// On-disk vault. The account list is encrypted with a random vault key; the vault key is stored
/// wrapped by either Windows Hello (platform layer) or an Argon2id password key.
/// </summary>
public sealed class VaultFile
{
    public int Version { get; set; } = 1;
    public VaultProtection Protection { get; set; }

    /// <summary>Password mode only.</summary>
    public KdfParameters? Kdf { get; set; }

    /// <summary>Windows Hello mode only: random challenge that the Hello key signs to derive the wrapping key.</summary>
    public byte[]? HelloChallenge { get; set; }

    public byte[] WrappedKey { get; set; } = [];
    public byte[] Data { get; set; } = [];

    private static readonly byte[] DataAad = "authenticator-vault-v1"u8.ToArray();

    public static byte[] Serialize(VaultFile file) => JsonSerializer.SerializeToUtf8Bytes(file, CoreJsonContext.Default.VaultFile);

    public static VaultFile Deserialize(ReadOnlySpan<byte> json) =>
        JsonSerializer.Deserialize(json, CoreJsonContext.Default.VaultFile) ?? throw new InvalidDataException("Vault file is empty.");

    public void SealContents(ReadOnlySpan<byte> vaultKey, IEnumerable<Account> accounts)
    {
        var contents = new VaultContents { Accounts = accounts.ToList() };
        var plaintext = JsonSerializer.SerializeToUtf8Bytes(contents, CoreJsonContext.Default.VaultContents);
        try
        {
            Data = Aead.Seal(vaultKey, plaintext, DataAad);
        }
        finally
        {
            CryptographicOperations.ZeroMemory(plaintext);
        }
    }

    public List<Account> OpenContents(ReadOnlySpan<byte> vaultKey)
    {
        var plaintext = Aead.Open(vaultKey, Data, DataAad);
        try
        {
            return JsonSerializer.Deserialize(plaintext, CoreJsonContext.Default.VaultContents)?.Accounts ?? [];
        }
        finally
        {
            CryptographicOperations.ZeroMemory(plaintext);
        }
    }
}

public sealed class VaultContents
{
    [JsonPropertyOrder(0)]
    public int Version { get; set; } = 1;

    public List<Account> Accounts { get; set; } = [];
}
