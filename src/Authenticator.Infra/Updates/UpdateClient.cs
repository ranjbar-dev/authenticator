using System.Net.Http.Headers;
using System.Reflection;
using System.Security.Cryptography;
using System.Text.Json;

namespace Authenticator.Infra.Updates;

public sealed record ReleaseInfo(Version Version, string Tag, Uri ExeUrl, Uri ChecksumUrl, Uri PageUrl);

/// <summary>Reads releases from GitHub. The repository is public, so no token is needed.</summary>
public static class UpdateClient
{
    public const string Repository = "ranjbar-dev/authenticator";
    public const string ExeAssetName = "Authenticator.exe";
    public const string ChecksumAssetName = "Authenticator.exe.sha256";

    public static readonly Uri ReleasesPage = new($"https://github.com/{Repository}/releases");

    private static readonly HttpClient Http = CreateClient();

    /// <summary>The running app's version, normalized to four parts (CI sets it to 1.1.&lt;build&gt;).</summary>
    public static Version CurrentVersion { get; } =
        Normalize(Assembly.GetEntryAssembly()?.GetName().Version ?? new Version(0, 0));

    /// <summary>The latest published release, or null if there is none or it lacks the expected assets.</summary>
    public static async Task<ReleaseInfo?> GetLatestAsync(CancellationToken cancellationToken = default)
    {
        using var response = await Http.GetAsync($"https://api.github.com/repos/{Repository}/releases/latest", cancellationToken);
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound)
            return null;
        response.EnsureSuccessStatusCode();

        await using var stream = await response.Content.ReadAsStreamAsync(cancellationToken);
        using var json = await JsonDocument.ParseAsync(stream, cancellationToken: cancellationToken);
        var root = json.RootElement;

        var tag = root.GetProperty("tag_name").GetString() ?? "";
        if (root.TryGetProperty("draft", out var draft) && draft.GetBoolean() ||
            root.TryGetProperty("prerelease", out var pre) && pre.GetBoolean() ||
            ParseTag(tag) is not { } version)
            return null;

        Uri? exe = null, checksum = null;
        foreach (var asset in root.GetProperty("assets").EnumerateArray())
        {
            var name = asset.GetProperty("name").GetString();
            var url = asset.GetProperty("browser_download_url").GetString();
            if (url == null)
                continue;
            if (name == ExeAssetName) exe = new Uri(url);
            else if (name == ChecksumAssetName) checksum = new Uri(url);
        }
        if (exe == null || checksum == null)
            return null;

        var page = root.TryGetProperty("html_url", out var html) && html.GetString() is { } pageUrl ? new Uri(pageUrl) : ReleasesPage;
        return new ReleaseInfo(version, tag, exe, checksum, page);
    }

    /// <summary>
    /// Downloads the release exe to <paramref name="destination"/>, verifying its SHA-256 against the
    /// published checksum file. Nothing is left at <paramref name="destination"/> unless it verified.
    /// </summary>
    public static async Task DownloadAsync(ReleaseInfo release, string destination, CancellationToken cancellationToken = default)
    {
        var expected = await GetExpectedHashAsync(release, cancellationToken);
        var partial = destination + ".partial";
        try
        {
            using (var response = await Http.GetAsync(release.ExeUrl, HttpCompletionOption.ResponseHeadersRead, cancellationToken))
            {
                response.EnsureSuccessStatusCode();
                await using var source = await response.Content.ReadAsStreamAsync(cancellationToken);
                await using var target = new FileStream(partial, FileMode.Create, FileAccess.Write, FileShare.None);
                await source.CopyToAsync(target, cancellationToken);
            }

            string actual;
            await using (var file = File.OpenRead(partial))
                actual = Convert.ToHexStringLower(await SHA256.HashDataAsync(file, cancellationToken));
            if (!string.Equals(actual, expected, StringComparison.OrdinalIgnoreCase))
                throw new InvalidDataException("The downloaded update didn't match its published checksum, so it was discarded.");

            File.Move(partial, destination, overwrite: true);
        }
        finally
        {
            if (File.Exists(partial))
                File.Delete(partial);
        }
    }

    /// <summary>"v1.1.42" → 1.1.42.0. Returns null for tags that aren't versions.</summary>
    public static Version? ParseTag(string tag) =>
        Version.TryParse(tag.TrimStart('v', 'V'), out var version) ? Normalize(version) : null;

    /// <summary>Fills missing parts with 0 so 1.1.5 and 1.1.5.0 compare as equal.</summary>
    public static Version Normalize(Version v) =>
        new(Math.Max(0, v.Major), Math.Max(0, v.Minor), Math.Max(0, v.Build), Math.Max(0, v.Revision));

    private static async Task<string> GetExpectedHashAsync(ReleaseInfo release, CancellationToken cancellationToken)
    {
        // Format written by CI: "<64 hex chars>  Authenticator.exe"
        var text = await Http.GetStringAsync(release.ChecksumUrl, cancellationToken);
        var hash = text.Trim().Split(' ', '\t', '\r', '\n')[0];
        if (hash.Length != 64 || !hash.All(Uri.IsHexDigit))
            throw new InvalidDataException("The release's checksum file is malformed.");
        return hash;
    }

    private static HttpClient CreateClient()
    {
        var client = new HttpClient { Timeout = TimeSpan.FromMinutes(5) };
        client.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue("Authenticator-Updater", CurrentVersion.ToString()));
        client.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));
        return client;
    }
}
