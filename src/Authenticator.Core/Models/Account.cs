using System.Text.Json.Serialization;

namespace Authenticator.Core.Models;

public enum OtpType
{
    Totp,
    Hotp,
}

public enum OtpAlgorithm
{
    Sha1,
    Sha256,
    Sha512,
}

/// <summary>A single authenticator account. <see cref="Secret"/> holds raw key bytes, never Base32.</summary>
public sealed class Account
{
    public const int DefaultDigits = 6;
    public const int DefaultPeriod = 30;

    public Guid Id { get; set; } = Guid.NewGuid();
    public string Issuer { get; set; } = "";
    public string Name { get; set; } = "";
    public byte[] Secret { get; set; } = [];
    public OtpType Type { get; set; } = OtpType.Totp;
    public OtpAlgorithm Algorithm { get; set; } = OtpAlgorithm.Sha1;
    public int Digits { get; set; } = DefaultDigits;

    /// <summary>TOTP time step in seconds.</summary>
    public int Period { get; set; } = DefaultPeriod;

    /// <summary>HOTP: the counter value used for the <em>next</em> generated code.</summary>
    public long Counter { get; set; }

    public bool Pinned { get; set; }
    public int SortOrder { get; set; }
    public DateTimeOffset CreatedAt { get; set; } = DateTimeOffset.UtcNow;

    /// <summary>Issuer if present, otherwise the account name. Used for avatars and titles.</summary>
    [JsonIgnore]
    public string DisplayTitle => string.IsNullOrWhiteSpace(Issuer) ? Name : Issuer;

    public Account Clone()
    {
        var copy = (Account)MemberwiseClone();
        copy.Secret = (byte[])Secret.Clone();
        return copy;
    }

    /// <summary>Overwrites the secret in memory. Best effort; the GC may have copied it before.</summary>
    public void Wipe() => Array.Clear(Secret);
}
