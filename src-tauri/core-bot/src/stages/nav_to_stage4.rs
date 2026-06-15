use super::*;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    ctx.logger
        .info("[Nav→Stage4] Starting navigation to Design & Paint Car Selection...");
    ctx.set_sub_state(Some("Navigation → Design & Paint".to_string()));

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

fn navigate(ctx: &BotFSMContext) -> bool {
    let mut pad = ctx.pad.lock().unwrap();

    // ── Step 0: 1s delay before starting ────────────────────────────────
    pad.sleep_responsive(1.000);

    // ── Step 1: RB × 3 → "My Horizon" ─────────────────────
    pad.navigate(crate::controller::BUTTON_RB, 2);
    pad.sleep_responsive(0.800);

    // ── Step 2: A → go to Home screen / Horizon festival ─────────────────
    pad.press_a();
    pad.sleep_responsive(0.400);

    // ── Step 3: A → confirm ───────────────────────────────────────────────
    pad.press_a();
    pad.sleep_responsive(1.500);

    // Drop pad lock before waiting, because waiting could take a while
    drop(pad);

    ctx.logger.info("[Nav→Stage4] Waiting for Horizon House/Garage to load (detecting 'car_selection_menu.png')...");
    let mut loaded = false;
    let max_attempts = 30;

    for attempt in 1..=max_attempts {
        if ctx.is_stop_requested.load(Ordering::Relaxed) {
            return false;
        }

        ctx.logger.info(&format!(
            "[Nav→Stage4] Checking for Horizon House/Garage load (attempt {}/{})...",
            attempt, max_attempts
        ));

        let frame = {
            let mut capture = ctx.capture.lock().unwrap();
            capture.grab_frame()
        };

        if let Some(frame) = frame {
            if is_on_screen(ctx, &frame, "car_selection_menu.png", 0.80, None) {
                loaded = true;
                ctx.logger.info(&format!(
                    "[Nav→Stage4] Loaded successfully! Found 'car_selection_menu.png' at attempt {}.",
                    attempt
                ));
                break;
            }
        } else {
            ctx.logger.warn(&format!(
                "[Nav→Stage4] Failed to grab frame during loading wait (attempt {}/{}).",
                attempt, max_attempts
            ));
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    if !loaded {
        ctx.logger.error("[Nav→Stage4] Timeout waiting for Horizon House/Garage load after 30 attempts!");
        return false;
    }

    // Add a short delay after loaded to ensure game UI is responsive
    std::thread::sleep(Duration::from_millis(500));

    // Re-lock pad to continue navigation
    let mut pad = ctx.pad.lock().unwrap();

    // ── Step 4: RB × 2 → "Cars" tab ────────────────────────────
    pad.navigate(crate::controller::BUTTON_RB, 2);
    pad.sleep_responsive(0.650);

    // ── Step 5: ↓↓ → select "Design & Paint" ─────────
    pad.press_dpad_down();
    pad.sleep_responsive(0.400);
    pad.press_dpad_down();
    pad.sleep_responsive(0.400);

    // ── Step 6: A → enter Design & Paint ─────────────────────────────────
    pad.press_a();
    pad.sleep_responsive(1.000);

    // ── Step 7: hold ↓ 2s → scroll to "Select Car" ──────
    pad.hold_dpad_down(2.0);
    pad.sleep_responsive(0.500);

    // ── Step 8: A → enter Select Car → Stage 4 starts ────────────────────
    pad.press_a();
    pad.sleep_responsive(1.000);
    true
}
