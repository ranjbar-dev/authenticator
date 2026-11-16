using System.Runtime.InteropServices;

namespace Authenticator.Infra.Imaging;

/// <summary>GDI capture of the whole virtual desktop (all monitors) in physical pixels.</summary>
public static partial class ScreenCapture
{
    public sealed record Capture(PixelImage Image, int Left, int Top);

    public static Capture CaptureVirtualScreen()
    {
        int left = GetSystemMetrics(SM_XVIRTUALSCREEN), top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        int width = GetSystemMetrics(SM_CXVIRTUALSCREEN), height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        var screenDc = GetDC(0);
        var memoryDc = CreateCompatibleDC(screenDc);
        var info = new BITMAPINFOHEADER
        {
            biSize = (uint)Marshal.SizeOf<BITMAPINFOHEADER>(),
            biWidth = width,
            biHeight = -height, // top-down
            biPlanes = 1,
            biBitCount = 32,
        };
        var bitmap = CreateDIBSection(screenDc, ref info, 0, out var bits, 0, 0);
        try
        {
            if (bitmap == 0)
                throw new InvalidOperationException("Could not capture the screen.");
            var previous = SelectObject(memoryDc, bitmap);
            BitBlt(memoryDc, 0, 0, width, height, screenDc, left, top, SRCCOPY | CAPTUREBLT);
            SelectObject(memoryDc, previous);

            var pixels = new byte[width * height * 4];
            Marshal.Copy(bits, pixels, 0, pixels.Length);
            for (var i = 3; i < pixels.Length; i += 4)
                pixels[i] = 255; // GDI leaves alpha at 0
            return new Capture(new PixelImage(pixels, width, height), left, top);
        }
        finally
        {
            if (bitmap != 0)
                DeleteObject(bitmap);
            DeleteDC(memoryDc);
            ReleaseDC(0, screenDc);
        }
    }

    private const int SM_XVIRTUALSCREEN = 76, SM_YVIRTUALSCREEN = 77, SM_CXVIRTUALSCREEN = 78, SM_CYVIRTUALSCREEN = 79;
    private const uint SRCCOPY = 0x00CC0020, CAPTUREBLT = 0x40000000;

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAPINFOHEADER
    {
        public uint biSize;
        public int biWidth, biHeight;
        public ushort biPlanes, biBitCount;
        public uint biCompression, biSizeImage;
        public int biXPelsPerMeter, biYPelsPerMeter;
        public uint biClrUsed, biClrImportant;
    }

    [LibraryImport("user32.dll")] private static partial int GetSystemMetrics(int index);
    [LibraryImport("user32.dll")] private static partial nint GetDC(nint hwnd);
    [LibraryImport("user32.dll")] private static partial int ReleaseDC(nint hwnd, nint hdc);
    [LibraryImport("gdi32.dll")] private static partial nint CreateCompatibleDC(nint hdc);
    [LibraryImport("gdi32.dll")] private static partial nint CreateDIBSection(nint hdc, ref BITMAPINFOHEADER info, uint usage, out nint bits, nint section, uint offset);
    [LibraryImport("gdi32.dll")] private static partial nint SelectObject(nint hdc, nint obj);
    [LibraryImport("gdi32.dll")] [return: MarshalAs(UnmanagedType.Bool)] private static partial bool BitBlt(nint dst, int x, int y, int w, int h, nint src, int sx, int sy, uint rop);
    [LibraryImport("gdi32.dll")] [return: MarshalAs(UnmanagedType.Bool)] private static partial bool DeleteObject(nint obj);
    [LibraryImport("gdi32.dll")] [return: MarshalAs(UnmanagedType.Bool)] private static partial bool DeleteDC(nint hdc);
}
