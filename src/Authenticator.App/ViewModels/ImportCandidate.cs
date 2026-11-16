using Authenticator.Core.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Authenticator.App.ViewModels;

/// <summary>One row in the import review dialog.</summary>
public sealed partial class ImportCandidate : ObservableObject
{
    public ImportCandidate(Account account, bool isDuplicate)
    {
        Account = account;
        IsDuplicate = isDuplicate;
        Include = !isDuplicate;
        Issuer = account.Issuer;
        Name = account.Name;
    }

    public Account Account { get; }
    public bool IsDuplicate { get; }

    [ObservableProperty] public partial bool Include { get; set; }
    [ObservableProperty] public partial string Issuer { get; set; }
    [ObservableProperty] public partial string Name { get; set; }

    public string Title => string.IsNullOrWhiteSpace(Issuer) ? Name : Issuer;

    public string Initial
    {
        get
        {
            var first = Title.FirstOrDefault(char.IsLetterOrDigit);
            return first == '\0' ? "?" : char.ToUpperInvariant(first).ToString();
        }
    }

    /// <summary>ListViewItem uses this as its accessible name.</summary>
    public override string ToString() => Title;

    public string Details
    {
        get
        {
            var kind = Account.Type == OtpType.Totp ? "Time-based" : "Counter-based";
            var extras = new List<string> { kind };
            if (Account.Digits != 6) extras.Add($"{Account.Digits} digits");
            if (Account.Algorithm != OtpAlgorithm.Sha1) extras.Add(Account.Algorithm.ToString().ToUpperInvariant());
            if (Account.Type == OtpType.Totp && Account.Period != 30) extras.Add($"{Account.Period}s");
            return string.Join(" · ", extras);
        }
    }
}
