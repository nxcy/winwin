use windows::{
    core::*, Win32::Foundation::*, Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
};

pub trait Window {
    fn new(hwnd: HWND) -> Result<Self>
    where
        Self: Sized;
    fn on_render(&mut self);
    fn on_resize(&mut self, size: (u32, u32));
}

pub fn run<T: Window>(width: u32, height: u32, title: &str) -> Result<()> {
    unsafe {
        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc::<T>),
            hInstance: GetModuleHandleW(None)?.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            lpszClassName: w!("ClassName"),
            ..Default::default()
        });

        let mut window_rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false)?;

        let title = HSTRING::from(title);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("ClassName"),
            &title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
            None,
            None,
            GetModuleHandleW(None)?,
            None,
        );

        let mut window = T::new(hwnd)?;

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut window as *mut T as isize);

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        loop {
            match GetMessageW(&mut msg, None, 0, 0).0 {
                -1 => return Err(Error::from_win32()),
                0 => break,
                _ => {
                    DispatchMessageW(&msg);
                }
            }
        }
    }

    Ok(())
}

extern "system" fn wndproc<T: Window>(
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
            std::ptr::NonNull::<T>::new(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut T)
        {
            match msg {
                WM_PAINT => {
                    ptr.as_mut().on_render();
                    return LRESULT(0);
                }
                WM_SIZE => {
                    ptr.as_mut().on_resize((
                        (lparam.0 & 0xFFFF) as u32,
                        ((lparam.0 >> 16) & 0xFFFF) as u32,
                    ));
                    return LRESULT(0);
                }
                _ => {}
            }
        }

        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}
