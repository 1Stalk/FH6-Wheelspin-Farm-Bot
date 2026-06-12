use super::*;

pub fn run(ctx: &BotFSMContext) -> StageResult {
    ctx.logger.info("[Nav→Stage3] Starting navigation to Collection Journal...");
    ctx.set_sub_state(Some("Navigation → Car Collection".to_string()));

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

    // ── Step 1: ← → navigate left in pause menu (1st page) ──────────────
    pad.press_dpad_left();
    pad.sleep_responsive(0.400);

    // ── Step 2: A → open Collection Journal ──────────────────────────────
    pad.press_a();
    pad.sleep_responsive(1.200);

    // ── Step 3: → → "Explorer/Adventurer" ─────────
    pad.press_dpad_right();
    pad.sleep_responsive(0.400);

    // ── Step 4: A → open Explorer section ────────────────────────────────
    pad.press_a();
    pad.sleep_responsive(1.200);

    // ── Step 5: ↓ → move to "Car Collection" ──────────
    pad.press_dpad_down();
    pad.sleep_responsive(0.400);

    // ── Step 6: A → open Car Collection → Stage 3 starts ─────────────────
    pad.press_a();
    pad.sleep_responsive(1.500);
    true
}
