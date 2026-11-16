using System.Collections.ObjectModel;
using Authenticator.App.Services;
using Authenticator.Core.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Authenticator.App.ViewModels;

public sealed partial class MainViewModel : ObservableObject
{
    private readonly List<AccountItemViewModel> _all = [];

    public ObservableCollection<AccountItemViewModel> Pinned { get; } = [];
    public ObservableCollection<AccountItemViewModel> Others { get; } = [];

    [ObservableProperty] public partial string SearchText { get; set; } = "";
    [ObservableProperty] public partial bool HasAccounts { get; set; }
    [ObservableProperty] public partial bool HasPinned { get; set; }
    [ObservableProperty] public partial bool NoResults { get; set; }
    [ObservableProperty] public partial bool IsSearching { get; set; }

    /// <summary>At least one 30-second code is on screen, so the shared time strip is meaningful.</summary>
    [ObservableProperty] public partial bool HasStandardTotp { get; set; }

    public int Count => _all.Count;

    public bool CanReorder => !IsSearching;

    partial void OnSearchTextChanged(string value) => ApplyFilter();

    partial void OnIsSearchingChanged(bool value) => OnPropertyChanged(nameof(CanReorder));

    public void Load()
    {
        _all.Clear();
        _all.AddRange(AccountStore.Ordered().Select(a => new AccountItemViewModel(a)));
        ApplyFilter();
    }

    public void Tick(DateTimeOffset now, bool showNextCode)
    {
        foreach (var item in _all)
            item.Tick(now, showNextCode);
    }

    public async Task TogglePinAsync(AccountItemViewModel item)
    {
        item.Account.Pinned = !item.Account.Pinned;
        // The boundary between sections: pinning lands at the end of the pinned section,
        // unpinning lands at the top of the rest.
        _all.Remove(item);
        _all.Insert(_all.Count(i => i.Account.Pinned), item);
        item.Refresh();
        ApplyFilter();
        await AccountStore.ReorderAsync(_all.Select(i => i.Account));
    }

    public async Task RemoveAsync(AccountItemViewModel item)
    {
        _all.Remove(item);
        Pinned.Remove(item);
        Others.Remove(item);
        UpdateFlags();
        await AccountStore.RemoveAsync(item.Account);
    }

    /// <summary>Call after the user drag-reordered one of the lists.</summary>
    public Task PersistOrderAsync()
    {
        _all.Clear();
        _all.AddRange(Pinned);
        _all.AddRange(Others);
        return AccountStore.ReorderAsync(_all.Select(i => i.Account));
    }

    public AccountItemViewModel? Find(Account account) => _all.FirstOrDefault(i => i.Account == account);

    private void ApplyFilter()
    {
        var query = SearchText.Trim();
        IsSearching = query.Length > 0;
        var visible = IsSearching ? _all.Where(i => i.Matches(query)).ToList() : _all;

        Sync(Pinned, visible.Where(i => i.Account.Pinned).ToList());
        Sync(Others, visible.Where(i => !i.Account.Pinned).ToList());
        UpdateFlags();
    }

    private void UpdateFlags()
    {
        HasAccounts = _all.Count > 0;
        HasStandardTotp = Pinned.Concat(Others).Any(i => i.IsTotp && !i.HasCustomPeriod);
        OnPropertyChanged(nameof(Count));
        HasPinned = Pinned.Count > 0;
        NoResults = HasAccounts && Pinned.Count == 0 && Others.Count == 0;
    }

    /// <summary>Updates the collection in place so the ListView keeps scroll position and animations.</summary>
    private static void Sync(ObservableCollection<AccountItemViewModel> target, IReadOnlyList<AccountItemViewModel> desired)
    {
        for (var i = target.Count - 1; i >= 0; i--)
        {
            if (!desired.Contains(target[i]))
                target.RemoveAt(i);
        }
        for (var i = 0; i < desired.Count; i++)
        {
            if (i < target.Count && target[i] == desired[i])
                continue;
            var existing = target.IndexOf(desired[i]);
            if (existing >= 0)
                target.Move(existing, i);
            else
                target.Insert(i, desired[i]);
        }
    }
}
