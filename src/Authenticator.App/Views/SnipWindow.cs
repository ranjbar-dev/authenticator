using System.Runtime.InteropServices.WindowsRuntime;
using Authenticator.App.Native;
using Authenticator.Infra.Imaging;
using Microsoft.UI;
using Microsoft.UI.Input;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.Graphics;
using Windows.System;
using WinRT.Interop;
using Path = Microsoft.UI.Xaml.Shapes.Path;

namespace Authenticator.App.Views;

/// <summary>
/// Full-desktop overlay over a frozen screenshot. Drag a rectangle around a QR code, click to scan everything,
/// or press Esc to cancel. The screenshot only ever lives in memory.
/// </summary>
internal sealed partial class SnipWindow : Window
{
    private const double MinSelection = 8;

    private readonly ScreenCapture.Capture _capture;
    private readonly TaskCompletionSource<PixelImage?> _result = new();
    private readonly SnipSurface _surface = new();
    private readonly RectangleGeometry _hole = new();
    private readonly Rectangle _selection;
    private readonly Rectangle[] _handles;
    private readonly Border _sizeLabel;
    private readonly TextBlock _sizeText;
    private readonly Border _hint;
    private Point _start;
    private bool _dragging;
    private bool _done;

    private SnipWindow(ScreenCapture.Capture capture)
    {
        _capture = capture;

        var bitmap = new WriteableBitmap(capture.Image.Width, capture.Image.Height);
        using (var stream = bitmap.PixelBuffer.AsStream())
            stream.Write(capture.Image.Bgra);
        bitmap.Invalidate();

        var dim = new Path
        {
            // Brand-tinted dim rather than flat black.
            Fill = new SolidColorBrush(Windows.UI.Color.FromArgb(0x9C, 0x12, 0x0E, 0x1A)),
            Data = new GeometryGroup
            {
                FillRule = FillRule.EvenOdd,
                Children = { new RectangleGeometry { Rect = new Rect(0, 0, 100000, 100000) }, _hole },
            },
            IsHitTestVisible = false,
        };
        var accent = Windows.UI.Color.FromArgb(0xFF, 0xA7, 0x8B, 0xFA);
        var appFont = (FontFamily)Application.Current.Resources["AppFontFamily"];
        var codeFont = (FontFamily)Application.Current.Resources["CodeFontFamily"];

        _selection = new Rectangle
        {
            Stroke = new SolidColorBrush(accent),
            StrokeThickness = 1.5,
            Fill = new SolidColorBrush(Windows.UI.Color.FromArgb(0x14, 0xA7, 0x8B, 0xFA)),
            RadiusX = 6,
            RadiusY = 6,
            Visibility = Visibility.Collapsed,
            IsHitTestVisible = false,
        };
        _handles = Enumerable.Range(0, 4).Select(_ => new Rectangle
        {
            Width = 10,
            Height = 10,
            RadiusX = 3,
            RadiusY = 3,
            Fill = new SolidColorBrush(Colors.White),
            Stroke = new SolidColorBrush(accent),
            StrokeThickness = 1.5,
            Visibility = Visibility.Collapsed,
            IsHitTestVisible = false,
        }).ToArray();
        _sizeText = new TextBlock
        {
            FontFamily = codeFont,
            FontSize = 12,
            Foreground = new SolidColorBrush(Colors.White),
        };
        _sizeLabel = new Border
        {
            Background = new SolidColorBrush(Windows.UI.Color.FromArgb(0xF0, 0x1E, 0x1A, 0x24)),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(8, 4, 8, 4),
            Visibility = Visibility.Collapsed,
            IsHitTestVisible = false,
            Child = _sizeText,
        };
        _hint = new Border
        {
            Background = new SolidColorBrush(Windows.UI.Color.FromArgb(0xF0, 0x1E, 0x1A, 0x24)),
            BorderBrush = new SolidColorBrush(Windows.UI.Color.FromArgb(0x22, 0xFF, 0xFF, 0xFF)),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(12),
            Padding = new Thickness(18, 11, 18, 11),
            IsHitTestVisible = false,
            Child = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                Spacing = 18,
                Children =
                {
                    HintItem("Drag", "around the QR code", appFont, codeFont),
                    HintItem("Click", "to scan everything", appFont, codeFont),
                    HintItem("Esc", "to cancel", appFont, codeFont),
                },
            },
        };

        _surface.Background = new SolidColorBrush(Colors.Transparent);
        _surface.Children.Add(dim);
        _surface.Children.Add(_selection);
        foreach (var handle in _handles)
            _surface.Children.Add(handle);
        _surface.Children.Add(_sizeLabel);
        _surface.Children.Add(_hint);
        _surface.PointerPressed += OnPointerPressed;
        _surface.PointerMoved += OnPointerMoved;
        _surface.PointerReleased += OnPointerReleased;
        _surface.SizeChanged += (_, _) => PositionHint();

        var root = new Grid
        {
            Background = new SolidColorBrush(Colors.Black),
            Children = { new Image { Source = bitmap, Stretch = Stretch.Fill }, _surface },
        };
        var escape = new KeyboardAccelerator { Key = VirtualKey.Escape };
        escape.Invoked += (_, e) =>
        {
            e.Handled = true;
            Finish(null);
        };
        root.KeyboardAccelerators.Add(escape);
        Content = root;

        var presenter = OverlappedPresenter.Create();
        presenter.SetBorderAndTitleBar(false, false);
        presenter.IsResizable = false;
        presenter.IsMaximizable = false;
        presenter.IsMinimizable = false;
        presenter.IsAlwaysOnTop = true;
        AppWindow.SetPresenter(presenter);
        AppWindow.IsShownInSwitchers = false;
        AppWindow.MoveAndResize(new RectInt32(capture.Left, capture.Top, capture.Image.Width, capture.Image.Height));

        Activated += (_, e) =>
        {
            if (e.WindowActivationState == WindowActivationState.Deactivated)
                Finish(null); // user switched away
        };
        Closed += (_, _) => _result.TrySetResult(null);
    }

    /// <summary>Captures the screen, lets the user pick a region, and returns it (or null if cancelled).</summary>
    public static async Task<PixelImage?> PickAsync()
    {
        var capture = await Task.Run(ScreenCapture.CaptureVirtualScreen);
        var window = new SnipWindow(capture);
        window.Activate();
        Win32.SetForegroundWindow(WindowNative.GetWindowHandle(window));
        try
        {
            return await window._result.Task;
        }
        finally
        {
            capture.Image.Wipe();
        }
    }

    /// <summary>A keycap-style key name followed by what it does.</summary>
    private static StackPanel HintItem(string key, string action, FontFamily appFont, FontFamily codeFont) => new()
    {
        Orientation = Orientation.Horizontal,
        Spacing = 7,
        Children =
        {
            new Border
            {
                Background = new SolidColorBrush(Windows.UI.Color.FromArgb(0x26, 0xFF, 0xFF, 0xFF)),
                CornerRadius = new CornerRadius(5),
                Padding = new Thickness(7, 2, 7, 3),
                Child = new TextBlock { Text = key, FontFamily = codeFont, FontSize = 12, Foreground = new SolidColorBrush(Colors.White) },
            },
            new TextBlock
            {
                Text = action,
                FontFamily = appFont,
                FontSize = 13,
                VerticalAlignment = VerticalAlignment.Center,
                Foreground = new SolidColorBrush(Windows.UI.Color.FromArgb(0xD9, 0xFF, 0xFF, 0xFF)),
            },
        },
    };

    /// <summary>Screen pixels per DIP on this surface (the image is stretched to fill the window).</summary>
    private double PixelsPerDip => _surface.ActualWidth > 0 ? _capture.Image.Width / _surface.ActualWidth : 1;

    private void PositionHint()
    {
        // Center the hint near the top of the primary monitor, which sits at (0,0) in virtual-screen coordinates.
        var scale = PixelsPerDip;
        var primaryWidth = Win32.GetSystemMetrics(0 /* SM_CXSCREEN */) / scale;
        _hint.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
        Canvas.SetLeft(_hint, -_capture.Left / scale + (primaryWidth - _hint.DesiredSize.Width) / 2);
        Canvas.SetTop(_hint, -_capture.Top / scale + 32);
    }

    private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        var point = e.GetCurrentPoint(_surface);
        if (point.Properties.IsRightButtonPressed)
        {
            Finish(null);
            return;
        }
        _surface.CapturePointer(e.Pointer);
        _start = point.Position;
        _dragging = true;
        _hint.Visibility = Visibility.Collapsed;
        UpdateSelection(_start);
    }

    private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_dragging)
            UpdateSelection(e.GetCurrentPoint(_surface).Position);
    }

    private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (!_dragging)
            return;
        _dragging = false;
        _surface.ReleasePointerCapture(e.Pointer);

        var rect = ToRect(_start, e.GetCurrentPoint(_surface).Position);
        if (rect.Width < MinSelection && rect.Height < MinSelection)
        {
            // A click without dragging scans the whole desktop.
            Finish(_capture.Image.Crop(0, 0, _capture.Image.Width, _capture.Image.Height));
            return;
        }

        var scale = PixelsPerDip;
        Finish(_capture.Image.Crop(
            (int)Math.Floor(rect.X * scale), (int)Math.Floor(rect.Y * scale),
            (int)Math.Ceiling(rect.Width * scale), (int)Math.Ceiling(rect.Height * scale)));
    }

    private void UpdateSelection(Point current)
    {
        var rect = ToRect(_start, current);
        _hole.Rect = rect;
        Canvas.SetLeft(_selection, rect.X);
        Canvas.SetTop(_selection, rect.Y);
        _selection.Width = rect.Width;
        _selection.Height = rect.Height;
        _selection.Visibility = Visibility.Visible;

        var corners = new[] { (rect.Left, rect.Top), (rect.Right, rect.Top), (rect.Left, rect.Bottom), (rect.Right, rect.Bottom) };
        for (var i = 0; i < _handles.Length; i++)
        {
            Canvas.SetLeft(_handles[i], corners[i].Item1 - 5);
            Canvas.SetTop(_handles[i], corners[i].Item2 - 5);
            _handles[i].Visibility = Visibility.Visible;
        }

        // Live size in real screen pixels, tucked under the bottom-right corner.
        var scale = PixelsPerDip;
        _sizeText.Text = $"{Math.Round(rect.Width * scale)} × {Math.Round(rect.Height * scale)}";
        _sizeLabel.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
        Canvas.SetLeft(_sizeLabel, Math.Max(0, rect.Right - _sizeLabel.DesiredSize.Width));
        Canvas.SetTop(_sizeLabel, rect.Bottom + 10);
        _sizeLabel.Visibility = Visibility.Visible;
    }

    private static Rect ToRect(Point a, Point b) =>
        new(Math.Min(a.X, b.X), Math.Min(a.Y, b.Y), Math.Abs(a.X - b.X), Math.Abs(a.Y - b.Y));

    private void Finish(PixelImage? image)
    {
        if (_done)
            return;
        _done = true;
        _result.TrySetResult(image);
        Close();
    }

    /// <summary>Canvas with a crosshair cursor (ProtectedCursor is only settable from a subclass).</summary>
    private sealed partial class SnipSurface : Canvas
    {
        public SnipSurface() => ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Cross);
    }
}
