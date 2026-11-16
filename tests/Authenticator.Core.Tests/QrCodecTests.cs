using Authenticator.Infra.Imaging;

namespace Authenticator.Core.Tests;

public class QrCodecTests
{
    private const string Payload = "otpauth://totp/GitHub:larry?secret=JBSWY3DPEHPK3PXP&issuer=GitHub";

    /// <summary>Renders the QR (dark on white) into a canvas of the given size at the given position.</summary>
    private static PixelImage Render(string text, int moduleSize, int canvasWidth, int canvasHeight, int left, int top, bool keepQuietZone = true)
    {
        var modules = QrCodec.Encode(text);
        var count = modules.GetLength(0);
        var margin = keepQuietZone ? 0 : 2; // Encode adds a 2-module quiet zone
        var pixels = new byte[canvasWidth * canvasHeight * 4];
        Array.Fill(pixels, (byte)255);

        for (var my = margin; my < count - margin; my++)
        for (var mx = margin; mx < count - margin; mx++)
        {
            if (!modules[mx, my])
                continue;
            for (var dy = 0; dy < moduleSize; dy++)
            for (var dx = 0; dx < moduleSize; dx++)
            {
                var x = left + (mx - margin) * moduleSize + dx;
                var y = top + (my - margin) * moduleSize + dy;
                var i = (y * canvasWidth + x) * 4;
                pixels[i] = pixels[i + 1] = pixels[i + 2] = 0;
            }
        }
        return new PixelImage(pixels, canvasWidth, canvasHeight);
    }

    [Fact]
    public void Round_trips_a_qr_code()
    {
        var image = Render(Payload, 6, 400, 400, 40, 40);
        Assert.Equal([Payload], QrCodec.Decode(image));
    }

    [Fact]
    public void Decodes_tight_snip_without_quiet_zone()
    {
        // A user dragging right on the edge of the code cuts off the white border scanners need.
        var modules = QrCodec.Encode(Payload).GetLength(0) - 4;
        var image = Render(Payload, 4, modules * 4, modules * 4, 0, 0, keepQuietZone: false);
        Assert.Equal([Payload], QrCodec.Decode(image));
    }

    [Fact]
    public void Finds_small_code_on_large_multi_monitor_capture()
    {
        // Click-to-scan on a 6400×1440 desktop with a ~3 px-per-module code, as measured on real hardware.
        var image = Render(Payload, 3, 6400, 1440, 2510, 290);
        Assert.Equal([Payload], QrCodec.Decode(image));
    }

    [Fact]
    public void Returns_nothing_for_blank_image()
    {
        var blank = new PixelImage(Enumerable.Repeat((byte)255, 300 * 200 * 4).ToArray(), 300, 200);
        Assert.Empty(QrCodec.Decode(blank));
    }
}
