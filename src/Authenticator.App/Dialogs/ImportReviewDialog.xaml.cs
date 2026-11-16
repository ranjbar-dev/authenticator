using System.ComponentModel;
using Authenticator.App.Services;
using Authenticator.App.ViewModels;
using Authenticator.Core.Import;
using Authenticator.Core.Models;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Authenticator.App.Dialogs;

/// <summary>Lets the user check, rename and deselect accounts found in a QR code or backup before saving.</summary>
public sealed partial class ImportReviewDialog : ContentDialog
{
    private readonly List<ImportCandidate> _candidates;

    private ImportReviewDialog(IReadOnlyList<Account> accounts, IReadOnlyList<string> warnings)
    {
        InitializeComponent();

        var existing = App.Current!.Vault.Accounts;
        _candidates = accounts.Select(a => new ImportCandidate(a, AccountMatcher.IsDuplicateOfAny(a, existing))).ToList();
        foreach (var candidate in _candidates)
            candidate.PropertyChanged += OnCandidateChanged;
        CandidateList.ItemsSource = _candidates;

        var duplicates = _candidates.Count(c => c.IsDuplicate);
        Title = accounts.Count == 1 ? "Add this account?" : $"Found {accounts.Count} accounts";
        SummaryText.Text = duplicates == 0
            ? "Check the names, and untick anything you don't want to add."
            : $"{duplicates} of these {(duplicates == 1 ? "is" : "are")} already in Authenticator and won't be added unless you tick {(duplicates == 1 ? "it" : "them")}.";

        WarningsBar.Message = string.Join("\n", warnings);
        WarningsBar.IsOpen = warnings.Count > 0;
        UpdatePrimaryButton();
    }

    public static async Task<IReadOnlyList<Account>> ShowAsync(XamlRoot root, IReadOnlyList<Account> accounts, IReadOnlyList<string> warnings)
    {
        var dialog = DialogService.Prepare(new ImportReviewDialog(accounts, warnings), root);
        if (await dialog.ShowAsync() != ContentDialogResult.Primary)
            return [];

        return dialog._candidates.Where(c => c.Include).Select(c =>
        {
            c.Account.Issuer = c.Issuer.Trim();
            c.Account.Name = c.Name.Trim();
            return c.Account;
        }).ToList();
    }

    private void OnCandidateChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(ImportCandidate.Include))
            UpdatePrimaryButton();
    }

    private void UpdatePrimaryButton()
    {
        var count = _candidates.Count(c => c.Include);
        PrimaryButtonText = count switch
        {
            0 => "Import",
            1 => "Import 1 account",
            _ => $"Import {count} accounts",
        };
        IsPrimaryButtonEnabled = count > 0;
    }
}
