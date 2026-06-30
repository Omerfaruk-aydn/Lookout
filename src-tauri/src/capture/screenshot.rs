use crate::error::LookoutError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(target_os = "windows")]
pub async fn capture_region(hwnd: isize, region: Rect) -> Result<Vec<u8>, LookoutError> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, SRCCOPY,
    };
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    unsafe {
        let hwnd_raw = HWND(hwnd as *mut _);

        if !IsWindow(hwnd_raw).as_bool() {
            return Err(LookoutError::WebullNotRunning);
        }

        let hdc = GetDC(hwnd_raw);
        if hdc.is_invalid() {
            return Err(LookoutError::CaptureFailed(
                "Failed to get device context".to_string(),
            ));
        }

        let mem_dc = CreateCompatibleDC(hdc);
        if mem_dc.is_invalid() {
            let _ = ReleaseDC(hwnd_raw, hdc);
            return Err(LookoutError::CaptureFailed(
                "Failed to create compatible DC".to_string(),
            ));
        }

        let bitmap = CreateCompatibleBitmap(hdc, region.width, region.height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(hwnd_raw, hdc);
            return Err(LookoutError::CaptureFailed(
                "Failed to create bitmap".to_string(),
            ));
        }

        let old_obj = SelectObject(mem_dc, bitmap);

        let result = BitBlt(
            mem_dc,
            0,
            0,
            region.width,
            region.height,
            hdc,
            region.x,
            region.y,
            SRCCOPY,
        );

        SelectObject(mem_dc, old_obj);

        if result.is_err() {
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(hwnd_raw, hdc);
            return Err(LookoutError::CaptureFailed("BitBlt failed".to_string()));
        }

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: region.width,
                biHeight: -region.height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };

        let buf_size = (region.width * region.height * 4) as usize;
        let mut pixel_data: Vec<u8> = vec![0u8; buf_size];

        let get_result = GetDIBits(
            mem_dc,
            bitmap,
            0,
            region.height as u32,
            Some(pixel_data.as_mut_ptr() as *mut _),
            &mut bmp_info as *mut _,
            DIB_RGB_COLORS,
        );

        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(hwnd_raw, hdc);

        if get_result == 0 {
            return Err(LookoutError::CaptureFailed(
                "GetDIBits failed".to_string(),
            ));
        }

        let png_data = encode_bgra_to_png(&pixel_data, region.width as u32, region.height as u32)?;
        Ok(png_data)
    }
}

#[cfg(not(target_os = "windows"))]
pub async fn capture_region(_hwnd: isize, _region: Rect) -> Result<Vec<u8>, LookoutError> {
    Err(LookoutError::CaptureFailed(
        "Screen capture is only supported on Windows".to_string(),
    ))
}

#[cfg(target_os = "windows")]
fn encode_bgra_to_png(bgra_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, LookoutError> {
    let mut rgba = bgra_data.to_vec();
    for chunk in rgba.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| LookoutError::CaptureFailed(format!("PNG encode header: {}", e)))?;
        writer
            .write_image_data(&rgba)
            .map_err(|e| LookoutError::CaptureFailed(format!("PNG encode data: {}", e)))?;
    }
    Ok(png_buf)
}
