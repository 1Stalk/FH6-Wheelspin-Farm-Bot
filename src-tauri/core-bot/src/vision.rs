use image::{imageops, GrayImage, Luma, RgbImage};

// Car selection grid layout constants (baseline 2560x1440 resolution)
pub const CAR_GRID_START_X: f32 = 758.0;
pub const CAR_GRID_START_Y: f32 = 445.0;
pub const CAR_CELL_W: f32 = 450.0;
pub const CAR_CELL_H: f32 = 350.0;

// Offsets for center adjustments
pub const CAR_MATCH_OFFSET_X: f32 = -14.0;
pub const CAR_MATCH_OFFSET_Y: f32 = 109.0;
pub const CAR_CURSOR_OFFSET_X: f32 = 203.0;
pub const CAR_CURSOR_OFFSET_Y: f32 = 170.0;

// Static templates include bytes
static TOYOTA_2019: &[u8] = include_bytes!("../templates/Toyota_2019.png");
static TOYOTA_2019_2: &[u8] = include_bytes!("../templates/Toyota_2019_2.png");
static NISSAN_1989: &[u8] = include_bytes!("../templates/Nissan_1989.png");
static NISSAN_1989_2: &[u8] = include_bytes!("../templates/Nissan_1989_2.png");
static NISSAN_BRAND_BIG: &[u8] = include_bytes!("../templates/nissan_brand_big.png");
static NISSAN_BRAND_BIG_2: &[u8] = include_bytes!("../templates/nissan_brand_big_2.png");
static NISSAN_BRAND_BIG_SELECTED: &[u8] =
    include_bytes!("../templates/nissan_brand_big_selected.png");
static NISSAN_BRAND_BIG_SELECTED_2: &[u8] =
    include_bytes!("../templates/nissan_brand_big_selected_2.png");
static AUTOPILOT_DRIVING: &[u8] = include_bytes!("../templates/autopilot_driving.png");
static AUTOPILOT_DRIVING_DISABLED: &[u8] =
    include_bytes!("../templates/autopilot_driving_disabled.png");
static AUTOPILOT_ICON: &[u8] = include_bytes!("../templates/autopilot_icon.png");
static BRAND_SELECTION_CURSOR: &[u8] =
    include_bytes!("../templates/brand_selection_cursor.png");
static CAR_CLASS_B: &[u8] = include_bytes!("../templates/car_class_b.png");
static CAR_FAVORITE_HEART: &[u8] = include_bytes!("../templates/car_favorite_heart.png");
static CAR_SELECTION_MENU: &[u8] = include_bytes!("../templates/car_selection_menu.png");
static CAR_SELECTION_MENU_SELECTED: &[u8] =
    include_bytes!("../templates/car_selection_menu_selected.png");
static COLLECTION_JOURNAL_MENU: &[u8] =
    include_bytes!("../templates/collection_journal_menu.png");
static DRIVING: &[u8] = include_bytes!("../templates/driving.png");
static ERROR: &[u8] = include_bytes!("../templates/error.png");
static EVENTLAB_RACE_ON_SCREEN: &[u8] =
    include_bytes!("../templates/eventlab_race_on_screen.png");
static JOURNAL_BRAND_CURSOR: &[u8] = include_bytes!("../templates/journal_brand_cursor.png");
static JOURNAL_CAR_CURSOR: &[u8] = include_bytes!("../templates/journal_car_cursor.png");
static JOURNAL_SUBARU_22B: &[u8] = include_bytes!("../templates/journal_subaru_22b.png");
static JOURNAL_SUBARU_22B_2: &[u8] = include_bytes!("../templates/journal_subaru_22b_2.png");
static JOURNAL_SUBARU_22B_SELECTED: &[u8] =
    include_bytes!("../templates/journal_subaru_22b_selected.png");
static JOURNAL_SUBARU_BRAND: &[u8] = include_bytes!("../templates/journal_subaru_brand.png");
static JOURNAL_SUBARU_BRAND_2: &[u8] =
    include_bytes!("../templates/journal_subaru_brand_2.png");
static JOURNAL_SUBARU_BRAND_SELECTED: &[u8] =
    include_bytes!("../templates/journal_subaru_brand_selected.png");
static JOURNAL_SUBARU_BRAND_SELECTED_2: &[u8] =
    include_bytes!("../templates/journal_subaru_brand_selected_2.png");
static JOURNAL_TOYOTA_BRAND: &[u8] = include_bytes!("../templates/journal_toyota_brand.png");
static JOURNAL_TOYOTA_BRAND_SELECTED: &[u8] =
    include_bytes!("../templates/journal_toyota_brand_selected.png");
static PAUSE_MENU: &[u8] = include_bytes!("../templates/pause_menu.png");
static PAUSE_MENU_1ST_PAGE: &[u8] = include_bytes!("../templates/pause_menu_1st_page.png");
static SPEND_SP_MENU: &[u8] = include_bytes!("../templates/spend_sp_menu.png");
static STAGE2_FINISH_BANNER: &[u8] = include_bytes!("../templates/stage2_finish_banner.png");
static STAGE2_FINISH_BANNER_2: &[u8] =
    include_bytes!("../templates/stage2_finish_banner_2.png");
static STAGE2_MAP_MENU: &[u8] = include_bytes!("../templates/stage2_map_menu.png");
static STAGE2_MAP_MENU_WRONG_BUTTON_SELECTED: &[u8] =
    include_bytes!("../templates/stage2_map_menu_wrong_button_selected.png");
static STAGE2_POST_FINISH: &[u8] = include_bytes!("../templates/stage2_post_finish.png");
static SUBARU_BRAND_BIG: &[u8] = include_bytes!("../templates/subaru_brand_big.png");
static SUBARU_BRAND_BIG_2: &[u8] = include_bytes!("../templates/subaru_brand_big_2.png");
static SUBARU_BRAND_BIG_SELECTED: &[u8] =
    include_bytes!("../templates/subaru_brand_big_selected.png");
static SUBARU_BRAND_BIG_SELECTED_2: &[u8] =
    include_bytes!("../templates/subaru_brand_big_selected_2.png");
static SUBARU_IMPREZA_1998: &[u8] = include_bytes!("../templates/subaru_impreza_1998.png");
static SUBARU_IMPREZA_1998_2: &[u8] =
    include_bytes!("../templates/subaru_impreza_1998_2.png");
static SUBARU_IMPREZA_NEW: &[u8] = include_bytes!("../templates/subaru_impreza_new.png");
static TALENT_CURSOR_CIRCLE: &[u8] = include_bytes!("../templates/talent_cursor_circle.png");
static TALENT_CURSOR_SQUARE: &[u8] = include_bytes!("../templates/talent_cursor_square.png");
static TOYOTA_BRAND_BIG: &[u8] = include_bytes!("../templates/toyota_brand_big.png");
static TOYOTA_BRAND_BIG_SELECTED: &[u8] =
    include_bytes!("../templates/toyota_brand_big_selected.png");

pub fn get_template_bytes(name: &str) -> Option<&'static [u8]> {
    let clean_name = name.split('/').last().unwrap_or(name).replace(".png", "");
    match clean_name.as_str() {
        "Toyota_2019" => Some(TOYOTA_2019),
        "Toyota_2019_2" => Some(TOYOTA_2019_2),
        "Nissan_1989" | "nissan_1989" => Some(NISSAN_1989),
        "Nissan_1989_2" | "nissan_1989_2" => Some(NISSAN_1989_2),
        "nissan_brand_big" => Some(NISSAN_BRAND_BIG),
        "nissan_brand_big_2" => Some(NISSAN_BRAND_BIG_2),
        "nissan_brand_big_selected" => Some(NISSAN_BRAND_BIG_SELECTED),
        "nissan_brand_big_selected_2" => Some(NISSAN_BRAND_BIG_SELECTED_2),
        "autopilot_driving" => Some(AUTOPILOT_DRIVING),
        "autopilot_driving_disabled" => Some(AUTOPILOT_DRIVING_DISABLED),
        "autopilot_icon" => Some(AUTOPILOT_ICON),
        "brand_selection_cursor" => Some(BRAND_SELECTION_CURSOR),
        "car_class_b" => Some(CAR_CLASS_B),
        "car_favorite_heart" => Some(CAR_FAVORITE_HEART),
        "car_selection_menu" => Some(CAR_SELECTION_MENU),
        "car_selection_menu_selected" => Some(CAR_SELECTION_MENU_SELECTED),
        "collection_journal_menu" => Some(COLLECTION_JOURNAL_MENU),
        "driving" => Some(DRIVING),
        "error" => Some(ERROR),
        "eventlab_race_on_screen" => Some(EVENTLAB_RACE_ON_SCREEN),
        "journal_brand_cursor" => Some(JOURNAL_BRAND_CURSOR),
        "journal_car_cursor" => Some(JOURNAL_CAR_CURSOR),
        "journal_subaru_22b" => Some(JOURNAL_SUBARU_22B),
        "journal_subaru_22b_2" => Some(JOURNAL_SUBARU_22B_2),
        "journal_subaru_22b_selected" => Some(JOURNAL_SUBARU_22B_SELECTED),
        "journal_subaru_brand" => Some(JOURNAL_SUBARU_BRAND),
        "journal_subaru_brand_2" => Some(JOURNAL_SUBARU_BRAND_2),
        "journal_subaru_brand_selected" => Some(JOURNAL_SUBARU_BRAND_SELECTED),
        "journal_subaru_brand_selected_2" => Some(JOURNAL_SUBARU_BRAND_SELECTED_2),
        "journal_toyota_brand" => Some(JOURNAL_TOYOTA_BRAND),
        "journal_toyota_brand_selected" => Some(JOURNAL_TOYOTA_BRAND_SELECTED),
        "pause_menu" => Some(PAUSE_MENU),
        "pause_menu_1st_page" => Some(PAUSE_MENU_1ST_PAGE),
        "spend_sp_menu" => Some(SPEND_SP_MENU),
        "stage2_finish_banner" => Some(STAGE2_FINISH_BANNER),
        "stage2_finish_banner_2" => Some(STAGE2_FINISH_BANNER_2),
        "stage2_map_menu" => Some(STAGE2_MAP_MENU),
        "stage2_map_menu_wrong_button_selected" => Some(STAGE2_MAP_MENU_WRONG_BUTTON_SELECTED),
        "stage2_post_finish" => Some(STAGE2_POST_FINISH),
        "subaru_brand_big" => Some(SUBARU_BRAND_BIG),
        "subaru_brand_big_2" => Some(SUBARU_BRAND_BIG_2),
        "subaru_brand_big_selected" => Some(SUBARU_BRAND_BIG_SELECTED),
        "subaru_brand_big_selected_2" => Some(SUBARU_BRAND_BIG_SELECTED_2),
        "subaru_impreza_1998" => Some(SUBARU_IMPREZA_1998),
        "subaru_impreza_1998_2" => Some(SUBARU_IMPREZA_1998_2),
        "subaru_impreza_new" => Some(SUBARU_IMPREZA_NEW),
        "talent_cursor_circle" => Some(TALENT_CURSOR_CIRCLE),
        "talent_cursor_square" => Some(TALENT_CURSOR_SQUARE),
        "toyota_brand_big" => Some(TOYOTA_BRAND_BIG),
        "toyota_brand_big_selected" => Some(TOYOTA_BRAND_BIG_SELECTED),
        _ => None,
    }
}

pub fn get_template_candidates(name: &str) -> Vec<&'static str> {
    let clean = name.replace(".png", "");
    match clean.as_str() {
        "brand_selection_cursor" => vec!["brand_selection_cursor", "journal_brand_cursor"],
        "toyota_brand_big" => vec!["toyota_brand_big", "journal_toyota_brand"],
        "toyota_brand_big_selected" => {
            vec!["toyota_brand_big_selected", "journal_toyota_brand_selected"]
        }
        "nissan_brand_big" => vec!["nissan_brand_big", "nissan_brand_big_2"],
        "nissan_brand_big_selected" => {
            vec!["nissan_brand_big_selected", "nissan_brand_big_selected_2"]
        }
        "Nissan_1989" | "nissan_1989" => vec!["Nissan_1989", "Nissan_1989_2"],
        "subaru_brand_big" => vec!["subaru_brand_big", "subaru_brand_big_2"],
        "subaru_brand_big_selected" => {
            vec!["subaru_brand_big_selected", "subaru_brand_big_selected_2"]
        }
        "journal_subaru_brand" => vec!["journal_subaru_brand", "journal_subaru_brand_2"],
        "journal_subaru_brand_selected" => vec![
            "journal_subaru_brand_selected",
            "journal_subaru_brand_selected_2",
        ],
        "subaru_impreza_new" => vec![
            "subaru_impreza_new",
            "journal_subaru_22b",
            "journal_subaru_22b_2",
        ],
        "subaru_impreza_1998" => vec!["subaru_impreza_1998", "subaru_impreza_1998_2"],
        "car_selection_menu_selected" => vec!["car_selection_menu_selected", "journal_car_cursor"],
        "Toyota_2019" => vec!["Toyota_2019", "Toyota_2019_2"],
        "stage2_finish_banner" => vec!["stage2_finish_banner", "stage2_finish_banner_2"],
        _ => vec![Box::leak(clean.into_boxed_str())],
    }
}

pub fn load_template_grayscale(name: &str) -> Option<GrayImage> {
    let bytes = get_template_bytes(name)?;
    let dynamic_img = image::load_from_memory(bytes).ok()?;
    Some(dynamic_img.to_luma8())
}

use std::cell::RefCell;

struct CachedGrayFrame {
    rgb_ptr: usize,
    width: u32,
    height: u32,
    fingerprint: [u8; 12],
    gray: GrayImage,
}

thread_local! {
    static GRAY_CACHE: RefCell<Option<CachedGrayFrame>> = RefCell::new(None);
}

// Convert RGB frame to Grayscale GrayImage
pub fn rgb_to_grayscale(frame: &RgbImage) -> GrayImage {
    let rgb_ptr = frame.as_raw().as_ptr() as usize;
    let width = frame.width();
    let height = frame.height();

    // Grab first 4 pixels for fingerprint (if frame has at least 4 pixels)
    let mut fingerprint = [0u8; 12];
    let raw_bytes = frame.as_raw();
    if raw_bytes.len() >= 12 {
        fingerprint.copy_from_slice(&raw_bytes[0..12]);
    }

    GRAY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(ref c) = *cache {
            if c.rgb_ptr == rgb_ptr
                && c.width == width
                && c.height == height
                && c.fingerprint == fingerprint
            {
                return c.gray.clone();
            }
        }

        // Cache miss: convert full frame
        let mut gray = GrayImage::new(width, height);
        for (x, y, pixel) in frame.enumerate_pixels() {
            // Standard luma formula: Y = 0.299R + 0.587G + 0.114B
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            gray.put_pixel(x, y, Luma([luma]));
        }

        *cache = Some(CachedGrayFrame {
            rgb_ptr,
            width,
            height,
            fingerprint,
            gray: gray.clone(),
        });

        gray
    })
}

/// Grabs a region of interest (ROI) from a GrayImage.
pub fn crop_grayscale(img: &GrayImage, rx: u32, ry: u32, rw: u32, rh: u32) -> GrayImage {
    let mut cropped = GrayImage::new(rw, rh);
    for dy in 0..rh {
        let sy = ry + dy;
        if sy >= img.height() {
            continue;
        }
        for dx in 0..rw {
            let sx = rx + dx;
            if sx >= img.width() {
                continue;
            }
            cropped.put_pixel(dx, dy, *img.get_pixel(sx, sy));
        }
    }
    cropped
}

struct IntegralImage {
    sum: Vec<f64>,
    sum2: Vec<f64>,
    stride: usize,
}

impl IntegralImage {
    fn new(img: &GrayImage) -> Self {
        let w = img.width() as usize;
        let h = img.height() as usize;
        let stride = w + 1;
        let mut sum = vec![0.0f64; stride * (h + 1)];
        let mut sum2 = vec![0.0f64; stride * (h + 1)];

        for y in 0..h {
            let mut row_sum = 0.0f64;
            let mut row_sum2 = 0.0f64;
            for x in 0..w {
                let val = img.get_pixel(x as u32, y as u32)[0] as f64;
                row_sum += val;
                row_sum2 += val * val;

                let idx = (y + 1) * stride + (x + 1);
                let prev_row_idx = y * stride + (x + 1);

                sum[idx] = sum[prev_row_idx] + row_sum;
                sum2[idx] = sum2[prev_row_idx] + row_sum2;
            }
        }

        Self {
            sum,
            sum2,
            stride,
        }
    }

    fn get_patch_stats(&self, x: usize, y: usize, tw: usize, th: usize) -> (f64, f64) {
        let x0 = x;
        let y0 = y;
        let x1 = x + tw;
        let y1 = y + th;

        let idx00 = y0 * self.stride + x0;
        let idx01 = y0 * self.stride + x1;
        let idx10 = y1 * self.stride + x0;
        let idx11 = y1 * self.stride + x1;

        let s = self.sum[idx11] - self.sum[idx01] - self.sum[idx10] + self.sum[idx00];
        let s2 = self.sum2[idx11] - self.sum2[idx01] - self.sum2[idx10] + self.sum2[idx00];
        (s, s2)
    }
}

fn find_template_ncc_impl(
    frame: &RgbImage,
    template_name: &str,
    threshold: f32,
    region: Option<(i32, i32, i32, i32)>,
    baseline_res: (u32, u32),
    find_all: bool,
) -> Vec<(u32, u32, f32)> {
    let clean_name = template_name
        .split('/')
        .last()
        .unwrap_or(template_name)
        .replace(".png", "")
        .replace(".jpg", "");
    let actual_region = if (clean_name == "car_selection_menu_selected"
        || clean_name == "journal_car_cursor")
        && region.is_none()
    {
        Some((500, 0, 2060, 1440))
    } else {
        region
    };

    let candidates = get_template_candidates(template_name);
    let gray_frame = rgb_to_grayscale(frame);
    let frame_h = frame.height();
    let frame_w = frame.width();
    let scale = frame_h as f32 / baseline_res.1 as f32;

    let mut rx = 0;
    let mut ry = 0;
    let mut rw = frame_w;
    let mut rh = frame_h;

    if let Some((x, y, w, h)) = actual_region {
        let mut scaled_x = (x as f32 * scale) as i32;
        let mut scaled_y = (y as f32 * scale) as i32;
        let mut scaled_w = (w as f32 * scale) as i32;
        let mut scaled_h = (h as f32 * scale) as i32;

        scaled_x = scaled_x.max(0).min(frame_w as i32 - 1);
        scaled_y = scaled_y.max(0).min(frame_h as i32 - 1);
        scaled_w = scaled_w.max(1).min(frame_w as i32 - scaled_x);
        scaled_h = scaled_h.max(1).min(frame_h as i32 - scaled_y);

        rx = scaled_x as u32;
        ry = scaled_y as u32;
        rw = scaled_w as u32;
        rh = scaled_h as u32;
    }

    let search_area = crop_grayscale(&gray_frame, rx, ry, rw, rh);
    let mut matches = Vec::new();

    for cand in candidates {
        let Some(template) = load_template_grayscale(cand) else {
            continue;
        };

        let scaled_template = if (scale - 1.0).abs() > 0.01 {
            let new_w = (template.width() as f32 * scale).round() as u32;
            let new_h = (template.height() as f32 * scale).round() as u32;
            imageops::resize(
                &template,
                new_w.max(1),
                new_h.max(1),
                imageops::FilterType::Triangle,
            )
        } else {
            template
        };

        let tw = scaled_template.width();
        let th = scaled_template.height();
        if rw < tw || rh < th {
            continue;
        }

        // Determine downsample factor coarse_s
        let min_dim = tw.min(th);
        let coarse_s = if min_dim >= 40 {
            8
        } else if min_dim >= 20 {
            4
        } else if min_dim >= 10 {
            2
        } else {
            1
        };

        // Downsample for coarse search
        let dtw = (tw / coarse_s).max(1);
        let dth = (th / coarse_s).max(1);
        let ds_template = if coarse_s > 1 {
            imageops::resize(&scaled_template, dtw, dth, imageops::FilterType::Triangle)
        } else {
            scaled_template.clone()
        };

        let dsw = (search_area.width() / coarse_s).max(1);
        let dsh = (search_area.height() / coarse_s).max(1);
        let ds_search = if coarse_s > 1 {
            imageops::resize(&search_area, dsw, dsh, imageops::FilterType::Triangle)
        } else {
            search_area.clone()
        };

        if dsw < dtw || dsh < dth {
            continue;
        }

        // Perform NCC on downsampled images
        let n = (dtw * dth) as f64;
        let mut sum_t = 0.0f64;
        let mut sum_t2 = 0.0f64;
        let ds_temp_raw = ds_template.as_raw();
        for val_byte in ds_temp_raw {
            let val = *val_byte as f64;
            sum_t += val;
            sum_t2 += val * val;
        }
        let var_t = n * sum_t2 - sum_t * sum_t;
        if var_t <= 0.0 {
            continue;
        }
        let dev_t = var_t.sqrt();

        let integral = IntegralImage::new(&ds_search);

        let mut ds_matches = Vec::new();

        let ds_search_raw = ds_search.as_raw();
        let dsw_stride = ds_search.width() as usize;
        let dtw_usize = dtw as usize;
        let dth_usize = dth as usize;

        for y in 0..=(dsh - dth) {
            for x in 0..=(dsw - dtw) {
                let (sum_i, sum_i2) =
                    integral.get_patch_stats(x as usize, y as usize, dtw as usize, dth as usize);
                let var_i = n * sum_i2 - sum_i * sum_i;
                if var_i <= 0.0 {
                    continue;
                }
                let dev_i = var_i.sqrt();

                // Compute cross-correlation using direct raw pointer indexing and integer arithmetic
                let mut sum_ti_u64 = 0u64;
                unsafe {
                    let search_ptr = ds_search_raw.as_ptr();
                    let temp_ptr = ds_temp_raw.as_ptr();
                    for ty in 0..dth_usize {
                        let search_row_offset = (y as usize + ty) * dsw_stride + x as usize;
                        let temp_row_offset = ty * dtw_usize;
                        for tx in 0..dtw_usize {
                            let t_val = *temp_ptr.add(temp_row_offset + tx) as u64;
                            let i_val = *search_ptr.add(search_row_offset + tx) as u64;
                            sum_ti_u64 += t_val * i_val;
                        }
                    }
                }
                let sum_ti = sum_ti_u64 as f64;

                let num = n * sum_ti - sum_t * sum_i;
                let score = (num / (dev_t * dev_i)) as f32;

                // Slower threshold for downsampling step to reduce false negatives
                let ds_threshold = threshold - 0.15;
                if score >= ds_threshold {
                    ds_matches.push((x, y, score));
                }
            }
        }

        if ds_matches.is_empty() {
            continue;
        }

        // Sort coarse matches by score descending
        ds_matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Perform Coarse NMS to get a small set of unique peak regions
        let mut refined_candidates = Vec::new();
        for (cx, cy, score) in ds_matches {
            let mut too_close = false;
            for &(rcx, rcy, _) in &refined_candidates {
                let dx = (cx as i32 - rcx as i32).abs();
                let dy = (cy as i32 - rcy as i32).abs();
                if dx < 4 && dy < 4 {
                    too_close = true;
                    break;
                }
            }
            if !too_close {
                refined_candidates.push((cx, cy, score));
                if refined_candidates.len() >= 20 {
                    break;
                }
            }
        }

        // Refine candidates at full resolution
        let full_integral = IntegralImage::new(&search_area);
        let search_raw = search_area.as_raw();
        let temp_raw = scaled_template.as_raw();
        let sa_stride = search_area.width() as usize;
        let tw_usize = tw as usize;
        let th_usize = th as usize;

        // Precompute scaled_template stats
        let n_full = (tw * th) as f64;
        let mut sum_t_full = 0.0f64;
        let mut sum_t2_full = 0.0f64;
        for val_byte in temp_raw {
            let val = *val_byte as f64;
            sum_t_full += val;
            sum_t2_full += val * val;
        }
        let var_t_full = n_full * sum_t2_full - sum_t_full * sum_t_full;
        if var_t_full <= 0.0 {
            continue;
        }
        let dev_t_full = var_t_full.sqrt();

        for (ds_x, ds_y, _) in refined_candidates {
            let base_x = ds_x * coarse_s;
            let base_y = ds_y * coarse_s;

            let start_x = if base_x >= coarse_s {
                base_x - coarse_s
            } else {
                0
            };
            let start_y = if base_y >= coarse_s {
                base_y - coarse_s
            } else {
                0
            };
            let end_x = (base_x + coarse_s).min(search_area.width() - tw);
            let end_y = (base_y + coarse_s).min(search_area.height() - th);

            let mut best_ref_x = base_x;
            let mut best_ref_y = base_y;
            let mut best_ref_score = -1.0f32;

            for ref_y in start_y..=end_y {
                for ref_x in start_x..=end_x {
                    let (sum_i, sum_i2) = full_integral.get_patch_stats(
                        ref_x as usize,
                        ref_y as usize,
                        tw_usize,
                        th_usize,
                    );
                    let var_i = n_full * sum_i2 - sum_i * sum_i;
                    if var_i <= 0.0 {
                        continue;
                    }
                    let dev_i = var_i.sqrt();

                    let mut sum_ti_u64 = 0u64;
                    unsafe {
                        let search_ptr = search_raw.as_ptr();
                        let temp_ptr = temp_raw.as_ptr();
                        for ty in 0..th_usize {
                            let sa_row_offset = (ref_y as usize + ty) * sa_stride + ref_x as usize;
                            let temp_row_offset = ty * tw_usize;
                            for tx in 0..tw_usize {
                                let i_val = *search_ptr.add(sa_row_offset + tx) as u64;
                                let t_val = *temp_ptr.add(temp_row_offset + tx) as u64;
                                sum_ti_u64 += t_val * i_val;
                            }
                        }
                    }
                    let sum_ti = sum_ti_u64 as f64;

                    let num = n_full * sum_ti - sum_t_full * sum_i;
                    let score = (num / (dev_t_full * dev_i)) as f32;

                    if score > best_ref_score {
                        best_ref_score = score;
                        best_ref_x = ref_x;
                        best_ref_y = ref_y;
                    }
                }
            }

            if best_ref_score >= threshold {
                matches.push((
                    best_ref_x + tw / 2 + rx,
                    best_ref_y + th / 2 + ry,
                    best_ref_score,
                ));
            }
        }
    }

    if matches.is_empty() {
        return vec![];
    }

    matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    if !find_all {
        vec![matches[0]]
    } else {
        let mut filtered = Vec::new();
        for det in matches {
            let (cx, cy, score) = det;
            let mut overlap = false;
            for f in &filtered {
                let (fcx, fcy, _) = *f;
                let dx = (cx as i32 - fcx as i32).abs();
                let dy = (cy as i32 - fcy as i32).abs();
                if dx < 40 && dy < 40 {
                    overlap = true;
                    break;
                }
            }
            if !overlap {
                filtered.push((cx, cy, score));
            }
        }
        filtered
    }
}

pub fn find_template(
    frame: &RgbImage,
    template_name: &str,
    threshold: f32,
    region: Option<(i32, i32, i32, i32)>,
    baseline_res: (u32, u32),
) -> Option<(u32, u32)> {
    let res = find_template_ncc_impl(frame, template_name, threshold, region, baseline_res, false);
    if res.is_empty() {
        None
    } else {
        Some((res[0].0, res[0].1))
    }
}

pub fn find_all_matches(
    frame: &RgbImage,
    template_name: &str,
    threshold: f32,
    region: Option<(i32, i32, i32, i32)>,
    baseline_res: (u32, u32),
) -> Vec<(u32, u32, f32)> {
    find_template_ncc_impl(frame, template_name, threshold, region, baseline_res, true)
}

pub fn is_on_screen(
    frame: &RgbImage,
    template_name: &str,
    threshold: f32,
    region: Option<(i32, i32, i32, i32)>,
    baseline_res: (u32, u32),
) -> bool {
    find_template(frame, template_name, threshold, region, baseline_res).is_some()
}

pub fn find_brand_cursor_lime(frame: &RgbImage, scale: f32) -> Option<(i32, i32)> {
    // Baseline (2560x1440) parameters:
    let base_start_x = 330.0;
    let base_start_y = 336.0;
    let base_cell_w = 473.0;
    let base_cell_h = 76.0;
    let base_step_x = 469.33;
    let base_step_y = 72.0;

    let mut best_col = -1;
    let mut best_row = -1;
    let mut max_lime_count = 0;

    for row in 0..12 {
        for col in 0..4 {
            let cell_min_x = ((base_start_x + col as f32 * base_step_x) * scale) as u32;
            let cell_min_y = ((base_start_y + row as f32 * base_step_y) * scale) as u32;
            let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
            let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

            // Offset search vertically to avoid the top UI strip (ends around y=333 on 1440p)
            let y_start = cell_min_y + (4.0 * scale) as u32;
            let y_end = cell_max_y + (4.0 * scale) as u32;
            let x_start = cell_min_x.saturating_sub((4.0 * scale) as u32);
            let x_end = cell_max_x + (4.0 * scale) as u32;

            let mut lime_count = 0;
            for y in y_start..y_end {
                if y >= frame.height() { continue; }
                for x in x_start..x_end {
                    if x >= frame.width() { continue; }
                    let px = frame.get_pixel(x, y);
                    if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                        lime_count += 1;
                    }
                }
            }

            if lime_count > max_lime_count {
                max_lime_count = lime_count;
                best_col = col as i32;
                best_row = row as i32;
            }
        }
    }

    // Threshold of lime pixels to confirm presence of the cursor (e.g. 100 pixels scaled)
    let threshold = (100.0 * scale * scale) as u32;
    if max_lime_count >= threshold {
        Some((best_col, best_row))
    } else {
        None
    }
}

pub fn find_car_cursor_lime(frame: &RgbImage, scale: f32) -> Option<(i32, i32)> {
    let base_start_x = 533.0;
    let base_start_y = 270.0;
    let base_cell_w = 442.67;
    let base_cell_h = 336.0;
    let base_step_x = 442.67;
    let base_step_y = 336.0;

    let mut best_col = -1;
    let mut best_row = -1;
    let mut max_lime_count = 0;

    for row in 0..3 {
        for col in 0..4 {
            let cell_min_x = ((base_start_x + col as f32 * base_step_x) * scale) as u32;
            let cell_min_y = ((base_start_y + row as f32 * base_step_y) * scale) as u32;
            let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
            let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

            let mut lime_count = 0;
            for y in cell_min_y..cell_max_y {
                if y >= frame.height() { continue; }
                for x in cell_min_x..cell_max_x {
                    if x >= frame.width() { continue; }
                    let px = frame.get_pixel(x, y);
                    if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                        lime_count += 1;
                    }
                }
            }

            if lime_count > max_lime_count {
                max_lime_count = lime_count;
                best_col = col as i32;
                best_row = row as i32;
            }
        }
    }

    // Threshold of lime pixels to confirm presence of the cursor (e.g. 50 pixels scaled)
    let threshold = (50.0 * scale * scale) as u32;
    if max_lime_count >= threshold {
        Some((best_col, best_row))
    } else {
        None
    }
}

pub fn calculate_brand_navigation_offsets(
    frame: &RgbImage,
    target_pos: (u32, u32),
) -> Option<(i32, i32)> {
    let scale = frame.height() as f32 / 1440.0;

    let b_sx = target_pos.0 as f32 / scale;
    let b_sy = target_pos.1 as f32 / scale;

    // Target cell coordinates based on our calibrated parameters
    let target_col = ((b_sx - 566.5) / 469.33).round().max(0.0).min(3.0) as i32;
    let target_row = ((b_sy - 374.0) / 72.0).round().max(0.0).min(11.0) as i32;

    // Detect the current brand cursor position using lime pixel density
    let (cursor_col, cursor_row) = find_brand_cursor_lime(frame, scale)?;

    Some((target_col - cursor_col, target_row - cursor_row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_performance() {
        println!("Starting template matching performance test...");
        let frame = RgbImage::new(2560, 1440);
        let start = Instant::now();
        // Search for Toyota_2019 which is 24x184
        let res = find_template(&frame, "Toyota_2019.png", 0.85, None, (2560, 1440));
        println!(
            "find_template took: {} ms. Found: {:?}",
            start.elapsed().as_millis(),
            res
        );

        let start_all = Instant::now();
        let res_all = find_all_matches(&frame, "Toyota_2019.png", 0.85, None, (2560, 1440));
        println!(
            "find_all_matches took: {} ms. Found count: {}",
            start_all.elapsed().as_millis(),
            res_all.len()
        );
    }

    #[test]
    fn test_live_search() {
        println!("Initializing ScreenCapture for live test...");
        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if found {
            if let Some(frame) = capture.grab_frame() {
                println!(
                    "Frame successfully grabbed! Size: {}x{}",
                    frame.width(),
                    frame.height()
                );

                let start = Instant::now();
                let res = find_template(&frame, "Toyota_2019.png", 0.80, None, (2560, 1440));
                println!(
                    "find_template (Toyota_2019) took: {} ms. Match position: {:?}",
                    start.elapsed().as_millis(),
                    res
                );

                let start_all = Instant::now();
                let res_all = find_all_matches(&frame, "Toyota_2019.png", 0.80, None, (2560, 1440));
                println!("find_all_matches (Toyota_2019) took: {} ms. Match count: {}. Coordinates: {:?}", start_all.elapsed().as_millis(), res_all.len(), res_all);
            } else {
                println!("Error: Failed to grab frame from game window.");
            }
        } else {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
        }
    }

    #[test]
    fn test_diagnostic() {
        println!("Loading live_frame.png...");
        let frame_path = std::path::Path::new(
            "c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/scratch/live_frame.png",
        );
        if !frame_path.exists() {
            println!("live_frame.png does not exist.");
            return;
        }
        let dynamic_img = image::open(frame_path).expect("Failed to open live_frame.png");
        let rgb_frame = dynamic_img.to_rgb8();
        println!("Running find_all_matches for brand_selection_cursor...");
        let res1 = find_all_matches(&rgb_frame, "brand_selection_cursor.png", 0.50, None, (2560, 1440));
        println!("Result for brand_selection_cursor: {:?}", res1);

        println!("Running find_all_matches for journal_brand_cursor...");
        let res2 = find_all_matches(&rgb_frame, "journal_brand_cursor.png", 0.50, None, (2560, 1440));
        println!("Result for journal_brand_cursor: {:?}", res2);
    }

    #[test]
    fn test_calibrate_brand_grid() {
        println!("Initializing ScreenCapture for brand grid calibration...");
        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if !found {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
            return;
        }

        println!("Focusing game window...");
        capture.focus_game_window();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let mut frame = None;
        for attempt in 1..=15 {
            if let Some(f) = capture.grab_frame() {
                let cx = f.width() / 2;
                let cy = f.height() / 2;
                let p = f.get_pixel(cx, cy);
                let p2 = f.get_pixel(cx.saturating_sub(100), cy.saturating_sub(100));
                let p3 = f.get_pixel(cx.min(f.width() - 1), cy.min(f.height() - 1));
                
                // If any of these are not black (0,0,0), we captured the screen successfully!
                if p[0] != 0 || p[1] != 0 || p[2] != 0 || p2[0] != 0 || p2[1] != 0 || p2[2] != 0 || p3[0] != 0 || p3[1] != 0 || p3[2] != 0 {
                    frame = Some(f);
                    println!("Successfully grabbed non-black frame on attempt {}", attempt);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        let frame = match frame {
            Some(f) => f,
            None => {
                println!("Error: Failed to grab a non-black frame after 15 attempts.");
                return;
            }
        };

        println!(
            "Captured frame of size: {}x{}",
            frame.width(),
            frame.height()
        );

        let scratch_dir = std::path::Path::new("c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/scratch");
        if !scratch_dir.exists() {
            let _ = std::fs::create_dir_all(scratch_dir);
        }
        let _ = frame.save(scratch_dir.join("live_frame.png"));
        println!("Saved full frame to scratch/live_frame.png");

        println!("Running template match for brand_selection_cursor...");
        let scale = frame.height() as f32 / 1440.0;
        let t_pos1 = find_template(&frame, "brand_selection_cursor.png", 0.70, None, (2560, 1440));
        let t_pos2 = find_template(&frame, "journal_brand_cursor.png", 0.70, None, (2560, 1440));
        println!("  - brand_selection_cursor.png: raw={:?}, scaled=({:?})", 
                 t_pos1, t_pos1.map(|p| (p.0 as f32 / scale, p.1 as f32 / scale)));
        println!("  - journal_brand_cursor.png: raw={:?}, scaled=({:?})", 
                 t_pos2, t_pos2.map(|p| (p.0 as f32 / scale, p.1 as f32 / scale)));

        let mut highlight_img = frame.clone();

        // Baseline (2560x1440) parameters:
        let base_start_x = 330.0;
        let base_start_y = 336.0;
        let base_cell_w = 473.0;
        let base_cell_h = 76.0;
        let base_step_x = 469.33;
        let base_step_y = 72.0;

        let scale = frame.height() as f32 / 1440.0;

        // Scan all 48 cells for lime pixels
        let mut cell_lime_counts = Vec::new();
        let mut max_lime_count = 0;
        let mut best_col = -1;
        let mut best_row = -1;

        for row in 0..12 {
            for col in 0..4 {
                let cell_min_x = ((base_start_x + col as f32 * base_step_x) * scale) as u32;
                let cell_min_y = ((base_start_y + row as f32 * base_step_y) * scale) as u32;
                let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
                let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

                // Adjust search area vertically to avoid the top UI strip (y=237..333)
                // We offset by 4.0 scaled pixels to be below the strip safely
                let y_start = cell_min_y + (4.0 * scale) as u32;
                let y_end = cell_max_y + (4.0 * scale) as u32;
                let x_start = cell_min_x.saturating_sub((4.0 * scale) as u32);
                let x_end = cell_max_x + (4.0 * scale) as u32;

                let mut lime_count = 0;
                for y in y_start..y_end {
                    if y >= frame.height() { continue; }
                    for x in x_start..x_end {
                        if x >= frame.width() { continue; }
                        let px = frame.get_pixel(x, y);
                        // Use the official is_lime_pixel from core-bot crate stages module!
                        if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                            lime_count += 1;
                        }
                    }
                }

                cell_lime_counts.push((col, row, lime_count));
                if lime_count > max_lime_count {
                    max_lime_count = lime_count;
                    best_col = col as i32;
                    best_row = row as i32;
                }
            }
        }

        // Draw lime pixels in red on highlight_img for visualization
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let px = frame.get_pixel(x, y);
                if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                    highlight_img.put_pixel(x, y, image::Rgb([255, 0, 0]));
                }
            }
        }

        let _ = highlight_img.save(scratch_dir.join("highlighted_lime.png"));
        println!("Saved highlighted mask to scratch/highlighted_lime.png");

        println!("=== LIME DETECTION RESULTS ===");
        if best_col != -1 && max_lime_count > 50 {
            println!("DETECTION SUCCESS!");
            println!("Selected Cell: Column {}, Row {}", best_col, best_row);
            println!("Lime Pixel Count in Selected Cell: {}", max_lime_count);
        } else {
            println!("DETECTION FAILED: No cell has significant lime pixels (max count: {}).", max_lime_count);
        }

        // Print significant cells (count > 5) sorted by count descending
        cell_lime_counts.sort_by(|a, b| b.2.cmp(&a.2));
        println!("Top cells by lime pixel count:");
        for (c, r, count) in cell_lime_counts.iter().take(5) {
            if *count > 5 {
                println!("  Cell (Col {}, Row {}): {} pixels", c, r, count);
            }
        }
        println!("==============================");

        // Draw the calibrated 4x12 grid overlay
        let mut grid_img = frame.clone();
        let border_color = image::Rgb([0, 0, 255]); // blue grid
        let active_color = image::Rgb([255, 0, 0]); // red active cell

        for row in 0..12 {
            for col in 0..4 {
                let cell_min_x = ((base_start_x + col as f32 * base_step_x) * scale) as u32;
                let cell_min_y = ((base_start_y + row as f32 * base_step_y) * scale) as u32;
                let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
                let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

                let color = if col as i32 == best_col && row as i32 == best_row {
                    active_color
                } else {
                    border_color
                };

                // Draw a hollow rectangle for each cell
                // We'll draw 2-pixel thick border for visibility
                for thickness in 0..2 {
                    let mx = cell_min_x.saturating_add(thickness);
                    let my = cell_min_y.saturating_add(thickness);
                    let rx = cell_max_x.saturating_sub(thickness);
                    let ry = cell_max_y.saturating_sub(thickness);

                    let w_img = grid_img.width();
                    let h_img = grid_img.height();
                    
                    for x in mx..=rx {
                        if x < w_img {
                            if my < h_img { grid_img.put_pixel(x, my, color); }
                            if ry < h_img { grid_img.put_pixel(x, ry, color); }
                        }
                    }
                    for y in my..=ry {
                        if y < h_img {
                            if mx < w_img { grid_img.put_pixel(mx, y, color); }
                            if rx < w_img { grid_img.put_pixel(rx, y, color); }
                        }
                    }
                }
            }
        }

        let out_grid_path = std::path::Path::new("c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/src-tauri/scratch/grid_overlay.png");
        let _ = grid_img.save(&out_grid_path);
        println!("Saved grid overlay image (with red active cell) to: {:?}", out_grid_path);
    }

    #[test]
    fn test_live_subaru_navigation_path() {
        println!("Initializing ScreenCapture for live Subaru navigation test...");
        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if !found {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
            return;
        }

        // Focus window and grab frame
        capture.focus_game_window();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let mut frame = None;
        for _attempt in 1..=10 {
            if let Some(f) = capture.grab_frame() {
                let cx = f.width() / 2;
                let cy = f.height() / 2;
                let p = f.get_pixel(cx, cy);
                if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                    frame = Some(f);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let frame = match frame {
            Some(f) => f,
            None => {
                println!("Error: Failed to grab a non-black frame.");
                return;
            }
        };

        let baseline_res = (2560, 1440);
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        // 1. Search for Subaru template
        println!("Searching for template 'journal_subaru_brand.png'...");
        let subaru_pos = find_template(&frame, "journal_subaru_brand.png", 0.70, None, baseline_res);
        println!("  Template match result: {:?}", subaru_pos);

        let (sx, sy) = match subaru_pos {
            Some(s) => s,
            None => {
                println!("Error: 'journal_subaru_brand.png' not found on the screen. Verify it's visible.");
                return;
            }
        };

        // 2. Search for active brand cursor using lime outline
        println!("Detecting current brand selection cursor (lime pixel scanner)...");
        let cursor_cell = find_brand_cursor_lime(&frame, scale);
        println!("  Cursor cell result: {:?}", cursor_cell);

        let (cursor_col, cursor_row) = match cursor_cell {
            Some(c) => c,
            None => {
                println!("Error: Active cursor not found using lime pixel outline.");
                return;
            }
        };

        // 3. Compute target cell
        let b_sx = sx as f32 / scale;
        let b_sy = sy as f32 / scale;
        let target_col = ((b_sx - 566.5) / 469.33).round().max(0.0).min(3.0) as i32;
        let target_row = ((b_sy - 374.0) / 72.0).round().max(0.0).min(11.0) as i32;

        println!("=== NAVIGATION PATH CALCULATION ===");
        println!("Detected Current Cursor: Column {}, Row {}", cursor_col, cursor_row);
        println!("Target Brand Cell (Subaru): Column {}, Row {}", target_col, target_row);
        println!("Navigation Required:");
        println!("  Columns diff: {}", target_col - cursor_col);
        println!("  Rows diff: {}", target_row - cursor_row);
        println!("===================================");
    }

    #[test]
    fn test_calibrate_car_grid() {
        println!("Initializing ScreenCapture for car selection grid calibration...");
        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if !found {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
            return;
        }

        // Focus window and grab frame
        capture.focus_game_window();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let mut frame = None;
        for _attempt in 1..=10 {
            if let Some(f) = capture.grab_frame() {
                let cx = f.width() / 2;
                let cy = f.height() / 2;
                let p = f.get_pixel(cx, cy);
                if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                    frame = Some(f);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let frame = match frame {
            Some(f) => f,
            None => {
                println!("Error: Failed to grab a non-black frame.");
                return;
            }
        };

        let baseline_res = (2560, 1440);
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        let scratch_dir = std::path::Path::new("c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/scratch");
        let _ = frame.save(scratch_dir.join("car_live_frame.png"));

        println!("Searching for template 'car_selection_menu_selected.png'...");
        let cursor_pos = find_template(&frame, "car_selection_menu_selected.png", 0.70, None, baseline_res);
        
        if let Some(pos) = cursor_pos {
            let b_cx = pos.0 as f32 / scale;
            let b_cy = pos.1 as f32 / scale;

            println!("=== CAR CURSOR DETECTED ===");
            println!("Raw match center: {:?}", pos);
            println!("Scaled match center (2560x1440): ({:.1}, {:.1})", b_cx, b_cy);
            println!("===========================");

            // Save crop
            let pad = 50;
            let crop_x = (pos.0.saturating_sub(pad)).min(frame.width() - 1);
            let crop_y = (pos.1.saturating_sub(pad)).min(frame.height() - 1);
            let crop_w = (100 + pad * 2).min(frame.width() - crop_x);
            let crop_h = (100 + pad * 2).min(frame.height() - crop_y);

            let mut cropped = image::RgbImage::new(crop_w, crop_h);
            for dy in 0..crop_h {
                for dx in 0..crop_w {
                    let pixel = frame.get_pixel(crop_x + dx, crop_y + dy);
                    cropped.put_pixel(dx, dy, *pixel);
                }
            }
            let _ = cropped.save(scratch_dir.join("car_detected_cursor.png"));
            println!("Saved crop to scratch/car_detected_cursor.png");
        } else {
            println!("=== CAR CURSOR NOT DETECTED ===");
            println!("Verify that 'car_selection_menu_selected.png' is visible on screen.");
        }
    }

    #[test]
    fn test_calibrate_car_grid_via_class_b() {
        println!("Initializing ScreenCapture for Class B car selection grid calibration...");
        
        // Load and print template sizes
        if let Some(cursor_img) = load_template_grayscale("car_selection_menu_selected") {
            println!("Template 'car_selection_menu_selected.png' size: {}x{}", cursor_img.width(), cursor_img.height());
        }
        if let Some(b_img) = load_template_grayscale("car_class_b") {
            println!("Template 'car_class_b.png' size: {}x{}", b_img.width(), b_img.height());
        }

        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if !found {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
            return;
        }

        // Focus window and grab frame
        capture.focus_game_window();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let mut frame = None;
        for _attempt in 1..=10 {
            if let Some(f) = capture.grab_frame() {
                let cx = f.width() / 2;
                let cy = f.height() / 2;
                let p = f.get_pixel(cx, cy);
                if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                    frame = Some(f);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let frame = match frame {
            Some(f) => f,
            None => {
                println!("Error: Failed to grab a non-black frame.");
                return;
            }
        };

        let baseline_res = (2560, 1440);
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        let scratch_dir = std::path::Path::new("c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/src-tauri/scratch");
        let _ = frame.save(scratch_dir.join("car_live_frame.png"));

        println!("Running find_all_matches for 'car_class_b.png'...");
        // Search in the main grid region: (500, 0, 2060, 1440) scaled to save time and reduce background noise
        let search_region = Some((500, 0, 2060, 1440));
        let matches = find_all_matches(&frame, "car_class_b.png", 0.70, search_region, baseline_res);
        println!("Total raw matches found: {}", matches.len());

        if matches.is_empty() {
            println!("Error: No 'car_class_b.png' matches found. Verify that Class B cars are on screen.");
            return;
        }

        // Convert coordinates to baseline resolution (2560x1440) and filter false positives
        // Grid region is Y >= 400.0, and score >= 0.81
        let scaled_matches: Vec<(f32, f32, f32)> = matches.iter()
            .map(|&(x, y, score)| (x as f32 / scale, y as f32 / scale, score))
            .filter(|&(_sx, sy, score)| sy >= 400.0 && score >= 0.81)
            .collect();

        println!("Filtered matches in grid region (total {}):", scaled_matches.len());
        for (idx, &(sx, sy, score)) in scaled_matches.iter().enumerate() {
            println!("  Match {}: scaled=({:.1}, {:.1}), score={:.4}", idx, sx, sy, score);
        }

        // Draw circles/rectangles on overlay image around detected matches
        let mut overlay_img = frame.clone();
        let red_color = image::Rgb([255, 0, 0]);

        for &(sx, sy, _) in &scaled_matches {
            let rx = (sx * scale) as i32;
            let ry = (sy * scale) as i32;

            // Draw a small 10x10 crosshair/rect around matches
            for thickness in 0..2 {
                crate::stages::draw_rect(&mut overlay_img, rx - 10 + thickness, ry - 10 + thickness, 20 - 2 * thickness, 20 - 2 * thickness, red_color);
            }
        }

        let _ = overlay_img.save(scratch_dir.join("car_matches_detected.png"));
        println!("Saved matched template visualization to scratch/car_matches_detected.png");

        // Sort matches by X and Y to find unique columns and rows
        let mut x_coords: Vec<f32> = scaled_matches.iter().map(|m| m.0).collect();
        let mut y_coords: Vec<f32> = scaled_matches.iter().map(|m| m.1).collect();

        // Group X coordinates into unique columns (difference < 50px)
        x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut columns: Vec<f32> = Vec::new();
        for x in x_coords {
            if columns.is_empty() || x - columns.last().unwrap() > 50.0 {
                columns.push(x);
            } else {
                // Average the coordinate
                let last = columns.len() - 1;
                columns[last] = (columns[last] + x) / 2.0;
            }
        }

        // Group Y coordinates into unique rows (difference < 50px)
        y_coords.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut rows: Vec<f32> = Vec::new();
        for y in y_coords {
            if rows.is_empty() || y - rows.last().unwrap() > 50.0 {
                rows.push(y);
            } else {
                // Average the coordinate
                let last = rows.len() - 1;
                rows[last] = (rows[last] + y) / 2.0;
            }
        }

        println!("=== DETECTED GRID LAYOUT ===");
        println!("Detected Columns X coordinates: {:?}", columns);
        println!("Detected Rows Y coordinates: {:?}", rows);

        let mut final_col_step = 442.67;
        let mut final_row_step = 336.0;

        if columns.len() >= 2 {
            let col_step = (columns.last().unwrap() - columns.first().unwrap()) / (columns.len() - 1) as f32;
            println!("  Estimated Horizontal Column Step (W): {:.2} pixels", col_step);
            final_col_step = col_step;
        }
        if rows.len() >= 2 {
            let row_step = (rows.last().unwrap() - rows.first().unwrap()) / (rows.len() - 1) as f32;
            println!("  Estimated Vertical Row Step (H): {:.2} pixels", row_step);
            final_row_step = row_step;
        }

        // Calculate and draw the complete 4x3 grid overlay on final_grid_img based on our calibration!
        // We know Row 0, Column 0 cursor top-left is at center (543.0, 283.0).
        // Let's draw the grid!
        let mut final_grid_img = frame.clone();
        let border_color = image::Rgb([0, 0, 255]); // blue grid
        
        // Let's calculate cell start X and Y based on the Class B badge center at Col 0 Row 0 (which is 879.0, 577.0)
        // Since Class B template size is, say, 42x42 (let's verify from run), and it's near the bottom right of the cell.
        // Let's align cell top-left so that:
        // cell_min_x = Col_0_Class_B_X - offset_x
        // cell_min_y = Row_0_Class_B_Y - offset_y
        //
        // Let's use the cursor match (543.0, 283.0) as the reference for top-left.
        // Let's assume the cell starts at X = 543.0 - 90.0 = 453.0
        // and Y = 283.0 - 15.0 = 268.0
        // Let's verify how it looks. We will set the cell boundaries precisely.
        let start_x = 533.0;
        let start_y = 270.0;

        let cell_w = final_col_step; 
        let cell_h = final_row_step; 

        for r in 0..3 {
            for c in 0..4 {
                let cell_min_x = ((start_x + c as f32 * final_col_step) * scale) as u32;
                let cell_min_y = ((start_y + r as f32 * final_row_step) * scale) as u32;
                let cell_max_x = (cell_min_x as f32 + cell_w * scale) as u32;
                let cell_max_y = (cell_min_y as f32 + cell_h * scale) as u32;

                for thickness in 0..2 {
                    let mx = cell_min_x.saturating_add(thickness);
                    let my = cell_min_y.saturating_add(thickness);
                    let rx = cell_max_x.saturating_sub(thickness);
                    let ry = cell_max_y.saturating_sub(thickness);

                    let w_img = final_grid_img.width();
                    let h_img = final_grid_img.height();
                    
                    for x in mx..=rx {
                        if x < w_img {
                            if my < h_img { final_grid_img.put_pixel(x, my, border_color); }
                            if ry < h_img { final_grid_img.put_pixel(x, ry, border_color); }
                        }
                    }
                    for y in my..=ry {
                        if y < h_img {
                            if mx < w_img { final_grid_img.put_pixel(mx, y, border_color); }
                            if rx < w_img { final_grid_img.put_pixel(rx, y, border_color); }
                        }
                    }
                }
            }
        }

        let out_grid_path = scratch_dir.join("car_grid_overlay.png");
        let _ = final_grid_img.save(&out_grid_path);
        println!("Saved car grid overlay image to: {:?}", out_grid_path);
    }

    #[test]
    fn test_detect_car_cursor_lime() {
        println!("Initializing ScreenCapture for car selection lime cursor detection...");
        let mut capture = crate::capture::ScreenCapture::new();
        let found = capture.find_game_window();
        println!("Game window 'Forza Horizon 6' found: {}", found);
        if !found {
            println!("Error: Game window 'Forza Horizon 6' is not running or not found.");
            return;
        }

        // Focus window and grab frame
        capture.focus_game_window();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let mut frame = None;
        for _attempt in 1..=10 {
            if let Some(f) = capture.grab_frame() {
                let cx = f.width() / 2;
                let cy = f.height() / 2;
                let p = f.get_pixel(cx, cy);
                if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                    frame = Some(f);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let frame = match frame {
            Some(f) => f,
            None => {
                println!("Error: Failed to grab a non-black frame.");
                return;
            }
        };

        let scale = frame.height() as f32 / 1440.0;
        let scratch_dir = std::path::Path::new("c:/Users/PC/Documents/FH6-Wheelspin-Farm-Bot/src-tauri/scratch");

        let base_start_x = 533.0;
        let base_start_y = 270.0;
        let base_cell_w = 442.67;
        let base_cell_h = 336.0;
        let base_step_x = 442.67;
        let base_step_y = 336.0;

        let mut cell_lime_counts = Vec::new();
        let mut max_lime_count = 0;
        let mut best_col = -1;
        let mut best_row = -1;

        for row in 0..3 {
            for col in 0..4 {
                let cell_min_x = ((base_start_x + col as f32 * base_step_x) * scale) as u32;
                let cell_min_y = ((base_start_y + row as f32 * base_step_y) * scale) as u32;
                let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
                let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

                let mut lime_count = 0;
                for y in cell_min_y..cell_max_y {
                    if y >= frame.height() { continue; }
                    for x in cell_min_x..cell_max_x {
                        if x >= frame.width() { continue; }
                        let px = frame.get_pixel(x, y);
                        if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                            lime_count += 1;
                        }
                    }
                }

                cell_lime_counts.push((col, row, lime_count));
                if lime_count > max_lime_count {
                    max_lime_count = lime_count;
                    best_col = col as i32;
                    best_row = row as i32;
                }
            }
        }

        println!("=== LIME DETECTOR RESULTS ===");
        let threshold = (50.0 * scale * scale) as u32;
        if best_col != -1 && max_lime_count >= threshold {
            println!("SUCCESS: Detected Car Selection Cursor at Col {}, Row {} (lime count: {})", best_col, best_row, max_lime_count);
        } else {
            println!("FAILED: No cell has sufficient lime pixels (max count: {}, threshold: {})", max_lime_count, threshold);
        }

        // Print all cells count
        for (c, r, count) in &cell_lime_counts {
            println!("  Cell (Col {}, Row {}): {} lime pixels", c, r, count);
        }
        println!("=============================");

        // Create overlay diagnostic image
        let mut diag_img = frame.clone();
        let border_color = image::Rgb([0, 0, 255]); // blue grid
        let active_color = image::Rgb([0, 255, 0]); // green active cell

        // Draw lime pixels in magenta for visualization
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let px = frame.get_pixel(x, y);
                if crate::stages::is_lime_pixel(px[0], px[1], px[2]) {
                    diag_img.put_pixel(x, y, image::Rgb([255, 0, 255]));
                }
            }
        }

        for r in 0..3 {
            for c in 0..4 {
                let cell_min_x = ((base_start_x + c as f32 * base_step_x) * scale) as u32;
                let cell_min_y = ((base_start_y + r as f32 * base_step_y) * scale) as u32;
                let cell_max_x = (cell_min_x as f32 + base_cell_w * scale) as u32;
                let cell_max_y = (cell_min_y as f32 + base_cell_h * scale) as u32;

                let color = if c as i32 == best_col && r as i32 == best_row && max_lime_count >= threshold {
                    active_color
                } else {
                    border_color
                };

                for thickness in 0..3 {
                    let mx = cell_min_x.saturating_add(thickness);
                    let my = cell_min_y.saturating_add(thickness);
                    let rx = cell_max_x.saturating_sub(thickness);
                    let ry = cell_max_y.saturating_sub(thickness);

                    let w_img = diag_img.width();
                    let h_img = diag_img.height();
                    
                    for x in mx..=rx {
                        if x < w_img {
                            if my < h_img { diag_img.put_pixel(x, my, color); }
                            if ry < h_img { diag_img.put_pixel(x, ry, color); }
                        }
                    }
                    for y in my..=ry {
                        if y < h_img {
                            if mx < w_img { diag_img.put_pixel(mx, y, color); }
                            if rx < w_img { diag_img.put_pixel(rx, y, color); }
                        }
                    }
                }
            }
        }

        let out_path = scratch_dir.join("car_cursor_lime_detected.png");
        let _ = diag_img.save(&out_path);
        println!("Saved diagnostic image to: {:?}", out_path);
    }
}

