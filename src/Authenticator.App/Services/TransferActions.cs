using System.Security.Cryptography;
using Authenticator.App.Dialogs;
using Authenticator.Core.Backup;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Microsoft.UI.Xaml;
using Microsoft.Windows.Storage.Pickers;

namespace Authenticator.App.Services;

/// <summary>Import review, encrypted backup/restore and QR transfer. Shared by the main and settings pages.</summary>
internal static class TransferActions
{
    /// <summary>Shows the review dialog for parsed accounts and saves the ones the user keeps. Returns how many were added.</summary>
    public static async Task<int> ReviewAndAddAsync(XamlRoot root, IReadOnlyList<Account> accounts, IReadOnlyList<string> warnings)
    {
        var selected = await ImportReviewDialog.ShowAsync(root, accounts, warnings);
        if (selected.Count == 0)
            return 0;
        await AccountStore.AddAsync(selected);
        return selected.Count;
    }

    public static async Task ExportBackupAsync(XamlRoot root)
    {
        var settings = App.Current!.Settings;
        var accounts = App.Current.Vault.Accounts;
        if (accounts.Count == 0)
        {
            await DialogService.ShowMessageAsync(root, "Nothing to back up", "Add some accounts first.");
            return;
        }
        if (!await DialogService.ReauthenticateAsync(root, "Enter your app password to export a backup."))
            return;

        var password = await PasswordDialog.CreateAsync(root, "Protect your backup",
            "Choose a password for the backup file. You'll need it to restore — it can't be recovered if you forget it.", "Choose location…");
        if (password == null)
            return;

        var picker = new FileSavePicker(App.Current.Window.AppWindow.Id)
        {
            SuggestedFileName = $"Authenticator backup {DateTime.Now:yyyy-MM-dd}",
            DefaultFileExtension = BackupFile.Extension,
        };
        picker.FileTypeChoices.Add("Authenticator backup", [BackupFile.Extension]);
        var result = await picker.PickSaveFileAsync();
        if (result == null)
            return;

        var snapshot = accounts.Select(a => a.Clone()).ToList();
        try
        {
            var bytes = await Task.Run(() => BackupFile.Create(snapshot, password, DateTimeOffset.UtcNow));
            await File.WriteAllBytesAsync(result.Path, bytes);
        }
        catch (IOException ex)
        {
            await DialogService.ShowMessageAsync(root, "Couldn't save the backup", ex.Message);
            return;
        }
        finally
        {
            snapshot.ForEach(a => a.Wipe());
        }

        settings.LastBackupAt = DateTimeOffset.UtcNow;
        settings.Save();
        await DialogService.ShowMessageAsync(root, "Backup saved", $"{snapshot.Count} account{(snapshot.Count == 1 ? "" : "s")} saved to {Path.GetFileName(result.Path)}. Keep it somewhere safe, like a USB drive or cloud storage.");
    }

    public static bool IsImportablePath(string path) =>
        string.Equals(Path.GetExtension(path), BackupFile.Extension, StringComparison.OrdinalIgnoreCase) || FirstOrderVault.IsVaultPath(path);

    /// <summary>Picks an Authenticator backup or an Authenticator App · 2FA vault. Returns the number of accounts added.</summary>
    public static async Task<int> ImportBackupAsync(XamlRoot root)
    {
        var picker = new FileOpenPicker(App.Current!.Window.AppWindow.Id);
        picker.FileTypeFilter.Add(BackupFile.Extension);
        picker.FileTypeFilter.Add(FirstOrderVault.Extension);
        var result = await picker.PickSingleFileAsync();
        if (result == null)
            return 0;
        return await ImportFileAsync(root, result.Path);
    }

    /// <summary>Imports a <c>.authbackup</c> or <c>.vault</c> file. Returns the number of accounts added.</summary>
    public static async Task<int> ImportFileAsync(XamlRoot root, string path)
    {
        byte[] bytes;
        try
        {
            bytes = await File.ReadAllBytesAsync(path);
        }
        catch (IOException ex)
        {
            await DialogService.ShowMessageAsync(root, "Couldn't open the file", ex.Message);
            return 0;
        }

        return FirstOrderVault.IsVaultPath(path)
            ? await ImportVaultAsync(root, bytes)
            : await ImportBackupBytesAsync(root, bytes, Path.GetFileName(path));
    }

    private static async Task<int> ImportVaultAsync(XamlRoot root, byte[] bytes)
    {
        ParsedPayload payload;
        try
        {
            payload = await Task.Run(() => FirstOrderVault.Parse(bytes));
        }
        catch (OtpFormatException ex)
        {
            await DialogService.ShowMessageAsync(root, "Couldn't import the vault", ex.Message);
            return 0;
        }
        finally
        {
            CryptographicOperations.ZeroMemory(bytes);
        }
        return await ReviewAndAddAsync(root, payload.Accounts, payload.Warnings);
    }

    private static async Task<int> ImportBackupBytesAsync(XamlRoot root, byte[] bytes, string fileName)
    {
        BackupContents? contents = null;
        var password = await PasswordDialog.EnterAsync(root, "Enter backup password", $"Enter the password for {fileName}.", "Restore", async candidate =>
        {
            try
            {
                contents = await Task.Run(() => BackupFile.Open(bytes, candidate));
                return null;
            }
            catch (CryptographicException)
            {
                return "That password is incorrect, or the file is damaged.";
            }
            catch (InvalidDataException ex)
            {
                return ex.Message;
            }
        });
        if (password == null || contents == null)
            return 0;

        return await ReviewAndAddAsync(root, contents.Accounts, []);
    }

    public static async Task ShowTransferQrAsync(XamlRoot root)
    {
        var accounts = AccountStore.Ordered().ToList();
        if (accounts.Count == 0)
        {
            await DialogService.ShowMessageAsync(root, "Nothing to transfer", "Add some accounts first.");
            return;
        }
        if (!await DialogService.ReauthenticateAsync(root, "Enter your app password to show transfer codes."))
            return;

        using (App.Current!.Window.ForceCaptureProtection())
            await ExportQrDialog.ShowAsync(root, accounts);
    }
}
