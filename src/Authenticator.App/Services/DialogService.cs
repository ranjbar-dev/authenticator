using Authenticator.App.Dialogs;
using Authenticator.Core.Vault;
using Authenticator.Infra.Security;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace Authenticator.App.Services;

internal static class DialogService
{
    public static T Prepare<T>(T dialog, XamlRoot root) where T : ContentDialog
    {
        dialog.XamlRoot = root;
        dialog.Style = (Style)Application.Current.Resources["DefaultContentDialogStyle"];
        if (root.Content is FrameworkElement content)
            dialog.RequestedTheme = content.RequestedTheme;
        return dialog;
    }

    public static async Task ShowMessageAsync(XamlRoot root, string title, string message)
    {
        var dialog = Prepare(new ContentDialog
        {
            Title = title,
            Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap },
            CloseButtonText = "OK",
            DefaultButton = ContentDialogButton.Close,
        }, root);
        await dialog.ShowAsync();
    }

    public static async Task<bool> ConfirmAsync(XamlRoot root, string title, string message, string confirmText, bool destructive = false)
    {
        var dialog = Prepare(new ContentDialog
        {
            Title = title,
            Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap },
            PrimaryButtonText = confirmText,
            CloseButtonText = "Cancel",
            DefaultButton = destructive ? ContentDialogButton.Close : ContentDialogButton.Primary,
        }, root);
        if (destructive)
        {
            dialog.PrimaryButtonStyle = new Style(typeof(Button))
            {
                BasedOn = (Style)Application.Current.Resources["DefaultButtonStyle"],
                Setters = { new Setter(Control.ForegroundProperty, Application.Current.Resources["DangerBrush"]) },
            };
        }
        return await dialog.ShowAsync() == ContentDialogResult.Primary;
    }

    /// <summary>Hides any open content dialogs (used when the app locks).</summary>
    public static void CloseAll(XamlRoot root)
    {
        foreach (var popup in VisualTreeHelper.GetOpenPopupsForXamlRoot(root))
        {
            if (popup.Child is ContentDialog dialog)
                dialog.Hide();
        }
    }

    /// <summary>Asks for Windows Hello or the app password again before a sensitive action.</summary>
    public static async Task<bool> ReauthenticateAsync(XamlRoot root, string reason)
    {
        var vault = App.Current!.Vault;
        if (vault.Protection == VaultProtection.WindowsHello)
        {
            try
            {
                await vault.VerifyAsync();
                return true;
            }
            catch (OperationCanceledException)
            {
                return false;
            }
            catch (WindowsHelloException ex)
            {
                await ShowMessageAsync(root, "Couldn't verify it's you", ex.Message);
                return false;
            }
        }

        var password = await PasswordDialog.EnterAsync(root, "Confirm it's you", reason, "Continue", async candidate =>
        {
            try
            {
                await vault.VerifyAsync(candidate);
                return null;
            }
            catch (WrongPasswordException ex)
            {
                return ex.Message;
            }
        });
        return password != null;
    }
}
