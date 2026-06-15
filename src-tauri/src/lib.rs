use std::sync::Arc;
use tauri::{Manager, Emitter};
use core_bot::state_machine::BotFSM;

// Holds the background FSM instance in Tauri state
pub struct BotState(pub Arc<BotFSM>);

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn start_bot(state: tauri::State<'_, BotState>) {
    state.0.start();
}

#[tauri::command]
fn stop_bot(state: tauri::State<'_, BotState>) {
    state.0.stop();
}

#[tauri::command]
fn pause_bot(state: tauri::State<'_, BotState>) {
    state.0.pause();
}

#[tauri::command]
fn resume_bot(state: tauri::State<'_, BotState>) {
    state.0.resume();
}

#[tauri::command]
fn update_config(state: tauri::State<'_, BotState>, new_config: core_bot::config::BotConfig) {
    state.0.update_config(new_config);
}

#[tauri::command]
fn get_bot_status(state: tauri::State<'_, BotState>) -> serde_json::Value {
    let state_lock = state.0.state.lock().unwrap();
    let sub_state_lock = state.0.sub_state.lock().unwrap();
    let cycle_lock = state.0.current_cycle.lock().unwrap();
    let session_sp_lock = state.0.session_sp.lock().unwrap();
    let session_cr_lock = state.0.session_cr.lock().unwrap();
    let session_wspins_lock = state.0.session_wspins.lock().unwrap();
    let config_lock = state.0.config.lock().unwrap();

    serde_json::json!({
        "type": "status",
        "state": state_lock.as_str(),
        "sub_state": *sub_state_lock,
        "cycle": *cycle_lock,
        "session_sp": *session_sp_lock,
        "session_cr": *session_cr_lock,
        "session_wspins": *session_wspins_lock,
        "config": *config_lock,
    })
}

#[tauri::command]
fn run_gamepad_test(state: tauri::State<'_, BotState>) {
    let fsm = Arc::clone(&state.0);
    std::thread::spawn(move || {
        let current = {
            let s = fsm.state.lock().unwrap();
            s.clone()
        };
        if current != core_bot::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run gamepad test: bot is not idle");
            return;
        }
        
        fsm.set_state(core_bot::state_machine::FSMState::TestInputs);
        let ctx = fsm.clone_ptrs();
        core_bot::stages::run_gamepad_test(&ctx);
        fsm.set_state(core_bot::state_machine::FSMState::Idle);
    });
}

#[tauri::command]
fn run_nav_test(state: tauri::State<'_, BotState>, target: String) {
    let fsm = Arc::clone(&state.0);
    std::thread::spawn(move || {
        let current = {
            let s = fsm.state.lock().unwrap();
            s.clone()
        };
        if current != core_bot::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run nav test: bot is not idle");
            return;
        }

        fsm.set_state(core_bot::state_machine::FSMState::Running);
        let ctx = fsm.clone_ptrs();
        
        let _res = match target.as_str() {
            "nav_to_stage1" => core_bot::stages::run_stage_by_name(&ctx, "nav_to_stage1"),
            "nav_to_stage2" => core_bot::stages::run_stage_by_name(&ctx, "nav_to_stage2"),
            "nav_to_stage3" => core_bot::stages::run_stage_by_name(&ctx, "nav_to_stage3"),
            "nav_to_stage4" => core_bot::stages::run_stage_by_name(&ctx, "nav_to_stage4"),
            _ => core_bot::stages::StageResult::Failed,
        };
        
        fsm.set_state(core_bot::state_machine::FSMState::Idle);
    });
}

#[tauri::command]
fn run_cv_diagnostics(state: tauri::State<'_, BotState>) {
    let fsm = Arc::clone(&state.0);
    std::thread::spawn(move || {
        let current = {
            let s = fsm.state.lock().unwrap();
            s.clone()
        };
        if current != core_bot::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run CV diagnostics: bot is not idle");
            return;
        }

        fsm.set_state(core_bot::state_machine::FSMState::Running);
        fsm.logger.info("[CV-Test] Starting CV diagnostics...");

        let mut capture = fsm.capture.lock().unwrap();
        if !capture.find_game_window() {
            fsm.logger.error("[CV-Test] Forza Horizon 6 window not found!");
            fsm.set_state(core_bot::state_machine::FSMState::Idle);
            return;
        }

        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                fsm.logger.error("[CV-Test] Failed to grab frame!");
                fsm.set_state(core_bot::state_machine::FSMState::Idle);
                return;
            }
        };

        // Save the live frame to scratch directory for inspection
        let scratch_dir = std::path::Path::new("scratch");
        let _ = std::fs::create_dir_all(&scratch_dir);
        let save_path = scratch_dir.join("live_frame.png");
        if frame.save(&save_path).is_ok() {
            fsm.logger.info(&format!("[CV-Test] Saved current screenshot to {:?}", save_path));
        } else {
            fsm.logger.warn("[CV-Test] Failed to save screenshot to scratch/live_frame.png");
        }

        let baseline_res = fsm.config.lock().unwrap().baseline_resolution;
        let scale = frame.height() as f32 / baseline_res.1 as f32;

        fsm.logger.info("[CV-Test] Checking screen state templates...");
        let in_car_selection = core_bot::vision::is_on_screen(&frame, "car_selection_menu.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'car_selection_menu.png' (threshold 0.80): {}", in_car_selection));

        let brand_cursor = core_bot::vision::find_template(&frame, "brand_selection_cursor.png", 0.75, None, baseline_res);
        fsm.logger.info(&format!("  -> 'brand_selection_cursor.png' (threshold 0.75): {:?}", brand_cursor));

        let car_cursor = core_bot::vision::find_template(&frame, "car_selection_menu_selected.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'car_selection_menu_selected.png' (threshold 0.80): {:?}", car_cursor));

        // Create visual diagnostics overlay image
        let mut diag_frame = frame.clone();
        
        // Draw vertical grid boundary line at x = 500 (baseline)
        let grid_border_x = (500.0 * scale) as i32;
        core_bot::stages::draw_line(&mut diag_frame, grid_border_x, 0, grid_border_x, frame.height() as i32 - 1, image::Rgb([128, 128, 128]));

        // Draw car selection grid (3 rows x 4 columns)
        for r in 0..3 {
            for c in 0..4 {
                let cell_cx = core_bot::vision::CAR_GRID_START_X + c as f32 * core_bot::vision::CAR_CELL_W;
                let cell_cy = core_bot::vision::CAR_GRID_START_Y + r as f32 * core_bot::vision::CAR_CELL_H;

                let screen_cx = (cell_cx * scale) as i32;
                let screen_cy = (cell_cy * scale) as i32;

                let cell_w = (core_bot::vision::CAR_CELL_W * scale) as i32;
                let cell_h = (core_bot::vision::CAR_CELL_H * scale) as i32;

                let x1 = screen_cx - cell_w / 2;
                let y1 = screen_cy - cell_h / 2;

                // Draw cell box outline in blue
                core_bot::stages::draw_rect(&mut diag_frame, x1, y1, cell_w, cell_h, image::Rgb([0, 80, 150]));
                // Draw small cell center crosshair
                core_bot::stages::draw_crosshair(&mut diag_frame, screen_cx, screen_cy, 5, image::Rgb([0, 80, 150]));
            }
        }

        // Draw selection cursor outline if detected
        if let Some((cx, cy)) = car_cursor {
            let cx_i = cx as i32;
            let cy_i = cy as i32;
            let cursor_w = (406.0 * scale) as i32;
            let cursor_h = (340.0 * scale) as i32;
            // Green box highlight
            core_bot::stages::draw_rect_thick(&mut diag_frame, cx_i, cy_i, cursor_w, cursor_h, 3, image::Rgb([0, 255, 0]));
        }

        let mut cursor_col = -1;
        let mut cursor_row = -1;
        if let Some((cx, cy)) = car_cursor {
            let b_cx = cx as f32 / scale;
            let b_cy = cy as f32 / scale;
            let cursor_center_x = b_cx + core_bot::vision::CAR_CURSOR_OFFSET_X;
            let cursor_center_y = b_cy + core_bot::vision::CAR_CURSOR_OFFSET_Y;
            cursor_col = ((cursor_center_x - core_bot::vision::CAR_GRID_START_X) / core_bot::vision::CAR_CELL_W).round() as i32;
            cursor_row = ((cursor_center_y - core_bot::vision::CAR_GRID_START_Y) / core_bot::vision::CAR_CELL_H).round() as i32;
            fsm.logger.info(&format!("[CV-Test] Selection cursor detected at grid cell: col={}, row={}", cursor_col, cursor_row));
        } else {
            fsm.logger.info("[CV-Test] Selection cursor NOT detected on screen.");
        }

        #[derive(Clone)]
        struct DetectedCar {
            col: i32,
            row: i32,
            name: &'static str,
            is_favorite: bool,
            score: f32,
            cell_cx: f32,
            cell_cy: f32,
        }

        let templates = vec![
            ("Toyota_2019.png", "Toyota Tacoma"),
            ("Nissan_1989.png", "Nissan S-Cargo"),
            ("subaru_impreza_1998.png", "Subaru Impreza"),
        ];

        let mut candidates = Vec::new();

        fsm.logger.info("[CV-Test] Scanning all 12 grid cells for car matches...");
        for r in 0..3 {
            for c in 0..4 {
                let cell_cx = core_bot::vision::CAR_GRID_START_X + c as f32 * core_bot::vision::CAR_CELL_W;
                let cell_cy = core_bot::vision::CAR_GRID_START_Y + r as f32 * core_bot::vision::CAR_CELL_H;
                let cell_rx = (cell_cx - core_bot::vision::CAR_CELL_W / 2.0).max(0.0) as i32;
                let cell_ry = (cell_cy - core_bot::vision::CAR_CELL_H / 2.0).max(0.0) as i32;
                let cell_rw = core_bot::vision::CAR_CELL_W as i32;
                let cell_rh = core_bot::vision::CAR_CELL_H as i32;

                // Check for favorite heart (using threshold 0.90 as in stage 1)
                let is_favorite = core_bot::vision::is_on_screen(
                    &frame,
                    "car_favorite_heart.png",
                    0.90,
                    Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    baseline_res,
                );

                // Check for class B (used for Subaru check)
                let is_class_b = core_bot::vision::is_on_screen(
                    &frame,
                    "car_class_b.png",
                    0.80,
                    Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                    baseline_res,
                );

                for &(tpl, name) in &templates {
                    let cell_matches = core_bot::vision::find_all_matches(
                        &frame,
                        tpl,
                        0.70, // Scan down to 0.70 to log close hits
                        Some((cell_rx, cell_ry, cell_rw, cell_rh)),
                        baseline_res,
                    );

                    if !cell_matches.is_empty() {
                        let best_match = cell_matches
                            .iter()
                            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                            .unwrap();
                        let (mx, my, score) = *best_match;

                        fsm.logger.info(&format!(
                            "  -> Cell (col={}, row={}) MATCHED {}: score={:.4} at x={}, y={}, favorite={}, class_b={}",
                            c, r, name, score, mx, my, is_favorite, is_class_b
                        ));

                        // Qualifies if score >= 0.85, and for Subaru: not class B
                        let passes_threshold = score >= 0.85;
                        let is_valid_candidate = if tpl == "subaru_impreza_1998.png" {
                            passes_threshold && !is_class_b
                        } else {
                            passes_threshold
                        };

                        if is_valid_candidate {
                            candidates.push(DetectedCar {
                                col: c as i32,
                                row: r as i32,
                                name,
                                is_favorite,
                                score,
                                cell_cx,
                                cell_cy,
                            });

                            // Draw crosshair at match in template-specific color
                            let match_color = match tpl {
                                "Toyota_2019.png" => image::Rgb([240, 168, 74]), // Orange
                                "subaru_impreza_1998.png" => image::Rgb([254, 2, 136]), // Pink
                                _ => image::Rgb([44, 188, 164]), // Teal (Nissan)
                            };
                            core_bot::stages::draw_crosshair(&mut diag_frame, mx as i32, my as i32, 15, match_color);

                            // Draw favorite bounding box in Magenta, or general detection in Yellow
                            let cell_rx_s = (cell_rx as f32 * scale) as i32;
                            let cell_ry_s = (cell_ry as f32 * scale) as i32;
                            let cell_rw_s = (cell_rw as f32 * scale) as i32;
                            let cell_rh_s = (cell_rh as f32 * scale) as i32;
                            let rect_color = if is_favorite { image::Rgb([255, 0, 255]) } else { match_color };
                            core_bot::stages::draw_rect_thick(&mut diag_frame, cell_rx_s, cell_ry_s, cell_rw_s, cell_rh_s, 2, rect_color);
                        }
                    }
                }
            }
        }

        fsm.logger.info(&format!("[CV-Test] Cell scan complete. Found {} valid car candidates.", candidates.len()));

        // Emulate navigation priority
        if !candidates.is_empty() && cursor_col >= 0 && cursor_row >= 0 {
            // Emulate selection logic
            let target_car = if let Some(fav) = candidates.iter().find(|c| c.is_favorite && c.name != "Subaru Impreza") {
                Some(fav.clone())
            } else {
                // Otherwise, pick the closest candidate by Manhattan distance
                let mut sorted = candidates.clone();
                sorted.sort_by_key(|c| (c.col - cursor_col).abs() + (c.row - cursor_row).abs());
                Some(sorted[0].clone())
            };

            if let Some(target) = target_car {
                let cols_diff = target.col - cursor_col;
                let rows_diff = target.row - cursor_row;

                let mut moves = Vec::new();
                if cols_diff > 0 {
                    moves.push(format!("DPAD_RIGHT x {}", cols_diff));
                } else if cols_diff < 0 {
                    moves.push(format!("DPAD_LEFT x {}", cols_diff.abs()));
                }
                if rows_diff > 0 {
                    moves.push(format!("DPAD_DOWN x {}", rows_diff));
                } else if rows_diff < 0 {
                    moves.push(format!("DPAD_UP x {}", rows_diff.abs()));
                }

                let moves_str = if moves.is_empty() {
                    "None (already on target)".to_string()
                } else {
                    moves.join(", ")
                };

                fsm.logger.info(&format!(
                    "[CV-Test] SIMULATED SELECTION for {}:",
                    target.name
                ));
                fsm.logger.info(&format!(
                    "  -> Start: cursor at col={}, row={}",
                    cursor_col, cursor_row
                ));
                fsm.logger.info(&format!(
                    "  -> Target: {} at col={}, row={}, score={:.4} (favorite={})",
                    target.name, target.col, target.row, target.score, target.is_favorite
                ));
                fsm.logger.info(&format!(
                    "  -> Emulated navigation inputs: {}, then press A",
                    moves_str
                ));

                // Draw navigation path line on the diagnostics image
                if let Some(cursor_pos) = car_cursor {
                    let b_cx = cursor_pos.0 as f32 / scale;
                    let b_cy = cursor_pos.1 as f32 / scale;
                    let cursor_center_x = b_cx + core_bot::vision::CAR_CURSOR_OFFSET_X;
                    let cursor_center_y = b_cy + core_bot::vision::CAR_CURSOR_OFFSET_Y;

                    let start_x = (cursor_center_x * scale) as i32;
                    let start_y = (cursor_center_y * scale) as i32;
                    let end_x = (target.cell_cx * scale) as i32;
                    let end_y = (target.cell_cy * scale) as i32;

                    // Draw solid bright green line
                    core_bot::stages::draw_line(&mut diag_frame, start_x, start_y, end_x, end_y, image::Rgb([0, 255, 0]));
                    // Draw target crosshair
                    core_bot::stages::draw_crosshair(&mut diag_frame, end_x, end_y, 10, image::Rgb([0, 255, 0]));
                }
            }
        } else {
            fsm.logger.info("[CV-Test] Emulated selection skipped: either cursor or candidates not detected.");
        }

        // Save visual diagnostics image
        let diag_grid_path = scratch_dir.join("car_diagnostics_grid.png");
        if diag_frame.save(&diag_grid_path).is_ok() {
            fsm.logger.info(&format!("[CV-Test] Saved visual diagnostics grid to {:?}", diag_grid_path));
        } else {
            fsm.logger.warn("[CV-Test] Failed to save diagnostics grid to scratch/car_diagnostics_grid.png");
        }

        fsm.logger.info("[CV-Test] CV diagnostics completed.");
        fsm.set_state(core_bot::state_machine::FSMState::Idle);
    });
}

#[tauri::command]
fn check_vigem_status() -> bool {
    vigem_client::Client::connect().is_ok()
}

#[tauri::command]
fn start_vigem_install(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let emit_status = |status: &str, msg: Option<&str>| {
            let payload = serde_json::json!({
                "status": status,
                "message": msg.unwrap_or(""),
            });
            let _ = app.emit("vigem-install-status", payload);
        };

        emit_status("downloading", None);

        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("ViGEmBus_1.22.0_Setup.exe");

        // Use PowerShell to download the unified 1.22.0 installer
        let download_url = "https://github.com/nefarius/ViGEmBus/releases/download/v1.22.0/ViGEmBus_1.22.0_x64_x86_arm64.exe";
        let download_cmd = format!(
            "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            download_url,
            installer_path.to_string_lossy().replace('\\', "/")
        );

        let download_status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &download_cmd])
            .status();

        match download_status {
            Ok(status) if status.success() => {
                emit_status("installing", None);

                // Run the installer. Requires admin rights on execution (triggers UAC popup)
                let install_status = std::process::Command::new(&installer_path)
                    .status();

                match install_status {
                    Ok(inst_status) if inst_status.success() => {
                        let _ = std::fs::remove_file(&installer_path);
                        
                        // Check if we can connect now
                        if vigem_client::Client::connect().is_ok() {
                            emit_status("success", None);
                        } else {
                            emit_status("error", Some("Driver installer finished but ViGEmBus client connection failed. A system reboot might be required."));
                        }
                    }
                    Ok(inst_status) => {
                        let _ = std::fs::remove_file(&installer_path);
                        emit_status("error", Some(&format!("Installer failed with exit code: {:?}", inst_status.code())));
                    }
                    Err(e) => {
                        let _ = std::fs::remove_file(&installer_path);
                        emit_status("error", Some(&format!("Failed to execute installer: {:?}", e)));
                    }
                }
            }
            Ok(status) => {
                emit_status("error", Some(&format!("Download failed with exit code: {:?}", status.code())));
            }
            Err(e) => {
                emit_status("error", Some(&format!("Failed to start download process: {:?}", e)));
            }
        }
    });
}

#[tauri::command]
fn get_auth_token() -> String {
    "local_rust".to_string()
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let fsm = Arc::new(BotFSM::new(Some(app.handle().clone())));
            app.manage(BotState(fsm));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Ensure bot stops on window close
                if let Some(bot_state) = window.try_state::<BotState>() {
                    bot_state.0.stop();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_bot,
            stop_bot,
            pause_bot,
            resume_bot,
            update_config,
            get_bot_status,
            run_gamepad_test,
            run_nav_test,
            run_cv_diagnostics,
            get_auth_token,
            check_vigem_status,
            start_vigem_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
