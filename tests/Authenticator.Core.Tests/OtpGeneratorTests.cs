using System.Text;
using Authenticator.Core.Models;
using Authenticator.Core.Otp;

namespace Authenticator.Core.Tests;

public class OtpGeneratorTests
{
    private static readonly byte[] Sha1Key = Encoding.ASCII.GetBytes("12345678901234567890");
    private static readonly byte[] Sha256Key = Encoding.ASCII.GetBytes("12345678901234567890123456789012");
    private static readonly byte[] Sha512Key = Encoding.ASCII.GetBytes("1234567890123456789012345678901234567890123456789012345678901234");

    // RFC 4226 Appendix D.
    [Theory]
    [InlineData(0, "755224")]
    [InlineData(1, "287082")]
    [InlineData(2, "359152")]
    [InlineData(3, "969429")]
    [InlineData(4, "338314")]
    [InlineData(5, "254676")]
    [InlineData(6, "287922")]
    [InlineData(7, "162583")]
    [InlineData(8, "399871")]
    [InlineData(9, "520489")]
    public void Hotp_matches_rfc4226_vectors(long counter, string expected) =>
        Assert.Equal(expected, OtpGenerator.Hotp(Sha1Key, counter));

    // RFC 6238 Appendix B.
    [Theory]
    [InlineData(59L, "94287082", "46119246", "90693936")]
    [InlineData(1111111109L, "07081804", "68084774", "25091201")]
    [InlineData(1111111111L, "14050471", "67062674", "99943326")]
    [InlineData(1234567890L, "89005924", "91819424", "93441116")]
    [InlineData(2000000000L, "69279037", "90698825", "38618901")]
    [InlineData(20000000000L, "65353130", "77737706", "47863826")]
    public void Totp_matches_rfc6238_vectors(long unixTime, string sha1, string sha256, string sha512)
    {
        var now = DateTimeOffset.FromUnixTimeSeconds(unixTime);
        Assert.Equal(sha1, OtpGenerator.Totp(Sha1Key, now, 30, 8, OtpAlgorithm.Sha1));
        Assert.Equal(sha256, OtpGenerator.Totp(Sha256Key, now, 30, 8, OtpAlgorithm.Sha256));
        Assert.Equal(sha512, OtpGenerator.Totp(Sha512Key, now, 30, 8, OtpAlgorithm.Sha512));
    }

    [Fact]
    public void Seven_digit_codes_are_zero_padded()
    {
        var code = OtpGenerator.Totp(Sha1Key, DateTimeOffset.FromUnixTimeSeconds(1111111109), 30, 7);
        Assert.Equal(7, code.Length);
        Assert.Equal("7081804", code);
    }

    [Theory]
    [InlineData("123456", "123 456")]
    [InlineData("1234567", "123 4567")]
    [InlineData("12345678", "1234 5678")]
    public void Format_groups_digits(string code, string expected) => Assert.Equal(expected, OtpGenerator.Format(code));

    [Fact]
    public void Seconds_remaining_counts_down_within_period()
    {
        Assert.Equal(30, OtpGenerator.SecondsRemaining(DateTimeOffset.FromUnixTimeSeconds(60), 30));
        Assert.Equal(1, OtpGenerator.SecondsRemaining(DateTimeOffset.FromUnixTimeSeconds(89), 30));
    }

    [Fact]
    public void Base32_round_trips_and_is_lenient()
    {
        Assert.Equal(Sha1Key, Base32.Decode("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"));
        Assert.Equal(Sha1Key, Base32.Decode("gezd gnbv-gy3t qojq gezd gnbv gy3t qojq===="));
        Assert.Equal("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", Base32.Encode(Sha1Key));
        Assert.False(Base32.TryDecode("not base32!", out _));
        Assert.False(Base32.TryDecode("", out _));
    }
}
