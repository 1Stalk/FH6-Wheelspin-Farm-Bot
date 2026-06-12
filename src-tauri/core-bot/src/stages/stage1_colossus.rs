use super::*;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    let success = run_iteration(ctx);

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

fn run_driving_monitor(ctx: &BotFSMContext) -> bool {
    let mut hud_detected = false;
    let start_hud_wait = Instant::now();
    let max_hud_wait = Duration::from_secs(20);

    // 1. Wait for driving HUD
    while start_hud_wait.elapsed() < max_hud_wait {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        let mut capture = ctx.capture.lock().unwrap();
        if let Some(frame) = capture.grab_frame() {
            if check_driving_hud(ctx, &frame) {
                hud_detected = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if !hud_detected {
        ctx.logger.warn("Driving HUD not detected within timeout. Proceeding anyway.");
    }

    // 2. Enable autopilot (D-pad Down, then D-pad Left)
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_down();
        pad.sleep_responsive(0.400);
        pad.press_dpad_left();
        pad.sleep_responsive(1.000);
    }

    // 3. Check for autopilot confirmation
    let mut capture = ctx.capture.lock().unwrap();
    if let Some(frame) = capture.grab_frame() {
        if check_autopilot_color(ctx, &frame) {
            ctx.logger.info("Autopilot verified active.");
        }
    }
    drop(capture);

    // 4. Periodic monitoring loop
    let mut last_hud_time = Instant::now();
    let mut last_autopilot_active_time = Instant::now();
    let mut last_log_time = Instant::now() - Duration::from_secs(20);
    let log_interval = Duration::from_secs(20);

    let start_driving_time = Instant::now();
    let drive_duration = Duration::from_secs({
        let cfg = ctx.config.lock().unwrap();
        cfg.stage1_duration as u64 * 60
    });

    let mut last_loop_time = Instant::now();

    while !ctx.is_stop_requested.load(Ordering::Relaxed) {
        if start_driving_time.elapsed() >= drive_duration {
            break;
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        let is_driving = check_driving_hud(ctx, &frame);
        let is_autopilot = check_autopilot_color(ctx, &frame);
        drop(capture);

        let now = Instant::now();
        let elapsed_secs = now.duration_since(last_loop_time).as_secs_f64();
        last_loop_time = now;

        if is_driving || is_autopilot {
            last_hud_time = Instant::now();
            ctx.add_driving_time(elapsed_secs);

            if last_log_time.elapsed() >= log_interval {
                ctx.logger.info(&format!("Autopilot active: driving={}", is_driving));
                last_log_time = Instant::now();
            }

            if is_driving && !is_autopilot {
                if last_autopilot_active_time.elapsed() >= Duration::from_secs(3) {
                    {
                        let mut pad = ctx.pad.lock().unwrap();
                        pad.press_dpad_down();
                        pad.sleep_responsive(0.400);
                        pad.press_dpad_left();
                        pad.sleep_responsive(1.000);
                    }
                    last_autopilot_active_time = Instant::now();
                }
            } else {
                last_autopilot_active_time = Instant::now();
            }
        } else {
            let time_lost = last_hud_time.elapsed();
            if time_lost >= Duration::from_secs(15) {
                if attempt_recovery(ctx, |c, f| check_driving_hud(c, f) || check_autopilot_color(c, f)) {
                    last_hud_time = Instant::now();
                    last_autopilot_active_time = Instant::now();
                    last_loop_time = Instant::now();
                } else {
                    return false;
                }
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }

    true
}

fn check_driving_hud(ctx: &BotFSMContext, frame: &image::RgbImage) -> bool {
    is_on_screen(ctx, frame, "autopilot_driving_disabled.png", 0.80, None)
        || is_on_screen(ctx, frame, "autopilot_driving.png", 0.80, None)
}

fn ensure_menu_and_cursor(ctx: &BotFSMContext) -> bool {
    let max_wait = Duration::from_secs(30);
    let start = Instant::now();

    while start.elapsed() < max_wait {
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

        if is_on_screen(ctx, &frame, "stage2_map_menu.png", 0.92, None) {
            std::thread::sleep(Duration::from_millis(150));
            return true;
        }

        if is_on_screen(ctx, &frame, "stage2_map_menu_wrong_button_selected.png", 0.92, None) {
            let mut correct_found = false;
            for _attempt in 1..=3 {
                std::thread::sleep(Duration::from_millis(500));
                if let Some(frame_retry) = capture.grab_frame() {
                    if is_on_screen(ctx, &frame_retry, "stage2_map_menu.png", 0.92, None) {
                        correct_found = true;
                        break;
                    }
                }
            }
            if correct_found {
                std::thread::sleep(Duration::from_millis(150));
                return true;
            }

            drop(capture);
            {
                let mut pad = ctx.pad.lock().unwrap();
                pad.hold_dpad_up(2.000);
                pad.sleep_responsive(0.400);
            }
            continue;
        }

        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

fn run_difficulty_setup(ctx: &BotFSMContext) -> bool {
    let mut pad = ctx.pad.lock().unwrap();
    // 1. D-pad Down and A to enter "Difficulty and parameters" menu
    pad.press_dpad_down();
    pad.sleep_responsive(0.200);
    pad.press_a();
    
    // 2. Wait 2 seconds for animation
    pad.sleep_responsive(2.200);

    // 3. Press A to enter "Difficulty" settings
    pad.press_a();
    pad.sleep_responsive(1.000);

    // 4. Press D-pad Down
    pad.press_dpad_down();
    pad.sleep_responsive(0.400);

    // 5. Hold D-pad Right for 2 seconds
    pad.hold_dpad_right(2.000);
    pad.sleep_responsive(0.400);

    // 6. Press D-pad Left once
    pad.press_dpad_left();
    pad.sleep_responsive(0.400);

    // 7. Press Start/Menu button to save settings
    pad.press_start();
    pad.sleep_responsive(2.000);

    // 8. Press B, then B to exit back to main race menu
    pad.press_b();
    pad.sleep_responsive(1.000);
    pad.press_b();
    pad.sleep_responsive(1.500);
    true
}

fn exit_to_open_world(ctx: &BotFSMContext) -> bool {
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_start();
        pad.sleep_responsive(1.5);
    }

    let mut pause_menu_opened = false;
    let mut capture = ctx.capture.lock().unwrap();
    for _ in 0..10 {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }
        if let Some(frame) = capture.grab_frame() {
            if is_on_screen(ctx, &frame, "pause_menu.png", 0.85, None)
                || is_on_screen(ctx, &frame, "pause_menu_1st_page.png", 0.85, None)
            {
                pause_menu_opened = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    if !pause_menu_opened {
        ctx.logger.warn("Failed to detect pause menu.");
    }

    drop(capture);
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_dpad_right();
        pad.sleep_responsive(0.400);
        pad.press_a();
        pad.sleep_responsive(0.400);
        pad.press_a();
        pad.sleep_responsive(10.000);
    }

    let mut open_world_confirmed = false;
    let start_check = Instant::now();
    let max_check = Duration::from_secs(30);

    let mut capture = ctx.capture.lock().unwrap();
    while start_check.elapsed() < max_check {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        if let Some(frame) = capture.grab_frame() {
            if is_on_screen(ctx, &frame, "autopilot_icon.png", 0.80, None) {
                open_world_confirmed = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    if open_world_confirmed {
        if let Some(frame) = capture.grab_frame() {
            if check_autopilot_color(ctx, &frame) {
                drop(capture);
                {
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_dpad_down();
                    pad.sleep_responsive(0.400);
                    pad.press_dpad_left();
                    pad.sleep_responsive(1.000);
                }
            }
        }
        true
    } else {
        false
    }
}

fn return_to_hub(ctx: &BotFSMContext) -> bool {
    if !exit_to_open_world(ctx) {
        return false;
    }
    open_pause_menu(ctx)
}

fn run_iteration(ctx: &BotFSMContext) -> bool {
    // # 1. Map Menu sub-state
    ctx.set_sub_state(Some("In Map Menu".to_string()));
    ensure_menu_and_cursor(ctx);

    // # 2. Difficulty Settings sub-state
    ctx.set_sub_state(Some("Difficulty Settings".to_string()));
    run_difficulty_setup(ctx);

    // # 3. Map Menu sub-state re-check
    ctx.set_sub_state(Some("In Map Menu".to_string()));
    ensure_menu_and_cursor(ctx);

    // # 4. Press A to start the race
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
        pad.sleep_responsive(7.000);
    }

    // # 5. Driving (Autopilot) sub-state
    ctx.set_sub_state(Some("Driving (Autopilot)".to_string()));
    if !run_driving_monitor(ctx) {
        return false;
    }

    // # 6. Finished sub-state
    ctx.set_sub_state(Some("Finished".to_string()));
    if !ctx.is_stop_requested.load(Ordering::Relaxed) {
        if !return_to_hub(ctx) {
            return false;
        }
    }
    true
}
