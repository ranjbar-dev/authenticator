using System.Security.Cryptography;
using System.Text;
using Konscious.Security.Cryptography;

namespace Authenticator.Core.Crypto;

/// <summary>Argon2id parameters, stored alongside the data they protect.</summary>
public sealed class KdfParameters
{
    public string Name { get; set; } = "argon2id";
    public byte[] Salt { get; set; } = [];
    public int MemoryKiB { get; set; } = 64 * 1024;
    public int Iterations { get; set; } = 3;
    public int Parallelism { get; set; } = 1;

    public static KdfParameters CreateDefault() => new() { Salt = RandomNumberGenerator.GetBytes(16) };
}

public static class PasswordKdf
{
    public static byte[] DeriveKey(string password, KdfParameters parameters)
    {
        if (parameters.Name != "argon2id")
            throw new CryptographicException($"Unsupported key derivation \"{parameters.Name}\".");

        var passwordBytes = Encoding.UTF8.GetBytes(password);
        try
        {
            using var argon = new Argon2id(passwordBytes)
            {
                Salt = parameters.Salt,
                MemorySize = parameters.MemoryKiB,
                Iterations = parameters.Iterations,
                DegreeOfParallelism = parameters.Parallelism,
            };
            return argon.GetBytes(Aead.KeySize);
        }
        finally
        {
            CryptographicOperations.ZeroMemory(passwordBytes);
        }
    }
}

public static class PasswordStrength
{
    /// <summary>Rough 0–4 score: length plus character variety, with a penalty for repeated characters.</summary>
    public static int Score(string password)
    {
        if (string.IsNullOrEmpty(password))
            return 0;

        var classes = 0;
        if (password.Any(char.IsLower)) classes++;
        if (password.Any(char.IsUpper)) classes++;
        if (password.Any(char.IsDigit)) classes++;
        if (password.Any(c => !char.IsLetterOrDigit(c))) classes++;

        var unique = password.Distinct().Count();
        var effectiveLength = Math.Min(password.Length, unique * 2);

        var score = effectiveLength switch
        {
            < 8 => 0,
            < 10 => 1,
            < 13 => 2,
            < 16 => 3,
            _ => 4,
        };
        if (classes >= 3) score++;
        if (classes <= 1) score--;
        return Math.Clamp(score, 0, 4);
    }

    public static string Describe(int score) => score switch
    {
        0 => "Very weak",
        1 => "Weak",
        2 => "Fair",
        3 => "Strong",
        _ => "Very strong",
    };
}
