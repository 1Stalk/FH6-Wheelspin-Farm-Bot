use super::*;
use crate::controller::BUTTON_BACK;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    ctx.logger.info("[Nav→Stage2] Starting navigation to Custom Map EventLab...");
    ctx.set_sub_state(Some("Navigation → Custom Map".to_string()));

    let success = navigate(ctx);

    ctx.set_sub_state(None);
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    if success && !ctx.is_stop_requested.load(Ordering::Relaxed) {
        StageResult::Success
    } else {
        StageResult::Failed
    }
}

fn select_subaru_brand(ctx: &BotFSMContext) -> bool {
    let mut menu_open = false;

    // Retry loop to open filter menu in case first BACK press is ignored
    for _attempt in 1..=3 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press(BUTTON_BACK);
            pad.sleep_responsive(1.200);
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => continue,
        };

        // 1. Check if we are still in the main Car Selection menu
        if is_on_screen(ctx, &frame, "car_selection_menu.png", 0.80, None) {
            continue;
        }

        // 2. Check if either brand selection cursor or Subaru logo is visible
        if is_on_screen(ctx, &frame, "brand_selection_cursor.png", 0.80, None)
            || is_on_screen(ctx, &frame, "subaru_brand_big.png", 0.80, None)
            || is_on_screen(ctx, &frame, "subaru_brand_big_selected.png", 0.80, None)
        {
            menu_open = true;
            break;
        }
    }

    if !menu_open {
        return false;
    }

    // 2. Press Up once to scroll to the bottom page
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_up();
        pad.sleep_responsive(1.000);
    }

    // 3. Grab frame again after scrolling (retry up to 5x)
    let mut capture = ctx.capture.lock().unwrap();
    let mut frame = None;
    for _retry in 0..5 {
        if let Some(f) = capture.grab_frame() {
            frame = Some(f);
            break;
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    let frame = match frame {
        Some(f) => f,
        None => return false,
    };

    // Check if Subaru brand is already selected
    if is_on_screen(ctx, &frame, "subaru_brand_big_selected.png", 0.75, None) {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(1.500);
        return true;
    }

    // 4. Detect unselected logo and cursor
    let cursor_pos = find_template(ctx, &frame, "brand_selection_cursor.png", 0.75, None);
    let subaru_pos = find_template(ctx, &frame, "subaru_brand_big.png", 0.75, None);

    let (cx, cy) = match (cursor_pos, subaru_pos) {
        (Some(c), Some(s)) => (c, s),
        _ => return false,
    };

    // 5. Convert to baseline resolution coordinates (2560x1440)
    let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
    let scale = frame.height() as f32 / baseline_res.1 as f32;

    let b_cx = cx.0 as f32 / scale;
    let b_cy = cx.1 as f32 / scale;
    let b_sx = cy.0 as f32 / scale;
    let b_sy = cy.1 as f32 / scale;

    // Align cursor corner coordinates to cell center coordinates
    let cursor_center_x = b_cx + 231.0;
    let cursor_center_y = b_cy + 40.0;

    let dx = b_sx - cursor_center_x;
    let dy = b_sy - cursor_center_y;

    // 6. Calculate grid index offsets
    let cols_diff = (dx / 470.0).round() as i32;
    let rows_diff = (dy / 70.0).round() as i32;

    drop(capture);
    // 7. Move cursor to the target logo and confirm
    {
        let mut pad = ctx.pad.lock().unwrap();
        if rows_diff > 0 {
            pad.navigate(crate::controller::BUTTON_DPAD_DOWN, rows_diff as usize);
        } else if rows_diff < 0 {
            pad.navigate(crate::controller::BUTTON_DPAD_UP, rows_diff.abs() as usize);
        }

        if cols_diff > 0 {
            pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, cols_diff as usize);
        } else if cols_diff < 0 {
            pad.navigate(crate::controller::BUTTON_DPAD_LEFT, cols_diff.abs() as usize);
        }

        pad.press_a();
        pad.sleep_responsive(1.500); // Wait for filter to apply and grid to reload
    }
    true
}

fn find_and_select_subaru_car(ctx: &BotFSMContext, scan_only: bool) -> bool {
    let max_scrolls = if scan_only { 1 } else { 12 };

    for _scroll in 0..max_scrolls {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        // Find selection cursor
        let cursor_pos = find_template(ctx, &frame, "car_selection_menu_selected.png", 0.80, None);

        // Scan all 12 cells for candidates
        let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        let mut candidates = Vec::new();
        for r in 0..3 {
            for c in 0..4 {
                let cell_cx = crate::vision::CAR_GRID_START_X + c as f32 * crate::vision::CAR_CELL_W;
                let cell_cy = crate::vision::CAR_GRID_START_Y + r as f32 * crate::vision::CAR_CELL_H;
                let cell_rx = (cell_cx - crate::vision::CAR_CELL_W / 2.0).max(0.0) as i32;
                let cell_ry = (cell_cy - crate::vision::CAR_CELL_H / 2.0).max(0.0) as i32;
                let cell_rw = crate::vision::CAR_CELL_W as i32;
                let cell_rh = crate::vision::CAR_CELL_H as i32;

                let is_car = is_on_screen(ctx, &frame, "subaru_impreza_1998.png", 0.92, Some((cell_rx, cell_ry, cell_rw, cell_rh)));
                if is_car {
                    let is_class_b = is_on_screen(ctx, &frame, "car_class_b.png", 0.80, Some((cell_rx, cell_ry, cell_rw, cell_rh)));
                    if !is_class_b {
                        candidates.push((c, r));
                    }
                }
            }
        }

        if !candidates.is_empty() {
            if cursor_pos.is_none() {
                drop(capture);
                {
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_dpad_left();
                    pad.sleep_responsive(0.500);
                }
                continue;
            }

            let (cx, cy) = cursor_pos.unwrap();
            let b_cx = cx as f32 / scale;
            let b_cy = cy as f32 / scale;
            let cursor_center_x = b_cx + crate::vision::CAR_CURSOR_OFFSET_X;
            let cursor_center_y = b_cy + crate::vision::CAR_CURSOR_OFFSET_Y;

            let cursor_col = ((cursor_center_x - crate::vision::CAR_GRID_START_X) / crate::vision::CAR_CELL_W).round() as i32;
            let cursor_row = ((cursor_center_y - crate::vision::CAR_GRID_START_Y) / crate::vision::CAR_CELL_H).round() as i32;

            let target = candidates[0];
            let target_col = target.0;
            let target_row = target.1;

            let cols_diff = target_col - cursor_col;
            let rows_diff = target_row - cursor_row;

            drop(capture);
            {
                let mut pad = ctx.pad.lock().unwrap();
                if cols_diff > 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, cols_diff as usize);
                } else if cols_diff < 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_LEFT, cols_diff.abs() as usize);
                }

                if rows_diff > 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_DOWN, rows_diff as usize);
                } else if rows_diff < 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_UP, rows_diff.abs() as usize);
                }

                pad.sleep_responsive(0.500);
                pad.press_a();
                pad.sleep_responsive(1.000);
            }
            return true;
        }

        if scan_only {
            break;
        }

        drop(capture);
        {
            // Scroll right if no matches found
            let mut pad = ctx.pad.lock().unwrap();
            pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, 4);
            pad.sleep_responsive(1.000);
        }
    }

    false
}

fn close_car_selection_menus(ctx: &BotFSMContext) {
    let mut pad = ctx.pad.lock().unwrap();
    for _ in 0..8 {
        pad.press_b();
        pad.sleep_responsive(0.400);
    }
}

fn navigate(ctx: &BotFSMContext) -> bool {
    {
        let mut pad = ctx.pad.lock().unwrap();
        
        // ── Step 0: 1s delay before starting ────────────────────────────────
        pad.sleep_responsive(1.000);

        // ── Step 1: RB × 5 → "Creative Hub" ──────────────
        pad.navigate(crate::controller::BUTTON_RB, 4);
        pad.sleep_responsive(0.600);

        // ── Step 2: A → enter EventLab ───────────────────────────────────────
        pad.press_a();
        pad.sleep_responsive(1.200);

        // ── Step 3: A → participate in race ──────────────────────────────────
        pad.press_a();
        pad.sleep_responsive(1.200);

        // ── Step 4: hold RB ~2s → scroll to "My History" tab ────────────────
        pad.hold_button(crate::controller::BUTTON_RB, 2.0, 0.0);
        pad.sleep_responsive(0.500);
    }

    // ── Step 4.5: Wait for My History tab to load (eventlab_race_on_screen.png) ──
    let mut map_loaded = false;
    let mut capture = ctx.capture.lock().unwrap();
    for _tick in 0..40 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        if let Some(frame) = capture.grab_frame() {
            if is_on_screen(ctx, &frame, "eventlab_race_on_screen.png", 0.80, None) {
                map_loaded = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    if !map_loaded {
        ctx.logger.warn("[Nav→S2] Map not detected.");
    }

    drop(capture);
    {
        let mut pad = ctx.pad.lock().unwrap();
        
        // ── Step 5: A → select target map (first in My History) ──────────────
        pad.press_a();
        pad.sleep_responsive(2.500);

        // ── Step 6: A → single player ───────────────────────
        pad.press_a();
        pad.sleep_responsive(2.500);
    }

    // ── Step 7: Car selection ─────────────────────────────────────────────
    ctx.set_sub_state(Some("Searching New Subaru".to_string()));
    let car_selected = find_and_select_subaru_car(ctx, true);

    if !car_selected {
        ctx.set_sub_state(Some("Subaru Brand Selection".to_string()));
        if !select_subaru_brand(ctx) {
            close_car_selection_menus(ctx);
            return false;
        }

        ctx.set_sub_state(Some("Searching New Subaru".to_string()));
        if !find_and_select_subaru_car(ctx, false) {
            close_car_selection_menus(ctx);
            return false;
        }
    }

    {
        let pad = ctx.pad.lock().unwrap();
        
        // ── Step 8: Wait for race menu to load ───────────────────────────────
        pad.sleep_responsive(5.000);
    }
    true
}
