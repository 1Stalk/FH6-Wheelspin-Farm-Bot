use super::*;
use crate::controller::BUTTON_BACK;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    let num_to_buy = {
        let cfg = ctx.config.lock().unwrap();
        cfg.stage3_iterations as usize
    };

    let success = run_iteration(ctx, num_to_buy);

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

    // Retry loop to open filter menu (pressing BACK / Select button)
    for _attempt in 1..=3 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press(BUTTON_BACK);
            pad.sleep_responsive(0.700);
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => continue,
        };

        // 1. Check if we are still in the main Collection Journal menu
        if is_on_screen(ctx, &frame, "collection_journal_menu.png", 0.80, None) {
            continue;
        }

        // 2. Check if brand filter cursor or Subaru logo is visible
        if is_on_screen(ctx, &frame, "brand_selection_cursor.png", 0.80, None)
            || is_on_screen(ctx, &frame, "journal_subaru_brand.png", 0.80, None)
            || is_on_screen(ctx, &frame, "journal_subaru_brand_selected.png", 0.80, None)
        {
            menu_open = true;
            break;
        }
    }

    if !menu_open {
        return false;
    }

    // Press Up once to jump to bottom (S-Z brands)
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_up();
        pad.sleep_responsive(0.500);
    }

    // Grab frame after scrolling
    let mut capture = ctx.capture.lock().unwrap();
    let frame = match capture.grab_frame() {
        Some(f) => f,
        None => return false,
    };

    // Check if Subaru brand is already selected
    if is_on_screen(ctx, &frame, "journal_subaru_brand_selected.png", 0.80, None) {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(0.100);
        return true;
    }

    // Detect unselected logo and selection cursor
    let cursor_pos = find_template(ctx, &frame, "brand_selection_cursor.png", 0.80, None);
    let subaru_pos = find_template(ctx, &frame, "journal_subaru_brand.png", 0.80, None);

    let (cx, cy) = match (cursor_pos, subaru_pos) {
        (Some(c), Some(s)) => (c, s),
        _ => return false,
    };

    // Convert coordinates to baseline resolution (2560x1440)
    let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
    let scale = frame.height() as f32 / baseline_res.1 as f32;

    let b_cx = cx.0 as f32 / scale;
    let b_cy = cx.1 as f32 / scale;
    let b_sx = cy.0 as f32 / scale;
    let b_sy = cy.1 as f32 / scale;

    // Align cursor corner to brand cell center (reusing Stage 4 offsets as default)
    let cursor_center_x = b_cx + 231.0;
    let cursor_center_y = b_cy + 40.0;

    let dx = b_sx - cursor_center_x;
    let dy = b_sy - cursor_center_y;

    // Calculate grid index offsets
    let cols_diff = (dx / 470.0).round() as i32;
    let rows_diff = (dy / 70.0).round() as i32;

    drop(capture);
    // Move cursor to target and select
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
            pad.navigate(
                crate::controller::BUTTON_DPAD_LEFT,
                cols_diff.abs() as usize,
            );
        }

        pad.press_a();
        pad.sleep_responsive(0.500); // Wait for filter to apply and collection list to reload
    }
    true
}

fn find_and_select_target_car(ctx: &BotFSMContext) -> bool {
    let max_scrolls = 10;

    for scroll in 0..max_scrolls {
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

        // Match active selection cursor and target car
        let cursor_pos = find_template(ctx, &frame, "car_selection_menu_selected.png", 0.75, None);
        let car_matches = find_all_matches(ctx, &frame, "subaru_impreza_new.png", 0.75, None);

        if !car_matches.is_empty() {
            if cursor_pos.is_none() {
                drop(capture);
                {
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_dpad_down();
                    pad.sleep_responsive(0.500);
                }
                continue;
            }

            let (cx, cy) = cursor_pos.unwrap();
            let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
            let scale = frame.height() as f32 / baseline_res.1 as f32;

            let b_cx = cx as f32 / scale;
            let b_cy = cy as f32 / scale;

            // Calculate cursor col/row using calibrated template offsets
            let cursor_col = ((b_cx - 100.0 - 68.0) / 440.0).round().max(0.0).min(4.0) as i32;
            let cursor_row = ((b_cy - 192.0 - 48.0) / 384.0).round().max(0.0).min(2.0) as i32;

            // Map matches to grid columns and rows
            let mut grid_matches = Vec::new();
            for (idx, (sx, sy, score)) in car_matches.iter().enumerate() {
                let b_sx = *sx as f32 / scale;
                let b_sy = *sy as f32 / scale;

                let col_idx = ((b_sx - 393.0) / 440.0).round().max(0.0).min(4.0) as i32;
                let row_idx = ((b_sy - 459.0) / 384.0).round().max(0.0).min(2.0) as i32;

                grid_matches.push((col_idx, row_idx, b_sx, b_sy, *score, idx));
            }

            // Sort matches to find topmost-leftmost target car
            grid_matches.sort_by_key(|m| (m.1, m.0));
            let target = &grid_matches[0];

            let target_col = target.0;
            let target_row = target.1;

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
            }
            return true;
        }

        drop(capture);
        {
            // Scroll down if target car is not on screen
            ctx.logger.info(&format!(
                "Target car not found on screen. Scrolling down (attempt {}/{})...",
                scroll + 1,
                max_scrolls
            ));
            let mut pad = ctx.pad.lock().unwrap();
            pad.navigate(crate::controller::BUTTON_DPAD_DOWN, 1);
            pad.sleep_responsive(0.500); // Wait for page to scroll and settle
        }
    }

    false
}

fn run_purchase_sequence(ctx: &BotFSMContext, num_to_buy: usize) -> bool {
    for i in 1..=num_to_buy {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        ctx.logger
            .info(&format!("Purchasing car {}/{}...", i, num_to_buy));

        // Phase 1: Open the purchase details card
        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.press_start();
            pad.sleep_responsive(1.500); // Wait for transition animation to purchase screen
            pad.press_dpad_down();
            pad.sleep_responsive(0.300);
            // First A: initiates purchase flow
            pad.press_a();
            pad.sleep_responsive(1.000); // Give game time to show confirmation screen
        }

        // Phase 2: Credits check (separate macro block so wait has completed)
        let mut capture = ctx.capture.lock().unwrap();
        if let Some(frame) = capture.grab_frame() {
            if is_on_screen(ctx, &frame, "error.png", 0.95, None) {
                ctx.logger
                    .error("Out of credits! 'error.png' template detected. Stopping bot.");
                return false;
            }
        }
        drop(capture);

        // Phase 3: Confirm purchase
        {
            let mut pad = ctx.pad.lock().unwrap();
            // Second A: confirm purchase dialog
            pad.press_a();
            pad.sleep_responsive(3.000); // Crucial: wait for the transaction saving spinner to complete and the Congratulations card to appear
                                         // Third A: complete transaction / dismiss congratulations card
            pad.press_a();
            pad.sleep_responsive(1.500); // Wait for save/delivery animation to complete and return to Collection menu
        }
        ctx.add_session_cr(-86000);
        ctx.logger
            .info(&format!("Car {} successfully purchased.", i));
    }

    true
}

fn exit_to_open_world(ctx: &BotFSMContext) -> bool {
    let mut pad = ctx.pad.lock().unwrap();
    // Stage 3: Exiting Collection Journal to Open World by pressing B multiple times.
    for _ in 1..=6 {
        pad.press_b();
        pad.sleep_responsive(0.150);
    }
    std::thread::sleep(Duration::from_secs(1));
    true
}

fn return_to_hub(ctx: &BotFSMContext) -> bool {
    if !exit_to_open_world(ctx) {
        return false;
    }
    open_pause_menu(ctx)
}

fn run_iteration(ctx: &BotFSMContext, num_to_buy: usize) -> bool {
    // 1. Check if we are in Collection Journal
    ctx.set_sub_state(Some("Collection Menu".to_string()));
    let mut menu_detected = false;

    let mut capture = ctx.capture.lock().unwrap();
    for _ in 0..15 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        if let Some(frame) = capture.grab_frame() {
            if is_on_screen(ctx, &frame, "collection_journal_menu.png", 0.80, None) {
                menu_detected = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if !menu_detected {
        ctx.logger.warn("Failed to detect Collection Journal menu.");
    }
    drop(capture);

    // 2. Select Subaru Brand
    ctx.set_sub_state(Some("Subaru Brand Selection".to_string()));
    if !select_subaru_brand(ctx) {
        return false;
    }

    // 3. Select Subaru 22B-STI Version
    if !find_and_select_target_car(ctx) {
        return false;
    }

    // 4. Run purchase loop
    ctx.set_sub_state(Some("Buying Cars".to_string()));
    if !run_purchase_sequence(ctx, num_to_buy) {
        return_to_hub(ctx);
        return false;
    }

    if !ctx.is_stop_requested.load(Ordering::Relaxed) {
        if !return_to_hub(ctx) {
            return false;
        }
    }
    true
}
