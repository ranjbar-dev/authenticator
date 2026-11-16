using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Authenticator.Core.Otp;

namespace Authenticator.Core.Tests;

public class ImportTests
{
    [Fact]
    public void Parses_full_totp_uri()
    {
        var a = OtpAuthUri.Parse("otpauth://totp/ACME%20Co:john.doe@email.com?secret=HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ&issuer=ACME%20Co&algorithm=SHA256&digits=8&period=60");
        Assert.Equal(OtpType.Totp, a.Type);
        Assert.Equal("ACME Co", a.Issuer);
        Assert.Equal("john.doe@email.com", a.Name);
        Assert.Equal(OtpAlgorithm.Sha256, a.Algorithm);
        Assert.Equal(8, a.Digits);
        Assert.Equal(60, a.Period);
        Assert.Equal("HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ", Base32.Encode(a.Secret));
    }

    [Fact]
    public void Issuer_falls_back_to_label_prefix_and_defaults_apply()
    {
        var a = OtpAuthUri.Parse("otpauth://totp/GitHub%3Alarry?secret=JBSWY3DPEHPK3PXP");
        Assert.Equal("GitHub", a.Issuer);
        Assert.Equal("larry", a.Name);
        Assert.Equal(OtpAlgorithm.Sha1, a.Algorithm);
        Assert.Equal(6, a.Digits);
        Assert.Equal(30, a.Period);
    }

    [Fact]
    public void Parses_hotp_with_counter_and_requires_it()
    {
        var a = OtpAuthUri.Parse("otpauth://hotp/Example:alice?secret=JBSWY3DPEHPK3PXP&counter=42");
        Assert.Equal(OtpType.Hotp, a.Type);
        Assert.Equal(42, a.Counter);
        Assert.Throws<OtpFormatException>(() => OtpAuthUri.Parse("otpauth://hotp/Example:alice?secret=JBSWY3DPEHPK3PXP"));
    }

    [Theory]
    [InlineData("https://example.com")]
    [InlineData("otpauth://totp/Example?issuer=x")]
    [InlineData("otpauth://totp/Example?secret=!!!")]
    [InlineData("otpauth://steam/Example?secret=JBSWY3DPEHPK3PXP")]
    [InlineData("otpauth://totp/Example?secret=JBSWY3DPEHPK3PXP&digits=4")]
    public void Rejects_invalid_uris(string uri) => Assert.Throws<OtpFormatException>(() => OtpAuthUri.Parse(uri));

    [Fact]
    public void Build_and_parse_round_trip()
    {
        var original = new Account
        {
            Issuer = "My Bank", Name = "me@x.com", Secret = Base32.Decode("JBSWY3DPEHPK3PXP"),
            Type = OtpType.Hotp, Algorithm = OtpAlgorithm.Sha512, Digits = 8, Counter = 7,
        };
        var parsed = OtpAuthUri.Parse(OtpAuthUri.Build(original));
        Assert.Equal(original.Issuer, parsed.Issuer);
        Assert.Equal(original.Name, parsed.Name);
        Assert.Equal(original.Secret, parsed.Secret);
        Assert.Equal(original.Type, parsed.Type);
        Assert.Equal(original.Algorithm, parsed.Algorithm);
        Assert.Equal(original.Digits, parsed.Digits);
        Assert.Equal(original.Counter, parsed.Counter);
    }

    [Fact]
    public void Google_migration_round_trips_in_batches()
    {
        var accounts = Enumerable.Range(0, 11).Select(i => new Account
        {
            Issuer = $"Service {i}", Name = $"user{i}@mail.com",
            Secret = Base32.Decode("JBSWY3DPEHPK3PXP" + (char)('A' + i) + "A"),
            Type = i == 3 ? OtpType.Hotp : OtpType.Totp, Counter = i == 3 ? 5 : 0,
            Algorithm = i == 4 ? OtpAlgorithm.Sha256 : OtpAlgorithm.Sha1,
            Digits = i == 5 ? 8 : 6,
        }).ToList();

        var uris = GoogleMigration.Build(accounts, batchSize: 8);
        Assert.Equal(2, uris.Count);

        var first = GoogleMigration.Parse(uris[0]);
        var second = GoogleMigration.Parse(uris[1]);
        Assert.Equal(2, first.BatchSize);
        Assert.Equal(0, first.BatchIndex);
        Assert.True(first.HasMoreBatches);
        Assert.Equal(1, second.BatchIndex);
        Assert.False(second.HasMoreBatches);
        Assert.Equal(first.BatchId, second.BatchId);

        var all = first.Accounts.Concat(second.Accounts).ToList();
        Assert.Equal(accounts.Count, all.Count);
        for (var i = 0; i < accounts.Count; i++)
        {
            Assert.Equal(accounts[i].Issuer, all[i].Issuer);
            Assert.Equal(accounts[i].Name, all[i].Name);
            Assert.Equal(accounts[i].Secret, all[i].Secret);
            Assert.Equal(accounts[i].Type, all[i].Type);
            Assert.Equal(accounts[i].Counter, all[i].Counter);
            Assert.Equal(accounts[i].Algorithm, all[i].Algorithm);
            Assert.Equal(accounts[i].Digits, all[i].Digits);
        }
    }

    [Fact]
    public void Parses_real_google_authenticator_export()
    {
        // Encoded independently (Python) from Google's MigrationPayload schema: one TOTP account
        // "Example:alice@google.com", SHA1, six digits, batch 0 of 1, URL-encoded Base64 like real exports.
        const string uri = "otpauth-migration://offline?data=CjUKCkhlbGxvId6tvu8SGEV4YW1wbGU6YWxpY2VAZ29vZ2xlLmNvbRoHRXhhbXBsZSABKAEwAhABGAEgACi5YA%3D%3D";
        var payload = GoogleMigration.Parse(uri);
        var a = Assert.Single(payload.Accounts);
        Assert.Equal("Example", a.Issuer);
        Assert.Equal("alice@google.com", a.Name);
        Assert.Equal(OtpType.Totp, a.Type);
        Assert.Equal(6, a.Digits);
        Assert.Equal("JBSWY3DPEHPK3PXP", Base32.Encode(a.Secret));
    }

    [Fact]
    public void Payload_parser_reports_non_authenticator_qr()
    {
        var ex = Assert.Throws<OtpFormatException>(() => PayloadParser.Parse("https://example.com"));
        Assert.Contains("not an authenticator", ex.Message);
    }

    [Fact]
    public void Payload_parser_keeps_good_codes_when_one_is_bad()
    {
        var result = PayloadParser.Parse(["https://example.com", "otpauth://totp/A:b?secret=JBSWY3DPEHPK3PXP"]);
        Assert.Single(result.Accounts);
        Assert.Single(result.Warnings);
    }

    [Fact]
    public void Parses_first_order_vault()
    {
        // Same layout as a real export: "uri:" + otpauth link, then an empty "backup_codes:" line per account.
        const string text =
            "uri:otpauth://totp/Google%3Aalice%40gmail.com?secret=JBSWY3DPEHPK3PXP&issuer=Google&algorithm=SHA1&digits=6&period=30&websiteurl=Z29vZ2xlLmNvbQ==\n" +
            "backup_codes:\n" +
            "uri:otpauth://totp/Binance:bob@mail.com?secret=HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ&issuer=Binance&algorithm=SHA512&digits=6&period=30\n" +
            "backup_codes:\n";
        var payload = FirstOrderVault.Parse(FirstOrderVault.Encrypt(text));

        Assert.Equal(2, payload.Accounts.Count);
        Assert.Empty(payload.Warnings);
        Assert.Equal("Google", payload.Accounts[0].Issuer);
        Assert.Equal("alice@gmail.com", payload.Accounts[0].Name);
        Assert.Equal("JBSWY3DPEHPK3PXP", Base32.Encode(payload.Accounts[0].Secret));
        Assert.Equal(OtpAlgorithm.Sha512, payload.Accounts[1].Algorithm);
    }

    [Fact]
    public void First_order_vault_matches_real_export_bytes()
    {
        // Every real export starts with this ciphertext block (the fixed IV encrypting "uri:otpauth://to").
        var encrypted = FirstOrderVault.Encrypt("uri:otpauth://totp/A:b?secret=JBSWY3DPEHPK3PXP\nbackup_codes:\n");
        Assert.Equal("ad0ff386f541", Convert.ToHexStringLower(encrypted.AsSpan(0, 6)));
    }

    [Fact]
    public void First_order_vault_skips_bad_accounts_and_flags_backup_codes()
    {
        const string text =
            "uri:otpauth://totp/A:b?secret=JBSWY3DPEHPK3PXP\nbackup_codes:\n" +
            "uri:otpauth://totp/C:d?secret=!!!\nbackup_codes:12345678,87654321\n";
        var payload = FirstOrderVault.Parse(FirstOrderVault.Encrypt(text));
        Assert.Single(payload.Accounts);
        Assert.Equal(2, payload.Warnings.Count);
    }

    [Fact]
    public void First_order_vault_rejects_other_files()
    {
        Assert.Throws<OtpFormatException>(() => FirstOrderVault.Parse([1, 2, 3]));
        Assert.Throws<OtpFormatException>(() => FirstOrderVault.Parse(new byte[64]));
        Assert.Throws<OtpFormatException>(() => FirstOrderVault.Parse(FirstOrderVault.Encrypt("{\"not\":\"a vault\"}")));
    }

    [Fact]
    public void Duplicates_detected_by_secret_or_label()
    {
        var a = new Account { Issuer = "GitHub", Name = "larry", Secret = [1, 2, 3] };
        Assert.True(AccountMatcher.IsDuplicate(new Account { Issuer = "x", Name = "y", Secret = [1, 2, 3] }, a));
        Assert.True(AccountMatcher.IsDuplicate(new Account { Issuer = "github ", Name = "LARRY", Secret = [9] }, a));
        Assert.False(AccountMatcher.IsDuplicate(new Account { Issuer = "GitHub", Name = "other", Secret = [9] }, a));
    }
}
