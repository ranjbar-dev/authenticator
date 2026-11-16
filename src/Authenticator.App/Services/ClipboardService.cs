using Windows.ApplicationModel.DataTransfer;

namespace Authenticator.App.Services;

/// <summary>Copies codes without clipboard history/cloud sync, and clears them after the configured delay.</summary>
internal static class ClipboardService
{
    private static string? _lastCopied;
    private static int _generation;

    public static void CopyCode(string code)
    {
        var package = new DataPackage();
        package.SetText(code);
        Clipboard.SetContentWithOptions(package, new ClipboardContentOptions { IsAllowedInHistory = false, IsRoamable = false });
        _lastCopied = code;

        var seconds = App.Current!.Settings.ClipboardClearSeconds;
        if (seconds > 0)
            _ = ClearLaterAsync(++_generation, TimeSpan.FromSeconds(seconds));
    }

    public static void CopySecret(string text) => CopyCode(text);

    /// <summary>Clears the clipboard only if it still holds the code we put there.</summary>
    public static void ClearIfOurs()
    {
        try
        {
            if (_lastCopied == null)
                return;
            // Called during shutdown, so wait briefly rather than await.
            var content = Clipboard.GetContent();
            if (content.Contains(StandardDataFormats.Text))
            {
                var read = content.GetTextAsync().AsTask();
                if (read.Wait(TimeSpan.FromMilliseconds(500)) && read.Result == _lastCopied)
                    Clipboard.Clear();
            }
        }
        catch (Exception)
        {
            // Clipboard can be locked by another process; nothing more we can do.
        }
        _lastCopied = null;
    }

    private static async Task ClearLaterAsync(int generation, TimeSpan delay)
    {
        await Task.Delay(delay);
        if (generation != _generation || _lastCopied == null)
            return;
        try
        {
            var content = Clipboard.GetContent();
            if (content.Contains(StandardDataFormats.Text) && await content.GetTextAsync() == _lastCopied)
                Clipboard.Clear();
        }
        catch (Exception)
        {
            // See ClearIfOurs.
        }
        _lastCopied = null;
    }
}
