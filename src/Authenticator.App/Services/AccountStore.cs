using Authenticator.Core.Models;

namespace Authenticator.App.Services;

/// <summary>Mutations of the unlocked account list. Every change is persisted immediately.</summary>
internal static class AccountStore
{
    private static List<Account> Accounts => App.Current!.Vault.Accounts;

    /// <summary>Accounts in display order: pinned first, then by <see cref="Account.SortOrder"/>.</summary>
    public static IEnumerable<Account> Ordered() =>
        Accounts.OrderByDescending(a => a.Pinned).ThenBy(a => a.SortOrder);

    public static Task AddAsync(IEnumerable<Account> accounts)
    {
        var next = Accounts.Count == 0 ? 0 : Accounts.Max(a => a.SortOrder) + 1;
        foreach (var account in accounts)
        {
            account.SortOrder = next++;
            account.CreatedAt = DateTimeOffset.UtcNow;
            Accounts.Add(account);
        }
        return SaveAsync();
    }

    public static Task RemoveAsync(Account account)
    {
        Accounts.Remove(account);
        account.Wipe();
        return SaveAsync();
    }

    /// <summary>Re-numbers accounts to match the on-screen order.</summary>
    public static Task ReorderAsync(IEnumerable<Account> displayOrder)
    {
        var order = 0;
        foreach (var account in displayOrder)
            account.SortOrder = order++;
        return SaveAsync();
    }

    public static Task SaveAsync() => App.Current!.Vault.SaveAsync();
}
