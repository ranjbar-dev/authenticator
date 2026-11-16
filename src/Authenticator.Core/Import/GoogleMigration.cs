using System.Security.Cryptography;
using Authenticator.Core.Models;

namespace Authenticator.Core.Import;

/// <summary>
/// Google Authenticator "Transfer accounts" format: <c>otpauth-migration://offline?data=BASE64(MigrationPayload)</c>.
/// <code>
/// message MigrationPayload {
///   repeated OtpParameters otp_parameters = 1; int32 version = 2;
///   int32 batch_size = 3; int32 batch_index = 4; int32 batch_id = 5; }
/// message OtpParameters {
///   bytes secret = 1; string name = 2; string issuer = 3;
///   Algorithm algorithm = 4;   // 0 unspecified, 1 SHA1, 2 SHA256, 3 SHA512, 4 MD5
///   DigitCount digits = 5;     // 0 unspecified, 1 six, 2 eight
///   OtpType type = 6;          // 0 unspecified, 1 HOTP, 2 TOTP
///   int64 counter = 7; }
/// </code>
/// </summary>
public static class GoogleMigration
{
    public const string Scheme = "otpauth-migration://";

    /// <summary>Accounts per QR code when exporting; keeps each code comfortably scannable.</summary>
    public const int ExportBatchSize = 8;

    public static bool IsMigration(string text) => text.TrimStart().StartsWith(Scheme, StringComparison.OrdinalIgnoreCase);

    public static ParsedPayload Parse(string uri)
    {
        uri = uri.Trim();
        var dataIndex = uri.IndexOf("data=", StringComparison.OrdinalIgnoreCase);
        if (!IsMigration(uri) || dataIndex < 0)
            throw new OtpFormatException("This is not a Google Authenticator export code.");

        var data = uri[(dataIndex + 5)..];
        var amp = data.IndexOf('&');
        if (amp >= 0)
            data = data[..amp];

        byte[] bytes;
        try
        {
            var b64 = Uri.UnescapeDataString(data).Replace(' ', '+').Replace('-', '+').Replace('_', '/');
            b64 = b64.PadRight(b64.Length + (4 - b64.Length % 4) % 4, '=');
            bytes = Convert.FromBase64String(b64);
        }
        catch (FormatException)
        {
            throw new OtpFormatException("The export QR code is malformed.");
        }

        return Decode(bytes);
    }

    internal static ParsedPayload Decode(ReadOnlySpan<byte> bytes)
    {
        var result = new ParsedPayload();
        var reader = new ProtoReader(bytes);
        while (!reader.End)
        {
            var (field, wire) = reader.ReadTag();
            switch (field)
            {
                case 1 when wire == 2:
                    var account = DecodeParameters(reader.ReadBytes(), out var warning);
                    if (account != null)
                        result.Accounts.Add(account);
                    if (warning != null)
                        result.Warnings.Add(warning);
                    break;
                case 3 when wire == 0: result.BatchSize = (int)reader.ReadVarint(); break;
                case 4 when wire == 0: result.BatchIndex = (int)reader.ReadVarint(); break;
                case 5 when wire == 0: result.BatchId = (int)reader.ReadVarint(); break;
                default: reader.Skip(wire); break;
            }
        }
        return result;
    }

    private static Account? DecodeParameters(ReadOnlySpan<byte> bytes, out string? warning)
    {
        warning = null;
        var reader = new ProtoReader(bytes);
        byte[] secret = [];
        string name = "", issuer = "";
        int algorithm = 0, digits = 0, type = 0;
        long counter = 0;

        while (!reader.End)
        {
            var (field, wire) = reader.ReadTag();
            switch (field)
            {
                case 1 when wire == 2: secret = reader.ReadBytes().ToArray(); break;
                case 2 when wire == 2: name = reader.ReadString(); break;
                case 3 when wire == 2: issuer = reader.ReadString(); break;
                case 4 when wire == 0: algorithm = (int)reader.ReadVarint(); break;
                case 5 when wire == 0: digits = (int)reader.ReadVarint(); break;
                case 6 when wire == 0: type = (int)reader.ReadVarint(); break;
                case 7 when wire == 0: counter = (long)reader.ReadVarint(); break;
                default: reader.Skip(wire); break;
            }
        }

        // Google stores the label as "Issuer:account" in name; strip the redundant prefix.
        var (labelIssuer, labelName) = OtpAuthUri.SplitLabel(name);
        if (labelIssuer.Length > 0 && (issuer.Length == 0 || string.Equals(labelIssuer.Trim(), issuer.Trim(), StringComparison.OrdinalIgnoreCase)))
        {
            issuer = issuer.Length == 0 ? labelIssuer : issuer;
            name = labelName;
        }

        var title = string.IsNullOrWhiteSpace(issuer) ? name : issuer;
        if (secret.Length == 0)
        {
            warning = $"Skipped \"{title}\": it has no secret key.";
            return null;
        }
        if (algorithm == 4)
        {
            warning = $"Skipped \"{title}\": MD5 codes are not supported.";
            return null;
        }

        return new Account
        {
            Secret = secret,
            Issuer = issuer.Trim(),
            Name = name.Trim(),
            Type = type == 1 ? OtpType.Hotp : OtpType.Totp,
            Algorithm = algorithm switch { 2 => OtpAlgorithm.Sha256, 3 => OtpAlgorithm.Sha512, _ => OtpAlgorithm.Sha1 },
            Digits = digits == 2 ? 8 : 6,
            Counter = counter,
        };
    }

    /// <summary>True if Google Authenticator can represent this account (6/8 digits, 30 s period).</summary>
    public static bool IsExportable(Account account) =>
        account.Digits is 6 or 8 && (account.Type == OtpType.Hotp || account.Period == Account.DefaultPeriod);

    /// <summary>Builds one or more migration URIs (one per QR code) for the given accounts.</summary>
    public static IReadOnlyList<string> Build(IReadOnlyList<Account> accounts, int batchSize = ExportBatchSize)
    {
        var batches = accounts.Chunk(batchSize).ToList();
        var batchId = RandomNumberGenerator.GetInt32(1, int.MaxValue);
        var uris = new List<string>(batches.Count);

        for (var i = 0; i < batches.Count; i++)
        {
            var payload = new ProtoWriter();
            foreach (var account in batches[i])
            {
                var label = string.IsNullOrEmpty(account.Issuer) ? account.Name : $"{account.Issuer}:{account.Name}";
                var parameters = new ProtoWriter()
                    .Bytes(1, account.Secret)
                    .String(2, label)
                    .String(3, account.Issuer)
                    .Varint(4, account.Algorithm switch { OtpAlgorithm.Sha256 => 2UL, OtpAlgorithm.Sha512 => 3UL, _ => 1UL })
                    .Varint(5, account.Digits == 8 ? 2UL : 1UL)
                    .Varint(6, account.Type == OtpType.Hotp ? 1UL : 2UL)
                    .Varint(7, account.Type == OtpType.Hotp ? (ulong)account.Counter : 0UL)
                    .ToArray();
                payload.Bytes(1, parameters);
            }
            payload.Varint(2, 1).Varint(3, (ulong)batches.Count).Varint(4, (ulong)i).Varint(5, (ulong)batchId);

            uris.Add($"{Scheme}offline?data={Uri.EscapeDataString(Convert.ToBase64String(payload.ToArray()))}");
        }
        return uris;
    }
}
