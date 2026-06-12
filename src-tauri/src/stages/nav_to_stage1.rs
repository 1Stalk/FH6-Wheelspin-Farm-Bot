use super::*;
use crate::controller::BUTTON_BACK;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    ctx.logger
        .info("[Nav→Stage1] Starting navigation to Colossus race...");
    ctx.set_sub_state(Some("Navigation → Colossus".to_string()));

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

fn select_brand_filter(
    ctx: &BotFSMContext,
    brand_big: &str,
    brand_big_selected: &str,
    brand_name: &str,
) -> bool {
    ctx.logger.info(&format!(
        "Opening manufacturer filter menu for {}...",
        brand_name
    ));
    let mut menu_open = false;

    // Retry loop to open filter menu in case first BACK press is ignored
    for attempt in 1..=3 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        ctx.logger.info(&format!(
            "Manufacturer filter menu open attempt {}/3...",
            attempt
        ));

        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press(BUTTON_BACK);
            pad.sleep_responsive(1.200);
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                ctx.logger
                    .warn("  [select_brand_filter] Failed to grab frame.");
                continue;
            }
        };

        // 1. Check if we are still in the main Car Selection menu
        let in_selection = is_on_screen(ctx, &frame, "car_selection_menu.png", 0.80, None);
        let has_cursor = is_on_screen(ctx, &frame, "brand_selection_cursor.png", 0.80, None);
        let has_brand = is_on_screen(ctx, &frame, brand_big, 0.80, None);
        let has_brand_sel = is_on_screen(ctx, &frame, brand_big_selected, 0.80, None);

        ctx.logger.info(&format!("  [select_brand_filter] Status check: in_selection={}, has_cursor={}, has_brand={}, has_brand_sel={}", 
            in_selection, has_cursor, has_brand, has_brand_sel));

        if in_selection {
            ctx.logger.warn(&format!("Still in car selection menu ('car_selection_menu.png' detected) (attempt {}/3). Retrying...", attempt));
            continue;
        }

        // 2. Check if either brand selection cursor or brand logo is visible
        if has_cursor || has_brand || has_brand_sel {
            menu_open = true;
            ctx.logger.info("Filter menu successfully opened.");
            break;
        }

        ctx.logger.warn(&format!(
            "Filter menu did not open (attempt {}/3). Retrying...",
            attempt
        ));
    }

    if !menu_open {
        ctx.logger
            .error("Failed to open manufacturer filter menu after 3 attempts.");
        return false;
    }

    // 2. Press Up once to scroll to the bottom page
    ctx.logger
        .info("Pressing UP to jump to T-Z / N-Z brands at the bottom of the list.");
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_up();
        pad.sleep_responsive(1.000);
    }

    // 3. Grab frame again after scrolling (retry up to 5×)
    let mut capture = ctx.capture.lock().unwrap();
    let mut frame = None;
    for _retry in 0..5 {
        if let Some(f) = capture.grab_frame() {
            frame = Some(f);
            break;
        }
        ctx.logger.warn(&format!(
            "[Nav→S1] grab_frame returned None after scroll, retrying ({}/5)...",
            _retry + 1
        ));
        std::thread::sleep(Duration::from_millis(300));
    }

    let frame = match frame {
        Some(f) => f,
        None => {
            ctx.logger
                .error("[Nav→S1] Failed to grab frame after scrolling (5 retries exhausted).");
            return false;
        }
    };

    // Check if brand is already selected
    if is_on_screen(ctx, &frame, brand_big_selected, 0.80, None) {
        ctx.logger.info(&format!(
            "{} brand is already selected! Pressing A to confirm.",
            brand_name
        ));
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(1.500);
        return true;
    }

    // 4. Detect unselected logo and cursor
    let cursor_pos = find_template(ctx, &frame, "brand_selection_cursor.png", 0.75, None);
    let brand_pos = find_template(ctx, &frame, brand_big, 0.80, None);

    ctx.logger.info(&format!(
        "  [select_brand_filter] Cursor: {:?}, Brand: {:?}",
        cursor_pos, brand_pos
    ));

    let (cx, cy) = match (cursor_pos, brand_pos) {
        (Some(c), Some(b)) => (c, b),
        _ => {
            ctx.logger.error(&format!(
                "Failed to find templates on screen. Cursor: {}, Brand: {}",
                cursor_pos.is_some(),
                brand_pos.is_some()
            ));
            return false;
        }
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

    // 7. Move cursor to the target logo and confirm
    let mut pad = ctx.pad.lock().unwrap();
    if rows_diff > 0 {
        pad.navigate(crate::controller::BUTTON_DPAD_DOWN, rows_diff as usize);
    } else if rows_diff < 0 {
        pad.navigate(crate::controller::BUTTON_DPAD_UP, rows_diff.abs() as usize);
    }

    if cols_diff > 0 {
        pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, cols_diff as usize);
    } else if cols_diff < 0 {
        pad.navigate(
            crate::controller::BUTTON_DPAD_LEFT,
            cols_diff.abs() as usize,
        );
    }

    pad.press_a();
    pad.sleep_responsive(1.500); // Wait for filter to apply and grid to reload
    true
}

fn find_and_select_car(
    ctx: &BotFSMContext,
    car_template: &str,
    car_name_log: &str,
    scan_only: bool,
) -> bool {
    let max_scrolls = if scan_only { 1 } else { 12 };
    ctx.logger.info(&format!(
        "[find_car] Starting search for {}: scan_only={}, max_scrolls={}",
        car_name_log, scan_only, max_scrolls
    ));

    for scroll in 0..max_scrolls {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        ctx.logger.info(&format!(
            "[find_car] Scroll iteration {}/{}",
            scroll + 1,
            max_scrolls
        ));

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                ctx.logger
                    .warn("[find_car] Failed to grab frame. Retrying...");
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
                let cell_cx =
                    crate::vision::CAR_GRID_START_X + c as f32 * crate::vision::CAR_CELL_W;
                let cell_cy =
                    crate::vision::CAR_GRID_START_Y + r as f32 * crate::vision::CAR_CELL_H;
                let cell_rx = (cell_cx - crate::vision::CAR_CELL_W / 2.0).max(0.0) as i32;
                let cell_ry = (cell_cy - crate::vision::CAR_CELL_H / 2.0).max(0.0) as i32;
                let cell_rw = crate::vision::CAR_CELL_W as i32;
                let cell_rh = crate::vision::CAR_CELL_H as i32;

                let is_car = is_on_screen(
                    ctx,
                    &frame,
                    car_template,
                    0.92,
                    Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                );
                if is_car {
                    let is_favorite = is_on_screen(
                        ctx,
                        &frame,
                        "car_favorite_heart.png",
                        0.80,
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    );
                    candidates.push((c, r, is_favorite));
                }
            }
        }

        ctx.logger.info(&format!(
            "  [find_car] Cursor pos: {:?}, Car candidates: {:?}",
            cursor_pos, candidates
        ));

        if !candidates.is_empty() {
            if cursor_pos.is_none() {
                ctx.logger
                    .warn("Car matches found, but cursor not detected. Pressing DPAD_LEFT...");
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

            let cursor_col = ((cursor_center_x - crate::vision::CAR_GRID_START_X)
                / crate::vision::CAR_CELL_W)
                .round() as i32;
            let cursor_row = ((cursor_center_y - crate::vision::CAR_GRID_START_Y)
                / crate::vision::CAR_CELL_H)
                .round() as i32;

            // Pick target candidate
            let target = if let Some(fav) = candidates.iter().find(|&&(_, _, fav)| fav) {
                fav
            } else {
                candidates.sort_by_key(|c| (c.0, c.1));
                &candidates[0]
            };

            let target_col = target.0;
            let target_row = target.1;

            ctx.logger.info(&format!(
                "Target selected: col={}, row={}, is_favorite={}",
                target_col, target_row, target.2
            ));

            // Calculate navigation offsets
            let cols_diff = target_col - cursor_col;
            let rows_diff = target_row - cursor_row;

            drop(capture);
            {
                let mut pad = ctx.pad.lock().unwrap();
                // Move cursor to target car
                if cols_diff > 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, cols_diff as usize);
                } else if cols_diff < 0 {
                    pad.navigate(
                        crate::controller::BUTTON_DPAD_LEFT,
                        cols_diff.abs() as usize,
                    );
                }

                if rows_diff > 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_DOWN, rows_diff as usize);
                } else if rows_diff < 0 {
                    pad.navigate(crate::controller::BUTTON_DPAD_UP, rows_diff.abs() as usize);
                }

                pad.sleep_responsive(0.500); // Wait for cursor to settle

                // Press A once to select the car
                ctx.logger.info("Pressing A to select the car.");
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
            ctx.logger.info(&format!(
                "{} not found on current screen. Scrolling right (attempt {}/{})...",
                car_name_log,
                scroll + 1,
                max_scrolls
            ));
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

        // ── Step 1: RB × 4 → "Online" ──────────────────────────────
        pad.navigate(crate::controller::BUTTON_RB, 3);
        pad.sleep_responsive(0.600);

        // ── Step 2: D-pad Down → hover on Rivals ─────────────────────────────
        pad.press_dpad_down();
        pad.sleep_responsive(0.400);

        // ── Step 3: A → A → A → highway race selection ────────────────────────
        pad.press_a();
        pad.sleep_responsive(1.200);
        pad.press_a();
        pad.sleep_responsive(1.200);
        pad.press_a();
        pad.sleep_responsive(1.500);

        // ── Step 4: ← ← → select Colossus ───────────────────────────────────
        pad.press_dpad_left();
        pad.sleep_responsive(0.400);
        pad.press_dpad_left();
        pad.sleep_responsive(0.400);

        // ── Step 5: A → confirm Colossus ─────────────────────────────────────
        pad.press_a();
        pad.sleep_responsive(0.400);

        // ── Step 6: ← → select class R ───────────────────────────────────────
        pad.press_dpad_left();
        pad.sleep_responsive(6.000);

        // ── Step 7: Y → opponent selection menu ──────────────────────────────
        pad.press_y();
        pad.sleep_responsive(5.000);

        // ── Step 8: A → pick Top 1 World rival ───────────────────────────────
        pad.press_a();
        pad.sleep_responsive(5.000);

        // ── Step 9: A → confirm rival ────────────────────────────────────────
        pad.press_a();
        pad.sleep_responsive(1.500);
    }

    // ── Step 10: Car selection ────────────────────────────────────────────
    let car_id = {
        let cfg = ctx.config.lock().unwrap();
        cfg.stage1_car.clone()
    };

    let (car_template, car_name_log, brand_big, brand_big_selected, brand_name) =
        if car_id == "nissan_scargo_fe" {
            (
                "Nissan_1989.png",
                "Nissan S-Cargo",
                "nissan_brand_big.png",
                "nissan_brand_big_selected.png",
                "Nissan",
            )
        } else {
            (
                "Toyota_2019.png",
                "Toyota Tacoma",
                "toyota_brand_big.png",
                "toyota_brand_big_selected.png",
                "Toyota",
            )
        };

    ctx.set_sub_state(Some(format!("Searching {}", car_name_log)));
    let car_selected = find_and_select_car(ctx, car_template, car_name_log, true);

    if !car_selected {
        ctx.set_sub_state(Some(format!("{} Brand Selection", brand_name)));
        if !select_brand_filter(ctx, brand_big, brand_big_selected, brand_name) {
            close_car_selection_menus(ctx);
            return false;
        }

        ctx.set_sub_state(Some(format!("Searching {}", car_name_log)));
        if !find_and_select_car(ctx, car_template, car_name_log, false) {
            close_car_selection_menus(ctx);
            return false;
        }
    }

    {
        let pad = ctx.pad.lock().unwrap();
        pad.sleep_responsive(5.000);
    }
    true
}
