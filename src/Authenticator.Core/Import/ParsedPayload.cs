using Authenticator.Core.Models;

namespace Authenticator.Core.Import;

/// <summary>Accounts decoded from one or more QR payloads, plus Google batch info if present.</summary>
public sealed class ParsedPayload
{
    public List<Account> Accounts { get; } = [];
    public List<string> Warnings { get; } = [];

    /// <summary>Google export: total number of QR codes in this export (1 if not batched).</summary>
    public int BatchSize { get; set; } = 1;

    /// <summary>Google export: zero-based index of this QR code.</summary>
    public int BatchIndex { get; set; }

    public int BatchId { get; set; }

    public bool HasMoreBatches => BatchSize > 1 && BatchIndex + 1 < BatchSize;
}

public static class PayloadParser
{
    /// <summary>Parses decoded QR texts. Unrecognized texts are reported as warnings; throws only if nothing usable was found.</summary>
    public static ParsedPayload Parse(IEnumerable<string> texts)
    {
        var result = new ParsedPayload();
        var sawAny = false;
        string? lastError = null;

        foreach (var text in texts.Select(t => t.Trim()).Where(t => t.Length > 0).Distinct())
        {
            sawAny = true;
            try
            {
                if (GoogleMigration.IsMigration(text))
                {
                    var batch = GoogleMigration.Parse(text);
                    result.Accounts.AddRange(batch.Accounts);
                    result.Warnings.AddRange(batch.Warnings);
                    result.BatchSize = batch.BatchSize;
                    result.BatchIndex = batch.BatchIndex;
                    result.BatchId = batch.BatchId;
                }
                else if (OtpAuthUri.IsOtpAuth(text))
                {
                    result.Accounts.Add(OtpAuthUri.Parse(text));
                }
                else
                {
                    lastError = "This QR code is not an authenticator code.";
                }
            }
            catch (OtpFormatException ex)
            {
                lastError = ex.Message;
            }
        }

        if (!sawAny)
            throw new OtpFormatException("No QR code found.");
        if (result.Accounts.Count == 0)
            throw new OtpFormatException(result.Warnings.FirstOrDefault() ?? lastError ?? "No accounts found.");
        if (lastError != null)
            result.Warnings.Add(lastError);
        return result;
    }

    public static ParsedPayload Parse(string text) => Parse([text]);
}
