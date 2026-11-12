//! Windows-native selection. No pixels are read until pointer-up approves a
//! bounded rectangle and all selection windows have been hidden + DwmFlush'd.
use super::*;
use crate::{
    qr::{self, PixelLayout},
    ui::theme,
};
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::DwmFlush, Gdi::*},
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use windows_sys::{
    Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_FOCUS},
    core::BOOL,
};
use zeroize::{Zeroize, Zeroizing};

const CANCEL: usize = 100;
const TICK: usize = 1;
const KEY_COLOR: u32 = 0x00030201;
const INSTRUCTIONS: &str = "Drag around the QR code, then release to scan.\r\nEscape cancels. Ctrl+L locks the vault.\r\nOnly the selected region is captured, locally in memory.";
const CANCEL_LABEL: &str = "Cancel capture";
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn color(c: eframe::egui::Color32) -> u32 {
    u32::from(c.r()) | (u32::from(c.g()) << 8) | (u32::from(c.b()) << 16)
}
fn dip(n: f32, dpi: u32) -> i32 {
    (n * dpi as f32 / 96.0).round() as i32
}

struct DpiContext(DPI_AWARENESS_CONTEXT);
impl DpiContext {
    fn enter() -> Result<Self, CaptureError> {
        // Changes only this dedicated thread, never process/user OS settings.
        let old =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if old.is_null() {
            Err(CaptureError::Unavailable)
        } else {
            Ok(Self(old))
        }
    }
}
impl Drop for DpiContext {
    fn drop(&mut self) {
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Monitor {
    bounds: DesktopBounds,
    work: DesktopBounds,
}
unsafe extern "system" fn collect_monitor(h: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    unsafe {
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(h, &mut info) == 0 {
            return 0;
        }
        let bounds = |r: RECT| DesktopBounds {
            left: r.left,
            top: r.top,
            width: r.right.saturating_sub(r.left),
            height: r.bottom.saturating_sub(r.top),
        };
        let list = &mut *(data as *mut Vec<Monitor>);
        if list.len() >= 64 {
            return 0;
        }
        list.push(Monitor {
            bounds: bounds(info.rcMonitor),
            work: bounds(info.rcWork),
        });
        1
    }
}
fn monitors() -> Result<Vec<Monitor>, CaptureError> {
    let mut list: Vec<Monitor> = Vec::new();
    if unsafe {
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(collect_monitor),
            (&mut list as *mut Vec<Monitor>) as isize,
        )
    } == 0
        || list.is_empty()
    {
        return Err(CaptureError::Unavailable);
    }
    for monitor in &list {
        monitor
            .bounds
            .validate()
            .map_err(|_| CaptureError::Unavailable)?;
        monitor
            .work
            .validate()
            .map_err(|_| CaptureError::Unavailable)?;
    }
    list.sort_by_key(|m| (m.bounds.left, m.bounds.top, m.bounds.width, m.bounds.height));
    Ok(list)
}
pub(super) fn display_bounds() -> Result<Vec<DesktopBounds>, CaptureError> {
    let _dpi = DpiContext::enter()?;
    Ok(monitors()?.into_iter().map(|m| m.bounds).collect())
}

struct Object(HGDIOBJ);
impl Drop for Object {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DeleteObject(self.0);
            }
        }
    }
}
struct FontMemory(HANDLE);
impl FontMemory {
    fn load(bytes: &'static [u8]) -> Result<Self, CaptureError> {
        let mut count = 0;
        // Private process resource only; no OS font installation or disk write.
        let resource = Self(unsafe {
            AddFontMemResourceEx(
                bytes.as_ptr().cast(),
                bytes.len() as u32,
                null(),
                std::ptr::addr_of_mut!(count),
            )
        });
        if resource.0.is_null() {
            Err(CaptureError::Unavailable)
        } else {
            Ok(resource)
        }
    }
}
impl Drop for FontMemory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                RemoveFontMemResourceEx(self.0);
            }
        }
    }
}
fn native_weight(weight: theme::Weight) -> i32 {
    match weight {
        theme::Weight::Regular => 400,
        theme::Weight::Semibold => 600,
    }
}
fn native_font(role: theme::Typography, dpi: u32) -> Result<Object, CaptureError> {
    let family = match role.family {
        // These are the bundled files' Win32 family names (name-table id1).
        // Asking the regular family for weight600 can synthesize weight700.
        theme::TypeFamily::NotoSans => wide(match role.weight {
            theme::Weight::Regular => "Noto Sans",
            theme::Weight::Semibold => "Noto Sans SemiBold",
        }),
        theme::TypeFamily::Monospace => return Err(CaptureError::Unavailable),
    };
    // Explicit native-kind exception: STATIC and DrawText use GDI font cell
    // metrics, not egui's role.line_height. Native tracking remains zero. The
    // measured fit/realized face/weight contract is tested below and documented.
    let font = Object(unsafe {
        CreateFontW(
            -dip(role.size, dpi),
            0,
            0,
            0,
            native_weight(role.weight),
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            CLEARTYPE_QUALITY as u32,
            0,
            family.as_ptr(),
        )
    });
    if font.0.is_null() {
        Err(CaptureError::Unavailable)
    } else {
        Ok(font)
    }
}
struct ScreenDc(HDC);
impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(null_mut(), self.0);
        }
    }
}
struct MemoryDc(HDC);
impl Drop for MemoryDc {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.0);
        }
    }
}
struct Dib {
    bitmap: HBITMAP,
    bits: *mut u8,
    len: usize,
}
impl Drop for Dib {
    fn drop(&mut self) {
        // Flush GDI before clearing this mapped, selected-region-only allocation.
        unsafe {
            GdiFlush();
            std::slice::from_raw_parts_mut(self.bits, self.len).zeroize();
            DeleteObject(self.bitmap);
        }
    }
}
struct SelectionObject {
    dc: HDC,
    old: HGDIOBJ,
}
impl Drop for SelectionObject {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
        }
    }
}

fn capture(
    region: Region,
    original: &[Monitor],
    gate: &Gate,
) -> Result<Zeroizing<Vec<u8>>, CaptureError> {
    let len = region.byte_len()?;
    if gate.stopped() {
        return Err(CaptureError::Unavailable);
    }
    // Snapshot equality guards disconnect/move races before allocation and again
    // before BitBlt. WM_DISPLAYCHANGE independently cancels the selection loop.
    if monitors()? != original {
        return Err(CaptureError::DisplayChanged);
    }
    unsafe {
        let screen = ScreenDc(GetDC(null_mut()));
        if screen.0.is_null() {
            return Err(CaptureError::Unavailable);
        }
        let memory = MemoryDc(CreateCompatibleDC(screen.0));
        if memory.0.is_null() {
            return Err(CaptureError::Unavailable);
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: region.width as i32,
                biHeight: -(region.height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = null_mut();
        let bitmap = CreateDIBSection(memory.0, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if bitmap.is_null() || bits.is_null() {
            if !bitmap.is_null() {
                DeleteObject(bitmap);
            }
            return Err(CaptureError::Unavailable);
        }
        let dib = Dib {
            bitmap,
            bits: bits.cast(),
            len,
        };
        let old = SelectObject(memory.0, bitmap);
        if old.is_null() || old as isize == -1 {
            return Err(CaptureError::Unavailable);
        }
        let _selected = SelectionObject { dc: memory.0, old };
        if gate.stopped() {
            return Err(CaptureError::Unavailable);
        }
        if monitors()? != original {
            return Err(CaptureError::DisplayChanged);
        }
        if BitBlt(
            memory.0,
            0,
            0,
            region.width as i32,
            region.height as i32,
            screen.0,
            region.left,
            region.top,
            SRCCOPY | CAPTUREBLT,
        ) == 0
            || GdiFlush() == 0
        {
            return Err(CaptureError::Unavailable);
        }
        if gate.stopped() {
            return Err(CaptureError::Unavailable);
        }
        let mut pixels = Zeroizing::new(vec![0u8; len]);
        pixels.copy_from_slice(std::slice::from_raw_parts(dib.bits, len));
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        Ok(pixels)
    }
}

struct Overlay<'a> {
    gate: &'a Gate,
    bounds: DesktopBounds,
    monitors: Vec<Monitor>,
    owner: HWND,
    window: HWND,
    panel: HWND,
    border: HWND,
    button: HWND,
    font: Object,
    label_font: Object,
    _font_memory: [FontMemory; 2],
    start: Option<Point>,
    end: Point,
    outcome: Option<Result<Option<Region>, CaptureError>>,
    locked: bool,
}
impl Overlay<'_> {
    fn finish(&mut self, result: Result<Option<Region>, CaptureError>) {
        if self.outcome.is_some() {
            return;
        }
        self.outcome = Some(result);
        self.start = None;
        // Release before hide/destroy even on stolen capture, focus loss, native
        // lock, empty region, duplicate cancel or monitor change.
        unsafe {
            if GetCapture() == self.window {
                ReleaseCapture();
            }
            ShowWindow(self.border, SW_HIDE);
            ShowWindow(self.panel, SW_HIDE);
            ShowWindow(self.window, SW_HIDE);
            PostQuitMessage(0);
        }
    }
    fn cursor(&self) -> Option<Point> {
        let mut p = POINT::default();
        if unsafe { GetCursorPos(&mut p) } == 0 {
            None
        } else {
            Some(Point { x: p.x, y: p.y })
        }
    }
    fn event_point(&self, lp: LPARAM) -> Option<Point> {
        // Button-event coordinates belong to that event. GetCursorPos here can
        // already be at pointer-up when several real SendInput events queue in
        // one message pass, collapsing a legitimate drag into an empty region.
        // PMv2 ClientToScreen maps signed client pixels into physical desktop
        // pixels without applying the main egui window's app zoom/DPI scale.
        let mut p = POINT {
            x: (lp as u16 as i16) as i32,
            y: ((lp as u32 >> 16) as u16 as i16) as i32,
        };
        if unsafe { ClientToScreen(self.window, &mut p) } == 0 {
            None
        } else {
            Some(Point { x: p.x, y: p.y })
        }
    }
    fn tick(&mut self) {
        if self.gate.stopped() {
            self.locked = self.gate.locked();
            self.finish(Ok(None));
        } else if unsafe { IsWindow(self.owner) } == 0 {
            self.finish(Ok(None));
        } else if !monitors().is_ok_and(|m| m == self.monitors) {
            self.finish(Err(CaptureError::DisplayChanged));
        }
    }
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_NCCREATE {
            let create = &*(lp as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Overlay<'_>>;
        if pointer.is_null() {
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        // Win32 calls back synchronously during focus/release/creation. RefCell
        // rejects re-entry before creating a second mutable state reference.
        let Ok(mut borrowed) = (&*pointer).try_borrow_mut() else {
            return DefWindowProcW(hwnd, msg, wp, lp);
        };
        let state = &mut *borrowed;
        match msg {
            WM_TIMER => {
                state.tick();
                0
            }
            WM_COMMAND if wp & 0xffff == CANCEL => {
                state.finish(Ok(None));
                0
            }
            WM_CLOSE | WM_CANCELMODE => {
                state.finish(Ok(None));
                0
            }
            WM_DISPLAYCHANGE | WM_DEVICECHANGE => {
                state.finish(Err(CaptureError::DisplayChanged));
                0
            }
            WM_DPICHANGED => {
                state.finish(Err(CaptureError::DisplayChanged));
                0
            }
            WM_ACTIVATE if wp & 0xffff == WA_INACTIVE as usize && state.outcome.is_none() => {
                // Moving focus between our overlay/panel is allowed. Alt-Tab to
                // another app cancels instead of retaining hidden pointer capture.
                if lp != state.panel as isize && lp != state.window as isize {
                    state.finish(Ok(None));
                }
                0
            }
            WM_LBUTTONDOWN if hwnd == state.window && state.outcome.is_none() => {
                if let Some(p) = state.event_point(lp) {
                    state.start = Some(p);
                    state.end = p;
                    SetCapture(hwnd);
                    if GetCapture() != hwnd {
                        state.finish(Err(CaptureError::Unavailable));
                    }
                } else {
                    state.finish(Err(CaptureError::Unavailable));
                }
                0
            }
            WM_MOUSEMOVE if hwnd == state.window && state.start.is_some() => {
                if let Some(p) = state.event_point(lp) {
                    state.end = p;
                    InvalidateRect(state.border, null(), 1);
                } else {
                    state.finish(Err(CaptureError::Unavailable));
                }
                0
            }
            WM_LBUTTONUP if hwnd == state.window && state.start.is_some() => {
                let result = match (state.start, state.event_point(lp)) {
                    (Some(a), Some(b)) => Region::from_drag(a, b, state.bounds).map(Some),
                    _ => Err(CaptureError::Unavailable),
                };
                state.finish(result);
                0
            }
            WM_CAPTURECHANGED if state.start.is_some() && lp != hwnd as isize => {
                state.finish(Ok(None));
                0
            }
            WM_CTLCOLORSTATIC => {
                SetTextColor(wp as _, color(theme::TEXT));
                SetBkColor(wp as _, color(theme::SURFACE));
                SetDCBrushColor(wp as _, color(theme::SURFACE));
                GetStockObject(DC_BRUSH) as isize
            }
            WM_DRAWITEM => {
                let item = &*(lp as *const DRAWITEMSTRUCT);
                SetDCBrushColor(item.hDC, color(theme::BLUE));
                FillRect(item.hDC, &item.rcItem, GetStockObject(DC_BRUSH));
                SetTextColor(item.hDC, color(theme::ON_BLUE));
                SetBkMode(item.hDC, TRANSPARENT as i32);
                let old = SelectObject(item.hDC, state.label_font.0);
                let mut rect = item.rcItem;
                let text = wide(CANCEL_LABEL);
                DrawTextW(
                    item.hDC,
                    text.as_ptr(),
                    -1,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                SelectObject(item.hDC, old);
                if item.itemState & ODS_FOCUS != 0 {
                    InflateRect(&mut rect, -4, -4);
                    // Opaque dark-on-blue focus exceeds the graphics target;
                    // DrawFocusRect's XOR colors depend on the existing fill.
                    SetDCBrushColor(item.hDC, color(theme::ON_BLUE));
                    for _ in 0..theme::FOCUS_STROKE as i32 {
                        FrameRect(item.hDC, &rect, GetStockObject(DC_BRUSH));
                        InflateRect(&mut rect, -1, -1);
                    }
                }
                1
            }
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut paint);
                let mut client = RECT::default();
                GetClientRect(hwnd, &mut client);
                SetDCBrushColor(
                    dc,
                    if hwnd == state.border {
                        KEY_COLOR
                    } else if hwnd == state.panel {
                        color(theme::SURFACE)
                    } else {
                        0
                    },
                );
                FillRect(dc, &client, GetStockObject(DC_BRUSH));
                if hwnd == state.border
                    && let Some(a) = state.start
                {
                    // Paint geometry only, no background screenshot/cached image.
                    let b = state.end;
                    let mut r = RECT {
                        left: a.x.min(b.x) - state.bounds.left,
                        top: a.y.min(b.y) - state.bounds.top,
                        right: a.x.max(b.x) - state.bounds.left,
                        bottom: a.y.max(b.y) - state.bounds.top,
                    };
                    SetDCBrushColor(dc, color(theme::ON_BLUE));
                    FrameRect(dc, &r, GetStockObject(DC_BRUSH));
                    InflateRect(&mut r, -1, -1);
                    SetDCBrushColor(dc, color(theme::FOCUS));
                    for _ in 0..3 {
                        FrameRect(dc, &r, GetStockObject(DC_BRUSH));
                        InflateRect(&mut r, -1, -1);
                    }
                }
                EndPaint(hwnd, &paint);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

struct Windows {
    handles: Vec<HWND>,
    class: Vec<u16>,
    instance: HINSTANCE,
}
impl Drop for Windows {
    fn drop(&mut self) {
        unsafe {
            for &h in self.handles.iter().rev() {
                if IsWindow(h) != 0 {
                    KillTimer(h, TICK);
                    SetWindowLongPtrW(h, GWLP_USERDATA, 0);
                    DestroyWindow(h);
                }
            }
            UnregisterClassW(self.class.as_ptr(), self.instance);
        }
    }
}
fn select(owner: usize, gate: &Gate) -> Result<(Option<Region>, Vec<Monitor>, bool), CaptureError> {
    let bounds = desktop_bounds().map_err(|_| CaptureError::Unavailable)?;
    // WM_MOUSE* client coordinates are signed16 fields. Fail closed on an
    // exceptionally large virtual desktop rather than wrap the approved drag.
    if bounds.width > i16::MAX as i32 || bounds.height > i16::MAX as i32 {
        return Err(CaptureError::Unavailable);
    }
    let layout = monitors()?;
    if gate.stopped() {
        return Ok((None, layout, gate.locked()));
    }
    unsafe {
        if IsWindow(owner as _) == 0 {
            return Err(CaptureError::Unavailable);
        }
        let font_bytes = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
        let font_memory = FontMemory::load(font_bytes)?;
        let semibold = include_bytes!("../../../assets/fonts/NotoSans-SemiBold.ttf");
        let semibold_memory = FontMemory::load(semibold)?;
        let instance = GetModuleHandleW(null());
        let class = wide(&format!("AuthenticatorCapture{}", uuid::Uuid::new_v4()));
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_CROSS),
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err(CaptureError::Unavailable);
        }
        let storage = Box::new(RefCell::new(Overlay {
            gate,
            bounds,
            monitors: layout,
            owner: owner as _,
            window: null_mut(),
            panel: null_mut(),
            border: null_mut(),
            button: null_mut(),
            font: Object(null_mut()),
            label_font: Object(null_mut()),
            _font_memory: [font_memory, semibold_memory],
            start: None,
            end: Point { x: 0, y: 0 },
            outcome: None,
            locked: false,
        }));
        // HWNDs always die before their USERDATA pointee, including setup errors.
        let mut windows = Windows {
            handles: Vec::new(),
            class,
            instance,
        };
        let ptr = (&*storage as *const RefCell<Overlay<'_>>).cast();
        let mut state = storage.borrow_mut();
        let mut make = |ex, title: &str, style, x, y, w, h, parent| {
            let hwnd = CreateWindowExW(
                ex,
                windows.class.as_ptr(),
                wide(title).as_ptr(),
                style,
                x,
                y,
                w,
                h,
                parent,
                null_mut(),
                instance,
                ptr,
            );
            if !hwnd.is_null() {
                windows.handles.push(hwnd);
            }
            hwnd
        };
        state.window = make(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_APPWINDOW,
            "Authenticator region selection",
            WS_POPUP,
            bounds.left,
            bounds.top,
            bounds.width,
            bounds.height,
            null_mut(),
        );
        if state.window.is_null() {
            return Err(CaptureError::Unavailable);
        }
        if SetLayeredWindowAttributes(state.window, 0, theme::CAPTURE_DIM_ALPHA, LWA_ALPHA) == 0 {
            return Err(CaptureError::Unavailable);
        }
        state.border = make(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            "Selection boundary",
            WS_POPUP,
            bounds.left,
            bounds.top,
            bounds.width,
            bounds.height,
            state.window,
        );
        if state.border.is_null() {
            return Err(CaptureError::Unavailable);
        }
        if SetLayeredWindowAttributes(state.border, KEY_COLOR, 255, LWA_COLORKEY) == 0 {
            return Err(CaptureError::Unavailable);
        }
        let cursor = state.cursor().ok_or(CaptureError::Unavailable)?;
        let work = state
            .monitors
            .iter()
            .find(|m| {
                cursor.x >= m.bounds.left
                    && cursor.x < m.bounds.left + m.bounds.width
                    && cursor.y >= m.bounds.top
                    && cursor.y < m.bounds.top + m.bounds.height
            })
            .unwrap_or(&state.monitors[0])
            .work;
        state.panel = make(
            WS_EX_TOPMOST,
            "Screenshot to Scan",
            WS_POPUP | WS_CLIPCHILDREN,
            work.left,
            work.top,
            1,
            1,
            state.window,
        );
        if state.panel.is_null() {
            return Err(CaptureError::Unavailable);
        }
        let dpi = GetDpiForWindow(state.panel).max(96);
        let p = dip(theme::PADDING as f32, dpi);
        let height = dip(theme::CAPTURE_PANEL_HEIGHT, dpi);
        let width = dip(theme::CAPTURE_PANEL_WIDTH, dpi).min(work.width);
        if SetWindowPos(
            state.panel,
            HWND_TOPMOST,
            work.left + (work.width - width) / 2,
            work.top + p,
            width,
            height,
            SWP_NOACTIVATE,
        ) == 0
        {
            return Err(CaptureError::Unavailable);
        }
        state.font = native_font(theme::BODY, dpi)?;
        state.label_font = native_font(theme::LABEL, dpi)?;
        let instructions = CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            wide(INSTRUCTIONS).as_ptr(),
            WS_CHILD | WS_VISIBLE,
            p,
            p,
            width - 2 * p,
            dip(theme::CAPTURE_INSTRUCTIONS_HEIGHT, dpi),
            state.panel,
            null_mut(),
            instance,
            null(),
        );
        state.button = CreateWindowExW(
            0,
            wide("BUTTON").as_ptr(),
            wide(CANCEL_LABEL).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
            p,
            dip(theme::CAPTURE_CANCEL_TOP, dpi),
            width - 2 * p,
            dip(theme::CONTROL_HEIGHT, dpi),
            state.panel,
            CANCEL as _,
            instance,
            null(),
        );
        if instructions.is_null() || state.button.is_null() {
            return Err(CaptureError::Unavailable);
        }
        SendMessageW(instructions, WM_SETFONT, state.font.0 as usize, 1);
        SendMessageW(state.button, WM_SETFONT, state.label_font.0 as usize, 1);
        if SetTimer(state.window, TICK, 25, None) == 0 {
            return Err(CaptureError::Unavailable);
        }
        ShowWindow(state.window, SW_SHOW);
        ShowWindow(state.border, SW_SHOWNOACTIVATE);
        ShowWindow(state.panel, SW_SHOW);
        if SetForegroundWindow(state.panel) == 0 {
            return Err(CaptureError::Unavailable);
        }
        SetFocus(state.button);
        if GetFocus() != state.button {
            return Err(CaptureError::Unavailable);
        }
        drop(state);
        let mut msg = MSG::default();
        while storage.borrow().outcome.is_none() {
            let status = GetMessageW(&mut msg, null_mut(), 0, 0);
            if status <= 0 {
                let mut state = storage.borrow_mut();
                if state.outcome.is_none() {
                    state.finish(Err(CaptureError::Unavailable));
                }
                break;
            }
            if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE as usize {
                storage.borrow_mut().finish(Ok(None));
                continue;
            }
            if msg.message == WM_KEYDOWN
                && msg.wParam == b'L' as usize
                && GetKeyState(VK_CONTROL as i32) < 0
            {
                let mut state = storage.borrow_mut();
                state.locked = true;
                state.finish(Ok(None));
                continue;
            }
            let panel = storage.borrow().panel;
            if IsDialogMessageW(panel, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        let mut state = storage.borrow_mut();
        let result = state
            .outcome
            .take()
            .unwrap_or(Err(CaptureError::Unavailable));
        let layout = std::mem::take(&mut state.monitors);
        let locked = state.locked;
        drop(state);
        // Destroy all HWNDs before dropping their stable state pointer/fonts.
        drop(windows);
        if DwmFlush() < 0 {
            return Err(CaptureError::Unavailable);
        }
        result.map(|r| (r, layout, locked))
    }
}

pub(super) fn select_and_decode(owner: usize, gate: &Gate, repaint: &dyn Fn()) -> CaptureOutcome {
    let _dpi = match DpiContext::enter() {
        Ok(d) => d,
        Err(e) => return CaptureOutcome::CaptureFailure(e),
    };
    let (region, layout, locked) = match select(owner, gate) {
        Ok(r) => r,
        Err(e) => return CaptureOutcome::CaptureFailure(e),
    };
    if locked || gate.stopped() {
        return if locked {
            CaptureOutcome::Locked
        } else {
            gate.stop_outcome()
        };
    }
    let Some(region) = region else {
        return CaptureOutcome::Cancelled;
    };
    *gate.shared.region.lock().unwrap_or_else(|e| e.into_inner()) = Some(region);
    let pixels = match capture(region, &layout, gate) {
        Ok(p) => p,
        Err(e) => return CaptureOutcome::CaptureFailure(e),
    };
    if gate.stopped() {
        return gate.stop_outcome();
    }
    gate.shared.decoding.store(true, Ordering::SeqCst);
    repaint();
    // Bounded decoder is cooperative; token checked again before publication.
    match qr::decode_pixels(region.width, region.height, PixelLayout::Bgra8, pixels) {
        Ok(review) => CaptureOutcome::Review(review),
        Err(error) => CaptureOutcome::DecodeFailure(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_font_metrics_and_control_fit() -> Result<(), CaptureError> {
        let _dpi = DpiContext::enter()?;
        let _private_fonts = [
            FontMemory::load(include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf"))?,
            FontMemory::load(include_bytes!(
                "../../../assets/fonts/NotoSans-SemiBold.ttf"
            ))?,
        ];
        // A private memory DC only: no screen/window pixels are captured and
        // no user font/settings are installed. Probe the same production helper.
        let dc = MemoryDc(unsafe { CreateCompatibleDC(null_mut()) });
        if dc.0.is_null() {
            return Err(CaptureError::Unavailable);
        }
        assert_eq!(
            unsafe { GetMapMode(dc.0) },
            MM_TEXT,
            "probe logical units must be pixels"
        );
        for (name, role, text, height, flags) in [
            (
                "BODY",
                theme::BODY,
                INSTRUCTIONS,
                theme::CAPTURE_INSTRUCTIONS_HEIGHT,
                DT_CALCRECT | DT_WORDBREAK,
            ),
            (
                "LABEL",
                theme::LABEL,
                CANCEL_LABEL,
                theme::CONTROL_HEIGHT,
                DT_CALCRECT | DT_SINGLELINE,
            ),
        ] {
            let dpi = 96;
            let font = native_font(role, dpi)?;
            unsafe {
                let old = SelectObject(dc.0, font.0);
                if old.is_null() || old as isize == -1 {
                    return Err(CaptureError::Unavailable);
                }
                let _selected = SelectionObject { dc: dc.0, old };
                let mut metrics = TEXTMETRICW::default();
                let mut requested = LOGFONTW::default();
                if GetTextMetricsW(dc.0, &mut metrics) == 0
                    || GetObjectW(
                        font.0,
                        std::mem::size_of::<LOGFONTW>() as i32,
                        (&mut requested as *mut LOGFONTW).cast(),
                    ) == 0
                {
                    return Err(CaptureError::Unavailable);
                }
                let mut face = [0u16; 64];
                let count = GetTextFaceW(dc.0, face.len() as i32, face.as_mut_ptr());
                if count <= 0 {
                    return Err(CaptureError::Unavailable);
                }
                let face_end = face.iter().position(|&c| c == 0).unwrap_or(face.len());
                let realized_face = String::from_utf16_lossy(&face[..face_end]);
                assert!(
                    realized_face.starts_with("Noto Sans"),
                    "private bundled font must be realized"
                );
                assert_eq!(requested.lfWeight, native_weight(role.weight));
                assert_eq!(metrics.tmWeight, native_weight(role.weight));
                assert_eq!(requested.lfHeight, -dip(role.size, dpi));
                assert_eq!(
                    GetTextCharacterExtra(dc.0),
                    0,
                    "native kind deliberately uses zero default tracking"
                );
                let width = dip(
                    theme::CAPTURE_PANEL_WIDTH - 2.0 * f32::from(theme::PADDING),
                    dpi,
                );
                let mut widest = 0;
                for line in text.split("\r\n") {
                    let units: Vec<u16> = line.encode_utf16().collect();
                    let mut size = SIZE::default();
                    if GetTextExtentPoint32W(dc.0, units.as_ptr(), units.len() as i32, &mut size)
                        == 0
                    {
                        return Err(CaptureError::Unavailable);
                    }
                    widest = widest.max(size.cx);
                    assert!(size.cx <= width && size.cy <= dip(height, dpi));
                }
                let units = wide(text);
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: width,
                    bottom: 0,
                };
                let block = DrawTextW(
                    dc.0,
                    units.as_ptr(),
                    (units.len() - 1) as i32,
                    &mut rect,
                    flags,
                );
                assert!(block > 0 && block <= dip(height, dpi));
                assert!(rect.right <= width);
                assert_eq!(block, metrics.tmHeight * text.split("\r\n").count() as i32);
                println!(
                    "{name} requested96DPI face={realized_face}; weight={}; lfHeight={}; cell={} ascent={} descent={} internalLeading={} externalLeading={}; nativeTracking=0; widest={} block={} allocated={}x{}; ROOT role.line_height={} is not imposed on native controls",
                    metrics.tmWeight,
                    requested.lfHeight,
                    metrics.tmHeight,
                    metrics.tmAscent,
                    metrics.tmDescent,
                    metrics.tmInternalLeading,
                    metrics.tmExternalLeading,
                    widest,
                    block,
                    width,
                    dip(height, dpi),
                    role.line_height
                );
            }
        }
        Ok(())
    }
    #[test]
    fn native_layout_probe_geometry_only() -> Result<(), CaptureError> {
        let _dpi = DpiContext::enter()?;
        let desktop = desktop_bounds().map_err(|_| CaptureError::Unavailable)?;
        let layout = monitors()?;
        println!("Physical desktop {desktop:?}; monitors {layout:?}");
        assert!(!layout.is_empty());
        for dpi in [96, 144, 192] {
            assert_eq!(dip(96., dpi), dpi as i32);
            assert_eq!(dip(theme::CONTROL_HEIGHT, dpi), 36 * dpi as i32 / 96);
        }
        Ok(())
    }
    #[test]
    fn private_native_messages_release_pointer_and_hidden_lock_watch() -> Result<(), CaptureError> {
        let _dpi = DpiContext::enter()?;
        for message in [
            WM_CLOSE,
            WM_CANCELMODE,
            WM_DISPLAYCHANGE,
            WM_DPICHANGED,
            WM_TIMER,
        ] {
            let listener = crate::platform::lifecycle::Lifecycle::start(|| {})
                .map_err(|_| CaptureError::Unavailable)?;
            let task = CaptureTask::queued(None);
            let gate = Gate {
                shared: task.shared.clone(),
                watch: listener.watch(),
                deadline: Instant::now() + std::time::Duration::from_secs(60),
            };
            unsafe {
                let instance = GetModuleHandleW(null());
                let class = wide(&format!(
                    "AuthenticatorPrivateCaptureTest{}",
                    uuid::Uuid::new_v4()
                ));
                let wc = WNDCLASSW {
                    hInstance: instance,
                    lpfnWndProc: Some(procedure),
                    lpszClassName: class.as_ptr(),
                    ..Default::default()
                };
                assert_ne!(RegisterClassW(&wc), 0);
                let storage = Box::new(RefCell::new(Overlay {
                    gate: &gate,
                    bounds: DesktopBounds {
                        left: -100,
                        top: 0,
                        width: 200,
                        height: 200,
                    },
                    monitors: monitors()?,
                    owner: null_mut(),
                    window: null_mut(),
                    panel: null_mut(),
                    border: null_mut(),
                    button: null_mut(),
                    font: Object(null_mut()),
                    label_font: Object(null_mut()),
                    _font_memory: [FontMemory(null_mut()), FontMemory(null_mut())],
                    start: Some(Point { x: 0, y: 0 }),
                    end: Point { x: 10, y: 10 },
                    outcome: None,
                    locked: false,
                }));
                let mut windows = Windows {
                    handles: Vec::new(),
                    class,
                    instance,
                };
                let hwnd = CreateWindowExW(
                    0,
                    windows.class.as_ptr(),
                    wide("Private capture message test").as_ptr(),
                    WS_POPUP,
                    0,
                    0,
                    32,
                    32,
                    null_mut(),
                    null_mut(),
                    instance,
                    (&*storage as *const RefCell<Overlay<'_>>).cast(),
                );
                assert!(!hwnd.is_null());
                windows.handles.push(hwnd);
                storage.borrow_mut().owner = hwnd;
                storage.borrow_mut().window = hwnd;
                SetCapture(hwnd);
                assert_eq!(GetCapture(), hwnd);
                if message == WM_TIMER {
                    listener.notify_lock_for_test();
                }
                SendMessageW(hwnd, message, 0, 0);
                assert!(storage.borrow().outcome.is_some());
                assert!(storage.borrow().start.is_none());
                assert!(GetCapture().is_null());
                assert_eq!(storage.borrow().locked, message == WM_TIMER);
                drop(windows);
                assert_eq!(IsWindow(hwnd), 0);
                // finish posts only to this test's message queue.
                let mut msg = MSG::default();
                PeekMessageW(&mut msg, null_mut(), WM_QUIT, WM_QUIT, PM_REMOVE);
            }
        }
        Ok(())
    }
    #[test]
    fn no_pixel_allocation_on_invalid_region_or_invalid_owner() -> Result<(), CaptureError> {
        let listener = crate::platform::lifecycle::Lifecycle::start(|| {})
            .map_err(|_| CaptureError::Unavailable)?;
        let task = CaptureTask::queued(None);
        let gate = Gate {
            shared: task.shared.clone(),
            watch: listener.watch(),
            deadline: Instant::now() + std::time::Duration::from_secs(60),
        };
        assert!(matches!(
            capture(
                Region {
                    left: 0,
                    top: 0,
                    width: 3840,
                    height: 2160
                },
                &[],
                &gate
            ),
            Err(CaptureError::TooLarge)
        ));
        assert!(matches!(
            select_and_decode(0, &gate, &|| {}),
            CaptureOutcome::CaptureFailure(CaptureError::Unavailable)
        ));
        task.cancel();
        assert!(matches!(
            select_and_decode(0, &gate, &|| {}),
            CaptureOutcome::Cancelled
        ));
        Ok(())
    }
}
