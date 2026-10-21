use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use jpeg_encoder::{ColorType, Encoder};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

pub enum CaptureMode {
    PrimaryMonitor,
    VirtualDesktop,
}

pub struct ScreenCapture {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub mode: CaptureMode,
    last_frame_hash: AtomicU64,
    last_keyframe_time: std::sync::Mutex<std::time::Instant>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self::with_mode(CaptureMode::PrimaryMonitor)
    }

    pub fn with_mode(mode: CaptureMode) -> Self {
        unsafe {
            let (x, y, width, height) = match mode {
                CaptureMode::PrimaryMonitor => {
                    let w = GetSystemMetrics(SM_CXSCREEN);
                    let h = GetSystemMetrics(SM_CYSCREEN);
                    (0, 0, w, h)
                }
                CaptureMode::VirtualDesktop => {
                    let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                    let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                    let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                    let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
                    (vx, vy, vw, vh)
                }
            };

            Self {
                x,
                y,
                width,
                height,
                mode,
                last_frame_hash: AtomicU64::new(0),
                last_keyframe_time: std::sync::Mutex::new(std::time::Instant::now()),
            }
        }
    }

    /// Fast 64-bit sampled checksum for dirty frame detection
    fn compute_buffer_hash(buf: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        // Sample every 16th byte for ultra-fast dirty checking across millions of pixels
        for &byte in buf.iter().step_by(16) {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Captures the screen. If dirty detection is active and the frame hasn't changed,
    /// returns Ok(None) to avoid redundant JPEG encoding and network congestion.
    /// A forced keyframe is emitted at least once every 2 seconds.
    pub fn capture_frame_adaptive(&self, quality: u8, force_full: bool) -> Result<Option<Vec<u8>>, String> {
        unsafe {
            // Attach thread to active desktop (handles UAC prompts, lock screens, and services)
            #[cfg(target_os = "windows")]
            let maybe_desk = {
                use windows::Win32::System::StationsAndDesktops::{
                    OpenInputDesktop, SetThreadDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_ACCESS_FLAGS,
                };
                if let Ok(hdesk) = OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_ACCESS_FLAGS(0x01FF)) {
                    let _ = SetThreadDesktop(hdesk);
                    Some(hdesk)
                } else {
                    None
                }
            };

            let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
            if hdc_screen.0.is_null() {
                return Err("Failed to get screen DC".to_string());
            }

            // Dynamically ensure width & height are valid
            let cur_w = match self.mode {
                CaptureMode::PrimaryMonitor => GetSystemMetrics(SM_CXSCREEN),
                CaptureMode::VirtualDesktop => GetSystemMetrics(SM_CXVIRTUALSCREEN),
            };
            let cur_h = match self.mode {
                CaptureMode::PrimaryMonitor => GetSystemMetrics(SM_CYSCREEN),
                CaptureMode::VirtualDesktop => GetSystemMetrics(SM_CYVIRTUALSCREEN),
            };
            let width = if cur_w > 0 { cur_w } else { self.width.max(800) };
            let height = if cur_h > 0 { cur_h } else { self.height.max(600) };

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.0.is_null() {
                ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
                return Err("Failed to create memory DC".to_string());
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
            if hbitmap.0.is_null() {
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
                return Err("Failed to create compatible bitmap".to_string());
            }

            let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));

            // Copy screen region to memory DC (use CAPTUREBLT to capture layered & hardware-composited windows)
            use windows::Win32::Graphics::Gdi::ROP_CODE;
            let rop = ROP_CODE(SRCCOPY.0 | 0x40000000); // SRCCOPY | CAPTUREBLT
            let blt_res = BitBlt(
                hdc_mem,
                0,
                0,
                width,
                height,
                hdc_screen,
                self.x,
                self.y,
                rop,
            );
            if blt_res.is_err() {
                // Fallback to standard SRCCOPY if CAPTUREBLT is unsupported on virtual display
                let _ = BitBlt(
                    hdc_mem,
                    0,
                    0,
                    width,
                    height,
                    hdc_screen,
                    self.x,
                    self.y,
                    SRCCOPY,
                );
            }

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Top-down DIB
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

            let pixel_count = (width * height) as usize;
            let buf_size = pixel_count * 4;
            let mut bgra_buf: Vec<u8> = vec![0u8; buf_size];

            let lines = GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                height as u32,
                Some(bgra_buf.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Clean up GDI handles immediately
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            #[cfg(target_os = "windows")]
            if let Some(hdesk) = maybe_desk {
                use windows::Win32::System::StationsAndDesktops::CloseDesktop;
                let _ = CloseDesktop(hdesk);
            }

            if lines == 0 {
                return Err("GetDIBits failed to copy pixels".to_string());
            }

            // Check if frame changed (dirty detection)
            let current_hash = Self::compute_buffer_hash(&bgra_buf);
            let prev_hash = self.last_frame_hash.swap(current_hash, Ordering::Relaxed);

            let mut keyframe_guard = self.last_keyframe_time.lock().unwrap();
            let elapsed = keyframe_guard.elapsed();
            let need_keyframe = force_full || elapsed >= std::time::Duration::from_millis(2000);

            if !need_keyframe && current_hash == prev_hash {
                // Screen has not changed! Save CPU and bandwidth
                return Ok(None);
            }

            *keyframe_guard = std::time::Instant::now();

            // High-speed BGRA -> RGB conversion without vector re-allocations
            let mut rgb_buf = vec![0u8; pixel_count * 3];
            for i in 0..pixel_count {
                let bgra_idx = i * 4;
                let rgb_idx = i * 3;
                rgb_buf[rgb_idx] = bgra_buf[bgra_idx + 2];     // R
                rgb_buf[rgb_idx + 1] = bgra_buf[bgra_idx + 1]; // G
                rgb_buf[rgb_idx + 2] = bgra_buf[bgra_idx];     // B
            }

            // Compress to JPEG with requested quality
            let mut jpeg_bytes = Vec::with_capacity(65536);
            let mut cursor = Cursor::new(&mut jpeg_bytes);
            let encoder = Encoder::new(&mut cursor, quality);
            encoder
                .encode(&rgb_buf, width as u16, height as u16, ColorType::Rgb)
                .map_err(|e| format!("JPEG encode error: {:?}", e))?;

            Ok(Some(jpeg_bytes))
        }
    }

    /// Legacy synchronous capture (always returns JPEG bytes)
    pub fn capture_frame(&self, quality: u8) -> Result<Vec<u8>, String> {
        match self.capture_frame_adaptive(quality, true)? {
            Some(data) => Ok(data),
            None => Err("No frame captured".to_string()),
        }
    }
}
