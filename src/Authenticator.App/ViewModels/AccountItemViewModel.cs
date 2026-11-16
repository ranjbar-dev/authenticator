using Authenticator.Core.Models;
using Authenticator.Core.Otp;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Authenticator.App.ViewModels;

public sealed partial class AccountItemViewModel : ObservableObject
{
    private const int ExpiringSeconds = 5;
    private static readonly TimeSpan HotpVisibleFor = TimeSpan.FromSeconds(30);

    private long _lastStep = -1;
    private DateTimeOffset _hotpHideAt;

    public AccountItemViewModel(Account account)
    {
        Account = account;
        Refresh();
    }

    public Account Account { get; }

    public bool IsTotp => Account.Type == OtpType.Totp;
    public bool IsHotp => Account.Type == OtpType.Hotp;
    public int Period => Account.Period;

    /// <summary>Non-30 s codes get their own small timer; 30 s codes share the page's time strip.</summary>
    public bool HasCustomPeriod => IsTotp && Account.Period != Account.DefaultPeriod;

    [ObservableProperty] public partial string Title { get; set; } = "";
    [ObservableProperty] public partial string Subtitle { get; set; } = "";
    [ObservableProperty] public partial string Initial { get; set; } = "";
    [ObservableProperty] public partial bool IsPinned { get; set; }
    [ObservableProperty] public partial string Code { get; set; } = "";
    [ObservableProperty] public partial string NextCode { get; set; } = "";
    [ObservableProperty] public partial bool ShowNextCode { get; set; }
    [ObservableProperty] public partial double Remaining { get; set; }
    [ObservableProperty] public partial string RemainingText { get; set; } = "";
    [ObservableProperty] public partial bool IsExpiring { get; set; }
    [ObservableProperty] public partial bool IsHotpRevealed { get; set; }
    [ObservableProperty] public partial string AccessibleName { get; set; } = "";
    [ObservableProperty] public partial bool IsHovered { get; set; }
    [ObservableProperty] public partial bool JustCopied { get; set; }

    /// <summary>The single line under the code: copied, next code, HOTP hide timer, or a hover hint.</summary>
    [ObservableProperty] public partial string Caption { get; set; } = "";

    partial void OnIsHoveredChanged(bool value) => UpdateCaption();

    partial void OnJustCopiedChanged(bool value) => UpdateCaption();

    private void UpdateCaption()
    {
        Caption = JustCopied ? "Copied"
            : IsHotp && IsHotpRevealed ? $"Hides in {Math.Max(0, (int)Math.Ceiling((_hotpHideAt - DateTimeOffset.UtcNow).TotalSeconds))} s"
            : ShowNextCode ? $"Next {NextCode}"
            : HasCustomPeriod ? $"{RemainingText} s left"
            : IsHovered && HasVisibleCode ? "Click to copy"
            : IsHovered && IsHotp ? "Click for a code"
            : "";
    }

    /// <summary>True when there is a code on screen that can be copied.</summary>
    public bool HasVisibleCode => IsTotp || IsHotpRevealed;

    /// <summary>Re-reads display fields after the account was edited.</summary>
    public void Refresh()
    {
        Title = string.IsNullOrWhiteSpace(Account.Issuer) ? Account.Name : Account.Issuer;
        Subtitle = string.IsNullOrWhiteSpace(Account.Issuer) ? "" : Account.Name;
        var first = Title.FirstOrDefault(char.IsLetterOrDigit);
        Initial = first == default ? "?" : char.ToUpperInvariant(first).ToString();
        IsPinned = Account.Pinned;
        _lastStep = -1;
        if (IsHotp && !IsHotpRevealed)
            Code = new string('•', Account.Digits / 2) + " " + new string('•', Account.Digits - Account.Digits / 2);
        UpdateAccessibleName();
    }

    public void Tick(DateTimeOffset now, bool showNextCodeSetting)
    {
        if (IsHotp)
        {
            if (IsHotpRevealed && now >= _hotpHideAt)
            {
                IsHotpRevealed = false;
                OnPropertyChanged(nameof(HasVisibleCode));
                Refresh();
            }
            UpdateCaption();
            return;
        }

        var remaining = OtpGenerator.SecondsRemaining(now, Account.Period);
        Remaining = remaining;
        var secondsLeft = (int)Math.Ceiling(remaining);
        RemainingText = secondsLeft.ToString();
        IsExpiring = secondsLeft <= ExpiringSeconds;
        ShowNextCode = showNextCodeSetting && IsExpiring;

        var step = OtpGenerator.TimeStep(now, Account.Period);
        if (step != _lastStep)
        {
            _lastStep = step;
            Code = OtpGenerator.Format(OtpGenerator.Totp(Account.Secret, now, Account.Period, Account.Digits, Account.Algorithm));
            NextCode = OtpGenerator.Format(OtpGenerator.Hotp(Account.Secret, step + 1, Account.Digits, Account.Algorithm));
            UpdateAccessibleName();
        }
        UpdateCaption();
    }

    /// <summary>
    /// HOTP: shows the code for the stored counter and advances the counter. The caller must persist the
    /// vault <em>before</em> displaying, so a crash can't cause the same counter to be reused.
    /// </summary>
    public string AdvanceHotp()
    {
        var code = OtpGenerator.Hotp(Account.Secret, Account.Counter, Account.Digits, Account.Algorithm);
        Account.Counter++;
        return code;
    }

    public void RevealHotp(string code, DateTimeOffset now)
    {
        Code = OtpGenerator.Format(code);
        IsHotpRevealed = true;
        _hotpHideAt = now + HotpVisibleFor;
        OnPropertyChanged(nameof(HasVisibleCode));
        UpdateAccessibleName();
    }

    /// <summary>ListViewItem uses this as its accessible name.</summary>
    public override string ToString() => AccessibleName;

    /// <summary>The digits currently on screen, without spaces.</summary>
    public string RawCode => Code.Replace(" ", "");

    public bool Matches(string query) =>
        Account.Issuer.Contains(query, StringComparison.CurrentCultureIgnoreCase) ||
        Account.Name.Contains(query, StringComparison.CurrentCultureIgnoreCase);

    private void UpdateAccessibleName()
    {
        var who = string.IsNullOrEmpty(Subtitle) ? Title : $"{Title}, {Subtitle}";
        // Digits separated by spaces so Narrator reads them one at a time.
        AccessibleName = HasVisibleCode
            ? $"{who}, code {string.Join(' ', RawCode.ToCharArray())}"
            : $"{who}, counter-based code hidden. Press Enter to get a code.";
    }
}
