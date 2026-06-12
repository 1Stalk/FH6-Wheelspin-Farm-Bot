use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use dxgi_capture_rs::{DXGIManager, CaptureError};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT, POINT};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetClientRect, SetForegroundWindow, ShowWindow, SW_SHOW};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use image::{RgbImage, ImageBuffer, Rgb};

const FH6_WINDOW_TITLE: &str = "Forza Horizon 6";

fn encode_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub struct ScreenCapture {
    manager: Option<DXGIManager>,
    hwnd: Option<HWND>,
    region: Option<(i32, i32, i32, i32)>, // (left, top, right, bottom) in screen coords
    last_frame: Option<RgbImage>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        let manager = match DXGIManager::new(10) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("[capture] Failed to initialize DXGIManager: {:?}", e);
                None
            }
        };
        Self {
            manager,
            hwnd: None,
            region: None,
            last_frame: None,
        }
    }

    pub fn find_game_window(&mut self) -> bool {
        let title_wide = encode_wide(FH6_WINDOW_TITLE);
        let hwnd = unsafe { FindWindowW(None, PCWSTR(title_wide.as_ptr())) };
        
        if hwnd.0 == 0 {
            self.hwnd = None;
            self.region = None;
            return false;
        }

        self.hwnd = Some(hwnd);

        // Try to get client rect in screen coordinates
        let mut rect = RECT::default();
        if unsafe { GetClientRect(hwnd, &mut rect) }.is_ok() {
            let mut tl = POINT { x: rect.left, y: rect.top };
            let mut br = POINT { x: rect.right, y: rect.bottom };
            
            unsafe {
                if ClientToScreen(hwnd, &mut tl).as_bool() && ClientToScreen(hwnd, &mut br).as_bool() {
                    self.region = Some((tl.x, tl.y, br.x, br.y));
                    return true;
                }
            }
        }

        // Fallback to extended frame bounds
        let mut bounds = RECT::default();
        let hr = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut bounds as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            )
        };

        if hr.is_ok() {
            self.region = Some((bounds.left, bounds.top, bounds.right, bounds.bottom));
            true
        } else {
            self.region = None;
            false
        }
    }

    pub fn focus_game_window(&self) -> bool {
        let Some(hwnd) = self.hwnd else {
            return false;
        };
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd).as_bool()
        }
    }

    pub fn window_region(&self) -> Option<(i32, i32, i32, i32)> {
        self.region
    }

    pub fn window_size(&self) -> Option<(u32, u32)> {
        self.region.map(|(l, t, r, b)| {
            ((r - l) as u32, (b - t) as u32)
        })
    }

    /// Safely crop a sub-region from the full screen BGRA frame and convert it to RGB.
    /// Coordinates are in screen coordinates.
    fn crop_to_rgb(
        &self,
        full_bgra: &[u8],
        full_w: u32,
        full_h: u32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) -> RgbImage {
        let width = (right - left) as u32;
        let height = (bottom - top) as u32;
        
        // Create an empty destination RGB image
        let mut img: RgbImage = ImageBuffer::new(width, height);
        
        for dy in 0..height {
            let sy = top + dy as i32;
            if sy < 0 || sy >= full_h as i32 {
                continue;
            }
            
            for dx in 0..width {
                let sx = left + dx as i32;
                if sx < 0 || sx >= full_w as i32 {
                    continue;
                }
                
                // Index in the source BGRA buffer (4 bytes per pixel)
                let src_idx = ((sy as u32 * full_w) + sx as u32) as usize * 4;
                if src_idx + 3 < full_bgra.len() {
                    let b = full_bgra[src_idx];
                    let g = full_bgra[src_idx + 1];
                    let r = full_bgra[src_idx + 2];
                    // Alpha is ignored
                    img.put_pixel(dx, dy, Rgb([r, g, b]));
                }
            }
        }
        
        img
    }

    pub fn grab_frame(&mut self) -> Option<RgbImage> {
        // If window region is not set, locate it
        if self.region.is_none() && !self.find_game_window() {
            return None;
        }

        if self.manager.is_none() {
            // Try to reinitialize DXGIManager if it was not ready
            self.manager = DXGIManager::new(10).ok();
        }
        
        let manager = self.manager.as_mut()?;

        let region = self.region?;
        let (left, top, right, bottom) = region;
        if right <= left || bottom <= top {
            return None;
        }

        match manager.capture_frame_components() {
            Ok((pixels, (fw, fh))) => {
                 let cropped = self.crop_to_rgb(&pixels, fw as u32, fh as u32, left, top, right, bottom);
                 self.last_frame = Some(cropped.clone());
                 Some(cropped)
            }
            Err(CaptureError::Timeout) => {
                 // Return cached frame immediately if screen hasn't updated
                 self.last_frame.clone()
            }
            Err(e) => {
                eprintln!("[capture] capture_frame_components error: {:?}", e);
                self.last_frame.clone()
            }
        }
    }

    pub fn grab_region(&mut self, x: i32, y: i32, w: i32, h: i32) -> Option<RgbImage> {
        if self.region.is_none() && !self.find_game_window() {
            return None;
        }

        let (win_left, win_top, _, _) = self.region?;
        let abs_left = win_left + x;
        let abs_top = win_top + y;
        let abs_right = abs_left + w;
        let abs_bottom = abs_top + h;

        if self.manager.is_none() {
            self.manager = DXGIManager::new(10).ok();
        }
        let manager = self.manager.as_mut()?;

        match manager.capture_frame_components() {
            Ok((pixels, (fw, fh))) => {
                 Some(self.crop_to_rgb(&pixels, fw as u32, fh as u32, abs_left, abs_top, abs_right, abs_bottom))
            }
            Err(CaptureError::Timeout) => {
                 // If the screen hasn't changed, we can crop from our cached full frame!
                 if let Some(ref full) = self.last_frame {
                     if x >= 0 && y >= 0 && x + w <= full.width() as i32 && y + h <= full.height() as i32 {
                         let mut cropped = ImageBuffer::new(w as u32, h as u32);
                         for dy in 0..h as u32 {
                             for dx in 0..w as u32 {
                                 cropped.put_pixel(dx, dy, *full.get_pixel((x as u32) + dx, (y as u32) + dy));
                             }
                         }
                         return Some(cropped);
                     }
                 }
                 None
            }
            Err(e) => {
                eprintln!("[capture] grab_region error: {:?}", e);
                None
            }
        }
    }
}
