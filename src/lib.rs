#![forbid(unsafe_op_in_unsafe_fn)]

use std::{marker::PhantomData, mem::size_of};

use windows::{
    core::*,
    Win32::{Foundation::*, System::LibraryLoader::*, UI::WindowsAndMessaging::*},
};

pub struct Window<EH: EventHandler> {
    hwnd: HWND,
    phantom_data: PhantomData<EH>,
}

pub trait EventHandler {
    fn on_paint(&mut self);
}

impl<EH: EventHandler> Window<EH> {
    pub fn new(size: (u32, u32), name: impl Param<PCWSTR>) -> Result<Self> {
        unsafe {
            RegisterClassExW(&WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc::<EH>),
                hInstance: GetModuleHandleW(None)?.into(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                lpszClassName: w!("winwin-rs-class-name"),
                ..Default::default()
            });

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: size.0 as i32,
                bottom: size.1 as i32,
            };
            AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false)?;

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("winwin-rs-class-name"),
                name,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                None,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            Ok(Self {
                hwnd,
                phantom_data: PhantomData,
            })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn run(self, mut event_handler: EH) {
        unsafe {
            SetWindowLongPtrW(
                self.hwnd,
                GWLP_USERDATA,
                &mut event_handler as *mut EH as isize,
            );

            let _ = ShowWindow(self.hwnd, SW_SHOW);

            let mut msg = MSG::default();
            while msg.message != WM_QUIT {
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}

unsafe extern "system" fn wndproc<EH: EventHandler>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
            return LRESULT(0);
        }

        if let Some(mut ptr) =
            std::ptr::NonNull::<EH>::new(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut EH)
        {
            match msg {
                WM_PAINT => {
                    ptr.as_mut().on_paint();
                    return LRESULT(0);
                }
                _ => {}
            }
        }

        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}
