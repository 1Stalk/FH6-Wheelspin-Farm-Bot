use super::*;
use crate::controller::BUTTON_BACK;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    let mut num_iterations = {
        let cfg = ctx.config.lock().unwrap();
        cfg.stage4_iterations as usize
    };
    if num_iterations == 0 {
        num_iterations = 999999;
    }

    let mut successful_spends = 0;
    let mut last_iteration_success = false;

    for i in 1..=num_iterations {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }

        ctx.logger.info(&format!(
            "--- Stage 4: Iteration {} of {} ---",
            i, num_iterations
        ));
        let is_last = i == num_iterations;

        let success = run_iteration(ctx, is_last);
        last_iteration_success = success;
        if success {
            successful_spends += 1;
        }

        if !success || ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }
    }

    if successful_spends > 0 && !ctx.is_stop_requested.load(Ordering::Relaxed) {
        let cleanup_ok = run_garage_cleanup(ctx, successful_spends, last_iteration_success);
        if !cleanup_ok {
            ctx.logger.error("Stage 4: garage cleanup failed.");
        }
    }

    if !ctx.is_stop_requested.load(Ordering::Relaxed) {
        if !return_to_hub(ctx) {
            ctx.logger.error("Stage 4: failed to return to Pause Menu.");
        }
    }

    ctx.set_sub_state(None);
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    StageResult::Success
}

fn select_subaru_brand(ctx: &BotFSMContext) -> bool {
    let mut menu_open = false;
    ctx.logger.info("[select_subaru_brand] Starting manufacturer selection...");

    // Retry loop to open filter menu in case first BACK press is ignored
    for attempt in 1..=3 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        ctx.logger.info(&format!(
            "[select_subaru_brand] Attempt {}/3: Pressing BACK button...",
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
                ctx.logger.warn("[select_subaru_brand] Grab frame failed, retrying...");
                continue;
            }
        };

        if is_on_screen(ctx, &frame, "car_selection_menu.png", 0.80, None) {
            continue;
        }

        let cursor_visible = is_on_screen(ctx, &frame, "brand_selection_cursor.png", 0.80, None);
        let subaru_visible = is_on_screen(ctx, &frame, "subaru_brand_big.png", 0.80, None);
        let subaru_selected = is_on_screen(ctx, &frame, "subaru_brand_big_selected.png", 0.80, None);
        ctx.logger.info(&format!(
            "[select_subaru_brand] Brand menu check: cursor={}, subaru={}, subaru_selected={}",
            cursor_visible, subaru_visible, subaru_selected
        ));

        if cursor_visible || subaru_visible || subaru_selected {
            menu_open = true;
            break;
        }
    }

    if !menu_open {
        ctx.logger.error("[select_subaru_brand] Failed to open filter menu after 3 attempts.");
        return false;
    }

    // Press Up to scroll to the bottom of the list (wraps around to end)
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_up();
        pad.sleep_responsive(1.000);
    }

    // Grab frame after scrolling (retry up to 5x)
    let mut capture = ctx.capture.lock().unwrap();
    let mut frame_opt = None;
    for _retry in 0..5 {
        if let Some(f) = capture.grab_frame() {
            frame_opt = Some(f);
            break;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    let frame = match frame_opt {
        Some(f) => f,
        None => {
            ctx.logger.error("[select_subaru_brand] Failed to grab frame after scroll.");
            return false;
        }
    };

    // If Subaru is already selected, just confirm
    if is_on_screen(ctx, &frame, "subaru_brand_big_selected.png", 0.80, None) {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(0.800);
        ctx.logger.info("[select_subaru_brand] Subaru was already selected. Confirmed.");
        return true;
    }

    // Find Subaru logo and calculate navigation offsets
    let subaru_pos = find_template(ctx, &frame, "subaru_brand_big.png", 0.70, None);
    let (sx, sy) = match subaru_pos {
        Some(s) => s,
        None => {
            ctx.logger.error("[select_subaru_brand] Could not find Subaru logo on screen.");
            return false;
        }
    };

    let offsets = crate::vision::calculate_brand_navigation_offsets(&frame, (sx, sy));
    let (cols_diff, rows_diff) = match offsets {
        Some(o) => o,
        None => {
            ctx.logger.error("[select_subaru_brand] Could not detect active brand cursor.");
            return false;
        }
    };

    ctx.logger.info(&format!(
        "[select_subaru_brand] Navigating to Subaru: cols_diff={}, rows_diff={}",
        cols_diff, rows_diff
    ));

    drop(capture);
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
        pad.sleep_responsive(0.800);
    }
    ctx.logger.info("[select_subaru_brand] Subaru brand selected.");
    true
}


fn find_and_select_new_car(ctx: &BotFSMContext) -> bool {
    let max_scrolls = 12;
    let start_total = std::time::Instant::now();
    ctx.logger
        .info("[find_and_select_new_car] Starting search for a new Subaru car...");

    for scroll in 0..max_scrolls {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        ctx.logger.info(&format!(
            "[find_and_select_new_car] Scroll iteration {}/12. Grabbing frame...",
            scroll + 1
        ));
        let start_grab = std::time::Instant::now();
        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => {
                ctx.logger.info(&format!(
                    "[find_and_select_new_car] Frame grabbed in {} ms.",
                    start_grab.elapsed().as_millis()
                ));
                f
            }
            None => {
                ctx.logger.warn(&format!(
                    "[find_and_select_new_car] Frame grab failed, retrying in 100ms. Took {} ms.",
                    start_grab.elapsed().as_millis()
                ));
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        // Find selection cursor using lime outline
        let cursor_pos = crate::vision::find_car_cursor_lime(&frame, scale);

        // Scan all 12 cells for candidates
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
                    "subaru_impreza_new.png",
                    0.80,
                    Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                );
                if is_car {
                    let is_favorite = is_on_screen(
                        ctx,
                        &frame,
                        "car_favorite_heart.png",
                        0.90,
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    );
                    let is_class_b = is_on_screen(
                        ctx,
                        &frame,
                        "car_class_b.png",
                        0.80,
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    );

                    let is_candidate = !is_favorite && is_class_b;
                    ctx.logger.info(&format!(
                        "[find_and_select_new_car] Evaluated cell (col={}, row={}): favorite={}, classB={}, is_candidate={}",
                        c, r, is_favorite, is_class_b, is_candidate
                    ));
                    if is_candidate {
                        candidates.push((c, r));
                    }
                }
            }
        }

        if !candidates.is_empty() {
            if cursor_pos.is_none() {
                ctx.logger.warn("[find_and_select_new_car] Target car found, but cursor not found! Nudging DPAD Left to reset selection cursor...");
                drop(capture);
                {
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_dpad_left();
                    pad.sleep_responsive(0.500);
                }
                continue;
            }

            let (cursor_col, cursor_row) = cursor_pos.unwrap();

            // Sort candidates by Manhattan-distance priority from cursor (same column first, then adjacent columns)
            candidates.sort_by_key(|c| ((c.0 as i32 - cursor_col).abs(), (c.1 as i32 - cursor_row).abs()));
            let target = candidates[0];
            let target_col = target.0;
            let target_row = target.1;

            let cols_diff = target_col - cursor_col;
            let rows_diff = target_row - cursor_row;

            ctx.logger.info(&format!(
                "[find_and_select_new_car] Found target car at (col={}, row={}). Navigating: cols_diff={}, rows_diff={}",
                target_col, target_row, cols_diff, rows_diff
            ));

            drop(capture);
            let start_nav = std::time::Instant::now();
            {
                let mut pad = ctx.pad.lock().unwrap();
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
            ctx.logger.info(&format!(
                "[find_and_select_new_car] Navigation complete in {} ms. Total search time: {} ms.",
                start_nav.elapsed().as_millis(),
                start_total.elapsed().as_millis()
            ));
            return true;
        }

        drop(capture);
        {
            // Scroll right if no candidates found on screen
            ctx.logger.info(&format!("New target candidate car not found on current screen. Scrolling right (attempt {}/12)...", scroll + 1));
            let start_scroll = std::time::Instant::now();
            let mut pad = ctx.pad.lock().unwrap();
            pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, 4);
            pad.sleep_responsive(1.000); // Wait for grid to scroll and settle
            ctx.logger.info(&format!(
                "[find_and_select_new_car] Scroll right + sleep took {} ms.",
                start_scroll.elapsed().as_millis()
            ));
        }
    }
    ctx.logger.error(&format!(
        "[find_and_select_new_car] Finished 12 scrolls. Target car NOT found. Total time: {} ms.",
        start_total.elapsed().as_millis()
    ));
    false
}

fn get_talent_sp_cost(col: i32, row: i32) -> i32 {
    match (col, row) {
        (0, 3) | (1, 3) => 1,
        (1, 2) => 3,
        (1, 1) => 5,
        (0, 0) | (1, 0) => 10,
        _ => 0,
    }
}

fn run_spend_sp_macro(ctx: &BotFSMContext, is_last: bool) -> bool {
    // 1. Confirm taking the car and enter modifications menu
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.sleep_responsive(1.000);
        pad.press_a();
        pad.sleep_responsive(4.500);

        pad.press_b();
        pad.sleep_responsive(1.000);

        pad.press_dpad_up();
        pad.sleep_responsive(0.450);
        pad.press_a();
        pad.sleep_responsive(0.750);

        pad.hold_dpad_down(1.6);
        pad.sleep_responsive(0.500);
        pad.press_a();
        pad.sleep_responsive(1.500);
    }

    // 2. Dynamic CV talent purchasing loop
    let target_path = vec![(0, 3), (1, 3), (1, 2), (1, 1), (1, 0), (0, 0)];
    let mut empty_cells = std::collections::HashSet::new();
    empty_cells.insert((2, 0));
    empty_cells.insert((0, 1));

    let mut previously_purchased = std::collections::HashSet::new();
    let mut initialized = false;

    let mut last_target = None;
    let mut expected_col = None;
    let mut expected_row = None;
    let mut attempts_on_current_target = 0;
    let mut cursor_not_found_count = 0;
    let max_loop = 25;

    for loop_idx in 0..max_loop {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        ctx.logger.info(&format!(
            "[Stage4 Upgrade] Iteration {}/{}",
            loop_idx + 1,
            max_loop
        ));

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                ctx.logger
                    .warn("[Stage4 Upgrade] Failed to grab frame, retrying in 100ms...");
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        ctx.logger.info(&format!(
            "[Stage4 Upgrade] Expected cursor at: col={:?}, row={:?}",
            expected_col, expected_row
        ));

        // Locate the selection cursor
        let cursor_info = find_best_cursor(&frame, expected_col, expected_row, scale, baseline_res);
        if cursor_info.is_none() {
            cursor_not_found_count += 1;
            ctx.logger.warn(&format!(
                "[Stage4 Upgrade] Cursor not found (attempt {}/20). Waiting 200ms...",
                cursor_not_found_count
            ));
            if cursor_not_found_count > 20 {
                ctx.logger.error("[Stage4 Upgrade] Cursor not found after 20 consecutive attempts. Exiting Stage 4.");
                return false;
            }
            drop(capture);
            std::thread::sleep(Duration::from_millis(200));
            continue;
        }

        cursor_not_found_count = 0;
        let (cursor_col, cursor_row, cur_x, cur_y, score, cursor_type) = cursor_info.unwrap();
        ctx.logger.info(&format!(
            "[Stage4 Upgrade] Cursor found: col={}, row={}, type='{}', match_x={}, match_y={}, score={:.4}",
            cursor_col, cursor_row, cursor_type, cur_x, cur_y, score
        ));
        expected_col = Some(cursor_col);
        expected_row = Some(cursor_row);

        // Read current state of all 16 skills on screen
        let mut grid_states = std::collections::HashMap::new();
        for r in 0..4 {
            for c in 0..4 {
                let state = get_cell_state(&frame, c, r, scale, &empty_cells);
                grid_states.insert((c, r), state);
            }
        }

        if !initialized {
            for r in 0..4 {
                for c in 0..4 {
                    if grid_states.get(&(c, r)) == Some(&CellState::Purchased) {
                        previously_purchased.insert((c, r));
                    }
                }
            }
            initialized = true;
        } else {
            for &(tc, tr) in &target_path {
                if grid_states.get(&(tc, tr)) == Some(&CellState::Purchased)
                    && !previously_purchased.contains(&(tc, tr))
                {
                    let cost = get_talent_sp_cost(tc as i32, tr as i32);
                    ctx.add_session_sp(-cost);
                    if tc == 0 && tr == 0 {
                        ctx.add_session_wspins(1);
                    }
                    previously_purchased.insert((tc, tr));
                    ctx.logger.info(&format!(
                        "[Stage4 Upgrade] Detected purchase of talent ({}, {}). Deducted {} SP. New Wheelspins? {}",
                        tc, tr, cost, tc == 0 && tr == 0
                    ));
                }
            }
        }

        // Print target path status for debugging
        let path_status: Vec<String> = target_path
            .iter()
            .map(|&(tc, tr)| {
                format!(
                    "({},{})={:?}",
                    tc,
                    tr,
                    grid_states
                        .get(&(tc, tr))
                        .copied()
                        .unwrap_or(CellState::Locked)
                )
            })
            .collect();
        ctx.logger.info(&format!(
            "[Stage4 Upgrade] Path states: {}",
            path_status.join(", ")
        ));

        // Determine next upgrade to purchase along TARGET_PATH
        let mut next_target = None;
        for &(tc, tr) in &target_path {
            if grid_states
                .get(&(tc, tr))
                .copied()
                .unwrap_or(CellState::Locked)
                != CellState::Purchased
            {
                next_target = Some((tc, tr));
                break;
            }
        }

        // Check if all target upgrades are purchased
        if next_target.is_none() {
            ctx.logger.info(
                "[Stage4 Upgrade] All target talents in path are purchased! Exiting purchase loop.",
            );
            break;
        }

        let (target_col, target_row) = next_target.unwrap();
        let target_state = grid_states
            .get(&(target_col, target_row))
            .copied()
            .unwrap_or(CellState::Locked);
        ctx.logger.info(&format!(
            "[Stage4 Upgrade] Current target is: col={}, row={}. Cell state is: {:?}",
            target_col, target_row, target_state
        ));

        if next_target == last_target {
            if cursor_col == target_col as i32 && cursor_row == target_row as i32 {
                attempts_on_current_target += 1;
                ctx.logger.info(&format!(
                    "[Stage4 Upgrade] Cursor is on target (col={}, row={}) but cell state is still {:?}. Attempt {}/3.",
                    target_col, target_row, target_state, attempts_on_current_target
                ));
                if attempts_on_current_target > 3 {
                    ctx.logger.error(&format!(
                        "[Stage4 Upgrade] Stuck on target (col={}, row={}) for 3 consecutive attempts. Exiting Stage 4.",
                        target_col, target_row
                    ));
                    return false;
                }
            }
        } else {
            last_target = next_target;
            attempts_on_current_target = 0;
        }

        // Upgrade logic
        let mut purchase_attempted = false;
        if cursor_col == target_col as i32 && cursor_row == target_row as i32 {
            if let CellState::Available = target_state {
                ctx.logger.info(&format!("[Stage4 Upgrade] Cursor is on target and cell is Available. Pressing A to buy talent (col={}, row={}).", target_col, target_row));
                drop(capture);
                {
                    // Apply upgrade purchase
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_a();
                    pad.sleep_responsive(0.500);
                }
                purchase_attempted = true;
            } else {
                ctx.logger.warn(&format!(
                    "[Stage4 Upgrade] Cursor is on target (col={}, row={}) but cell state is {:?} (not Available). Sleeping 300ms.",
                    target_col, target_row, target_state
                ));
                drop(capture);
                std::thread::sleep(Duration::from_millis(300));
            }
        } else {
            let path_opt = find_grid_path(
                (cursor_col, cursor_row),
                (target_col as i32, target_row as i32),
                &empty_cells,
            );

            drop(capture);
            {
                let mut pad = ctx.pad.lock().unwrap();
                if let Some(path) = path_opt {
                    ctx.logger.info(&format!(
                        "[Stage4 Upgrade] Cursor is at ({},{}), target is at ({},{}). Found path: {:?}",
                        cursor_col, cursor_row, target_col, target_row, path
                    ));

                    let mut current = (cursor_col, cursor_row);
                    for next in path {
                        let col_step = next.0 - current.0;
                        let row_step = next.1 - current.1;

                        let button = if col_step > 0 {
                            crate::controller::BUTTON_DPAD_RIGHT
                        } else if col_step < 0 {
                            crate::controller::BUTTON_DPAD_LEFT
                        } else if row_step > 0 {
                            crate::controller::BUTTON_DPAD_DOWN
                        } else {
                            crate::controller::BUTTON_DPAD_UP
                        };

                        if !pad.press(button) {
                            return false;
                        }
                        current = next;
                    }
                } else {
                    ctx.logger.warn(&format!(
                        "[Stage4 Upgrade] No path found from ({},{}) to ({},{}). Falling back to direct navigation.",
                        cursor_col, cursor_row, target_col, target_row
                    ));

                    let cols_diff = target_col as i32 - cursor_col;
                    let rows_diff = target_row as i32 - cursor_row;

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
                }

                pad.sleep_responsive(0.150);
                ctx.logger.info(&format!(
                    "[Stage4 Upgrade] Pressing A to buy talent (col={}, row={}) after navigation.",
                    target_col, target_row
                ));
                pad.press_a();
                pad.sleep_responsive(0.350);
            }
            expected_col = Some(target_col as i32);
            expected_row = Some(target_row as i32);
            purchase_attempted = true;
        }

        // Verification check: verify cursor actually landed on target cell
        if purchase_attempted {
            ctx.logger.info("[Stage4 Upgrade] Purchase was attempted. Running menu presence verification check...");
            let mut capture = ctx.capture.lock().unwrap();
            if let Some(frame_check) = capture.grab_frame() {
                if !is_on_screen(ctx, &frame_check, "spend_sp_menu.png", 0.80, None) {
                    ctx.logger.warn("[Stage4 Upgrade] 'spend_sp_menu.png' not detected on screen. A popup (like upgrade confirmation dialog) might be open. Pressing A to dismiss...");
                    drop(capture);
                    {
                        let mut pad = ctx.pad.lock().unwrap();
                        pad.press_a();
                        pad.sleep_responsive(0.600);
                    }
                    let mut capture = ctx.capture.lock().unwrap();
                    if let Some(frame_retry) = capture.grab_frame() {
                        if is_on_screen(ctx, &frame_retry, "spend_sp_menu.png", 0.80, None) {
                            ctx.logger.error("[Stage4 Upgrade] Out of Skill Points! Detected and closed the error popup. Aborting SP spending.");
                            return false;
                        } else {
                            ctx.logger.error("[Stage4 Upgrade] Still failed to detect 'spend_sp_menu.png' after pressing A. Exiting Stage 4.");
                            return false;
                        }
                    }
                } else {
                    ctx.logger
                        .info("[Stage4 Upgrade] Menu presence verified successfully.");
                    std::thread::sleep(Duration::from_millis(150));
                }
            }
        }
    }

    // Post-spend cleanup
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_b();
        pad.sleep_responsive(1.000);

        pad.press_b();
        pad.sleep_responsive(1.000);

        pad.press_dpad_up();
        pad.sleep_responsive(0.300);
        pad.press_a();
        pad.sleep_responsive(0.500);

        pad.press_a();
        pad.sleep_responsive(0.500);
        pad.press_a();
        pad.sleep_responsive(0.500);
    }

    if is_last {
        return true;
    }

    // Loop exit or reset
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_b();
        pad.sleep_responsive(1.200);

        pad.press_dpad_down();
        pad.sleep_responsive(0.300);
        pad.press_dpad_down();
        pad.sleep_responsive(0.300);
        pad.press_a();
        pad.sleep_responsive(0.500);

        pad.hold_dpad_down(1.6);
        pad.sleep_responsive(0.500);
        pad.press_a();
        pad.sleep_responsive(1.000);
    }

    true
}

pub(crate) fn find_grid_path(
    start: (i32, i32),
    target: (i32, i32),
    empty_cells: &std::collections::HashSet<(usize, usize)>,
) -> Option<Vec<(i32, i32)>> {
    let mut queue = std::collections::VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    let mut parent = std::collections::HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(curr) = queue.pop_front() {
        if curr == target {
            let mut path = Vec::new();
            let mut node = curr;
            while node != start {
                path.push(node);
                node = *parent.get(&node)?;
            }
            path.reverse();
            return Some(path);
        }

        let (c, r) = curr;
        let neighbors = vec![
            (c + 1, r),
            (c - 1, r),
            (c, r + 1),
            (c, r - 1),
        ];

        for (nc, nr) in neighbors {
            if nc >= 0 && nc < 4 && nr >= 0 && nr < 4 {
                if !empty_cells.contains(&(nc as usize, nr as usize)) {
                    if !visited.contains(&(nc, nr)) {
                        visited.insert((nc, nr));
                        parent.insert((nc, nr), curr);
                        queue.push_back((nc, nr));
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn find_best_cursor(
    frame: &image::RgbImage,
    _expected_col: Option<i32>,
    _expected_row: Option<i32>,
    scale: f32,
    _baseline_res: (u32, u32),
) -> Option<(i32, i32, u32, u32, f32, &'static str)> {
    const TALENT_GRID_START_X: f32 = 500.0;
    const TALENT_GRID_START_Y: f32 = 320.0;
    const TALENT_CELL_W: f32 = 154.0;
    const TALENT_CELL_H: f32 = 154.0;

    let mut best_col = -1;
    let mut best_row = -1;
    let mut max_lime_count = 0;

    for r in 0..4 {
        for c in 0..4 {
            let cell_center_x = TALENT_GRID_START_X + c as f32 * TALENT_CELL_W;
            let cell_center_y = TALENT_GRID_START_Y + r as f32 * TALENT_CELL_H;

            let screen_cx = (cell_center_x * scale) as i32;
            let screen_cy = (cell_center_y * scale) as i32;

            let scan_w = (170.0 * scale) as i32;
            let scan_h = (170.0 * scale) as i32;

            let x1 = (screen_cx - scan_w / 2)
                .max(0)
                .min(frame.width() as i32 - 1) as u32;
            let y1 = (screen_cy - scan_h / 2)
                .max(0)
                .min(frame.height() as i32 - 1) as u32;
            let x2 = (screen_cx + scan_w / 2).max(1).min(frame.width() as i32) as u32;
            let y2 = (screen_cy + scan_h / 2).max(1).min(frame.height() as i32) as u32;

            let mut lime_count = 0;
            for y in y1..y2 {
                for x in x1..x2 {
                    let px = frame.get_pixel(x, y);
                    if super::is_lime_pixel(px[0], px[1], px[2]) {
                        lime_count += 1;
                    }
                }
            }

            if lime_count > max_lime_count {
                max_lime_count = lime_count;
                best_col = c as i32;
                best_row = r as i32;
            }
        }
    }

    let lime_threshold = (1000.0 * scale * scale) as u32;
    if max_lime_count >= lime_threshold {
        let center_x = TALENT_GRID_START_X + best_col as f32 * TALENT_CELL_W;
        let center_y = TALENT_GRID_START_Y + best_row as f32 * TALENT_CELL_H;
        let cx = (center_x * scale) as u32;
        let cy = (center_y * scale) as u32;
        Some((best_col, best_row, cx, cy, max_lime_count as f32, "lime"))
    } else {
        None
    }
}

fn run_iteration(ctx: &BotFSMContext, is_last: bool) -> bool {
    // # Step 1: Open Manufacturer filter and select Subaru
    ctx.set_sub_state(Some("Subaru Brand Selection".to_string()));
    if !select_subaru_brand(ctx) {
        return false;
    }

    // # Step 2: Navigate the car grid to find a new car
    ctx.set_sub_state(Some("Searching New Subaru".to_string()));
    if !find_and_select_new_car(ctx) {
        return false;
    }

    // # Step 3: Run SP spending and favoriting macro
    ctx.set_sub_state(Some("Spending Skill Points".to_string()));
    if !run_spend_sp_macro(ctx, is_last) {
        return false;
    }

    true
}

fn find_and_select_player_subaru(ctx: &BotFSMContext) -> bool {
    let max_scrolls = 12;

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

        let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        // Find selection cursor using lime outline
        let cursor_pos = crate::vision::find_car_cursor_lime(&frame, scale);

        // Scan all 12 cells for candidates
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

                let matches = find_all_matches(
                    ctx,
                    &frame,
                    "subaru_impreza_1998.png",
                    0.85,
                    Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                );
                if !matches.is_empty() {
                    let best_score = matches[0].2;
                    let is_class_b = is_on_screen(
                        ctx,
                        &frame,
                        "car_class_b.png",
                        0.80,
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    );
                    if !is_class_b {
                        candidates.push((c, r, best_score));
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

            let (cursor_col, cursor_row) = cursor_pos.unwrap();

            // Sort candidates by match score descending, fallback to Manhattan-distance if scores are very close
            candidates.sort_by(|a, b| {
                let score_diff = b.2 - a.2;
                if score_diff.abs() < 0.015 {
                    let dist_a = (a.0 as i32 - cursor_col).abs() + (a.1 as i32 - cursor_row).abs();
                    let dist_b = (b.0 as i32 - cursor_col).abs() + (b.1 as i32 - cursor_row).abs();
                    dist_a.cmp(&dist_b)
                } else {
                    b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
                }
            });

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

                pad.sleep_responsive(0.500);
                pad.press_a();
                pad.sleep_responsive(0.500);
                pad.press_a();
                pad.sleep_responsive(4.000);

                pad.press_a();
                pad.sleep_responsive(1.500);
            }

            if !select_subaru_brand(ctx) {
                return false;
            }
            return true;
        }

        drop(capture);
        {
            let mut pad = ctx.pad.lock().unwrap();
            pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, 4);
            pad.sleep_responsive(1.000);
        }
    }
    false
}

fn delete_used_subaru_cars(ctx: &BotFSMContext, count: usize) -> bool {
    for _delete_idx in 1..=count {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        let mut found = false;
        let max_scrolls = 6;

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

            let baseline_res = ctx.config.lock().unwrap().baseline_resolution;
            let scale = frame.height() as f32 / baseline_res.1 as f32;

            // Find selection cursor using lime outline
            let cursor_pos = crate::vision::find_car_cursor_lime(&frame, scale);

            // Scan all 12 cells for candidates
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
                        "subaru_impreza_new.png",
                        0.80,
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    );
                    if is_car {
                        let is_favorite = is_on_screen(
                            ctx,
                            &frame,
                            "car_favorite_heart.png",
                            0.90,
                            Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                        );
                        let is_class_b = is_on_screen(
                            ctx,
                            &frame,
                            "car_class_b.png",
                            0.80,
                            Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                        );

                        let is_candidate = is_favorite && is_class_b;
                        if is_candidate {
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

                let (cursor_col, cursor_row) = cursor_pos.unwrap();

                // Sort candidates by Manhattan-distance priority from cursor
                candidates.sort_by_key(|c| ((c.0 as i32 - cursor_col).abs(), (c.1 as i32 - cursor_row).abs()));
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

                    pad.sleep_responsive(0.250);

                    pad.press_a();
                    pad.sleep_responsive(0.350);

                    pad.navigate(crate::controller::BUTTON_DPAD_DOWN, 4);
                    pad.sleep_responsive(0.200);

                    pad.press_a();
                    pad.sleep_responsive(0.350);

                    pad.press_dpad_down();
                    pad.sleep_responsive(0.200);

                    pad.press_a();
                    pad.sleep_responsive(0.300);
                }
                found = true;

                // Wait for the car selection menu to reload and the selection cursor to appear
                ctx.logger.info("[delete_used_subaru_cars] Waiting for menu to reload after deletion...");
                let mut cursor_loaded = false;
                for attempt in 1..=25 {
                    if ctx.is_stop_requested.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                    let frame_opt = {
                        let mut capture = ctx.capture.lock().unwrap();
                        capture.grab_frame()
                    };
                    if let Some(frame) = frame_opt {
                        let scale = frame.height() as f32 / 1440.0;
                        if crate::vision::find_car_cursor_lime(&frame, scale).is_some() {
                            cursor_loaded = true;
                            ctx.logger.info(&format!(
                                "[delete_used_subaru_cars] Menu reloaded (cursor found) at attempt {}.",
                                attempt
                            ));
                            break;
                        }
                    }
                }
                if !cursor_loaded {
                    ctx.logger.warn("[delete_used_subaru_cars] Timeout waiting for cursor after deletion.");
                }
                // Add a small extra sleep for stability
                std::thread::sleep(Duration::from_millis(300));

                break;
            }

            drop(capture);
            {
                let mut pad = ctx.pad.lock().unwrap();
                pad.navigate(crate::controller::BUTTON_DPAD_RIGHT, 4);
                pad.sleep_responsive(1.000);
            }
        }

        if !found {
            return false;
        }
    }
    true
}

fn run_garage_cleanup(ctx: &BotFSMContext, count: usize, last_iteration_success: bool) -> bool {
    if !last_iteration_success {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_b();
        pad.sleep_responsive(1.000);
        pad.press_b();
        pad.sleep_responsive(1.000);
        pad.press_dpad_up();
        pad.sleep_responsive(0.350);
        pad.press_dpad_up();
        pad.sleep_responsive(0.350);
        pad.press_a();
        pad.sleep_responsive(1.000);
    }

    // # Step 1: Open brands and select Subaru
    if !select_subaru_brand(ctx) {
        return false;
    }

    // # Step 2: Select player's main Subaru and activate it
    if !find_and_select_player_subaru(ctx) {
        return false;
    }

    // # Step 3: Find and delete the used Subaru cars
    if !delete_used_subaru_cars(ctx, count) {
        return false;
    }

    // # Step 4: Exit back to Home menu
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_b();
        pad.sleep_responsive(1.000);
    }
    true
}

fn exit_to_open_world(ctx: &BotFSMContext) -> bool {
    let mut pad = ctx.pad.lock().unwrap();
    for _ in 1..=8 {
        pad.press_b();
        pad.sleep_responsive(0.150);
    }
    std::thread::sleep(Duration::from_secs(2));
    true
}

fn return_to_hub(ctx: &BotFSMContext) -> bool {
    if !exit_to_open_world(ctx) {
        return false;
    }
    open_pause_menu(ctx)
}
