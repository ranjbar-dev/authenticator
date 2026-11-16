using Authenticator.Infra.Security;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Navigation;
using Windows.System;

namespace Authenticator.App.Views;

public sealed partial class SetupPage : Page
{
    private bool _helloAvailable;
    private bool _busy;

    public SetupPage()
    {
        InitializeComponent();
    }

    protected override async void OnNavigatedTo(NavigationEventArgs e)
    {
        _helloAvailable = await WindowsHello.IsAvailableAsync();
        HelloButton.Visibility = _helloAvailable ? Visibility.Visible : Visibility.Collapsed;
        if (!_helloAvailable)
        {
            // Nothing to choose between: show the password form straight away.
            PasswordOption.Visibility = Visibility.Collapsed;
            NoHelloText.Visibility = Visibility.Visible;
            ShowPasswordForm(true);
        }
    }

    /// <summary>Content column: 380 wide at most, 32 px gutters, always centered.</summary>
    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs e) =>
        Column.Width = Math.Max(240, Math.Min(380, e.NewSize.Width - 64));

    private void ShowPasswordForm(bool show)
    {
        PasswordPanel.Visibility = show ? Visibility.Visible : Visibility.Collapsed;
        PasswordChevron.Glyph = show ? ((char)0xE70E).ToString() : ((char)0xE70D).ToString();
        ErrorBar.IsOpen = false;
        if (show)
            NewPassword.Focus(FocusState.Programmatic);
    }

    private void OnUsePasswordClick(object sender, RoutedEventArgs e) =>
        ShowPasswordForm(PasswordPanel.Visibility != Visibility.Visible);

    private async void OnSetUpHelloClick(object sender, RoutedEventArgs e)
    {
        HelloChevron.Visibility = Visibility.Collapsed;
        HelloBusy.IsActive = true;
        await RunAsync(() => App.Current!.Vault.CreateWithHelloAsync());
        HelloBusy.IsActive = false;
        HelloChevron.Visibility = Visibility.Visible;
    }

    private async void OnCreateWithPasswordClick(object sender, RoutedEventArgs e)
    {
        if (!Checklist.IsValid)
            return;
        var password = NewPassword.Password;
        CreateBusy.Visibility = Visibility.Visible;
        CreateBusy.IsActive = true;
        await RunAsync(() => App.Current!.Vault.CreateWithPasswordAsync(password));
        CreateBusy.IsActive = false;
        CreateBusy.Visibility = Visibility.Collapsed;
    }

    private void OnConfirmKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == VirtualKey.Enter)
            OnCreateWithPasswordClick(sender, e);
    }

    private void OnPasswordChanged(object sender, RoutedEventArgs e)
    {
        Checklist.Update(NewPassword.Password, ConfirmPassword.Password);
        CreateButton.IsEnabled = Checklist.IsValid && !_busy;
    }

    private async Task RunAsync(Func<Task> create)
    {
        if (_busy)
            return;
        _busy = true;
        HelloButton.IsEnabled = PasswordOption.IsEnabled = CreateButton.IsEnabled = false;
        ErrorBar.IsOpen = false;
        try
        {
            await create();
            App.Current!.Window.OnUnlocked();
        }
        catch (OperationCanceledException)
        {
            // User dismissed Windows Hello.
        }
        catch (Exception ex) when (ex is WindowsHelloException or IOException or UnauthorizedAccessException)
        {
            ErrorBar.Title = "Setup didn't finish";
            ErrorBar.Message = ex.Message;
            ErrorBar.IsOpen = true;
        }
        finally
        {
            _busy = false;
            HelloButton.IsEnabled = PasswordOption.IsEnabled = true;
            CreateButton.IsEnabled = Checklist.IsValid;
        }
    }
}
