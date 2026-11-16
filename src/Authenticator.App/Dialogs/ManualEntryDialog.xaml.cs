using Authenticator.App.Services;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Authenticator.Core.Otp;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Authenticator.App.Dialogs;

/// <summary>Add an account by setup key (or pasted otpauth:// link), or edit an existing account's details.</summary>
public sealed partial class ManualEntryDialog : ContentDialog
{
    private readonly bool _editing;
    private readonly DispatcherQueueTimer _previewTimer;
    private byte[]? _secret;
    private bool _filling;

    private Account? Result { get; set; }

    private ManualEntryDialog(Account? existing)
    {
        InitializeComponent();

        _previewTimer = DispatcherQueue.CreateTimer();
        _previewTimer.Interval = TimeSpan.FromSeconds(1);
        _previewTimer.Tick += (_, _) => UpdatePreview();
        Opened += (_, _) => _previewTimer.Start();
        Closed += (_, _) => _previewTimer.Stop();

        if (existing != null)
        {
            _editing = true;
            Title = "Edit account";
            PrimaryButtonText = "Save";
            SecretPanel.Visibility = Visibility.Collapsed;
            TypeButtons.IsEnabled = false;
            CounterWarning.Visibility = Visibility.Visible;
            _secret = (byte[])existing.Secret.Clone();
            Fill(existing);
        }
        Validate();
    }

    public static async Task<Account?> ShowAsync(XamlRoot root, Account? existing)
    {
        var dialog = DialogService.Prepare(new ManualEntryDialog(existing), root);
        return await dialog.ShowAsync() == ContentDialogResult.Primary ? dialog.Result : null;
    }

    private bool IsHotp => TypeButtons.SelectedIndex == 1;

    private void Fill(Account account)
    {
        _filling = true;
        IssuerBox.Text = account.Issuer;
        NameBox.Text = account.Name;
        TypeButtons.SelectedIndex = account.Type == OtpType.Hotp ? 1 : 0;
        AlgorithmBox.SelectedIndex = (int)account.Algorithm;
        DigitsBox.SelectedIndex = Math.Clamp(account.Digits - 6, 0, 2);
        PeriodBox.Value = account.Period;
        CounterBox.Value = account.Counter;
        if (account.Algorithm != OtpAlgorithm.Sha1 || account.Digits != 6 || account.Period != 30 || account.Type == OtpType.Hotp)
            AdvancedExpander.IsExpanded = true;
        _filling = false;
    }

    private void OnSecretChanged(object sender, TextChangedEventArgs e)
    {
        var text = SecretBox.Text.Trim();
        SecretError.Visibility = Visibility.Collapsed;

        if (OtpAuthUri.IsOtpAuth(text))
        {
            try
            {
                var account = OtpAuthUri.Parse(text);
                _secret = account.Secret;
                Fill(account);
                SecretBox.Text = Base32.Group(Base32.Encode(account.Secret));
            }
            catch (OtpFormatException ex)
            {
                _secret = null;
                ShowSecretError(ex.Message);
            }
        }
        else if (text.Length == 0)
        {
            _secret = null;
        }
        else if (Base32.TryDecode(text, out var secret))
        {
            _secret = secret;
        }
        else
        {
            _secret = null;
            ShowSecretError("Setup keys only contain the letters A–Z and the digits 2–7.");
        }
        Validate();
    }

    private void ShowSecretError(string message)
    {
        SecretError.Text = message;
        SecretError.Visibility = Visibility.Visible;
    }

    private void OnFieldChanged(object sender, object e)
    {
        if (!_filling)
            Validate();
    }

    private void OnNumberChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (!_filling)
            Validate();
    }

    private void Validate()
    {
        PeriodBox.Visibility = IsHotp ? Visibility.Collapsed : Visibility.Visible;
        CounterBox.Visibility = IsHotp ? Visibility.Visible : Visibility.Collapsed;
        CounterWarning.Visibility = _editing && IsHotp ? Visibility.Visible : Visibility.Collapsed;
        PreviewLabel.Text = IsHotp ? "First code" : "Current code";

        var hasLabel = IssuerBox.Text.Trim().Length > 0 || NameBox.Text.Trim().Length > 0;
        var numbersValid = IsHotp ? !double.IsNaN(CounterBox.Value) : !double.IsNaN(PeriodBox.Value) && PeriodBox.Value >= 1;
        IsPrimaryButtonEnabled = _secret != null && hasLabel && numbersValid;
        UpdatePreview();
    }

    private void UpdatePreview()
    {
        var account = Build();
        PreviewCode.Text = account == null
            ? "––– –––"
            : OtpGenerator.Format(OtpGenerator.Current(account, DateTimeOffset.UtcNow));
    }

    private Account? Build()
    {
        if (_secret == null || (IsHotp ? double.IsNaN(CounterBox.Value) : double.IsNaN(PeriodBox.Value) || PeriodBox.Value < 1))
            return null;
        return new Account
        {
            Issuer = IssuerBox.Text.Trim(),
            Name = NameBox.Text.Trim(),
            Secret = (byte[])_secret.Clone(),
            Type = IsHotp ? OtpType.Hotp : OtpType.Totp,
            Algorithm = (OtpAlgorithm)Math.Max(0, AlgorithmBox.SelectedIndex),
            Digits = 6 + Math.Max(0, DigitsBox.SelectedIndex),
            Period = IsHotp ? Account.DefaultPeriod : (int)PeriodBox.Value,
            Counter = IsHotp ? (long)CounterBox.Value : 0,
        };
    }

    private void OnPrimaryClick(ContentDialog sender, ContentDialogButtonClickEventArgs args)
    {
        Result = Build();
        if (Result == null)
            args.Cancel = true;
    }
}
