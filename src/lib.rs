use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW,
            LoadCursorW, PeekMessageW, RegisterClassExW, SetWindowLongPtrW, ShowWindow,
            TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, MSG,
            PM_REMOVE, SW_SHOW, WINDOW_EX_STYLE, WM_QUIT, WNDCLASSEXW, WS_CAPTION, WS_MINIMIZEBOX,
            WS_OVERLAPPED, WS_SYSMENU,
        },
    },
};

pub trait App: Sized {
    fn new(hwnd: HWND) -> Result<Self>;
    fn wndproc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

pub fn run<A: App>(width: u32, height: u32, title: &str) -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = w!("AppClassName");
        let window_style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;

        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc::<A>),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            lpszClassName: class_name,
            ..Default::default()
        });

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        AdjustWindowRect(&mut rect, window_style, false)?;

        let title = HSTRING::from(title);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            &title,
            window_style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            instance,
            None,
        );

        let mut app = A::new(hwnd)?;

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut app as *mut A as isize);

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while msg.message != WM_QUIT {
            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    Ok(())
}

extern "system" fn wndproc<A: App>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match std::ptr::NonNull::<A>::new(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut A) {
            Some(mut ptr) => ptr.as_mut().wndproc(hwnd, msg, wparam, lparam),
            None => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
