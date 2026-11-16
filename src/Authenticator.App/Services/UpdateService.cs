using Authenticator.Infra.Settings;
using Authenticator.Infra.Updates;
using Microsoft.UI.Dispatching;

namespace Authenticator.App.Services;

public enum UpdateState
{
    Unsupported,
    Idle,
    Checking,
    UpToDate,
    Downloading,
    Ready,
    Failed,
}

/// <summary>
/// Background self-update from GitHub releases: check shortly after start and every 6 hours, download and
/// verify silently, then offer "Restart to update". If the user just quits, the update is applied on exit.
/// </summary>
internal sealed class UpdateService
{
    private static readonly TimeSpan FirstCheckDelay = TimeSpan.FromSeconds(20);
    private static readonly TimeSpan CheckInterval = TimeSpan.FromHours(6);

    private readonly AppSettings _settings;
    private readonly DispatcherQueueTimer _timer;
    private bool _busy;

    public UpdateService(AppSettings settings, DispatcherQueue dispatcher)
    {
        _settings = settings;
        _timer = dispatcher.CreateTimer();
        _timer.Tick += async (_, _) =>
        {
            _timer.Interval = CheckInterval;
            if (_settings.AutoUpdate)
                await CheckAsync();
        };
        State = UpdateInstaller.IsSupported ? UpdateState.Idle : UpdateState.Unsupported;
    }

    public UpdateState State { get; private set; }
    public Version CurrentVersion => UpdateClient.CurrentVersion;
    public ReleaseInfo? Latest { get; private set; }
    public string? Error { get; private set; }

    /// <summary>Version that will run after a restart.</summary>
    public string? ReadyVersion => State == UpdateState.Ready ? _settings.PendingUpdateVersion : null;

    /// <summary>Raised on the UI thread whenever <see cref="State"/> changes.</summary>
    public event EventHandler? Changed;

    public void Start()
    {
        if (State == UpdateState.Unsupported)
            return;

        // A verified download from an earlier session that is still newer than us is ready to go.
        if (UpdateInstaller.HasPending &&
            UpdateClient.ParseTag(_settings.PendingUpdateVersion ?? "") is { } pending && pending > CurrentVersion)
        {
            SetState(UpdateState.Ready);
        }
        else
        {
            UpdateInstaller.DiscardPending();
            ForgetPending();
        }

        _timer.Interval = FirstCheckDelay;
        _timer.Start();
    }

    public async Task CheckAsync()
    {
        if (State == UpdateState.Unsupported || _busy)
            return;
        _busy = true;
        Error = null;
        SetState(UpdateState.Checking);
        try
        {
            Latest = await UpdateClient.GetLatestAsync();
            if (Latest == null || Latest.Version <= CurrentVersion)
            {
                SetState(UpdateState.UpToDate);
                return;
            }
            if (UpdateInstaller.HasPending && UpdateClient.ParseTag(_settings.PendingUpdateVersion ?? "") == Latest.Version)
            {
                SetState(UpdateState.Ready);
                return;
            }

            SetState(UpdateState.Downloading);
            await UpdateClient.DownloadAsync(Latest, UpdateInstaller.PendingPath!);
            _settings.PendingUpdateVersion = Latest.Version.ToString(3);
            _settings.Save();
            SetState(UpdateState.Ready);
        }
        catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException or IOException
                                       or UnauthorizedAccessException or InvalidDataException or System.Text.Json.JsonException
                                       or KeyNotFoundException or InvalidOperationException)
        {
            Error = ex is HttpRequestException or TaskCanceledException
                ? "Couldn't reach GitHub. Check your internet connection."
                : ex.Message;
            SetState(UpdateState.Failed);
        }
        finally
        {
            _busy = false;
        }
    }

    /// <summary>Swaps in the new exe, starts it, and quits this instance.</summary>
    public void RestartToUpdate()
    {
        if (State != UpdateState.Ready)
            return;
        var app = App.Current!;
        app.Vault.Flush();
        try
        {
            if (!UpdateInstaller.ApplyPending())
                return;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            Error = $"Couldn't install the update: {ex.Message}";
            SetState(UpdateState.Failed);
            return;
        }
        ForgetPending();
        UpdateInstaller.LaunchReplacement();
        app.Window.Quit();
    }

    /// <summary>Called on exit: if an update is waiting, put it in place for the next launch.</summary>
    public void ApplyOnExit()
    {
        if (State != UpdateState.Ready)
            return;
        try
        {
            if (UpdateInstaller.ApplyPending())
                ForgetPending();
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            // Leave it pending; we'll try again next time.
        }
    }

    private void ForgetPending()
    {
        if (_settings.PendingUpdateVersion == null)
            return;
        _settings.PendingUpdateVersion = null;
        _settings.Save();
    }

    private void SetState(UpdateState state)
    {
        State = state;
        Changed?.Invoke(this, EventArgs.Empty);
    }
}
