using Authenticator.App.Controls;
using Authenticator.App.Services;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Authenticator.App.Dialogs;

/// <summary>Password prompts: choose a new password (with strength meter) or enter an existing one.</summary>
internal static class PasswordDialog
{
    /// <summary>Asks for a new password twice. Returns null if cancelled.</summary>
    public static async Task<string?> CreateAsync(XamlRoot root, string title, string message, string confirmText)
    {
        var password = new PasswordBox { Header = "Password" };
        var confirm = new PasswordBox { Header = "Confirm password" };
        var checklist = new PasswordChecklist();

        var dialog = DialogService.Prepare(new ContentDialog
        {
            Title = title,
            Content = new StackPanel
            {
                Spacing = 12,
                MinWidth = 320,
                Children =
                {
                    new TextBlock
                    {
                        Text = message,
                        Style = (Style)Application.Current.Resources["LeadTextStyle"],
                    },
                    password,
                    confirm,
                    checklist,
                },
            },
            PrimaryButtonText = confirmText,
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
            IsPrimaryButtonEnabled = false,
        }, root);

        void Update()
        {
            checklist.Update(password.Password, confirm.Password);
            dialog.IsPrimaryButtonEnabled = checklist.IsValid;
        }
        password.PasswordChanged += (_, _) => Update();
        confirm.PasswordChanged += (_, _) => Update();
        dialog.Opened += (_, _) => password.Focus(FocusState.Programmatic);

        return await dialog.ShowAsync() == ContentDialogResult.Primary ? password.Password : null;
    }

    /// <summary>
    /// Asks for an existing password. <paramref name="validate"/> returns an error message to keep the dialog
    /// open, or null to accept. Returns null if cancelled.
    /// </summary>
    public static async Task<string?> EnterAsync(XamlRoot root, string title, string message, string confirmText, Func<string, Task<string?>> validate)
    {
        var password = new PasswordBox { PlaceholderText = "Password" };
        var error = new TextBlock
        {
            Foreground = (Microsoft.UI.Xaml.Media.Brush)Application.Current.Resources["DangerBrush"],
            Style = (Style)Application.Current.Resources["CaptionTextBlockStyle"],
            Visibility = Visibility.Collapsed,
            TextWrapping = TextWrapping.Wrap,
        };
        var busy = new ProgressRing { IsActive = false, Width = 20, Height = 20, HorizontalAlignment = HorizontalAlignment.Left };

        var dialog = DialogService.Prepare(new ContentDialog
        {
            Title = title,
            Content = new StackPanel
            {
                Spacing = 10,
                MinWidth = 320,
                Children = { new TextBlock { Text = message, Style = (Style)Application.Current.Resources["LeadTextStyle"] }, password, error, busy },
            },
            PrimaryButtonText = confirmText,
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
        }, root);

        dialog.PrimaryButtonClick += async (_, args) =>
        {
            var deferral = args.GetDeferral();
            busy.IsActive = true;
            dialog.IsPrimaryButtonEnabled = false;
            var problem = password.Password.Length == 0 ? "Enter the password." : await validate(password.Password);
            busy.IsActive = false;
            dialog.IsPrimaryButtonEnabled = true;
            if (problem != null)
            {
                error.Text = problem;
                error.Visibility = Visibility.Visible;
                password.SelectAll();
                password.Focus(FocusState.Programmatic);
                args.Cancel = true;
            }
            deferral.Complete();
        };
        dialog.Opened += (_, _) => password.Focus(FocusState.Programmatic);

        return await dialog.ShowAsync() == ContentDialogResult.Primary ? password.Password : null;
    }
}
