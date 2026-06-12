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

// Convert RGB frame to Grayscale GrayImage
pub fn rgb_to_grayscale(frame: &RgbImage) -> GrayImage {
    let mut gray = GrayImage::new(frame.width(), frame.height());
    for (x, y, pixel) in frame.enumerate_pixels() {
        // Standard luma formula: Y = 0.299R + 0.587G + 0.114B
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;
        let luma = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
        gray.put_pixel(x, y, Luma([luma]));
    }
    gray
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

        let tw = scaled_template.width();
        let th = scaled_template.height();
        if rw < tw || rh < th {
            continue;
        }

        let search_area = crop_grayscale(&gray_frame, rx, ry, rw, rh);

        // Determine downsample factor coarse_s
        let min_dim = tw.min(th);
        let coarse_s = if min_dim >= 64 {
            8
        } else if min_dim >= 32 {
            4
        } else if min_dim >= 16 {
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

                // Compute cross-correlation using direct raw buffer indexing
                let mut sum_ti = 0.0f64;
                for ty in 0..dth_usize {
                    let search_offset = (y as usize + ty) * dsw_stride + x as usize;
                    let temp_offset = ty * dtw_usize;
                    for tx in 0..dtw_usize {
                        let t_val = ds_temp_raw[temp_offset + tx] as f64;
                        let i_val = ds_search_raw[search_offset + tx] as f64;
                        sum_ti += t_val * i_val;
                    }
                }

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

                    let mut sum_ti = 0.0f64;
                    for ty in 0..th_usize {
                        let sa_row_offset = (ref_y as usize + ty) * sa_stride + ref_x as usize;
                        let temp_row_offset = ty * tw_usize;
                        for tx in 0..tw_usize {
                            let i_val = search_raw[sa_row_offset + tx] as f64;
                            let t_val = temp_raw[temp_row_offset + tx] as f64;
                            sum_ti += t_val * i_val;
                        }
                    }

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
        println!("Running find_all_matches on live_frame...");
        let res = find_all_matches(&rgb_frame, "Toyota_2019.png", 0.70, None, (2560, 1440));
        println!("Result of find_all_matches: {:?}", res);
    }
}
