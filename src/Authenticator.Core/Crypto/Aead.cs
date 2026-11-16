using System.Security.Cryptography;

namespace Authenticator.Core.Crypto;

/// <summary>AES-256-GCM. Output layout: nonce (12) | ciphertext | tag (16).</summary>
public static class Aead
{
    public const int KeySize = 32;
    private const int NonceSize = 12;
    private const int TagSize = 16;

    public static byte[] NewKey() => RandomNumberGenerator.GetBytes(KeySize);

    public static byte[] Seal(ReadOnlySpan<byte> key, ReadOnlySpan<byte> plaintext, ReadOnlySpan<byte> associatedData = default)
    {
        var output = new byte[NonceSize + plaintext.Length + TagSize];
        var nonce = output.AsSpan(0, NonceSize);
        RandomNumberGenerator.Fill(nonce);
        using var aes = new AesGcm(key, TagSize);
        aes.Encrypt(nonce, plaintext, output.AsSpan(NonceSize, plaintext.Length), output.AsSpan(NonceSize + plaintext.Length), associatedData);
        return output;
    }

    /// <exception cref="CryptographicException">Wrong key or tampered data.</exception>
    public static byte[] Open(ReadOnlySpan<byte> key, ReadOnlySpan<byte> sealedData, ReadOnlySpan<byte> associatedData = default)
    {
        if (sealedData.Length < NonceSize + TagSize)
            throw new CryptographicException("Encrypted data is truncated.");
        var plaintext = new byte[sealedData.Length - NonceSize - TagSize];
        using var aes = new AesGcm(key, TagSize);
        aes.Decrypt(sealedData[..NonceSize], sealedData[NonceSize..^TagSize], sealedData[^TagSize..], plaintext, associatedData);
        return plaintext;
    }
}
