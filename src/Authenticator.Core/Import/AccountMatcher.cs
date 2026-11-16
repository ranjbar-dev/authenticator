using Authenticator.Core.Models;

namespace Authenticator.Core.Import;

public static class AccountMatcher
{
    /// <summary>Same secret, or same issuer + name (case-insensitive).</summary>
    public static bool IsDuplicate(Account candidate, Account existing) =>
        candidate.Secret.AsSpan().SequenceEqual(existing.Secret) ||
        (string.Equals(candidate.Issuer.Trim(), existing.Issuer.Trim(), StringComparison.OrdinalIgnoreCase) &&
         string.Equals(candidate.Name.Trim(), existing.Name.Trim(), StringComparison.OrdinalIgnoreCase));

    public static bool IsDuplicateOfAny(Account candidate, IEnumerable<Account> existing) =>
        existing.Any(e => IsDuplicate(candidate, e));
}
