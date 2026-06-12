use std::time::{Instant, Duration};
use std::sync::atomic::Ordering;
use crate::state_machine::{BotFSMContext, FSMState};

pub mod nav_to_stage1;
pub mod stage1_colossus;
pub mod nav_to_stage2;
pub mod stage2_skill_points;
pub mod nav_to_stage3;
pub mod stage3_buy_cars;
pub mod nav_to_stage4;
pub mod stage4_spend_sp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StageResult {
    Success,
    Skipped,
    Failed,
}

// Global FSM run loop called by state_machine background thread
pub fn run_loop(ctx: &BotFSMContext) {
    let (stages_enabled, loop_count) = {
        let cfg = ctx.config.lock().unwrap();
        (cfg.stages_enabled.clone(), cfg.loop_count)
    };

    let only_stage1 = stages_enabled.get("stage1").copied().unwrap_or(true)
        && !stages_enabled.get("stage2").copied().unwrap_or(false)
        && !stages_enabled.get("stage3").copied().unwrap_or(false)
        && !stages_enabled.get("stage4").copied().unwrap_or(false);

    let only_stage2 = stages_enabled.get("stage2").copied().unwrap_or(true)
        && !stages_enabled.get("stage1").copied().unwrap_or(false)
        && !stages_enabled.get("stage3").copied().unwrap_or(false)
        && !stages_enabled.get("stage4").copied().unwrap_or(false);

    let only_stage3 = stages_enabled.get("stage3").copied().unwrap_or(true)
        && !stages_enabled.get("stage1").copied().unwrap_or(false)
        && !stages_enabled.get("stage2").copied().unwrap_or(false)
        && !stages_enabled.get("stage4").copied().unwrap_or(false);

    let only_stage4 = stages_enabled.get("stage4").copied().unwrap_or(true)
        && !stages_enabled.get("stage1").copied().unwrap_or(false)
        && !stages_enabled.get("stage2").copied().unwrap_or(false)
        && !stages_enabled.get("stage3").copied().unwrap_or(false);

    let max_cycles = if only_stage1 || only_stage2 || only_stage3 || only_stage4 {
        1
    } else {
        loop_count
    };

    let only_single = only_stage2 || only_stage3 || only_stage4;

    while !ctx.is_stop_requested.load(Ordering::Relaxed) {
        let cycle = {
            let mut cur_cycle = ctx.current_cycle.lock().unwrap();
            *cur_cycle += 1;
            *cur_cycle
        };

        ctx.logger.info(&format!("--- Cycle {} started ---", cycle));
        ctx.set_state(FSMState::Running);

        let mut current_stage = if only_single {
            if only_stage2 {
                "nav_to_stage2".to_string()
            } else if only_stage3 {
                "nav_to_stage3".to_string()
            } else {
                "nav_to_stage4".to_string()
            }
        } else {
            "nav_to_stage1".to_string()
        };

        loop {
            if ctx.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }
            ctx.wait_if_paused();
            if ctx.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }

            let is_enabled = if current_stage.starts_with("nav_to_") {
                let target_stage = &current_stage["nav_to_".len()..];
                stages_enabled.get(target_stage).copied().unwrap_or(true)
            } else {
                stages_enabled.get(&current_stage).copied().unwrap_or(true)
            };

            if !is_enabled {
                ctx.logger.info(&format!("[{}] skipped (disabled in config)", current_stage));
                if let Some(next) = get_next_stage_transition(&current_stage, StageResult::Skipped) {
                    current_stage = next;
                    continue;
                } else {
                    break;
                }
            }

            let state_enum = match current_stage.as_str() {
                "stage1" => FSMState::Stage1,
                "stage2" => FSMState::Stage2,
                "stage3" => FSMState::Stage3,
                "stage4" => FSMState::Stage4,
                _ => FSMState::Running,
            };

            ctx.set_state(state_enum);
            ctx.logger.info(&format!("[{}] starting", current_stage));

            let result = run_stage_by_name(ctx, &current_stage);
            ctx.logger.info(&format!("[{}] completed with result: {:?}", current_stage, result));

            match result {
                StageResult::Success => {
                    if let Some(next) = get_next_stage_transition(&current_stage, StageResult::Success) {
                        current_stage = next;
                    } else {
                        break;
                    }
                }
                StageResult::Skipped => {
                    if let Some(next) = get_next_stage_transition(&current_stage, StageResult::Skipped) {
                        current_stage = next;
                    } else {
                        break;
                    }
                }
                StageResult::Failed => {
                    ctx.set_state(FSMState::Error);
                    if let Some(ref handle) = ctx.app_handle {
                        use serde_json::json;
                        use tauri::Emitter;
                        let _ = handle.emit("error", json!({
                            "type": "error",
                            "stage": current_stage,
                            "message": format!("Stage {} failed during execution.", current_stage)
                        }));
                    }
                    ctx.is_stop_requested.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }

        ctx.logger.info(&format!("--- Cycle {} complete ---", cycle));

        if max_cycles > 0 && cycle as i32 >= max_cycles {
            ctx.logger.info(&format!("Reached target cycle count ({}). Stopping.", max_cycles));
            break;
        }
    }

    ctx.set_state(FSMState::Idle);
}

fn get_next_stage_transition(current_stage: &str, result: StageResult) -> Option<String> {
    match (current_stage, result) {
        ("nav_to_stage1", StageResult::Success) => Some("stage1".to_string()),
        ("nav_to_stage1", StageResult::Skipped) => Some("nav_to_stage2".to_string()),
        ("stage1", StageResult::Success) | ("stage1", StageResult::Skipped) => Some("nav_to_stage2".to_string()),
        ("nav_to_stage2", StageResult::Success) => Some("stage2".to_string()),
        ("nav_to_stage2", StageResult::Skipped) => Some("nav_to_stage3".to_string()),
        ("stage2", StageResult::Success) | ("stage2", StageResult::Skipped) => Some("nav_to_stage3".to_string()),
        ("nav_to_stage3", StageResult::Success) => Some("stage3".to_string()),
        ("nav_to_stage3", StageResult::Skipped) => Some("nav_to_stage4".to_string()),
        ("stage3", StageResult::Success) | ("stage3", StageResult::Skipped) => Some("nav_to_stage4".to_string()),
        ("nav_to_stage4", StageResult::Success) => Some("stage4".to_string()),
        ("nav_to_stage4", StageResult::Skipped) => None,
        ("stage4", StageResult::Success) | ("stage4", StageResult::Skipped) => None,
        _ => None,
    }
}

pub fn run_stage_by_name(ctx: &BotFSMContext, name: &str) -> StageResult {
    match name {
        "nav_to_stage1" => nav_to_stage1::run(ctx),
        "stage1" => stage1_colossus::run(ctx),
        "nav_to_stage2" => nav_to_stage2::run(ctx),
        "stage2" => stage2_skill_points::run(ctx),
        "nav_to_stage3" => nav_to_stage3::run(ctx),
        "stage3" => stage3_buy_cars::run(ctx),
        "nav_to_stage4" => nav_to_stage4::run(ctx),
        "stage4" => stage4_spend_sp::run(ctx),
        _ => StageResult::Failed,
    }
}

// ---------------------------------------------------------------------------
// Grayscale & HSV Helpers
// ---------------------------------------------------------------------------

pub(crate) fn rgb_to_opencv_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;

    let max = r_f.max(g_f).max(b_f);
    let min = r_f.min(g_f).min(b_f);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { (delta / max) * 255.0 };

    let mut h = if delta == 0.0 {
        0.0
    } else if (max - r_f).abs() < 0.0001 {
        60.0 * ((g_f - b_f) / delta)
    } else if (max - g_f).abs() < 0.0001 {
        60.0 * (((b_f - r_f) / delta) + 2.0)
    } else {
        60.0 * (((r_f - g_f) / delta) + 4.0)
    };

    if h < 0.0 {
        h += 360.0;
    }

    let h_opencv = (h / 2.0).round().min(180.0) as u8;
    let s_opencv = s.round().min(255.0) as u8;
    let v_opencv = v.round().min(255.0) as u8;

    (h_opencv, s_opencv, v_opencv)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CellState {
    Empty,
    Locked,
    Available,
    Purchased,
}

pub(crate) fn get_cell_state(frame: &image::RgbImage, c: usize, r: usize, scale: f32, empty_cells: &std::collections::HashSet<(usize, usize)>) -> CellState {
    if empty_cells.contains(&(c, r)) {
        return CellState::Empty;
    }

    const TALENT_GRID_START_X: f32 = 500.0;
    const TALENT_GRID_START_Y: f32 = 320.0;
    const TALENT_CELL_W: f32 = 154.0;
    const TALENT_CELL_H: f32 = 154.0;

    let cell_center_x = TALENT_GRID_START_X + c as f32 * TALENT_CELL_W;
    let cell_center_y = TALENT_GRID_START_Y + r as f32 * TALENT_CELL_H;

    let screen_cx = (cell_center_x * scale) as i32;
    let screen_cy = (cell_center_y * scale) as i32;

    let frame_h = frame.height() as i32;
    let frame_w = frame.width() as i32;
    let crop_size = (50.0 * scale) as i32;

    let x1 = (screen_cx - crop_size / 2).max(0).min(frame_w - 1);
    let y1 = (screen_cy - crop_size / 2).max(0).min(frame_h - 1);
    let x2 = (screen_cx + crop_size / 2).max(1).min(frame_w);
    let y2 = (screen_cy + crop_size / 2).max(1).min(frame_h);

    if x2 <= x1 || y2 <= y1 {
        return CellState::Locked;
    }

    let tw = (x2 - x1) as u32;
    let th = (y2 - y1) as u32;
    let total_pixels = tw * th;

    let mut pink_count = 0;
    let mut white_count = 0;

    for dy in 0..th {
        for dx in 0..tw {
            let px = frame.get_pixel((x1 as u32) + dx, (y1 as u32) + dy);
            let (h, s, v) = rgb_to_opencv_hsv(px[0], px[1], px[2]);

            if h >= 140 && h <= 175 && s >= 80 && v >= 80 {
                pink_count += 1;
            }
            if s <= 45 && v >= 195 {
                white_count += 1;
            }
        }
    }

    let pink_ratio = pink_count as f32 / total_pixels as f32;
    let white_ratio = white_count as f32 / total_pixels as f32;

    if pink_ratio > 0.12 {
        CellState::Purchased
    } else if white_ratio > 0.15 {
        CellState::Available
    } else {
        CellState::Locked
    }
}

pub(crate) fn check_autopilot_color(ctx: &BotFSMContext, frame: &image::RgbImage) -> bool {
    let baseline_res = {
        let cfg = ctx.config.lock().unwrap();
        cfg.baseline_resolution
    };
    let pos = match crate::vision::find_template(frame, "autopilot_driving.png", 0.80, None, baseline_res) {
        Some(p) => p,
        None => return false,
    };

    let cx = pos.0;
    let cy = pos.1;

    let template = match crate::vision::load_template_grayscale("autopilot_driving") {
        Some(t) => t,
        None => return false,
    };

    let orig_w = template.width();
    let orig_h = template.height();

    let frame_h = frame.height();
    let frame_w = frame.width();
    let scale = frame_h as f32 / baseline_res.1 as f32;

    let tw = ((orig_w as f32 * scale).round() as u32).max(1);
    let th = ((orig_h as f32 * scale).round() as u32).max(1);

    let x = (cx as i32 - (tw as i32) / 2).max(0).min(frame_w as i32 - tw as i32) as u32;
    let y = (cy as i32 - (th as i32) / 2).max(0).min(frame_h as i32 - th as i32) as u32;

    let mut in_range_count = 0;
    let total_pixels = tw * th;

    for dy in 0..th {
        for dx in 0..tw {
            let px = frame.get_pixel(x + dx, y + dy);
            let (h, s, v) = rgb_to_opencv_hsv(px[0], px[1], px[2]);
            if h >= 80 && h <= 105 && s >= 80 && v >= 80 {
                in_range_count += 1;
            }
        }
    }

    let ratio = in_range_count as f32 / total_pixels as f32;
    ratio >= 0.08
}

// ---------------------------------------------------------------------------
// Vision Wrapper Helpers
// ---------------------------------------------------------------------------

pub(crate) fn is_on_screen(ctx: &BotFSMContext, frame: &image::RgbImage, template_name: &str, threshold: f32, region: Option<(i32, i32, i32, i32)>) -> bool {
    let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
    crate::vision::is_on_screen(frame, template_name, threshold, region, baseline_res)
}

pub(crate) fn find_template(ctx: &BotFSMContext, frame: &image::RgbImage, template_name: &str, threshold: f32, region: Option<(i32, i32, i32, i32)>) -> Option<(u32, u32)> {
    let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
    crate::vision::find_template(frame, template_name, threshold, region, baseline_res)
}

pub(crate) fn find_all_matches(ctx: &BotFSMContext, frame: &image::RgbImage, template_name: &str, threshold: f32, region: Option<(i32, i32, i32, i32)>) -> Vec<(u32, u32, f32)> {
    let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
    crate::vision::find_all_matches(frame, template_name, threshold, region, baseline_res)
}

// ---------------------------------------------------------------------------
// Recovery & Hub Navigation
// ---------------------------------------------------------------------------

pub(crate) fn attempt_recovery<F>(ctx: &BotFSMContext, is_healthy_fn: F) -> bool
where
    F: Fn(&BotFSMContext, &image::RgbImage) -> bool,
{
    ctx.logger.warn("Recovery: initiating procedure...");

    let mut capture = ctx.capture.lock().unwrap();
    let frame = match capture.grab_frame() {
        Some(f) => f,
        None => {
            ctx.logger.error("Recovery: frame grab returned None — aborting.");
            return false;
        }
    };

    let in_pause = is_on_screen(ctx, &frame, "pause_menu.png", 0.85, None);
    let in_pause_1st = is_on_screen(ctx, &frame, "pause_menu_1st_page.png", 0.85, None);

    if in_pause || in_pause_1st {
        ctx.logger.info("Recovery: Pause Menu detected — pressing B to return to driving.");
        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press_b();
            pad.sleep_responsive(2.0);
        }

        if let Some(f) = capture.grab_frame() {
            if is_healthy_fn(ctx, &f) {
                ctx.logger.info("Recovery: success — returned from Pause Menu.");
                return true;
            }
        }
    }

    ctx.logger.info("Recovery: Step 1 — pressing A.");
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(2.0);
    }

    if let Some(f) = capture.grab_frame() {
        if is_healthy_fn(ctx, &f) {
            ctx.logger.info("Recovery: success after Step 1 (A).");
            return true;
        }
    }

    for step in 2..=4 {
        ctx.logger.info(&format!("Recovery: Step {} — pressing B.", step));
        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press_b();
            pad.sleep_responsive(2.0);
        }

        if let Some(f) = capture.grab_frame() {
            if is_healthy_fn(ctx, &f) {
                ctx.logger.info(&format!("Recovery: success after Step {} (B).", step));
                return true;
            }
        }
    }

    ctx.logger.error("Recovery: all steps failed — still in unknown state.");
    false
}

pub(crate) fn open_pause_menu(ctx: &BotFSMContext) -> bool {
    let max_attempts = 5;
    for attempt in 1..=max_attempts {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        ctx.logger.info(&format!("Hub: pressing Start to open Pause Menu (attempt {}/{})...", attempt, max_attempts));
        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press_start();
        }

        std::thread::sleep(Duration::from_secs(1));

        for _ in 0..6 {
            if ctx.is_stop_requested.load(Ordering::Relaxed) {
                return false;
            }
            let mut capture = ctx.capture.lock().unwrap();
            if let Some(frame) = capture.grab_frame() {
                if is_on_screen(ctx, &frame, "pause_menu.png", 0.85, None)
                    || is_on_screen(ctx, &frame, "pause_menu_1st_page.png", 0.85, None)
                {
                    ctx.logger.info("Hub: Pause Menu confirmed.");
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    ctx.logger.error("Hub: failed to confirm Pause Menu after all attempts.");
    false
}
pub fn run_gamepad_test(ctx: &BotFSMContext) {
    ctx.logger.info("[Test-Inputs] Starting controller testing sequence...");

    // 1. Connect virtual gamepad if not connected
    {
        let mut pad = ctx.pad.lock().unwrap();
        ctx.logger.info("[Test-Inputs] Connecting virtual controller (ViGEmBus)...");
        if let Err(e) = pad.try_connect() {
            ctx.logger.error(&format!("[Test-Inputs] Failed to connect virtual controller: {}", e));
            return;
        }
    }

    // 2. Set cycle to 0 and sub-state to "0" (step 0)
    {
        let mut cycle = ctx.current_cycle.lock().unwrap();
        *cycle = 0;
    }
    ctx.set_sub_state(Some("0".to_string()));

    ctx.logger.info("[Test-Inputs] Step 0: 2-second pause...");
    // Sleep responsive checking for stop request
    {
        let check_interval = Duration::from_millis(50);
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if ctx.is_stop_requested.load(Ordering::Relaxed) {
                ctx.logger.warn("[Test-Inputs] Test cancelled during pause.");
                return;
            }
            std::thread::sleep(check_interval);
        }
    }

    if ctx.is_stop_requested.load(Ordering::Relaxed) {
        return;
    }

    // 3. Test pressing of D-pad buttons: UP, DOWN, LEFT, RIGHT
    ctx.logger.info("[Test-Inputs] Testing all 4 D-pad buttons...");
    {
        let mut pad = ctx.pad.lock().unwrap();
        ctx.logger.info("  -> Pressing D-pad UP");
        if !pad.press_dpad_up() { return; }
        ctx.logger.info("  -> Pressing D-pad DOWN");
        if !pad.press_dpad_down() { return; }
        ctx.logger.info("  -> Pressing D-pad LEFT");
        if !pad.press_dpad_left() { return; }
        ctx.logger.info("  -> Pressing D-pad RIGHT");
        if !pad.press_dpad_right() { return; }
    }

    if ctx.is_stop_requested.load(Ordering::Relaxed) {
        return;
    }

    // 4. Test pressing of A, B, X, Y buttons
    ctx.logger.info("[Test-Inputs] Testing buttons A, B, X, Y...");
    {
        let mut pad = ctx.pad.lock().unwrap();
        ctx.logger.info("  -> Pressing A");
        if !pad.press_a() { return; }
        ctx.logger.info("  -> Pressing B");
        if !pad.press_b() { return; }
        ctx.logger.info("  -> Pressing X");
        if !pad.press_x() { return; }
        ctx.logger.info("  -> Pressing Y");
        if !pad.press_y() { return; }
    }

    if ctx.is_stop_requested.load(Ordering::Relaxed) {
        return;
    }

    // 5. Test pressing of RB, LB shoulders
    ctx.logger.info("[Test-Inputs] Testing buttons LB, RB...");
    {
        let mut pad = ctx.pad.lock().unwrap();
        ctx.logger.info("  -> Pressing LB");
        if !pad.press_lb() { return; }
        ctx.logger.info("  -> Pressing RB");
        if !pad.press_rb() { return; }
    }

    if ctx.is_stop_requested.load(Ordering::Relaxed) {
        return;
    }

    // 6. Test quick stick movements: Left stick & Right stick in 4 directions sequentially
    ctx.logger.info("[Test-Inputs] Testing Left Stick quick movements (UP, DOWN, LEFT, RIGHT)...");
    {
        let mut pad = ctx.pad.lock().unwrap();
        
        ctx.logger.info("  -> Left Stick UP");
        pad.apply_stick("left_y", 1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("left_y", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Left Stick DOWN");
        pad.apply_stick("left_y", -1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("left_y", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Left Stick LEFT");
        pad.apply_stick("left_x", -1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("left_x", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Left Stick RIGHT");
        pad.apply_stick("left_x", 1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("left_x", 0.0);
        if !pad.sleep_responsive(0.10) { return; }
    }

    ctx.logger.info("[Test-Inputs] Testing Right Stick quick movements (UP, DOWN, LEFT, RIGHT)...");
    {
        let mut pad = ctx.pad.lock().unwrap();
        
        ctx.logger.info("  -> Right Stick UP");
        pad.apply_stick("right_y", 1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("right_y", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Right Stick DOWN");
        pad.apply_stick("right_y", -1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("right_y", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Right Stick LEFT");
        pad.apply_stick("right_x", -1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("right_x", 0.0);
        if !pad.sleep_responsive(0.10) { return; }

        ctx.logger.info("  -> Right Stick RIGHT");
        pad.apply_stick("right_x", 1.0);
        if !pad.sleep_responsive(0.15) { return; }
        pad.apply_stick("right_x", 0.0);
        if !pad.sleep_responsive(0.10) { return; }
    }

    // Clean up controller state
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    ctx.logger.info("[Test-Inputs] Gamepad test sequence complete.");
}

// ---------------------------------------------------------------------------
// Drawing Utility Helpers
// ---------------------------------------------------------------------------

pub fn draw_line(img: &mut image::RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgb<u8>) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut cx = x1;
    let mut cy = y1;

    loop {
        if cx >= 0 && cx < img.width() as i32 && cy >= 0 && cy < img.height() as i32 {
            img.put_pixel(cx as u32, cy as u32, color);
        }

        if cx == x2 && cy == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

pub fn draw_rect(img: &mut image::RgbImage, x: i32, y: i32, w: i32, h: i32, color: image::Rgb<u8>) {
    let x2 = x + w - 1;
    let y2 = y + h - 1;
    draw_line(img, x, y, x2, y, color);
    draw_line(img, x, y2, x2, y2, color);
    draw_line(img, x, y, x, y2, color);
    draw_line(img, x2, y, x2, y2, color);
}

pub fn draw_rect_thick(img: &mut image::RgbImage, x: i32, y: i32, w: i32, h: i32, thickness: i32, color: image::Rgb<u8>) {
    for t in 0..thickness {
        draw_rect(img, x + t, y + t, w - 2 * t, h - 2 * t, color);
    }
}

pub fn draw_crosshair(img: &mut image::RgbImage, cx: i32, cy: i32, size: i32, color: image::Rgb<u8>) {
    draw_line(img, cx - size, cy, cx + size, cy, color);
    draw_line(img, cx, cy - size, cx, cy + size, color);
}

pub(crate) fn is_lime_pixel(r: u8, g: u8, b: u8) -> bool {
    let (h, s, v) = rgb_to_opencv_hsv(r, g, b);
    h >= 30 && h <= 45 && s >= 150 && v >= 150
}
