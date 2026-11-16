using ZXing;
using ZXing.Common;
using ZXing.QrCode;
using ZXing.QrCode.Internal;

namespace Authenticator.Infra.Imaging;

public static class QrCodec
{
    /// <summary>
    /// Decodes every QR code in the image. Tries the image as-is, then with an added white quiet zone
    /// (tight snips often cut it off), then upscaled for small codes. Handles inverted codes.
    /// </summary>
    public static IReadOnlyList<string> Decode(PixelImage image)
    {
        var gray = image.ToGray();
        try
        {
            foreach (var (pixels, width, height) in Variants(gray, image.Width, image.Height))
            {
                var found = TryDecode(pixels, width, height);
                if (found.Count > 0)
                    return found;
            }

            // Whole-desktop captures (e.g. 6400×1440 across three monitors): a small QR among lots of other
            // content often isn't found in one pass, so scan overlapping tiles.
            return image.Width > TileSize * 3 / 2 || image.Height > TileSize * 3 / 2
                ? DecodeTiles(gray, image.Width, image.Height)
                : [];
        }
        finally
        {
            Array.Clear(gray);
        }
    }

    /// <summary>Encodes text as a QR module matrix (true = dark), including a quiet zone.</summary>
    public static bool[,] Encode(string text)
    {
        var hints = new Dictionary<EncodeHintType, object>
        {
            [EncodeHintType.MARGIN] = 2,
            [EncodeHintType.ERROR_CORRECTION] = ErrorCorrectionLevel.M,
            [EncodeHintType.CHARACTER_SET] = "UTF-8",
        };
        var matrix = new QRCodeWriter().encode(text, BarcodeFormat.QR_CODE, 0, 0, hints);
        var modules = new bool[matrix.Width, matrix.Height];
        for (var x = 0; x < matrix.Width; x++)
            for (var y = 0; y < matrix.Height; y++)
                modules[x, y] = matrix[x, y];
        return modules;
    }

    private static List<string> TryDecode(byte[] gray, int width, int height, bool multiple = true)
    {
        var reader = new BarcodeReaderGeneric
        {
            AutoRotate = true,
            Options = new DecodingOptions
            {
                PossibleFormats = [BarcodeFormat.QR_CODE],
                TryHarder = true,
                TryInverted = true,
            },
        };
        var source = new RGBLuminanceSource(gray, width, height, RGBLuminanceSource.BitmapFormat.Gray8);

        var results = multiple ? reader.DecodeMultiple(source)?.Select(r => r.Text).ToList() ?? [] : [];
        if (results.Count == 0 && reader.Decode(source) is { } single)
            results.Add(single.Text);
        return results.Where(t => !string.IsNullOrEmpty(t)).Distinct().ToList();
    }

    private const int TileSize = 960;

    private static List<string> DecodeTiles(byte[] gray, int width, int height)
    {
        var tiles = (from y in TileOffsets(height) from x in TileOffsets(width) select (x, y)).ToList();

        // Native resolution first (fast); upscaled tiles only if that finds nothing (small codes).
        // Tiles are independent, so decode them in parallel; one code per tile is enough since
        // overlapping tiles pick up the others.
        foreach (var factor in new[] { 1, 2 })
        {
            var perTile = new List<string>[tiles.Count];
            Parallel.For(0, tiles.Count, i =>
            {
                var (x, y) = tiles[i];
                int w = Math.Min(TileSize, width - x), h = Math.Min(TileSize, height - y);
                var tile = new byte[w * h];
                for (var row = 0; row < h; row++)
                    Buffer.BlockCopy(gray, (y + row) * width + x, tile, row * w, w);

                if (factor == 1)
                {
                    perTile[i] = TryDecode(tile, w, h, multiple: false);
                }
                else
                {
                    var scaled = Scale(tile, w, h, factor);
                    perTile[i] = TryDecode(scaled.Pixels, scaled.Width, scaled.Height, multiple: false);
                    Array.Clear(scaled.Pixels);
                }
                Array.Clear(tile);
            });

            var results = perTile.SelectMany(r => r).Distinct().ToList();
            if (results.Count > 0)
                return results;
        }
        return [];
    }

    /// <summary>Tile origins with 50% overlap, so a code up to half a tile wide is always fully inside one tile.</summary>
    private static IEnumerable<int> TileOffsets(int length)
    {
        if (length <= TileSize)
        {
            yield return 0;
            yield break;
        }
        for (var offset = 0; ; offset += TileSize / 2)
        {
            if (offset + TileSize >= length)
            {
                yield return length - TileSize;
                yield break;
            }
            yield return offset;
        }
    }

    private static IEnumerable<(byte[] Pixels, int Width, int Height)> Variants(byte[] gray, int width, int height)
    {
        yield return (gray, width, height);
        if (width > TileSize * 3 / 2 || height > TileSize * 3 / 2)
            yield break;

        var padded = Pad(gray, width, height, Math.Max(16, Math.Max(width, height) / 8));
        yield return padded;

        if (Math.Min(width, height) < 400)
        {
            var factor = Math.Min(4, Math.Max(2, 600 / Math.Max(1, Math.Min(width, height))));
            yield return Scale(padded.Pixels, padded.Width, padded.Height, factor);
        }
    }

    private static (byte[] Pixels, int Width, int Height) Pad(byte[] gray, int width, int height, int border)
    {
        int w = width + border * 2, h = height + border * 2;
        var output = new byte[w * h];
        Array.Fill(output, (byte)255);
        for (var y = 0; y < height; y++)
            Buffer.BlockCopy(gray, y * width, output, (y + border) * w + border, width);
        return (output, w, h);
    }

    private static (byte[] Pixels, int Width, int Height) Scale(byte[] gray, int width, int height, int factor)
    {
        int w = width * factor, h = height * factor;
        var output = new byte[w * h];
        for (var y = 0; y < h; y++)
        {
            var src = (y / factor) * width;
            var dst = y * w;
            for (var x = 0; x < w; x++)
                output[dst + x] = gray[src + x / factor];
        }
        return (output, w, h);
    }
}
