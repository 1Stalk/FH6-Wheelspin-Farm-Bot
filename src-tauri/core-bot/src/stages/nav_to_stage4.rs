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
    pad.sleep_responsive(5.000);

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
