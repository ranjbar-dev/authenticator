using System.Buffers.Binary;
using System.Security.Cryptography;
using Authenticator.Core.Models;

namespace Authenticator.Core.Otp;

/// <summary>HOTP (RFC 4226) and TOTP (RFC 6238) code generation.</summary>
public static class OtpGenerator
{
    public const int MinDigits = 6;
    public const int MaxDigits = 8;

    public static string Hotp(ReadOnlySpan<byte> secret, long counter, int digits = 6, OtpAlgorithm algorithm = OtpAlgorithm.Sha1)
    {
        if (digits is < MinDigits or > MaxDigits)
            throw new ArgumentOutOfRangeException(nameof(digits));

        Span<byte> message = stackalloc byte[8];
        BinaryPrimitives.WriteInt64BigEndian(message, counter);

        Span<byte> hash = stackalloc byte[64];
        var length = algorithm switch
        {
            OtpAlgorithm.Sha1 => HMACSHA1.HashData(secret, message, hash),
            OtpAlgorithm.Sha256 => HMACSHA256.HashData(secret, message, hash),
            OtpAlgorithm.Sha512 => HMACSHA512.HashData(secret, message, hash),
            _ => throw new ArgumentOutOfRangeException(nameof(algorithm)),
        };
        hash = hash[..length];

        // Dynamic truncation (RFC 4226 §5.3).
        var offset = hash[^1] & 0x0F;
        var binary = ((hash[offset] & 0x7F) << 24)
                     | (hash[offset + 1] << 16)
                     | (hash[offset + 2] << 8)
                     | hash[offset + 3];

        var modulo = digits switch { 6 => 1_000_000, 7 => 10_000_000, _ => 100_000_000 };
        return (binary % modulo).ToString().PadLeft(digits, '0');
    }

    public static long TimeStep(DateTimeOffset now, int period) => now.ToUnixTimeSeconds() / period;

    public static string Totp(ReadOnlySpan<byte> secret, DateTimeOffset now, int period = 30, int digits = 6, OtpAlgorithm algorithm = OtpAlgorithm.Sha1) =>
        Hotp(secret, TimeStep(now, period), digits, algorithm);

    /// <summary>Fractional seconds left in the current TOTP window, in (0, period].</summary>
    public static double SecondsRemaining(DateTimeOffset now, int period)
    {
        var ms = now.ToUnixTimeMilliseconds() % (period * 1000L);
        return period - ms / 1000.0;
    }

    /// <summary>The current code for a TOTP account, or the code for the stored counter of an HOTP account.</summary>
    public static string Current(Account account, DateTimeOffset now) => account.Type == OtpType.Totp
        ? Totp(account.Secret, now, account.Period, account.Digits, account.Algorithm)
        : Hotp(account.Secret, account.Counter, account.Digits, account.Algorithm);

    /// <summary>"123456" → "123 456", "12345678" → "1234 5678", "1234567" → "123 4567".</summary>
    public static string Format(string code)
    {
        if (code.Length < 6)
            return code;
        var split = code.Length / 2;
        return $"{code[..split]} {code[split..]}";
    }
}
