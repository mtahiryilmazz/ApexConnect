use std::io::Cursor;
use jpeg_encoder::{ColorType, Encoder};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct ScreenCapture {
    pub width: i32,
    pub height: i32,
}

impl ScreenCapture {
    pub fn new() -> Self {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            Self { width, height }
        }
    }

    pub fn capture_frame(&self, quality: u8) -> Result<Vec<u8>, String> {
        unsafe {
            let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
            if hdc_screen.0.is_null() {
                return Err("Failed to get screen DC".to_string());
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.0.is_null() {
                ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
                return Err("Failed to create memory DC".to_string());
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, self.width, self.height);
            if hbitmap.0.is_null() {
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
                return Err("Failed to create compatible bitmap".to_string());
            }

            let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));

            // Copy screen to memory DC
            let _ = BitBlt(
                hdc_mem,
                0,
                0,
                self.width,
                self.height,
                hdc_screen,
                0,
                0,
                SRCCOPY,
            );

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.width,
                    biHeight: -self.height, // Top-down DIB
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
                }; 1],
            };

            let buf_size = (self.width * self.height * 4) as usize;
            let mut bgra_buf: Vec<u8> = vec![0u8; buf_size];

            let lines = GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                self.height as u32,
                Some(bgra_buf.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Clean up GDI handles
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

            if lines == 0 {
                return Err("GetDIBits failed to copy pixels".to_string());
            }

            // Convert BGRA to RGB
            let mut rgb_buf = Vec::with_capacity((self.width * self.height * 3) as usize);
            for chunk in bgra_buf.chunks_exact(4) {
                rgb_buf.push(chunk[2]); // R
                rgb_buf.push(chunk[1]); // G
                rgb_buf.push(chunk[0]); // B
            }

            // Compress to JPEG
            let mut jpeg_bytes = Vec::new();
            let mut cursor = Cursor::new(&mut jpeg_bytes);
            let encoder = Encoder::new(&mut cursor, quality);
            encoder
                .encode(&rgb_buf, self.width as u16, self.height as u16, ColorType::Rgb)
                .map_err(|e| format!("JPEG encode error: {:?}", e))?;

            Ok(jpeg_bytes)
        }
    }
}
