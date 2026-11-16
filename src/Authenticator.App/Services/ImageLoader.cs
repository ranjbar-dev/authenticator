using System.Runtime.InteropServices.WindowsRuntime;
using Authenticator.Infra.Imaging;
using Windows.ApplicationModel.DataTransfer;
using Windows.Graphics.Imaging;
using Windows.Storage;
using Windows.Storage.Streams;

namespace Authenticator.App.Services;

/// <summary>Loads images (files, clipboard, drag &amp; drop) into memory for QR decoding. Nothing is written to disk.</summary>
internal static class ImageLoader
{
    public static readonly string[] Extensions = [".png", ".jpg", ".jpeg", ".bmp", ".gif", ".webp", ".tif", ".tiff"];

    public static bool IsImagePath(string path) => Extensions.Contains(Path.GetExtension(path), StringComparer.OrdinalIgnoreCase);

    public static async Task<PixelImage> FromFileAsync(string path)
    {
        using var stream = await FileRandomAccessStream.OpenAsync(path, FileAccessMode.Read);
        return await DecodeAsync(stream);
    }

    public static async Task<PixelImage> FromStreamReferenceAsync(RandomAccessStreamReference reference)
    {
        using var stream = await reference.OpenReadAsync();
        return await DecodeAsync(stream);
    }

    /// <summary>Clipboard content: a bitmap, a copied image file, or text (an otpauth link pasted directly).</summary>
    public static async Task<(PixelImage? Image, string? Text)> FromDataViewAsync(DataPackageView view)
    {
        if (view.Contains(StandardDataFormats.Bitmap))
            return (await FromStreamReferenceAsync(await view.GetBitmapAsync()), null);

        if (view.Contains(StandardDataFormats.StorageItems))
        {
            var items = await view.GetStorageItemsAsync();
            var file = items.OfType<StorageFile>().FirstOrDefault(f => IsImagePath(f.Path));
            if (file != null)
                return (await FromFileAsync(file.Path), null);
        }

        if (view.Contains(StandardDataFormats.Text))
            return (null, await view.GetTextAsync());

        return (null, null);
    }

    private static async Task<PixelImage> DecodeAsync(IRandomAccessStream stream)
    {
        var decoder = await BitmapDecoder.CreateAsync(stream);
        using var bitmap = await decoder.GetSoftwareBitmapAsync(BitmapPixelFormat.Bgra8, BitmapAlphaMode.Straight);
        var pixels = new byte[bitmap.PixelWidth * bitmap.PixelHeight * 4];
        bitmap.CopyToBuffer(pixels.AsBuffer());
        return new PixelImage(pixels, bitmap.PixelWidth, bitmap.PixelHeight);
    }
}
