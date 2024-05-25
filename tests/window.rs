use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{DefWindowProcW, PostQuitMessage, WM_DESTROY},
};

#[test]
fn test() {
    winwin::run::<App>(400, 300, "Window").unwrap();
}

struct App;
impl winwin::App for App {
    fn new(_hwnd: HWND) -> windows::core::Result<Self> {
        Ok(App)
    }

    fn wndproc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_DESTROY => unsafe {
                PostQuitMessage(0);
                LRESULT(0)
            },
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }
}