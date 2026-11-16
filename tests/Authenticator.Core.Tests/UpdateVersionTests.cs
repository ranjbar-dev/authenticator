using Authenticator.Infra.Updates;

namespace Authenticator.Core.Tests;

public class UpdateVersionTests
{
    [Theory]
    [InlineData("v1.1.42", "1.1.42.0")]
    [InlineData("1.1.42", "1.1.42.0")]
    [InlineData("V2.0", "2.0.0.0")]
    public void Parses_release_tags(string tag, string expected) =>
        Assert.Equal(Version.Parse(expected), UpdateClient.ParseTag(tag));

    [Theory]
    [InlineData("latest")]
    [InlineData("")]
    [InlineData("v1.x")]
    public void Rejects_non_version_tags(string tag) => Assert.Null(UpdateClient.ParseTag(tag));

    [Fact]
    public void Three_and_four_part_versions_compare_equal()
    {
        // The assembly reports 1.1.5.0; the tag says v1.1.5. Without normalizing, 1.1.5 < 1.1.5.0 and
        // the app would download the release it is already running, forever.
        var running = UpdateClient.Normalize(new Version(1, 1, 5, 0));
        Assert.Equal(running, UpdateClient.ParseTag("v1.1.5"));
        Assert.False(UpdateClient.ParseTag("v1.1.5") > running);
        Assert.True(UpdateClient.ParseTag("v1.1.6") > running);
        Assert.True(UpdateClient.ParseTag("v1.1.1") > UpdateClient.Normalize(new Version(1, 0, 0, 0)));
    }
}
