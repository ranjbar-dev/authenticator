namespace Authenticator.Infra.Imaging;

/// <summary>An in-memory 32-bit BGRA image. Never written to disk.</summary>
public sealed class PixelImage(byte[] bgra, int width, int height)
{
    public byte[] Bgra { get; } = bgra;
    public int Width { get; } = width;
    public int Height { get; } = height;

    public PixelImage Crop(int x, int y, int width, int height)
    {
        x = Math.Clamp(x, 0, Width);
        y = Math.Clamp(y, 0, Height);
        width = Math.Clamp(width, 0, Width - x);
        height = Math.Clamp(height, 0, Height - y);

        var output = new byte[width * height * 4];
        for (var row = 0; row < height; row++)
            Buffer.BlockCopy(Bgra, ((y + row) * Width + x) * 4, output, row * width * 4, width * 4);
        return new PixelImage(output, width, height);
    }

    /// <summary>Converts to 8-bit luminance, compositing transparency over white.</summary>
    public byte[] ToGray()
    {
        var gray = new byte[Width * Height];
        for (int i = 0, p = 0; i < gray.Length; i++, p += 4)
        {
            int b = Bgra[p], g = Bgra[p + 1], r = Bgra[p + 2], a = Bgra[p + 3];
            var lum = (r * 77 + g * 150 + b * 29) >> 8;
            gray[i] = (byte)((lum * a + 255 * (255 - a)) / 255);
        }
        return gray;
    }

    /// <summary>Clears every byte. Call once decoding is done.</summary>
    public void Wipe() => Array.Clear(Bgra);
}
