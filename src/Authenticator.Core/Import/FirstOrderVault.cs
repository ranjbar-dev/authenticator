using System.Security.Cryptography;
using System.Text;

namespace Authenticator.Core.Import;

/// <summary>
/// Reads <c>.vault</c> exports from "Authenticator App · 2FA" (FirstOrder AI) made without a vault password.
/// The file is AES-256-CBC (PKCS7) with a fixed app-wide key and IV, over UTF-8 text with one block per account:
/// <c>uri:otpauth://…</c> followed by <c>backup_codes:…</c>.
/// </summary>
public static class FirstOrderVault
{
    public const string Extension = ".vault";

    private const string UriPrefix = "uri:";
    private const string BackupCodesPrefix = "backup_codes:";

    // Recovered from the app (v3.2.0); identical across exports, not tied to the device or account.
    private static readonly byte[] Key = Convert.FromHexString("5bd08577d89efa343561bda140aa709c6aaab4ae6985519f6a7eda3407180454");
    private static readonly byte[] Iv = Convert.FromHexString("6f965fa334bc2ec490733bd55ef8b855");

    public static bool IsVaultPath(string path) => string.Equals(Path.GetExtension(path), Extension, StringComparison.OrdinalIgnoreCase);

    /// <exception cref="OtpFormatException">Not a readable vault, or it holds no usable accounts.</exception>
    public static ParsedPayload Parse(byte[] fileBytes)
    {
        var plaintext = Decrypt(fileBytes);
        try
        {
            return ParseText(Encoding.UTF8.GetString(plaintext));
        }
        finally
        {
            CryptographicOperations.ZeroMemory(plaintext);
        }
    }

    /// <summary>Encrypts vault text the way the app does. Used by tests.</summary>
    public static byte[] Encrypt(string text)
    {
        using var aes = Aes.Create();
        aes.Key = Key;
        return aes.EncryptCbc(Encoding.UTF8.GetBytes(text), Iv);
    }

    private static byte[] Decrypt(byte[] fileBytes)
    {
        if (fileBytes.Length == 0 || fileBytes.Length % 16 != 0)
            throw new OtpFormatException("This file is not an Authenticator App · 2FA vault.");
        try
        {
            using var aes = Aes.Create();
            aes.Key = Key;
            return aes.DecryptCbc(fileBytes, Iv);
        }
        catch (CryptographicException)
        {
            throw new OtpFormatException("Couldn't open this vault. If you set a vault password when exporting it, export it again without one.");
        }
    }

    internal static ParsedPayload ParseText(string text)
    {
        if (!text.StartsWith(UriPrefix, StringComparison.Ordinal))
            throw new OtpFormatException("Couldn't open this vault. If you set a vault password when exporting it, export it again without one.");

        var result = new ParsedPayload();
        var skipped = 0;
        var hasBackupCodes = false;
        foreach (var rawLine in text.Split('\n'))
        {
            var line = rawLine.Trim();
            if (line.StartsWith(UriPrefix, StringComparison.Ordinal))
            {
                try
                {
                    result.Accounts.Add(OtpAuthUri.Parse(line[UriPrefix.Length..]));
                }
                catch (OtpFormatException)
                {
                    skipped++;
                }
            }
            else if (line.Length > 0 && line != BackupCodesPrefix)
            {
                // Codes either follow "backup_codes:" on the same line or on the lines after it.
                hasBackupCodes = true;
            }
        }

        if (result.Accounts.Count == 0)
            throw new OtpFormatException(skipped > 0 ? "None of the accounts in this vault could be read." : "This vault has no accounts.");
        if (skipped > 0)
            result.Warnings.Add($"{skipped} account{(skipped == 1 ? "" : "s")} in the vault couldn't be read and {(skipped == 1 ? "was" : "were")} skipped.");
        if (hasBackupCodes)
            result.Warnings.Add("Backup codes saved in the vault aren't imported. Keep a copy of them elsewhere.");
        return result;
    }
}
