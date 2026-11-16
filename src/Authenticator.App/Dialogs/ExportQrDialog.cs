using Authenticator.App.Services;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Authenticator.App.Dialogs;

/// <summary>
/// "Transfer to phone": pick accounts, then page through Google Authenticator-compatible QR codes.
/// Codes are only shown on screen — never saved as a file.
/// </summary>
internal static class ExportQrDialog
{
    public static async Task ShowAsync(XamlRoot root, IReadOnlyList<Account> accounts)
    {
        var checks = accounts.Select(a =>
        {
            var exportable = GoogleMigration.IsExportable(a);
            return new CheckBox
            {
                Content = exportable ? a.DisplayTitle : $"{a.DisplayTitle} — not supported by Google Authenticator",
                IsChecked = exportable,
                IsEnabled = exportable,
                Tag = a,
            };
        }).ToList();

        var pickPanel = new StackPanel { Spacing = 8 };
        pickPanel.Children.Add(new InfoBar
        {
            IsOpen = true,
            IsClosable = false,
            Severity = InfoBarSeverity.Warning,
            Title = "Keep these codes private",
            Message = "Anyone who sees or photographs them can copy your accounts. Only show them to your own phone.",
        });
        pickPanel.Children.Add(new TextBlock
        {
            Text = "On your phone, open Google Authenticator → Transfer accounts → Import accounts, then scan each code.",
            TextWrapping = TextWrapping.Wrap,
            Style = (Style)Application.Current.Resources["SecondaryTextStyle"],
            TextTrimming = TextTrimming.None,
        });
        var list = new StackPanel();
        checks.ForEach(c => list.Children.Add(c));
        pickPanel.Children.Add(new ScrollViewer { Content = list, MaxHeight = 260 });

        var qrHost = new ContentControl { HorizontalAlignment = HorizontalAlignment.Center };
        var pageText = new TextBlock { HorizontalAlignment = HorizontalAlignment.Center };
        var previous = new Button { Content = "Previous" };
        var next = new Button { Content = "Next" };
        var qrPanel = new StackPanel
        {
            Spacing = 12,
            Visibility = Visibility.Collapsed,
            Children =
            {
                qrHost,
                pageText,
                new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, HorizontalAlignment = HorizontalAlignment.Center, Children = { previous, next } },
            },
        };

        var dialog = DialogService.Prepare(new ContentDialog
        {
            Title = "Transfer to phone",
            Content = new Grid { MinWidth = 320, Children = { pickPanel, qrPanel } },
            PrimaryButtonText = "Show QR codes",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
        }, root);

        foreach (var check in checks)
        {
            check.Checked += (_, _) => UpdateEnabled();
            check.Unchecked += (_, _) => UpdateEnabled();
        }
        UpdateEnabled();

        IReadOnlyList<string> uris = [];
        var index = 0;

        void UpdateEnabled() => dialog.IsPrimaryButtonEnabled = checks.Any(c => c.IsChecked == true);

        void ShowPage()
        {
            qrHost.Content = QrDialog.QrView(uris[index]);
            pageText.Text = $"Code {index + 1} of {uris.Count}";
            previous.IsEnabled = index > 0;
            next.IsEnabled = index < uris.Count - 1;
            previous.Visibility = next.Visibility = uris.Count > 1 ? Visibility.Visible : Visibility.Collapsed;
        }

        previous.Click += (_, _) => { index--; ShowPage(); };
        next.Click += (_, _) => { index++; ShowPage(); };

        dialog.PrimaryButtonClick += (_, args) =>
        {
            args.Cancel = true; // stay open and switch to the QR view
            var selected = checks.Where(c => c.IsChecked == true).Select(c => (Account)c.Tag).ToList();
            uris = GoogleMigration.Build(selected);
            index = 0;
            pickPanel.Visibility = Visibility.Collapsed;
            qrPanel.Visibility = Visibility.Visible;
            dialog.PrimaryButtonText = "";
            dialog.CloseButtonText = "Done";
            ShowPage();
        };

        await dialog.ShowAsync();
    }
}
