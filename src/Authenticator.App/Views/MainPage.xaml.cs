using Authenticator.App.Dialogs;
using Authenticator.App.Services;
using Authenticator.App.ViewModels;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Authenticator.Infra.Imaging;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Navigation;
using Microsoft.Windows.Storage.Pickers;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Windows.System;
using DispatcherQueueTimer = Microsoft.UI.Dispatching.DispatcherQueueTimer;

namespace Authenticator.App.Views;

public sealed partial class MainPage : Page
{
    private const double TwoColumnWidth = 720;

    private readonly DispatcherQueueTimer _ticker;
    private readonly DispatcherQueueTimer _toastTimer;
    private readonly DispatcherQueueTimer _copiedTimer;
    private AccountItemViewModel? _contextItem;
    private AccountItemViewModel? _copiedItem;

    public MainViewModel ViewModel { get; } = new();

    public MainPage()
    {
        InitializeComponent();

        _ticker = DispatcherQueue.CreateTimer();
        _ticker.Interval = TimeSpan.FromMilliseconds(200);
        _ticker.Tick += (_, _) => Tick();

        _toastTimer = DispatcherQueue.CreateTimer();
        _toastTimer.Interval = TimeSpan.FromSeconds(1.6);
        _toastTimer.IsRepeating = false;
        _toastTimer.Tick += (_, _) => Toast.Opacity = 0;

        _copiedTimer = DispatcherQueue.CreateTimer();
        _copiedTimer.Interval = TimeSpan.FromSeconds(1.6);
        _copiedTimer.IsRepeating = false;
        _copiedTimer.Tick += (_, _) => ClearCopied();

        var settingsShortcut = new KeyboardAccelerator { Key = (VirtualKey)188, Modifiers = VirtualKeyModifiers.Control };
        settingsShortcut.Invoked += (_, e) =>
        {
            e.Handled = true;
            App.Current!.Window.NavigateToSettings();
        };
        KeyboardAccelerators.Add(settingsShortcut);
    }

    private static MainWindow Window => App.Current!.Window;

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        ViewModel.Load();
        Tick();
        _ticker.Start();
        var hotkey = App.Current!.Settings.ScanHotkey.ToString();
        ScanMenuItem.KeyboardAcceleratorTextOverride = hotkey;
        HotkeyRun.Text = hotkey;
        UpdateBackupReminder();
        if (Window.ClockWarning is { } warning)
            ShowClockWarning(warning);

        App.Current!.Updates.Changed += OnUpdateChanged;
        OnUpdateChanged(null, EventArgs.Empty);
    }

    protected override void OnNavigatedFrom(NavigationEventArgs e)
    {
        _ticker.Stop();
        App.Current!.Updates.Changed -= OnUpdateChanged;
    }

    private bool _updateBarDismissed;

    private void OnUpdateChanged(object? sender, EventArgs e)
    {
        var updates = App.Current!.Updates;
        if (updates.ReadyVersion is { } version && !_updateBarDismissed)
        {
            UpdateBar.Message = $"Version {version} is downloaded. It installs when you restart, or the next time you quit.";
            UpdateBar.IsOpen = true;
        }
        else if (updates.State != Services.UpdateState.Ready)
        {
            UpdateBar.IsOpen = false;
        }
    }

    private void OnRestartToUpdateClick(object sender, RoutedEventArgs e) => App.Current!.Updates.RestartToUpdate();

    private void Tick()
    {
        var now = DateTimeOffset.UtcNow;
        ViewModel.Tick(now, App.Current!.Settings.ShowNextCode);

        // One shared countdown for every 30-second code, drawn as a strip (transform-only, so it's cheap).
        var remaining = Core.Otp.OtpGenerator.SecondsRemaining(now, Account.DefaultPeriod);
        var fraction = (float)(remaining / Account.DefaultPeriod);
        var expiring = remaining <= 5;
        StripFill.Scale = StripWarning.Scale = new System.Numerics.Vector3(fraction, 1, 1);
        StripWarning.Visibility = expiring ? Visibility.Visible : Visibility.Collapsed;

        var count = ViewModel.Count;
        StatusText.Text = count == 0 ? "No accounts yet"
            : !ViewModel.HasStandardTotp ? Plural(count, "account")
            : $"{Plural(count, "account")}  ·  new codes in {(int)Math.Ceiling(remaining)} s";
    }

    private static string Plural(int n, string word) => n == 1 ? $"1 {word}" : $"{n} {word}s";

    // ---------------------------------------------------------------- x:Bind helpers

    public static Visibility Not(bool value) => value ? Visibility.Collapsed : Visibility.Visible;

    public static Visibility VisibleIfText(string value) => string.IsNullOrEmpty(value) ? Visibility.Collapsed : Visibility.Visible;

    public static string PinLabel(bool pinned) => pinned ? "Unpin" : "Pin to top";

    public static double HoverOpacity(bool hovered) => hovered ? 1 : 0;

    /// <summary>
    /// Which code layer is visible: 0 ink (rest), 1 accent (hover), 2 warning (about to change),
    /// 3 muted (counter-based code still hidden). Layering avoids resolving theme brushes in code.
    /// </summary>
    public static Visibility CodeLayer(bool hovered, bool expiring, bool visible, int layer)
    {
        var active = !visible ? 3 : expiring ? 2 : hovered ? 1 : 0;
        return active == layer ? Visibility.Visible : Visibility.Collapsed;
    }

    // ---------------------------------------------------------------- Row feel: hover, press, copied

    private static readonly System.Numerics.Vector3 Pressed = new(0.985f, 0.985f, 1);

    private void OnRowLoaded(object sender, RoutedEventArgs e)
    {
        var row = (FrameworkElement)sender;
        row.ScaleTransition = new Vector3Transition { Duration = TimeSpan.FromMilliseconds(120) };
        row.CenterPoint = new System.Numerics.Vector3((float)row.ActualWidth / 2, (float)row.ActualHeight / 2, 0);
        row.SizeChanged += (_, args) =>
            row.CenterPoint = new System.Numerics.Vector3((float)args.NewSize.Width / 2, (float)args.NewSize.Height / 2, 0);
    }

    private void OnRowPointerEntered(object sender, PointerRoutedEventArgs e)
    {
        if (((FrameworkElement)sender).DataContext is AccountItemViewModel item)
            item.IsHovered = true;
    }

    private void OnRowPointerExited(object sender, PointerRoutedEventArgs e)
    {
        var row = (FrameworkElement)sender;
        row.Scale = System.Numerics.Vector3.One;
        if (row.DataContext is AccountItemViewModel item)
            item.IsHovered = false;
    }

    private void OnRowPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (e.GetCurrentPoint((UIElement)sender).Properties.IsLeftButtonPressed)
            ((UIElement)sender).Scale = Pressed;
    }

    private void OnRowPointerReleased(object sender, PointerRoutedEventArgs e) =>
        ((UIElement)sender).Scale = System.Numerics.Vector3.One;

    private void ClearCopied()
    {
        if (_copiedItem != null)
            _copiedItem.JustCopied = false;
        _copiedItem = null;
    }

    // ---------------------------------------------------------------- Notices

    public void ShowClockWarning(string message)
    {
        ClockBar.Message = message;
        ClockBar.IsOpen = true;
        Window.ClockWarning = null;
    }

    private void ShowNotice(InfoBarSeverity severity, string title, string message)
    {
        NoticeBar.Severity = severity;
        NoticeBar.Title = title;
        NoticeBar.Message = message;
        NoticeBar.IsOpen = true;
    }

    private void ShowToast(string text)
    {
        ToastText.Text = text;
        Toast.Opacity = 1;
        // The toast itself is hidden from assistive tech; announce it instead.
        FrameworkElementAutomationPeer.FromElement(this)?.RaiseNotificationEvent(
            AutomationNotificationKind.ActionCompleted, AutomationNotificationProcessing.ImportantMostRecent, text, "toast");
        _toastTimer.Stop();
        _toastTimer.Start();
    }

    private void UpdateBackupReminder()
    {
        var settings = App.Current!.Settings;
        var now = DateTimeOffset.UtcNow;
        var stale = settings.LastBackupAt is not { } last || now - last > TimeSpan.FromDays(90);
        var snoozed = settings.BackupReminderDismissedAt is { } dismissed && now - dismissed < TimeSpan.FromDays(30);
        BackupBar.IsOpen = ViewModel.HasAccounts && stale && !snoozed;
    }

    private void OnUpdateBarClosed(InfoBar sender, object args) => _updateBarDismissed = true;

    private void OnBackupReminderClosed(InfoBar sender, object args)
    {
        var settings = App.Current!.Settings;
        settings.BackupReminderDismissedAt = DateTimeOffset.UtcNow;
        settings.Save();
    }

    private async void OnBackupNowClick(object sender, RoutedEventArgs e)
    {
        await TransferActions.ExportBackupAsync(XamlRoot);
        UpdateBackupReminder();
    }

    // ---------------------------------------------------------------- Codes

    private async void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is AccountItemViewModel item)
            await CopyOrRevealAsync(item);
    }

    private async Task CopyOrRevealAsync(AccountItemViewModel item)
    {
        if (item.HasVisibleCode)
            Copy(item);
        else
            await RevealHotpAsync(item);
    }

    private async Task RevealHotpAsync(AccountItemViewModel item)
    {
        var code = item.AdvanceHotp();
        try
        {
            // Persist the advanced counter before showing the code so it's never reused.
            await AccountStore.SaveAsync();
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            item.Account.Counter--;
            ShowNotice(InfoBarSeverity.Error, "Couldn't save", ex.Message);
            return;
        }
        item.RevealHotp(code, DateTimeOffset.UtcNow);
        Copy(item);
    }

    private void Copy(AccountItemViewModel item)
    {
        ClipboardService.CopyCode(item.RawCode);

        // Confirm inline on the row itself, and announce it for screen readers.
        ClearCopied();
        _copiedItem = item;
        item.JustCopied = true;
        _copiedTimer.Stop();
        _copiedTimer.Start();
        FrameworkElementAutomationPeer.FromElement(this)?.RaiseNotificationEvent(
            AutomationNotificationKind.ActionCompleted, AutomationNotificationProcessing.ImportantMostRecent,
            $"{item.Title} code copied", "copied");
    }

    // ---------------------------------------------------------------- Context menu

    private void OnContextFlyoutOpening(object? sender, object e) =>
        _contextItem = (sender as FlyoutBase)?.Target?.DataContext as AccountItemViewModel;

    private async void OnCopyMenuClick(object sender, RoutedEventArgs e)
    {
        if (_contextItem != null)
            await CopyOrRevealAsync(_contextItem);
    }

    private async void OnPinMenuClick(object sender, RoutedEventArgs e)
    {
        if (_contextItem != null)
            await ViewModel.TogglePinAsync(_contextItem);
    }

    private async void OnEditMenuClick(object sender, RoutedEventArgs e)
    {
        if (_contextItem is not { } item)
            return;
        var edited = await ManualEntryDialog.ShowAsync(XamlRoot, item.Account);
        if (edited == null)
            return;

        var account = item.Account;
        account.Issuer = edited.Issuer;
        account.Name = edited.Name;
        account.Algorithm = edited.Algorithm;
        account.Digits = edited.Digits;
        account.Period = edited.Period;
        account.Counter = edited.Counter;
        edited.Wipe();
        item.Refresh();
        Tick();
        await AccountStore.SaveAsync();
    }

    private async void OnShowQrMenuClick(object sender, RoutedEventArgs e)
    {
        if (_contextItem is not { } item)
            return;
        if (!await DialogService.ReauthenticateAsync(XamlRoot, "Enter your app password to show this account's secret."))
            return;
        using (Window.ForceCaptureProtection())
            await QrDialog.ShowAsync(XamlRoot, item.Account);
    }

    private async void OnDeleteMenuClick(object sender, RoutedEventArgs e)
    {
        if (_contextItem is not { } item)
            return;
        var confirmed = await DialogService.ConfirmAsync(XamlRoot, $"Delete {item.Title}?",
            "You'll lose these codes and may be locked out of the account unless 2FA is turned off there first or you have a backup. This can't be undone.",
            "Delete", destructive: true);
        if (confirmed)
        {
            await ViewModel.RemoveAsync(item);
            UpdateBackupReminder();
        }
    }

    private async void OnDragItemsCompleted(ListViewBase sender, DragItemsCompletedEventArgs args) =>
        await ViewModel.PersistOrderAsync();

    private void OnWrapGridSizeChanged(object sender, SizeChangedEventArgs e)
    {
        if (sender is ItemsWrapGrid panel)
        {
            var columns = e.NewSize.Width >= TwoColumnWidth ? 2 : 1;
            panel.ItemWidth = Math.Floor(e.NewSize.Width / columns);
        }
    }

    // ---------------------------------------------------------------- Toolbar

    private async void OnScanClick(object sender, RoutedEventArgs e)
    {
        BatchBar.IsOpen = false;
        await Window.StartScanAsync();
    }

    private async void OnPasteClick(object sender, RoutedEventArgs e) => await ImportFromClipboardAsync();

    private async void OnOpenImageClick(object sender, RoutedEventArgs e) => await PickImageAsync();

    private async void OnAddManualClick(object sender, RoutedEventArgs e) => await AddManualAsync();

    private void OnLockClick(object sender, RoutedEventArgs e) => Window.Lock();

    private void OnSettingsClick(object sender, RoutedEventArgs e) => Window.NavigateToSettings();

    // ---------------------------------------------------------------- Keyboard

    private static bool IsTextInputFocused(XamlRoot root) =>
        FocusManager.GetFocusedElement(root) is TextBox or PasswordBox or AutoSuggestBox;

    private async void OnPasteAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        // Images are always imported; text pastes normally when a text box has focus.
        // Handled must be decided synchronously, before any await.
        var clipboard = Clipboard.GetContent();
        var hasImage = clipboard.Contains(StandardDataFormats.Bitmap) || clipboard.Contains(StandardDataFormats.StorageItems);
        if (IsTextInputFocused(XamlRoot) && !hasImage)
            return;
        args.Handled = true;
        await ImportFromClipboardAsync();
    }

    private async void OnAddManualAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        await AddManualAsync();
    }

    private async void OnOpenImageAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        await PickImageAsync();
    }

    private void OnSearchAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        SearchBox.Focus(FocusState.Keyboard);
    }

    private void OnLockAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        Window.Lock();
    }

    /// <summary>Type-to-search: letters typed anywhere on the page go to the search box.</summary>
    private void OnCharacterReceived(UIElement sender, CharacterReceivedRoutedEventArgs args)
    {
        if (IsTextInputFocused(XamlRoot) || !char.IsLetterOrDigit(args.Character))
            return;
        args.Handled = true;
        SearchBox.Text += args.Character;
        SearchBox.Focus(FocusState.Keyboard);
        if (FindDescendant<TextBox>(SearchBox) is { } input)
            input.SelectionStart = input.Text.Length;
    }

    private static T? FindDescendant<T>(DependencyObject root) where T : DependencyObject
    {
        for (var i = 0; i < Microsoft.UI.Xaml.Media.VisualTreeHelper.GetChildrenCount(root); i++)
        {
            var child = Microsoft.UI.Xaml.Media.VisualTreeHelper.GetChild(root, i);
            if (child is T match)
                return match;
            if (FindDescendant<T>(child) is { } nested)
                return nested;
        }
        return null;
    }

    // ---------------------------------------------------------------- Import

    private void OnDragOver(object sender, DragEventArgs e)
    {
        if (e.DataView.Contains(StandardDataFormats.StorageItems) || e.DataView.Contains(StandardDataFormats.Bitmap))
        {
            e.AcceptedOperation = DataPackageOperation.Copy;
            e.DragUIOverride.Caption = "Import";
        }
    }

    private async void OnDrop(object sender, DragEventArgs e)
    {
        if (!e.DataView.Contains(StandardDataFormats.StorageItems) && !e.DataView.Contains(StandardDataFormats.Bitmap))
            return;
        var deferral = e.GetDeferral();
        try
        {
            if (e.DataView.Contains(StandardDataFormats.StorageItems))
            {
                var items = await e.DataView.GetStorageItemsAsync();
                var file = items.OfType<StorageFile>().FirstOrDefault(f => TransferActions.IsImportablePath(f.Path));
                if (file != null)
                {
                    await ImportFileAsync(file.Path);
                    return;
                }
            }

            var (image, _) = await ImageLoader.FromDataViewAsync(e.DataView);
            if (image == null)
                ShowNotice(InfoBarSeverity.Warning, "Can't import this file", "Drop an image of a QR code, an Authenticator backup (.authbackup) or an Authenticator App · 2FA vault (.vault).");
            else
                await ImportFromImageAsync(image);
        }
        catch (Exception ex)
        {
            ShowNotice(InfoBarSeverity.Error, "Couldn't open the image", ex.Message);
        }
        finally
        {
            deferral.Complete();
        }
    }

    private async Task ImportFileAsync(string path)
    {
        NoticeBar.IsOpen = false;
        var added = await TransferActions.ImportFileAsync(XamlRoot, path);
        ViewModel.Load();
        Tick();
        UpdateBackupReminder();
        if (added > 0)
            ShowToast(added == 1 ? "Account added" : $"{added} accounts added");
    }

    private async Task ImportFromClipboardAsync()
    {
        try
        {
            var (image, text) = await ImageLoader.FromDataViewAsync(Clipboard.GetContent());
            if (image != null)
                await ImportFromImageAsync(image);
            else if (text != null && (OtpAuthUri.IsOtpAuth(text) || GoogleMigration.IsMigration(text)))
                await ImportTextsAsync([text.Trim()]);
            else
                ShowNotice(InfoBarSeverity.Informational, "No image on the clipboard",
                    "Take a screenshot of the QR code first (Win+Shift+S), then paste it here.");
        }
        catch (Exception ex)
        {
            ShowNotice(InfoBarSeverity.Error, "Couldn't read the clipboard", ex.Message);
        }
    }

    private async Task PickImageAsync()
    {
        var picker = new FileOpenPicker(Window.AppWindow.Id);
        foreach (var extension in ImageLoader.Extensions)
            picker.FileTypeFilter.Add(extension);
        var result = await picker.PickSingleFileAsync();
        if (result == null)
            return;
        try
        {
            await ImportFromImageAsync(await ImageLoader.FromFileAsync(result.Path));
        }
        catch (Exception ex)
        {
            ShowNotice(InfoBarSeverity.Error, "Couldn't open the image", ex.Message);
        }
    }

    /// <summary>Decodes QR codes in an in-memory image and walks the user through importing them.</summary>
    public async Task ImportFromImageAsync(PixelImage image)
    {
        IReadOnlyList<string> texts;
        try
        {
            texts = await Task.Run(() => QrCodec.Decode(image));
        }
        finally
        {
            image.Wipe();
        }

        if (texts.Count == 0)
        {
            ShowNotice(InfoBarSeverity.Warning, "No QR code found",
                "Make sure the whole QR code is visible and not too small, then try again.");
            return;
        }
        await ImportTextsAsync(texts);
    }

    private async Task ImportTextsAsync(IReadOnlyList<string> texts)
    {
        ParsedPayload payload;
        try
        {
            payload = PayloadParser.Parse(texts);
        }
        catch (OtpFormatException ex)
        {
            ShowNotice(InfoBarSeverity.Warning, "Couldn't import", ex.Message);
            return;
        }

        NoticeBar.IsOpen = false;
        var added = await TransferActions.ReviewAndAddAsync(XamlRoot, payload.Accounts, payload.Warnings);
        ViewModel.Load();
        Tick();
        UpdateBackupReminder();

        if (payload.HasMoreBatches)
        {
            BatchBar.Message = $"That was QR code {payload.BatchIndex + 1} of {payload.BatchSize} from Google Authenticator. Show the next one on your phone, then scan it.";
            BatchBar.IsOpen = true;
        }
        else
        {
            BatchBar.IsOpen = false;
        }

        if (added > 0)
            ShowToast(added == 1 ? "Account added" : $"{added} accounts added");
    }

    private async Task AddManualAsync()
    {
        var account = await ManualEntryDialog.ShowAsync(XamlRoot, null);
        if (account == null)
            return;

        if (AccountMatcher.IsDuplicateOfAny(account, App.Current!.Vault.Accounts) &&
            !await DialogService.ConfirmAsync(XamlRoot, "Add a duplicate?", "You already have an account with the same name or secret key.", "Add anyway"))
            return;

        await AccountStore.AddAsync([account]);
        ViewModel.Load();
        Tick();
        UpdateBackupReminder();
        ShowToast("Account added");
    }
}
