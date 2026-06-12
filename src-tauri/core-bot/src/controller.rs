use std::sync::Arc;
use std::time::{Instant, Duration};
use rand::prelude::*;
use rand_distr::{LogNormal, Distribution};
use noise::{NoiseFn, OpenSimplex};
use rsruckig::ruckig::Ruckig;
use rsruckig::input_parameter::InputParameter;
use rsruckig::output_parameter::OutputParameter;
use rsruckig::result::RuckigResult;
use vigem_client::{Client, Xbox360Wired, XGamepad, XButtons, TargetId};

// XInput Button Constants
pub const BUTTON_DPAD_UP: u16 = 0x0001;
pub const BUTTON_DPAD_DOWN: u16 = 0x0002;
pub const BUTTON_DPAD_LEFT: u16 = 0x0004;
pub const BUTTON_DPAD_RIGHT: u16 = 0x0008;
pub const BUTTON_START: u16 = 0x0010;
pub const BUTTON_BACK: u16 = 0x0020;
pub const BUTTON_LEFT_THUMB: u16 = 0x0040;
pub const BUTTON_RIGHT_THUMB: u16 = 0x0080;
pub const BUTTON_LB: u16 = 0x0100;
pub const BUTTON_RB: u16 = 0x0200;
pub const BUTTON_A: u16 = 0x1000;
pub const BUTTON_B: u16 = 0x2000;
pub const BUTTON_X: u16 = 0x4000;
pub const BUTTON_Y: u16 = 0x8000;

pub struct GamepadWrapper {
    target: Option<Xbox360Wired<Client>>,
    gamepad_state: XGamepad,
    is_paused: bool,
    pressed_buttons_mask: u16,
}

impl GamepadWrapper {
    pub fn new() -> Self {
        let client = Client::connect().ok();
        let target = client.and_then(|c| {
            let id = TargetId::XBOX360_WIRED;
            let mut target = Xbox360Wired::new(c, id);
            if target.plugin().is_ok() && target.wait_ready().is_ok() {
                Some(target)
            } else {
                None
            }
        });

        Self {
            target,
            gamepad_state: XGamepad::default(),
            is_paused: false,
            pressed_buttons_mask: 0,
        }
    }

    pub fn try_connect(&mut self) -> Result<(), String> {
        if self.target.is_some() {
            return Ok(());
        }
        let client = Client::connect().map_err(|e| format!("Failed to connect to ViGEmBus driver: {:?}", e))?;
        let id = TargetId::XBOX360_WIRED;
        let mut target = Xbox360Wired::new(client, id);
        target.plugin().map_err(|e| format!("Failed to plugin virtual controller: {:?}", e))?;
        target.wait_ready().map_err(|e| format!("Virtual controller wait_ready failed: {:?}", e))?;
        self.target = Some(target);
        Ok(())
    }

    pub fn press_button(&mut self, button_mask: u16) {
        self.pressed_buttons_mask |= button_mask;
        self.gamepad_state.buttons = XButtons { raw: self.pressed_buttons_mask };
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn release_button(&mut self, button_mask: u16) {
        self.pressed_buttons_mask &= !button_mask;
        self.gamepad_state.buttons = XButtons { raw: self.pressed_buttons_mask };
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn left_joystick(&mut self, x: i16, y: i16) {
        self.gamepad_state.thumb_lx = x;
        self.gamepad_state.thumb_ly = y;
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn right_joystick(&mut self, x: i16, y: i16) {
        self.gamepad_state.thumb_rx = x;
        self.gamepad_state.thumb_ry = y;
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn left_trigger_float(&mut self, value: f32) {
        let val_u8 = (value.clamp(0.0, 1.0) * 255.0) as u8;
        self.gamepad_state.left_trigger = val_u8;
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn right_trigger_float(&mut self, value: f32) {
        let val_u8 = (value.clamp(0.0, 1.0) * 255.0) as u8;
        self.gamepad_state.right_trigger = val_u8;
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn reset(&mut self) {
        self.pressed_buttons_mask = 0;
        self.gamepad_state = XGamepad::default();
        if !self.is_paused {
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }

    pub fn pause_inputs(&mut self) {
        if !self.is_paused {
            self.is_paused = true;
            if let Some(ref mut target) = self.target {
                let _ = target.update(&XGamepad::default());
            }
        }
    }

    pub fn resume_inputs(&mut self) {
        if self.is_paused {
            self.is_paused = false;
            if let Some(ref mut target) = self.target {
                let _ = target.update(&self.gamepad_state);
            }
        }
    }
}

// Lognormal sampling parameters (NIH keystroke dynamics)
pub struct HumanTiming;

impl HumanTiming {
    pub const BUTTON_HOLD: (f64, f64) = (0.15, 70.0);
    pub const BUTTON_GAP: (f64, f64) = (0.45, 160.0);
    pub const NAV_PAUSE: (f64, f64) = (0.50, 10.0);
    pub const STAGE_JITTER: (f64, f64) = (0.30, 800.0);
    pub const THINK_PAUSE: (f64, f64) = (0.40, 1800.0);

    pub fn sample(s: f64, scale: f64, floor_ms: f64) -> f64 {
        let mu = scale.ln();
        let sigma = s;
        let log_normal = LogNormal::new(mu, sigma).unwrap();
        let mut rng = thread_rng();
        let ms = log_normal.sample(&mut rng);
        let ms = ms.max(floor_ms);
        ms / 1000.0
    }

    pub fn sleep(profile: (f64, f64), floor_ms: f64) {
        let secs = Self::sample(profile.0, profile.1, floor_ms);
        std::thread::sleep(Duration::from_secs_f64(secs));
    }
}

pub struct GamepadController {
    pub pad: GamepadWrapper,
    cancel_fn: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    simplex_clock_t: f64,
    simplex: OpenSimplex,
}

impl GamepadController {
    pub fn new() -> Self {
        let controller = Self {
            pad: GamepadWrapper::new(),
            cancel_fn: None,
            simplex_clock_t: 0.0,
            simplex: OpenSimplex::new(thread_rng().gen()),
        };
        // Allow OS to recognize controller
        HumanTiming::sleep(HumanTiming::STAGE_JITTER, 500.0);
        controller
    }

    pub fn try_connect(&mut self) -> Result<(), String> {
        self.pad.try_connect()
    }

    pub fn set_cancel_fn(&mut self, fn_ptr: Option<Arc<dyn Fn() -> bool + Send + Sync>>) {
        self.cancel_fn = fn_ptr;
    }

    pub fn pause_inputs(&mut self) {
        self.pad.pause_inputs();
    }

    pub fn resume_inputs(&mut self) {
        self.pad.resume_inputs();
    }

    fn check_cancelled(&self) -> bool {
        if let Some(ref cancel) = self.cancel_fn {
            cancel()
        } else {
            false
        }
    }

    pub fn sleep_responsive(&self, duration_s: f64) -> bool {
        let check_interval = 0.05f64;
        let mut elapsed = 0.0;
        while elapsed < duration_s {
            if self.check_cancelled() {
                return false;
            }
            let sleep_time = check_interval.min(duration_s - elapsed);
            std::thread::sleep(Duration::from_secs_f64(sleep_time));
            elapsed += sleep_time;
        }
        true
    }

    // --- Noise & Tremor --------------------------------------------------------

    pub fn next_simplex_noise(&mut self, amplitude: f64, dt: f64) -> f64 {
        // Advances t each sample and produces correlated noise (Tremor)
        self.simplex_clock_t += dt * 10.0;
        self.simplex.get([self.simplex_clock_t, 0.0]) * amplitude
    }

    pub fn apply_stick(&mut self, axis: &str, value: f64) {
        let scaled = (value.clamp(-1.0, 1.0) * 32767.0) as i16;
        match axis {
            "left_x" => self.pad.left_joystick(scaled, 0),
            "left_y" => self.pad.left_joystick(0, scaled),
            "right_x" => self.pad.right_joystick(scaled, 0),
            "right_y" => self.pad.right_joystick(0, scaled),
            _ => {}
        }
    }

    pub fn apply_trigger(&mut self, side: &str, value: f64) {
        let val = value.clamp(0.0, 1.0) as f32;
        match side {
            "left" => self.pad.left_trigger_float(val),
            "right" => self.pad.right_trigger_float(val),
            _ => {}
        }
    }

    // --- Basic Button press ----------------------------------------------------

    pub fn press(&mut self, button_mask: u16) -> bool {
        if self.check_cancelled() {
            return false;
        }
        self.pad.press_button(button_mask);
        let hold_s = HumanTiming::sample(HumanTiming::BUTTON_HOLD.0, HumanTiming::BUTTON_HOLD.1, 50.0);
        if !self.sleep_responsive(hold_s) {
            self.pad.release_button(button_mask);
            return false;
        }
        self.pad.release_button(button_mask);
        let gap_s = HumanTiming::sample(HumanTiming::BUTTON_GAP.0, HumanTiming::BUTTON_GAP.1, 60.0);
        self.sleep_responsive(gap_s)
    }

    pub fn press_a(&mut self) -> bool { self.press(BUTTON_A) }
    pub fn press_b(&mut self) -> bool { self.press(BUTTON_B) }
    pub fn press_x(&mut self) -> bool { self.press(BUTTON_X) }
    pub fn press_y(&mut self) -> bool { self.press(BUTTON_Y) }
    pub fn press_dpad_up(&mut self) -> bool { self.press(BUTTON_DPAD_UP) }
    pub fn press_dpad_down(&mut self) -> bool { self.press(BUTTON_DPAD_DOWN) }
    pub fn press_dpad_left(&mut self) -> bool { self.press(BUTTON_DPAD_LEFT) }
    pub fn press_dpad_right(&mut self) -> bool { self.press(BUTTON_DPAD_RIGHT) }
    pub fn press_lb(&mut self) -> bool { self.press(BUTTON_LB) }
    pub fn press_rb(&mut self) -> bool { self.press(BUTTON_RB) }
    pub fn press_start(&mut self) -> bool { self.press(BUTTON_START) }

    pub fn navigate(&mut self, button_mask: u16, count: usize) -> bool {
        for i in 0..count {
            if !self.press(button_mask) {
                return false;
            }
            if i < count - 1 {
                let nav_s = HumanTiming::sample(HumanTiming::NAV_PAUSE.0, HumanTiming::NAV_PAUSE.1, 60.0);
                if !self.sleep_responsive(nav_s) {
                    return false;
                }
            }
        }
        true
    }

    pub fn hold_button(&mut self, button_mask: u16, duration_s: f64, jitter_ratio: f64) -> bool {
        if self.check_cancelled() {
            return false;
        }
        let mut actual_duration = duration_s;
        if jitter_ratio > 0.0 {
            let mut rng = thread_rng();
            actual_duration = duration_s * rng.gen_range((1.0 - jitter_ratio)..(1.0 + jitter_ratio));
        }

        self.pad.press_button(button_mask);
        let success = self.sleep_responsive(actual_duration);
        self.pad.release_button(button_mask);
        if !success {
            return false;
        }
        let gap_s = HumanTiming::sample(HumanTiming::BUTTON_GAP.0, HumanTiming::BUTTON_GAP.1, 60.0);
        self.sleep_responsive(gap_s)
    }

    pub fn hold_dpad_up(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_DPAD_UP, duration_s, 0.10) }
    pub fn hold_dpad_down(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_DPAD_DOWN, duration_s, 0.10) }
    pub fn hold_dpad_left(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_DPAD_LEFT, duration_s, 0.10) }
    pub fn hold_dpad_right(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_DPAD_RIGHT, duration_s, 0.10) }
    pub fn hold_a(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_A, duration_s, 0.10) }
    pub fn hold_b(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_B, duration_s, 0.10) }
    pub fn hold_x(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_X, duration_s, 0.10) }
    pub fn hold_y(&mut self, duration_s: f64) -> bool { self.hold_button(BUTTON_Y, duration_s, 0.10) }

    // --- Trajectories & Smooth Motion -----------------------------------------

    fn ruckig_move_limits(
        &mut self,
        axis: &str,
        start: f64,
        target: f64,
        max_vel: Option<f64>,
        max_acc: Option<f64>,
        max_jerk: Option<f64>,
    ) -> bool {
        let cycle_s = 0.005;
        let mut otg = Ruckig::new(1, cycle_s, true);
        let mut input = InputParameter::new(1);
        let mut output = OutputParameter::new(1);

        input.current_position[0] = start;
        input.current_velocity[0] = 0.0;
        input.current_acceleration[0] = 0.0;
        input.target_position[0] = target;
        input.target_velocity[0] = 0.0;
        input.target_acceleration[0] = 0.0;

        input.max_velocity[0] = max_vel.unwrap_or(3.5);
        input.max_acceleration[0] = max_acc.unwrap_or(12.0);
        input.max_jerk[0] = max_jerk.unwrap_or(40.0);

        loop {
            if self.check_cancelled() {
                self.apply_stick(axis, 0.0);
                return false;
            }

            match otg.update(&input, &mut output) {
                Ok(RuckigResult::Working) => {
                    let noise = self.next_simplex_noise(0.008, cycle_s);
                    let pos = output.new_position[0] + noise;
                    self.apply_stick(axis, pos);
                    
                    input.current_position = output.new_position.clone();
                    input.current_velocity = output.new_velocity.clone();
                    input.current_acceleration = output.new_acceleration.clone();
                    
                    std::thread::sleep(Duration::from_secs_f64(cycle_s));
                }
                Ok(RuckigResult::Finished) => {
                    break;
                }
                Ok(other) => {
                    eprintln!("[controller] Ruckig error result: {:?}", other);
                    break;
                }
                Err(e) => {
                    eprintln!("[controller] Ruckig error: {:?}", e);
                    break;
                }
            }
        }

        self.apply_stick(axis, target);
        true
    }

    fn ruckig_move(&mut self, axis: &str, start: f64, target: f64) -> bool {
        let move_time = HumanTiming::sample(0.20, 250.0, 100.0); // 100-250ms
        let dist = (target - start).abs().max(0.01);
        let max_vel = dist / move_time;
        let max_acc = max_vel / move_time;
        let max_jerk = max_acc / move_time;

        self.ruckig_move_limits(axis, start, target, Some(max_vel), Some(max_acc), Some(max_jerk))
    }

    fn micro_hold(&mut self, axis: &str, position: f64, hold_s: f64) -> bool {
        let cycle_s = 0.010;
        let end = Instant::now() + Duration::from_secs_f64(hold_s);
        while Instant::now() < end {
            if self.check_cancelled() {
                self.apply_stick(axis, 0.0);
                return false;
            }
            let noise = self.next_simplex_noise(0.005, cycle_s);
            self.apply_stick(axis, position + noise);
            std::thread::sleep(Duration::from_secs_f64(cycle_s));
        }
        self.apply_stick(axis, position);
        true
    }

    pub fn move_stick(&mut self, axis: &str, target: f64, current: f64) -> bool {
        let movement = target - current;
        let mut rng = thread_rng();
        let do_overshoot = movement.abs() > 0.1 && rng.gen_bool(0.35);

        if do_overshoot {
            let overshoot_frac = HumanTiming::sample(0.30, 0.08, 20.0);
            let overshoot_frac = overshoot_frac.clamp(0.02, 0.20);
            let overshoot_target = (target + movement * overshoot_frac).clamp(-1.0, 1.0);

            if !self.ruckig_move(axis, current, overshoot_target) {
                return false;
            }

            let hold_s = HumanTiming::sample(0.30, 60.0, 30.0);
            if !self.micro_hold(axis, overshoot_target, hold_s.max(0.03)) {
                return false;
            }

            if !self.ruckig_move_limits(axis, overshoot_target, target, Some(2.0), None, None) {
                return false;
            }
        } else {
            if !self.ruckig_move(axis, current, target) {
                return false;
            }
        }

        let settle_s = HumanTiming::sample(0.25, 40.0, 20.0);
        self.micro_hold(axis, target, settle_s.max(0.02))
    }

    pub fn release_stick(&mut self, axis: &str) -> bool {
        self.move_stick(axis, 0.0, 0.0) // current defaults to 0.0 for releasing
    }

    // --- Triggers --------------------------------------------------------------

    pub fn press_trigger(
        &mut self,
        side: &str,
        peak: f64,
        hold_duration_s: Option<f64>,
        speed: f64,
    ) -> bool {
        let cycle_s = 0.005;

        // Finger dynamics are faster than thumb sticks. Calculate limits based on human duration.
        let squeeze_time = HumanTiming::sample(0.20, 80.0, 30.0) / speed;
        let squeeze_vel = peak / squeeze_time;
        let squeeze_acc = squeeze_vel / squeeze_time;
        let squeeze_jerk = squeeze_acc / squeeze_time;
        let noise_squeeze = 0.006;

        // --- Squeeze in ---
        let mut otg = Ruckig::new(1, cycle_s, true);
        let mut input = InputParameter::new(1);
        let mut output = OutputParameter::new(1);

        input.current_position[0] = 0.0;
        input.current_velocity[0] = 0.0;
        input.current_acceleration[0] = 0.0;
        input.target_position[0] = peak;
        input.target_velocity[0] = 0.0;
        input.target_acceleration[0] = 0.0;

        input.max_velocity[0] = squeeze_vel;
        input.max_acceleration[0] = squeeze_acc;
        input.max_jerk[0] = squeeze_jerk;

        loop {
            if self.check_cancelled() {
                self.apply_trigger(side, 0.0);
                return false;
            }

            match otg.update(&input, &mut output) {
                Ok(RuckigResult::Working) => {
                    let noise = self.next_simplex_noise(noise_squeeze, cycle_s);
                    let pos = output.new_position[0] + noise;
                    self.apply_trigger(side, pos);

                    input.current_position = output.new_position.clone();
                    input.current_velocity = output.new_velocity.clone();
                    input.current_acceleration = output.new_acceleration.clone();

                    std::thread::sleep(Duration::from_secs_f64(cycle_s));
                }
                Ok(RuckigResult::Finished) => {
                    break;
                }
                Ok(other) => {
                    eprintln!("[controller] Trigger Ruckig squeeze error result: {:?}", other);
                    break;
                }
                Err(e) => {
                    eprintln!("[controller] Trigger Ruckig squeeze error: {:?}", e);
                    break;
                }
            }
        }

        self.apply_trigger(side, peak);

        // --- Hold ---
        let hold_s = match hold_duration_s {
            Some(h) => h,
            None => HumanTiming::sample(0.35, 150.0, 60.0),
        };
        let end = Instant::now() + Duration::from_secs_f64(hold_s);
        let tremor_std = 0.004;

        while Instant::now() < end {
            if self.check_cancelled() {
                self.apply_trigger(side, 0.0);
                return false;
            }
            let noise = self.next_simplex_noise(tremor_std, 0.010);
            let val = (peak + noise).clamp(0.0, 1.0);
            self.apply_trigger(side, val);
            std::thread::sleep(Duration::from_millis(10));
        }

        self.apply_trigger(side, peak);

        // --- Release ---
        let release_time = HumanTiming::sample(0.15, 100.0, 40.0) / speed;
        let release_vel = peak / release_time;
        let release_acc = release_vel / release_time;
        let release_jerk = release_acc / release_time;

        let mut otg2 = Ruckig::new(1, cycle_s, true);
        let mut input2 = InputParameter::new(1);
        let mut output2 = OutputParameter::new(1);

        input2.current_position[0] = peak;
        input2.current_velocity[0] = 0.0;
        input2.current_acceleration[0] = 0.0;
        input2.target_position[0] = 0.0;
        input2.target_velocity[0] = 0.0;
        input2.target_acceleration[0] = 0.0;

        input2.max_velocity[0] = release_vel;
        input2.max_acceleration[0] = release_acc;
        input2.max_jerk[0] = release_jerk;

        loop {
            if self.check_cancelled() {
                self.apply_trigger(side, 0.0);
                return false;
            }

            match otg2.update(&input2, &mut output2) {
                Ok(RuckigResult::Working) => {
                    let noise = self.next_simplex_noise(noise_squeeze, cycle_s);
                    let pos = output2.new_position[0] + noise;
                    self.apply_trigger(side, pos.clamp(0.0, 1.0));

                    input2.current_position = output2.new_position.clone();
                    input2.current_velocity = output2.new_velocity.clone();
                    input2.current_acceleration = output2.new_acceleration.clone();

                    std::thread::sleep(Duration::from_secs_f64(cycle_s));
                }
                Ok(RuckigResult::Finished) => {
                    break;
                }
                Ok(other) => {
                    eprintln!("[controller] Trigger Ruckig release error result: {:?}", other);
                    break;
                }
                Err(e) => {
                    eprintln!("[controller] Trigger Ruckig release error: {:?}", e);
                    break;
                }
            }
        }

        self.apply_trigger(side, 0.0);
        true
    }

    pub fn press_lt(&mut self, peak: f64, hold_s: Option<f64>, speed: f64) -> bool {
        self.press_trigger("left", peak, hold_s, speed)
    }

    pub fn press_rt(&mut self, peak: f64, hold_s: Option<f64>, speed: f64) -> bool {
        self.press_trigger("right", peak, hold_s, speed)
    }

    pub fn think(&mut self) -> bool {
        let think_s = HumanTiming::sample(HumanTiming::THINK_PAUSE.0, HumanTiming::THINK_PAUSE.1, 500.0);
        self.sleep_responsive(think_s)
    }

    pub fn stage_jitter(&mut self) -> bool {
        let jitter_s = HumanTiming::sample(HumanTiming::STAGE_JITTER.0, HumanTiming::STAGE_JITTER.1, 500.0);
        self.sleep_responsive(jitter_s)
    }

    pub fn reset(&mut self) {
        self.pad.reset();
    }
}
