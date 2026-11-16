using System.Security.Cryptography;
using Authenticator.Core.Backup;
using Authenticator.Core.Crypto;
using Authenticator.Core.Models;
using Authenticator.Core.Vault;

namespace Authenticator.Core.Tests;

public class CryptoTests
{
    // Cheap KDF parameters keep the tests fast; production uses KdfParameters.CreateDefault().
    private static KdfParameters FastKdf() => new() { Salt = RandomNumberGenerator.GetBytes(16), MemoryKiB = 1024, Iterations = 1 };

    private static List<Account> SampleAccounts() =>
    [
        new() { Issuer = "GitHub", Name = "larry", Secret = [1, 2, 3, 4, 5], Pinned = true },
        new() { Issuer = "Bank", Name = "me", Secret = [9, 8, 7], Type = OtpType.Hotp, Counter = 12 },
    ];

    [Fact]
    public void Aead_round_trips_and_detects_tampering()
    {
        var key = Aead.NewKey();
        var sealedData = Aead.Seal(key, "hello"u8, "aad"u8);
        Assert.Equal("hello"u8.ToArray(), Aead.Open(key, sealedData, "aad"u8));

        sealedData[^1] ^= 1;
        Assert.ThrowsAny<CryptographicException>(() => Aead.Open(key, sealedData, "aad"u8));
        Assert.ThrowsAny<CryptographicException>(() => Aead.Open(Aead.NewKey(), Aead.Seal(key, "x"u8)));
    }

    [Fact]
    public void Vault_contents_round_trip_through_json()
    {
        var key = Aead.NewKey();
        var file = new VaultFile { Protection = VaultProtection.Password, Kdf = FastKdf(), WrappedKey = [1] };
        file.SealContents(key, SampleAccounts());

        var reloaded = VaultFile.Deserialize(VaultFile.Serialize(file));
        var accounts = reloaded.OpenContents(key);

        Assert.Equal(VaultProtection.Password, reloaded.Protection);
        Assert.Equal(2, accounts.Count);
        Assert.True(accounts[0].Pinned);
        Assert.Equal(new byte[] { 1, 2, 3, 4, 5 }, accounts[0].Secret);
        Assert.Equal(OtpType.Hotp, accounts[1].Type);
        Assert.Equal(12, accounts[1].Counter);
    }

    [Fact]
    public void Backup_round_trips_and_rejects_wrong_password()
    {
        var bytes = BackupFile.Create(SampleAccounts(), "correct horse battery", DateTimeOffset.UnixEpoch, FastKdf());

        var contents = BackupFile.Open(bytes, "correct horse battery");
        Assert.Equal(2, contents.Accounts.Count);
        Assert.Equal("GitHub", contents.Accounts[0].Issuer);

        Assert.ThrowsAny<CryptographicException>(() => BackupFile.Open(bytes, "wrong password"));
        Assert.Throws<InvalidDataException>(() => BackupFile.Open("{\"format\":\"other\"}"u8, "x"));
        Assert.Throws<InvalidDataException>(() => BackupFile.Open("not json"u8, "x"));
    }

    [Fact]
    public void Password_kdf_is_deterministic_per_salt()
    {
        var kdf = FastKdf();
        Assert.Equal(PasswordKdf.DeriveKey("pw", kdf), PasswordKdf.DeriveKey("pw", kdf));
        Assert.NotEqual(PasswordKdf.DeriveKey("pw", kdf), PasswordKdf.DeriveKey("pw2", kdf));
    }

    [Theory]
    [InlineData("", 0)]
    [InlineData("aaaaaaaaaaaa", 0)]
    [InlineData("Tr0ub4dor&3xyz!", 4)]
    public void Password_strength_scores(string password, int expected) =>
        Assert.Equal(expected, PasswordStrength.Score(password));
}
