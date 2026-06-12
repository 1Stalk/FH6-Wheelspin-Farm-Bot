use super::*;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    let _only_stage2 = {
        let cfg = ctx.config.lock().unwrap();
        let stages = &cfg.stages_enabled;
        stages.get("stage2").copied().unwrap_or(true)
            && !stages.get("stage1").copied().unwrap_or(false)
            && !stages.get("stage3").copied().unwrap_or(false)
            && !stages.get("stage4").copied().unwrap_or(false)
    };

    let mut num_iterations = {
        let cfg = ctx.config.lock().unwrap();
        cfg.stage2_iterations as usize
    };
    if num_iterations == 0 {
        num_iterations = 999999;
    }

    if !ctx.is_stop_requested.load(Ordering::Relaxed) {
        run_stage2_difficulty_flow(ctx);
    }

    let mut success = true;
    for i in 1..=num_iterations {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }

        ctx.logger.info(&format!("--- Stage 2: Iteration {} of {} ---", i, num_iterations));

        let iter_success = run_macro_iteration(ctx);
        if !iter_success || ctx.is_stop_requested.load(Ordering::Relaxed) {
            success = false;
            break;
        }

        ctx.add_session_sp(10);

        if i < num_iterations {
            run_restart_transition(ctx);
        } else {
            if !run_finish_transition(ctx) {
                success = false;
            }
        }

        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }
    }

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

fn check_driving_hud(ctx: &BotFSMContext, frame: &image::RgbImage) -> bool {
    is_on_screen(ctx, frame, "autopilot_driving_disabled.png", 0.80, None)
        || is_on_screen(ctx, frame, "autopilot_driving.png", 0.80, None)
}

fn get_best_match_score(ctx: &BotFSMContext, frame: &image::RgbImage, template_name: &str) -> f32 {
    let matches = find_all_matches(ctx, frame, template_name, 0.40, None);
    if matches.is_empty() {
        0.0
    } else {
        matches[0].2
    }
}

fn detect_sub_state(ctx: &BotFSMContext, frame: &image::RgbImage, verbose: bool) -> Option<&'static str> {
    let map_score = get_best_match_score(ctx, frame, "stage2_map_menu.png");
    let post_finish_score = get_best_match_score(ctx, frame, "stage2_post_finish.png");
    let finish_banner_score = get_best_match_score(ctx, frame, "stage2_finish_banner.png");
    let autopilot_driving_score = get_best_match_score(ctx, frame, "autopilot_driving.png");
    let autopilot_driving_disabled_score = get_best_match_score(ctx, frame, "autopilot_driving_disabled.png");

    if verbose {
        ctx.logger.info(&format!(
            "[Stage2 CV] Scores -> map: {:.4} (th 0.85), post_finish: {:.4} (th 0.85), finish_banner: {:.4} (th 0.85), driving: {:.4} (th 0.80), disabled: {:.4} (th 0.80)",
            map_score, post_finish_score, finish_banner_score, autopilot_driving_score, autopilot_driving_disabled_score
        ));
    }

    if map_score >= 0.85 {
        return Some("In Map Menu");
    }
    if post_finish_score >= 0.85 {
        return Some("Results Menu");
    }
    if finish_banner_score >= 0.85 {
        return Some("Finished");
    }
    if autopilot_driving_score >= 0.80 || autopilot_driving_disabled_score >= 0.80 {
        return Some("Driving");
    }
    None
}

fn ensure_menu_and_cursor(ctx: &BotFSMContext) -> bool {
    let max_wait = Duration::from_secs(15);
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

        if is_on_screen(ctx, &frame, "stage2_map_menu.png", 0.85, None) {
            std::thread::sleep(Duration::from_millis(150));
            return true;
        }

        if is_on_screen(ctx, &frame, "stage2_map_menu_wrong_button_selected.png", 0.85, None) {
            let mut correct_found = false;
            for _attempt in 1..=3 {
                std::thread::sleep(Duration::from_millis(500));
                if let Some(frame_retry) = capture.grab_frame() {
                    if is_on_screen(ctx, &frame_retry, "stage2_map_menu.png", 0.85, None) {
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
    pad.sleep_responsive(1.000); // Wait for difficulty settings screen to load

    // 4. Press D-pad Down
    pad.press_dpad_down();
    pad.sleep_responsive(0.400);

    // 5. Hold D-pad Left for 2 seconds (Stage 2 specific)
    pad.hold_dpad_left(2.000);
    pad.sleep_responsive(0.400);

    // 6. Press Start/Menu button to save settings
    pad.press_start();
    pad.sleep_responsive(2.000); // Wait for save animation

    // 7. Press B, then B to exit back to main race menu
    pad.press_b();
    pad.sleep_responsive(1.000);
    pad.press_b();
    pad.sleep_responsive(1.500);
    true
}

fn run_stage2_difficulty_flow(ctx: &BotFSMContext) {
    ctx.set_sub_state(Some("In Map Menu".to_string()));
    if ensure_menu_and_cursor(ctx) {
        ctx.set_sub_state(Some("Difficulty Settings".to_string()));
        run_difficulty_setup(ctx);
        ctx.set_sub_state(Some("In Map Menu".to_string()));
        ensure_menu_and_cursor(ctx);
    }
}

fn run_restart_transition(ctx: &BotFSMContext) {
    ctx.logger.info("Stage 2: Sending restart gamepad input (X -> A)...");
    let mut pad = ctx.pad.lock().unwrap();
    // Pressing X to restart the custom map race.
    pad.press_x();
    pad.sleep_responsive(0.600); // Wait for confirmation dialog to appear
    
    // Confirming restart by pressing A.
    pad.press_a();
    pad.sleep_responsive(1.500); // Wait for reload process to initiate
    ctx.logger.info("Stage 2: Restart gamepad input completed.");
}

fn exit_to_open_world(ctx: &BotFSMContext) -> bool {
    {
        let mut pad = ctx.pad.lock().unwrap();
        // Stage 2 ends at the post-finish menu ("X Restart" / "A Continue").
        // Pressing A dismisses the screen and loads the open world.
        pad.press_a();
        pad.sleep_responsive(2.0);
    }

    let mut open_world_confirmed = false;
    let start = Instant::now();
    let max_check = Duration::from_secs(25);

    let mut capture = ctx.capture.lock().unwrap();
    while start.elapsed() < max_check {
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
    open_world_confirmed
}

fn return_to_hub(ctx: &BotFSMContext) -> bool {
    if !exit_to_open_world(ctx) {
        return false;
    }
    open_pause_menu(ctx)
}

fn run_finish_transition(ctx: &BotFSMContext) -> bool {
    return_to_hub(ctx)
}

fn run_macro_iteration(ctx: &BotFSMContext) -> bool {
    // --- SUB-STAGE 1: Map Menu ---
    ctx.set_sub_state(Some("In Map Menu".to_string()));

    let start_map = Instant::now();
    let max_map = Duration::from_secs(12);
    let mut _map_menu_loaded = false;
    let mut last_log = Instant::now();

    let mut capture = ctx.capture.lock().unwrap();
    while start_map.elapsed() < max_map {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }
        if let Some(frame) = capture.grab_frame() {
            let verbose = last_log.elapsed() >= Duration::from_secs(1);
            if verbose {
                last_log = Instant::now();
            }
            if detect_sub_state(ctx, &frame, verbose) == Some("In Map Menu") {
                _map_menu_loaded = true;
                std::thread::sleep(Duration::from_secs(1));
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    // Press A to start the race
    drop(capture);
    {
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_a();
    }

    // --- SUB-STAGE 2: Driving ---
    ctx.set_sub_state(Some("Driving".to_string()));
    ctx.logger.info("Sub-stage: Driving. Squeezing Right Trigger...");
    {
        // Squeeze RT to peak 1.0. We'll hold it in a loop with tremors.
        let mut pad = ctx.pad.lock().unwrap();
        pad.press_rt(1.0, Some(0.1), 10.0);
    }

    let start_drive = Instant::now();
    let mut last_hud_time = Instant::now();
    let max_drive = Duration::from_secs(35);
    let mut finished = false;
    let mut last_log_drive = Instant::now();

    ctx.logger.info("Stage 2: Starting driving phase...");

    while start_drive.elapsed() < max_drive {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }

        {
            // Squeeze accelerator (RT) and apply subtle Tremors to simulate human hold
            let mut pad = ctx.pad.lock().unwrap();
            pad.pad.right_trigger_float(1.0);
            let _ = pad.pad.resume_inputs();
        }

        let mut capture = ctx.capture.lock().unwrap();
        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
        };

        let verbose = last_log_drive.elapsed() >= Duration::from_secs(1);
        if verbose {
            last_log_drive = Instant::now();
        }
        let current_sub = detect_sub_state(ctx, &frame, verbose);
        if current_sub == Some("Finished") || current_sub == Some("Results Menu") {
            ctx.logger.info(&format!("Stage 2: Driving completed. Detected sub-state: {:?}", current_sub));
            finished = true;
            break;
        }

        let is_driving = check_driving_hud(ctx, &frame);
        drop(capture);

        if is_driving {
            last_hud_time = Instant::now();
        } else {
            let time_lost = last_hud_time.elapsed();
            if time_lost >= Duration::from_secs(15) {
                ctx.logger.warn(&format!("Stage 2: Autopilot inactive for {:.1}s. Initiating recovery...", time_lost.as_secs_f32()));
                {
                    // Release trigger before recovery to prevent continuous throttle
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.reset();
                }

                if attempt_recovery(ctx, check_driving_hud) {
                    // Recovery succeeded! Reset variables and re-squeeze trigger
                    last_hud_time = Instant::now();
                    let mut pad = ctx.pad.lock().unwrap();
                    pad.press_rt(1.0, Some(0.1), 10.0);
                } else {
                    ctx.logger.error("Stage 2: Recovery failed.");
                    return false;
                }
            }
        }

        // Sleep briefly (100ms) to allow other tasks to run and reduce CPU usage
        std::thread::sleep(Duration::from_millis(100));
    }

    {
        // Always release accelerator when exiting driving stage
        let mut pad = ctx.pad.lock().unwrap();
        pad.reset();
    }

    if !finished {
        ctx.logger.warn("Stage 2: Driving phase timed out (35s) without detecting end of race.");
    }

    // --- SUB-STAGE 3: Finished / Loading ---
    ctx.set_sub_state(Some("Finished".to_string()));
    ctx.logger.info("Sub-stage: Finished. Waiting for rewards screen to settle...");

    let start_finished = Instant::now();
    let max_finished = Duration::from_secs(15);
    let mut settled = false;
    let mut last_log_finished = Instant::now();

    let mut capture = ctx.capture.lock().unwrap();
    let mut last_logged_sub = None;
    while start_finished.elapsed() < max_finished {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            break;
        }

        if let Some(frame) = capture.grab_frame() {
            let verbose = last_log_finished.elapsed() >= Duration::from_secs(1);
            if verbose {
                last_log_finished = Instant::now();
            }
            let current_sub = detect_sub_state(ctx, &frame, verbose);
            if current_sub != last_logged_sub {
                ctx.logger.info(&format!("Stage 2: Screen state during transition: {:?}", current_sub));
                last_logged_sub = current_sub;
            }
            if current_sub == Some("Results Menu") {
                ctx.logger.info("Stage 2: Results Menu screen confirmed. Settling for 1.0s...");
                settled = true;
                // Let the menu animations finish and input lock release
                let settle_dur = Duration::from_secs(1);
                let start_settle = Instant::now();
                while start_settle.elapsed() < settle_dur {
                    if ctx.is_stop_requested.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if !settled {
        ctx.logger.error("Stage 2: Timed out (15s) waiting for Results Menu.");
    } else {
        ctx.logger.info("Stage 2: Screen settled. Ready for restart/exit.");
    }

    // --- SUB-STAGE 4: Post Finish Menu ---
    ctx.set_sub_state(Some("Results Menu".to_string()));
    settled
}
