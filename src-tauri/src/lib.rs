use std::sync::Arc;
use tauri::{Manager, Emitter};

pub mod capture;
pub mod config;
pub mod controller;
pub mod playback;
pub mod state_machine;
pub mod vision;
pub mod stages;

use crate::state_machine::BotFSM;

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
fn update_config(state: tauri::State<'_, BotState>, new_config: crate::config::BotConfig) {
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
        if current != crate::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run gamepad test: bot is not idle");
            return;
        }
        
        fsm.set_state(crate::state_machine::FSMState::TestInputs);
        let ctx = fsm.clone_ptrs();
        crate::stages::run_gamepad_test(&ctx);
        fsm.set_state(crate::state_machine::FSMState::Idle);
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
        if current != crate::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run nav test: bot is not idle");
            return;
        }

        fsm.set_state(crate::state_machine::FSMState::Running);
        let ctx = fsm.clone_ptrs();
        
        let _res = match target.as_str() {
            "nav_to_stage1" => crate::stages::run_stage_by_name(&ctx, "nav_to_stage1"),
            "nav_to_stage2" => crate::stages::run_stage_by_name(&ctx, "nav_to_stage2"),
            "nav_to_stage3" => crate::stages::run_stage_by_name(&ctx, "nav_to_stage3"),
            "nav_to_stage4" => crate::stages::run_stage_by_name(&ctx, "nav_to_stage4"),
            _ => crate::stages::StageResult::Failed,
        };
        
        fsm.set_state(crate::state_machine::FSMState::Idle);
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
        if current != crate::state_machine::FSMState::Idle {
            fsm.logger.warn("Cannot run CV diagnostics: bot is not idle");
            return;
        }

        fsm.set_state(crate::state_machine::FSMState::Running);
        fsm.logger.info("[CV-Test] Starting CV diagnostics...");

        let mut capture = fsm.capture.lock().unwrap();
        if !capture.find_game_window() {
            fsm.logger.error("[CV-Test] Forza Horizon 6 window not found!");
            fsm.set_state(crate::state_machine::FSMState::Idle);
            return;
        }

        let frame = match capture.grab_frame() {
            Some(f) => f,
            None => {
                fsm.logger.error("[CV-Test] Failed to grab frame!");
                fsm.set_state(crate::state_machine::FSMState::Idle);
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
        let in_car_selection = crate::vision::is_on_screen(&frame, "car_selection_menu.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'car_selection_menu.png' (threshold 0.80): {}", in_car_selection));

        let brand_cursor = crate::vision::find_template(&frame, "brand_selection_cursor.png", 0.75, None, baseline_res);
        fsm.logger.info(&format!("  -> 'brand_selection_cursor.png' (threshold 0.75): {:?}", brand_cursor));

        let nissan_brand = crate::vision::find_template(&frame, "nissan_brand_big.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'nissan_brand_big.png' (threshold 0.80): {:?}", nissan_brand));

        let nissan_brand_selected = crate::vision::find_template(&frame, "nissan_brand_big_selected.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'nissan_brand_big_selected.png' (threshold 0.80): {:?}", nissan_brand_selected));

        let car_cursor = crate::vision::find_template(&frame, "car_selection_menu_selected.png", 0.80, None, baseline_res);
        fsm.logger.info(&format!("  -> 'car_selection_menu_selected.png' (threshold 0.80): {:?}", car_cursor));

        // Create visual diagnostics overlay image
        let mut diag_frame = frame.clone();
        
        // Draw vertical grid boundary line at x = 500 (baseline)
        let grid_border_x = (500.0 * scale) as i32;
        crate::stages::draw_line(&mut diag_frame, grid_border_x, 0, grid_border_x, frame.height() as i32 - 1, image::Rgb([128, 128, 128]));

        // Draw car selection grid (3 rows x 4 columns)
        for r in 0..3 {
            for c in 0..4 {
                let cell_cx = crate::vision::CAR_GRID_START_X + c as f32 * crate::vision::CAR_CELL_W;
                let cell_cy = crate::vision::CAR_GRID_START_Y + r as f32 * crate::vision::CAR_CELL_H;

                let screen_cx = (cell_cx * scale) as i32;
                let screen_cy = (cell_cy * scale) as i32;

                let cell_w = (crate::vision::CAR_CELL_W * scale) as i32;
                let cell_h = (crate::vision::CAR_CELL_H * scale) as i32;

                let x1 = screen_cx - cell_w / 2;
                let y1 = screen_cy - cell_h / 2;

                // Draw cell box outline in blue
                crate::stages::draw_rect(&mut diag_frame, x1, y1, cell_w, cell_h, image::Rgb([0, 80, 150]));
                // Draw small cell center crosshair
                crate::stages::draw_crosshair(&mut diag_frame, screen_cx, screen_cy, 5, image::Rgb([0, 80, 150]));
            }
        }

        // Draw selection cursor outline if detected
        if let Some((cx, cy)) = car_cursor {
            let cx_i = cx as i32;
            let cy_i = cy as i32;
            let cursor_w = (406.0 * scale) as i32;
            let cursor_h = (340.0 * scale) as i32;
            // Green box highlight
            crate::stages::draw_rect_thick(&mut diag_frame, cx_i, cy_i, cursor_w, cursor_h, 3, image::Rgb([0, 255, 0]));
        }

        let mut cursor_col = -1;
        let mut cursor_row = -1;
        if let Some((cx, cy)) = car_cursor {
            let b_cx = cx as f32 / scale;
            let b_cy = cy as f32 / scale;
            let cursor_center_x = b_cx + crate::vision::CAR_CURSOR_OFFSET_X;
            let cursor_center_y = b_cy + crate::vision::CAR_CURSOR_OFFSET_Y;
            cursor_col = ((cursor_center_x - crate::vision::CAR_GRID_START_X) / crate::vision::CAR_CELL_W).round() as i32;
            cursor_row = ((cursor_center_y - crate::vision::CAR_GRID_START_Y) / crate::vision::CAR_CELL_H).round() as i32;
            fsm.logger.info(&format!("[CV-Test] Selection cursor detected at grid cell: col={}, row={}", cursor_col, cursor_row));
        } else {
            fsm.logger.info("[CV-Test] Selection cursor NOT detected on screen.");
        }

        fsm.logger.info("[CV-Test] Scanning all 12 grid cells for Nissan S-Cargo matches...");
        let mut total_matches = 0;

        for r in 0..3 {
            for c in 0..4 {
                let cell_cx = crate::vision::CAR_GRID_START_X + c as f32 * crate::vision::CAR_CELL_W;
                let cell_cy = crate::vision::CAR_GRID_START_Y + r as f32 * crate::vision::CAR_CELL_H;
                let cell_rx = (cell_cx - crate::vision::CAR_CELL_W / 2.0).max(0.0) as i32;
                let cell_ry = (cell_cy - crate::vision::CAR_CELL_H / 2.0).max(0.0) as i32;
                let cell_rw = crate::vision::CAR_CELL_W as i32;
                let cell_rh = crate::vision::CAR_CELL_H as i32;

                // Look for templates inside the cell bounds (down to 0.70 threshold for diagnostics)
                let cell_matches = crate::vision::find_all_matches(&frame, "Nissan_1989.png", 0.70, Some((cell_rx, cell_ry, cell_rw, cell_rh)), baseline_res);

                if !cell_matches.is_empty() {
                    total_matches += 1;
                    let best_match = cell_matches.iter().max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).unwrap();
                    let (sx, sy, score) = *best_match;

                    let is_favorite = crate::vision::is_on_screen(&frame, "car_favorite_heart.png", 0.80, Some((cell_rx, cell_ry, cell_rw, cell_rh)), baseline_res);

                    // Draw matched car crosshair in bright red
                    crate::stages::draw_crosshair(&mut diag_frame, sx as i32, sy as i32, 15, image::Rgb([255, 0, 0]));

                    // Draw favorite heart search bounding box (magenta if favorite, yellow if not)
                    let cell_rx_s = (cell_rx as f32 * scale) as i32;
                    let cell_ry_s = (cell_ry as f32 * scale) as i32;
                    let cell_rw_s = (cell_rw as f32 * scale) as i32;
                    let cell_rh_s = (cell_rh as f32 * scale) as i32;
                    let fav_color = if is_favorite { image::Rgb([255, 0, 255]) } else { image::Rgb([255, 255, 0]) };
                    crate::stages::draw_rect_thick(&mut diag_frame, cell_rx_s, cell_ry_s, cell_rw_s, cell_rh_s, 2, fav_color);

                    fsm.logger.info(&format!(
                        "  -> Cell (col={}, row={}) MATCHED S-Cargo: score={:.4} at x={}, y={} (favorite={})",
                        c, r, score, sx, sy, is_favorite
                    ));

                    if cursor_col >= 0 && cursor_row >= 0 {
                        let cols_diff = c as i32 - cursor_col;
                        let rows_diff = r as i32 - cursor_row;
                        fsm.logger.info(&format!(
                            "     Distance to cursor: cols_diff={}, rows_diff={}",
                            cols_diff, rows_diff
                        ));
                    }
                }
            }
        }
        fsm.logger.info(&format!("[CV-Test] Cell scan complete. Found {} matched cells.", total_matches));

        // Save visual diagnostics image
        let diag_grid_path = scratch_dir.join("car_diagnostics_grid.png");
        if diag_frame.save(&diag_grid_path).is_ok() {
            fsm.logger.info(&format!("[CV-Test] Saved visual diagnostics grid to {:?}", diag_grid_path));
        } else {
            fsm.logger.warn("[CV-Test] Failed to save diagnostics grid to scratch/car_diagnostics_grid.png");
        }

        fsm.logger.info("[CV-Test] CV diagnostics completed.");
        fsm.set_state(crate::state_machine::FSMState::Idle);
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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Ensure bot stops on window close
                if let Some(bot_state) = window.try_state::<BotState>() {
                    bot_state.0.stop();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
