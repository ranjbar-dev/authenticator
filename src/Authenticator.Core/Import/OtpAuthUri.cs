using System.Globalization;
using System.Text;
using Authenticator.Core.Models;
using Authenticator.Core.Otp;

namespace Authenticator.Core.Import;

/// <summary>Parses and builds <c>otpauth://</c> Key URIs (Google Authenticator "Key Uri Format").</summary>
public static class OtpAuthUri
{
    public const string Scheme = "otpauth://";

    public static bool IsOtpAuth(string text) => text.TrimStart().StartsWith(Scheme, StringComparison.OrdinalIgnoreCase);

    public static Account Parse(string uri)
    {
        uri = uri.Trim();
        if (!IsOtpAuth(uri))
            throw new OtpFormatException("This is not an otpauth:// link.");

        var rest = uri[Scheme.Length..];
        var slash = rest.IndexOf('/');
        if (slash < 0)
            throw new OtpFormatException("The otpauth:// link is missing its type.");

        var type = rest[..slash].ToLowerInvariant() switch
        {
            "totp" => OtpType.Totp,
            "hotp" => OtpType.Hotp,
            var other => throw new OtpFormatException($"Unsupported code type \"{other}\"."),
        };

        rest = rest[(slash + 1)..];
        var question = rest.IndexOf('?');
        var labelPart = question < 0 ? rest : rest[..question];
        var query = question < 0 ? "" : rest[(question + 1)..];

        var parameters = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        foreach (var pair in query.Split('&', StringSplitOptions.RemoveEmptyEntries))
        {
            var eq = pair.IndexOf('=');
            var key = eq < 0 ? pair : pair[..eq];
            var value = eq < 0 ? "" : pair[(eq + 1)..];
            parameters[Unescape(key)] = Unescape(value);
        }

        if (!parameters.TryGetValue("secret", out var secretText) || !Base32.TryDecode(secretText, out var secret))
            throw new OtpFormatException("The link has a missing or invalid secret key.");

        var (labelIssuer, name) = SplitLabel(Unescape(labelPart));
        parameters.TryGetValue("issuer", out var issuer);

        var account = new Account
        {
            Type = type,
            Secret = secret,
            Issuer = (string.IsNullOrWhiteSpace(issuer) ? labelIssuer : issuer).Trim(),
            Name = name.Trim(),
        };

        if (parameters.TryGetValue("algorithm", out var algorithm))
            account.Algorithm = ParseAlgorithm(algorithm);

        if (parameters.TryGetValue("digits", out var digits))
        {
            if (!int.TryParse(digits, NumberStyles.None, CultureInfo.InvariantCulture, out var d) || d is < OtpGenerator.MinDigits or > OtpGenerator.MaxDigits)
                throw new OtpFormatException($"Unsupported number of digits: {digits}.");
            account.Digits = d;
        }

        if (parameters.TryGetValue("period", out var period) && type == OtpType.Totp)
        {
            if (!int.TryParse(period, NumberStyles.None, CultureInfo.InvariantCulture, out var p) || p <= 0)
                throw new OtpFormatException($"Invalid period: {period}.");
            account.Period = p;
        }

        if (type == OtpType.Hotp)
        {
            if (!parameters.TryGetValue("counter", out var counter) ||
                !long.TryParse(counter, NumberStyles.None, CultureInfo.InvariantCulture, out var c))
                throw new OtpFormatException("Counter-based (HOTP) links must include a counter.");
            account.Counter = c;
        }

        return account;
    }

    public static string Build(Account account)
    {
        var type = account.Type == OtpType.Totp ? "totp" : "hotp";
        var label = string.IsNullOrEmpty(account.Issuer)
            ? Escape(account.Name)
            : $"{Escape(account.Issuer)}:{Escape(account.Name)}";

        var sb = new StringBuilder($"{Scheme}{type}/{label}?secret={Base32.Encode(account.Secret)}");
        if (!string.IsNullOrEmpty(account.Issuer))
            sb.Append("&issuer=").Append(Escape(account.Issuer));
        sb.Append("&algorithm=").Append(account.Algorithm.ToString().ToUpperInvariant());
        sb.Append("&digits=").Append(account.Digits.ToString(CultureInfo.InvariantCulture));
        if (account.Type == OtpType.Totp)
            sb.Append("&period=").Append(account.Period.ToString(CultureInfo.InvariantCulture));
        else
            sb.Append("&counter=").Append(account.Counter.ToString(CultureInfo.InvariantCulture));
        return sb.ToString();
    }

    internal static OtpAlgorithm ParseAlgorithm(string value) => value.ToUpperInvariant().Replace("-", "") switch
    {
        "SHA1" => OtpAlgorithm.Sha1,
        "SHA256" => OtpAlgorithm.Sha256,
        "SHA512" => OtpAlgorithm.Sha512,
        _ => throw new OtpFormatException($"Unsupported algorithm \"{value}\"."),
    };

    /// <summary>Splits "Issuer:account" (colon may be URL-encoded) into its parts.</summary>
    internal static (string Issuer, string Name) SplitLabel(string label)
    {
        var colon = label.IndexOf(':');
        return colon < 0 ? ("", label) : (label[..colon], label[(colon + 1)..]);
    }

    private static string Unescape(string value) => Uri.UnescapeDataString(value.Replace('+', ' '));

    private static string Escape(string value) => Uri.EscapeDataString(value);
}
