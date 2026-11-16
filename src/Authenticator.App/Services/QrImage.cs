using System.Runtime.InteropServices.WindowsRuntime;
using Authenticator.Infra.Imaging;
using Microsoft.UI.Xaml.Media.Imaging;

namespace Authenticator.App.Services;

internal static class QrImage
{
    /// <summary>Renders a black-on-white QR code (always light, for scanner contrast).</summary>
    public static WriteableBitmap Create(string text, int pixelsPerModule = 8)
    {
        var modules = QrCodec.Encode(text);
        int count = modules.GetLength(0), size = count * pixelsPerModule;
        var pixels = new byte[size * size * 4];

        for (var y = 0; y < size; y++)
        {
            for (var x = 0; x < size; x++)
            {
                var value = modules[x / pixelsPerModule, y / pixelsPerModule] ? (byte)0 : (byte)255;
                var i = (y * size + x) * 4;
                pixels[i] = pixels[i + 1] = pixels[i + 2] = value;
                pixels[i + 3] = 255;
            }
        }

        var bitmap = new WriteableBitmap(size, size);
        using (var stream = bitmap.PixelBuffer.AsStream())
            stream.Write(pixels);
        bitmap.Invalidate();
        return bitmap;
    }
}
