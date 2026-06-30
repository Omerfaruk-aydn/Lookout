use crate::error::LookoutError;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindow,
};

#[cfg(target_os = "windows")]
use std::sync::Mutex;

#[cfg(target_os = "windows")]
struct WindowSearchState {
    found_hwnd: Option<isize>,
}

#[cfg(target_os = "windows")]
static CACHED_HWND: Mutex<Option<isize>> = Mutex::new(None);

#[cfg(target_os = "windows")]
pub fn find_webull_window() -> Result<isize, LookoutError> {
    if let Ok(cached) = CACHED_HWND.lock() {
        if let Some(hwnd) = *cached {
            unsafe {
                if IsWindow(HWND(hwnd as *mut _)).as_bool() {
                    return Ok(hwnd);
                }
            }
        }
    }

    let state = std::sync::Arc::new(Mutex::new(WindowSearchState {
        found_hwnd: None,
    }));

    let state_clone = state.clone();

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            windows::Win32::Foundation::LPARAM(&state_clone as *const _ as isize),
        );
    }

    let locked = state.lock().map_err(|_| LookoutError::CaptureFailed("Failed to enumerate windows".to_string()))?;
    match locked.found_hwnd {
        Some(hwnd) => {
            if let Ok(mut cached) = CACHED_HWND.lock() {
                *cached = Some(hwnd);
            }
            Ok(hwnd)
        }
        None => Err(LookoutError::WebullNotRunning),
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_callback(
    hwnd: HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let state = &*(lparam.0 as *const std::sync::Arc<Mutex<WindowSearchState>>);

    let mut title_buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut title_buf) } as usize;
    if len == 0 {
        return windows::Win32::Foundation::BOOL(1);
    }

    let title = String::from_utf16_lossy(&title_buf[..len]);

    if title.contains("Webull") {
        if let Ok(mut locked) = state.lock() {
            locked.found_hwnd = Some(hwnd.0 as isize);
            return windows::Win32::Foundation::BOOL(0);
        }
    }

    windows::Win32::Foundation::BOOL(1)
}

#[cfg(not(target_os = "windows"))]
pub fn find_webull_window() -> Result<isize, LookoutError> {
    Err(LookoutError::CaptureFailed(
        "Screen capture is only supported on Windows".to_string(),
    ))
}

pub fn validate_hwnd(hwnd: isize) -> Result<(), LookoutError> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            if !IsWindow(HWND(hwnd as *mut _)).as_bool() {
                if let Ok(mut cached) = CACHED_HWND.lock() {
                    *cached = None;
                }
                return Err(LookoutError::WebullNotRunning);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
    }
    Ok(())
}
