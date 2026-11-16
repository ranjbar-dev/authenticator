using Authenticator.App.Services;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Authenticator.Core.Otp;
using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace Authenticator.App.Dialogs;

/// <summary>Shows one account's QR code and setup key (after re-authentication).</summary>
internal static class QrDialog
{
    public static async Task ShowAsync(XamlRoot root, Account account)
    {
        var key = Base32.Group(Base32.Encode(account.Secret));
        var copy = new Button { Content = "Copy key", HorizontalAlignment = HorizontalAlignment.Center };
        copy.Click += (_, _) => ClipboardService.CopySecret(key.Replace(" ", ""));

        var content = new StackPanel
        {
            Spacing = 12,
            MinWidth = 300,
            Children =
            {
                QrView(OtpAuthUri.Build(account)),
                new TextBlock
                {
                    Text = key,
                    FontFamily = (FontFamily)Application.Current.Resources["CodeFontFamily"],
                    FontSize = 16,
                    HorizontalAlignment = HorizontalAlignment.Center,
                    TextAlignment = TextAlignment.Center,
                    TextWrapping = TextWrapping.Wrap,
                    IsTextSelectionEnabled = true,
                },
                copy,
                new InfoBar
                {
                    IsOpen = true,
                    IsClosable = false,
                    Severity = InfoBarSeverity.Warning,
                    Message = "Anyone who sees this QR code or key can generate your codes. Close it when you're done.",
                },
            },
        };

        var dialog = DialogService.Prepare(new ContentDialog
        {
            Title = account.DisplayTitle,
            Content = content,
            CloseButtonText = "Done",
            DefaultButton = ContentDialogButton.Close,
        }, root);
        await dialog.ShowAsync();
    }

    /// <summary>A QR code on a white card so it scans in dark mode too.</summary>
    public static Border QrView(string text) => new()
    {
        Background = new SolidColorBrush(Colors.White),
        CornerRadius = new CornerRadius(16),
        Padding = new Thickness(12),
        HorizontalAlignment = HorizontalAlignment.Center,
        Child = new Image
        {
            Source = QrImage.Create(text),
            Width = 240,
            Height = 240,
            Stretch = Stretch.Uniform,
        },
    };
}
