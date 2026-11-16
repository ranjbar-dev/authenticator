using Authenticator.Core.Crypto;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Authenticator.App.Controls;

public sealed partial class PasswordChecklist : UserControl
{
    public const int MinLength = 8;

    public PasswordChecklist()
    {
        InitializeComponent();
        LengthText.Text = $"At least {MinLength} characters";
    }

    /// <summary>True when the password is long enough and both entries match.</summary>
    public bool IsValid { get; private set; }

    public void Update(string password, string confirm)
    {
        var score = password.Length == 0 ? -1 : PasswordStrength.Score(password);
        // Very weak and Weak both light one segment; Fair 2, Strong 3, Very strong 4.
        var lit = score < 0 ? 0 : Math.Max(1, score);
        var fills = new[] { Fill0, Fill1, Fill2, Fill3 };
        for (var i = 0; i < fills.Length; i++)
            fills[i].Opacity = i < lit ? 1 : 0;
        StrengthLabel.Text = score < 0 ? "Strength" : PasswordStrength.Describe(score);

        var longEnough = password.Length >= MinLength;
        var matches = password.Length > 0 && password == confirm;
        SetMet(LengthMet, LengthUnmet, longEnough);
        SetMet(MatchMet, MatchUnmet, matches);
        IsValid = longEnough && matches;
    }

    private static void SetMet(UIElement met, UIElement unmet, bool value)
    {
        met.Visibility = value ? Visibility.Visible : Visibility.Collapsed;
        unmet.Visibility = value ? Visibility.Collapsed : Visibility.Visible;
    }
}
