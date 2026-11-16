using Authenticator.App.Services;
using Authenticator.Core.Vault;
using Authenticator.Infra;
using Authenticator.Infra.Security;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Navigation;
using Windows.System;

namespace Authenticator.App.Views;

public sealed partial class UnlockPage : Page
{
    private bool _busy;

    public UnlockPage()
    {
        InitializeComponent();
    }

    private static VaultSession Vault => App.Current!.Vault;
    private static MainWindow Window => App.Current!.Window;
    private static bool IsHello => Vault.Protection == VaultProtection.WindowsHello;

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        HelloButton.Visibility = IsHello ? Visibility.Visible : Visibility.Collapsed;
        PasswordPanel.Visibility = IsHello ? Visibility.Collapsed : Visibility.Visible;
        Subtitle.Text = IsHello
            ? "Use Windows Hello to see your codes."
            : "Enter your app password to see your codes.";

        Window.Shown += OnWindowShown;
        Loaded += OnLoaded;
    }

    protected override void OnNavigatedFrom(NavigationEventArgs e)
    {
        Window.Shown -= OnWindowShown;
        Loaded -= OnLoaded;
    }

    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs e) =>
        Column.Width = Math.Max(240, Math.Min(356, e.NewSize.Width - 64));

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        if (Window.AppWindow.IsVisible)
            AutoPrompt();
    }

    private void OnWindowShown(object? sender, EventArgs e) => AutoPrompt();

    /// <summary>Prompts for Hello (or focuses the password box) as soon as the window is visible.</summary>
    private void AutoPrompt()
    {
        if (IsHello)
            _ = UnlockAsync(() => Vault.UnlockWithHelloAsync());
        else
            Password.Focus(FocusState.Programmatic);
    }

    private async void OnHelloClick(object sender, RoutedEventArgs e) =>
        await UnlockAsync(() => Vault.UnlockWithHelloAsync());

    private async void OnUnlockClick(object sender, RoutedEventArgs e)
    {
        var password = Password.Password;
        if (password.Length > 0)
            await UnlockAsync(() => Vault.UnlockWithPasswordAsync(password));
    }

    private void OnPasswordChanged(object sender, RoutedEventArgs e)
    {
        UnlockButton.IsEnabled = Password.Password.Length > 0 && !_busy;
        ErrorPanel.Visibility = Visibility.Collapsed;
    }

    private void OnPasswordKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == VirtualKey.Enter)
            OnUnlockClick(sender, e);
    }

    private void SetBusy(bool busy)
    {
        _busy = busy;
        HelloButton.IsEnabled = !busy;
        HelloIcon.Visibility = busy ? Visibility.Collapsed : Visibility.Visible;
        HelloBusy.IsActive = busy;
        HelloLabel.Text = busy ? "Waiting for Windows Hello…" : "Unlock with Windows Hello";

        PasswordBusy.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        PasswordBusy.IsActive = busy;
        UnlockLabel.Text = busy ? "Unlocking…" : "Unlock";
        UnlockButton.IsEnabled = !busy && Password.Password.Length > 0;
        Password.IsEnabled = !busy;
    }

    private async Task UnlockAsync(Func<Task> unlock)
    {
        if (_busy || Vault.IsUnlocked)
            return;
        SetBusy(true);
        ErrorPanel.Visibility = Visibility.Collapsed;
        try
        {
            await unlock();
            Window.OnUnlocked();
        }
        catch (OperationCanceledException)
        {
            // Hello prompt dismissed; leave the button for a retry.
        }
        catch (WrongPasswordException)
        {
            ShowError("That password isn't right. Check Caps Lock and try again.");
            SetBusy(false);
            Password.SelectAll();
            Password.Focus(FocusState.Programmatic);
            return;
        }
        catch (Exception ex) when (ex is WindowsHelloException or IOException or InvalidDataException or System.Text.Json.JsonException)
        {
            ShowError(ex.Message);
            ResetLink.Visibility = Visibility.Visible;
        }
        finally
        {
            if (_busy)
                SetBusy(false);
        }
    }

    private void ShowError(string message)
    {
        ErrorText.Text = message;
        ErrorPanel.Visibility = Visibility.Visible;
    }

    private async void OnResetClick(object sender, RoutedEventArgs e)
    {
        var confirmed = await DialogService.ConfirmAsync(XamlRoot, "Reset Authenticator?",
            "This permanently deletes all accounts stored on this PC. You can restore them afterwards from a backup file (Settings → Restore from file).",
            "Delete and reset", destructive: true);
        if (!confirmed)
            return;

        foreach (var path in new[] { AppPaths.Vault, AppPaths.VaultBackup })
        {
            if (File.Exists(path))
                File.Delete(path);
        }
        await WindowsHello.DeleteAsync();
        Frame.Navigate(typeof(SetupPage));
        Frame.BackStack.Clear();
    }
}
