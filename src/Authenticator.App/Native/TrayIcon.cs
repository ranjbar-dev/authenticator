using System.Runtime.InteropServices;

namespace Authenticator.App.Native;

internal enum TrayCommand
{
    Open = 1,
    Scan,
    Lock,
    Quit,
}

/// <summary>Notification-area icon with a right-click menu. Messages arrive through the owning window's <see cref="WindowMessageHook"/>.</summary>
internal sealed class TrayIcon : IDisposable
{
    public const uint CallbackMessage = Win32.WM_APP + 1;
    private static readonly uint TaskbarCreated = Win32.RegisterWindowMessageW("TaskbarCreated");

    private readonly nint _hwnd;
    private nint _icon;
    private bool _added;

    public event EventHandler<TrayCommand>? CommandInvoked;

    public TrayIcon(nint hwnd)
    {
        _hwnd = hwnd;
        Add();
    }

    public void ShowNotification(string title, string message)
    {
        var data = CreateData(Win32.NIF_INFO);
        data.szInfoTitle = title;
        data.szInfo = message;
        Win32.Shell_NotifyIconW(Win32.NIM_MODIFY, ref data);
    }

    /// <summary>Handles tray-related messages. Returns true if the message was ours.</summary>
    public bool HandleMessage(uint msg, nint lParam)
    {
        if (msg == TaskbarCreated)
        {
            // Explorer restarted; re-add the icon.
            _added = false;
            Add();
            return true;
        }
        if (msg != CallbackMessage)
            return false;

        switch ((uint)(lParam & 0xFFFF))
        {
            case Win32.NIN_SELECT:
            case Win32.NIN_KEYSELECT:
            case Win32.NIN_BALLOONUSERCLICK:
                CommandInvoked?.Invoke(this, TrayCommand.Open);
                break;
            case Win32.WM_CONTEXTMENU:
                ShowMenu();
                break;
        }
        return true;
    }

    private void Add()
    {
        if (_icon == 0)
        {
            var dpi = Win32.GetDpiForWindow(_hwnd);
            var size = Win32.GetSystemMetricsForDpi(Win32.SM_CXSMICON, dpi == 0 ? 96 : dpi);
            _icon = Win32.LoadImageW(0, Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico"), Win32.IMAGE_ICON, size, size, Win32.LR_LOADFROMFILE);
        }

        var data = CreateData(Win32.NIF_MESSAGE | Win32.NIF_ICON | Win32.NIF_TIP | Win32.NIF_SHOWTIP);
        data.uCallbackMessage = CallbackMessage;
        data.hIcon = _icon;
        data.szTip = "Authenticator";
        _added = Win32.Shell_NotifyIconW(Win32.NIM_ADD, ref data);

        data.uVersion = Win32.NOTIFYICON_VERSION_4;
        Win32.Shell_NotifyIconW(Win32.NIM_SETVERSION, ref data);
    }

    private void ShowMenu()
    {
        var menu = Win32.CreatePopupMenu();
        try
        {
            Win32.AppendMenuW(menu, Win32.MF_STRING, (nuint)TrayCommand.Open, "Open Authenticator");
            Win32.AppendMenuW(menu, Win32.MF_STRING, (nuint)TrayCommand.Scan, "Scan QR code from screen");
            Win32.AppendMenuW(menu, Win32.MF_STRING, (nuint)TrayCommand.Lock, "Lock now");
            Win32.AppendMenuW(menu, Win32.MF_SEPARATOR, 0, null);
            Win32.AppendMenuW(menu, Win32.MF_STRING, (nuint)TrayCommand.Quit, "Quit");

            Win32.GetCursorPos(out var point);
            // Required so the menu closes when the user clicks elsewhere.
            Win32.SetForegroundWindow(_hwnd);
            var command = Win32.TrackPopupMenuEx(menu, Win32.TPM_RIGHTBUTTON | Win32.TPM_RETURNCMD | Win32.TPM_NONOTIFY | Win32.TPM_BOTTOMALIGN, point.X, point.Y, _hwnd, 0);
            Win32.PostMessageW(_hwnd, Win32.WM_NULL, 0, 0);

            if (command != 0)
                CommandInvoked?.Invoke(this, (TrayCommand)command);
        }
        finally
        {
            Win32.DestroyMenu(menu);
        }
    }

    private Win32.NOTIFYICONDATAW CreateData(uint flags) => new()
    {
        cbSize = (uint)Marshal.SizeOf<Win32.NOTIFYICONDATAW>(),
        hWnd = _hwnd,
        uID = 1,
        uFlags = flags,
        szTip = "",
        szInfo = "",
        szInfoTitle = "",
    };

    public void Dispose()
    {
        if (_added)
        {
            var data = CreateData(0);
            Win32.Shell_NotifyIconW(Win32.NIM_DELETE, ref data);
            _added = false;
        }
        if (_icon != 0)
        {
            Win32.DestroyIcon(_icon);
            _icon = 0;
        }
    }
}
