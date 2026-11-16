namespace Authenticator.App.Native;

/// <summary>Subclasses a WinUI window's HWND so we can see raw Win32 messages (hotkeys, tray, session lock, sizing).</summary>
internal sealed class WindowMessageHook : IDisposable
{
    /// <summary>Return true if handled; <paramref name="result"/> is then returned from the window procedure.</summary>
    public delegate bool MessageHandler(uint msg, nuint wParam, nint lParam, out nint result);

    private readonly nint _hwnd;
    private readonly Win32.SubclassProc _proc; // keep the delegate alive while subclassed
    private readonly MessageHandler _handler;

    public WindowMessageHook(nint hwnd, MessageHandler handler)
    {
        _hwnd = hwnd;
        _handler = handler;
        _proc = WndProc;
        Win32.SetWindowSubclass(hwnd, _proc, 1, 0);
    }

    private nint WndProc(nint hwnd, uint msg, nuint wParam, nint lParam, nuint id, nuint refData) =>
        _handler(msg, wParam, lParam, out var result) ? result : Win32.DefSubclassProc(hwnd, msg, wParam, lParam);

    public void Dispose() => Win32.RemoveWindowSubclass(_hwnd, _proc, 1);
}
