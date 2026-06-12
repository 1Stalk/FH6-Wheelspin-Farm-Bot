use rand::prelude::*;
use crate::controller::{GamepadController, HumanTiming};

// Define structure for recording samples
#[derive(Debug, Clone, Copy)]
pub struct RecordingSample {
    pub dt: f32,
    pub lx: f32,
    pub ly: f32,
    pub rx: f32,
    pub ry: f32,
    pub lt: f32,
    pub rt: f32,
}

// Parse raw binary slice of f32s into samples
pub fn parse_recording(data: &[u8]) -> Vec<RecordingSample> {
    let mut samples = Vec::new();
    for chunk in data.chunks_exact(28) {
        let dt = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
        let lx = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
        let ly = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
        let rx = f32::from_le_bytes(chunk[12..16].try_into().unwrap());
        let ry = f32::from_le_bytes(chunk[16..20].try_into().unwrap());
        let lt = f32::from_le_bytes(chunk[20..24].try_into().unwrap());
        let rt = f32::from_le_bytes(chunk[24..28].try_into().unwrap());
        samples.push(RecordingSample { dt, lx, ly, rx, ry, lt, rt });
    }
    samples
}

// Static binary recording assets
static STICK_LEFT_X_NEG_FAST_01: &[u8] = include_bytes!("../recordings/stick_left_x_neg_fast_01.bin");
static STICK_LEFT_X_NEG_FAST_02: &[u8] = include_bytes!("../recordings/stick_left_x_neg_fast_02.bin");
static STICK_LEFT_X_NEG_FAST_03: &[u8] = include_bytes!("../recordings/stick_left_x_neg_fast_03.bin");
static STICK_LEFT_X_NEG_FAST_04: &[u8] = include_bytes!("../recordings/stick_left_x_neg_fast_04.bin");
static STICK_LEFT_X_NEG_FAST_05: &[u8] = include_bytes!("../recordings/stick_left_x_neg_fast_05.bin");

static STICK_LEFT_X_NEG_SLOW_01: &[u8] = include_bytes!("../recordings/stick_left_x_neg_slow_01.bin");
static STICK_LEFT_X_NEG_SLOW_02: &[u8] = include_bytes!("../recordings/stick_left_x_neg_slow_02.bin");
static STICK_LEFT_X_NEG_SLOW_03: &[u8] = include_bytes!("../recordings/stick_left_x_neg_slow_03.bin");
static STICK_LEFT_X_NEG_SLOW_04: &[u8] = include_bytes!("../recordings/stick_left_x_neg_slow_04.bin");
static STICK_LEFT_X_NEG_SLOW_05: &[u8] = include_bytes!("../recordings/stick_left_x_neg_slow_05.bin");

static STICK_LEFT_X_POS_FAST_01: &[u8] = include_bytes!("../recordings/stick_left_x_pos_fast_01.bin");
static STICK_LEFT_X_POS_FAST_02: &[u8] = include_bytes!("../recordings/stick_left_x_pos_fast_02.bin");
static STICK_LEFT_X_POS_FAST_03: &[u8] = include_bytes!("../recordings/stick_left_x_pos_fast_03.bin");
static STICK_LEFT_X_POS_FAST_04: &[u8] = include_bytes!("../recordings/stick_left_x_pos_fast_04.bin");
static STICK_LEFT_X_POS_FAST_05: &[u8] = include_bytes!("../recordings/stick_left_x_pos_fast_05.bin");

static STICK_LEFT_X_POS_SLOW_01: &[u8] = include_bytes!("../recordings/stick_left_x_pos_slow_01.bin");
static STICK_LEFT_X_POS_SLOW_02: &[u8] = include_bytes!("../recordings/stick_left_x_pos_slow_02.bin");
static STICK_LEFT_X_POS_SLOW_03: &[u8] = include_bytes!("../recordings/stick_left_x_pos_slow_03.bin");
static STICK_LEFT_X_POS_SLOW_04: &[u8] = include_bytes!("../recordings/stick_left_x_pos_slow_04.bin");
static STICK_LEFT_X_POS_SLOW_05: &[u8] = include_bytes!("../recordings/stick_left_x_pos_slow_05.bin");

static STICK_LEFT_Y_NEG_FAST_01: &[u8] = include_bytes!("../recordings/stick_left_y_neg_fast_01.bin");
static STICK_LEFT_Y_NEG_FAST_02: &[u8] = include_bytes!("../recordings/stick_left_y_neg_fast_02.bin");
static STICK_LEFT_Y_NEG_FAST_03: &[u8] = include_bytes!("../recordings/stick_left_y_neg_fast_03.bin");
static STICK_LEFT_Y_NEG_FAST_04: &[u8] = include_bytes!("../recordings/stick_left_y_neg_fast_04.bin");
static STICK_LEFT_Y_NEG_FAST_05: &[u8] = include_bytes!("../recordings/stick_left_y_neg_fast_05.bin");

static STICK_LEFT_Y_POS_FAST_01: &[u8] = include_bytes!("../recordings/stick_left_y_pos_fast_01.bin");
static STICK_LEFT_Y_POS_FAST_02: &[u8] = include_bytes!("../recordings/stick_left_y_pos_fast_02.bin");
static STICK_LEFT_Y_POS_FAST_03: &[u8] = include_bytes!("../recordings/stick_left_y_pos_fast_03.bin");
static STICK_LEFT_Y_POS_FAST_04: &[u8] = include_bytes!("../recordings/stick_left_y_pos_fast_04.bin");
static STICK_LEFT_Y_POS_FAST_05: &[u8] = include_bytes!("../recordings/stick_left_y_pos_fast_05.bin");

static STICK_RIGHT_X_NEG_FAST_01: &[u8] = include_bytes!("../recordings/stick_right_x_neg_fast_01.bin");
static STICK_RIGHT_X_NEG_FAST_02: &[u8] = include_bytes!("../recordings/stick_right_x_neg_fast_02.bin");
static STICK_RIGHT_X_NEG_FAST_03: &[u8] = include_bytes!("../recordings/stick_right_x_neg_fast_03.bin");
static STICK_RIGHT_X_NEG_FAST_04: &[u8] = include_bytes!("../recordings/stick_right_x_neg_fast_04.bin");
static STICK_RIGHT_X_NEG_FAST_05: &[u8] = include_bytes!("../recordings/stick_right_x_neg_fast_05.bin");

static STICK_RIGHT_X_POS_FAST_01: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_01.bin");
static STICK_RIGHT_X_POS_FAST_02: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_02.bin");
static STICK_RIGHT_X_POS_FAST_03: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_03.bin");
static STICK_RIGHT_X_POS_FAST_04: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_04.bin");
static STICK_RIGHT_X_POS_FAST_05: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_05.bin");
static STICK_RIGHT_X_POS_FAST_06: &[u8] = include_bytes!("../recordings/stick_right_x_pos_fast_06.bin");

static STICK_RIGHT_Y_NEG_FAST_01: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_01.bin");
static STICK_RIGHT_Y_NEG_FAST_02: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_02.bin");
static STICK_RIGHT_Y_NEG_FAST_03: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_03.bin");
static STICK_RIGHT_Y_NEG_FAST_04: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_04.bin");
static STICK_RIGHT_Y_NEG_FAST_05: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_05.bin");
static STICK_RIGHT_Y_NEG_FAST_06: &[u8] = include_bytes!("../recordings/stick_right_y_neg_fast_06.bin");

static STICK_RIGHT_Y_POS_FAST_01: &[u8] = include_bytes!("../recordings/stick_right_y_pos_fast_01.bin");
static STICK_RIGHT_Y_POS_FAST_02: &[u8] = include_bytes!("../recordings/stick_right_y_pos_fast_02.bin");
static STICK_RIGHT_Y_POS_FAST_03: &[u8] = include_bytes!("../recordings/stick_right_y_pos_fast_03.bin");
static STICK_RIGHT_Y_POS_FAST_04: &[u8] = include_bytes!("../recordings/stick_right_y_pos_fast_04.bin");
static STICK_RIGHT_Y_POS_FAST_05: &[u8] = include_bytes!("../recordings/stick_right_y_pos_fast_05.bin");

static TRIGGER_LT_FULL_01: &[u8] = include_bytes!("../recordings/trigger_lt_full_01.bin");
static TRIGGER_LT_FULL_02: &[u8] = include_bytes!("../recordings/trigger_lt_full_02.bin");
static TRIGGER_LT_FULL_03: &[u8] = include_bytes!("../recordings/trigger_lt_full_03.bin");
static TRIGGER_LT_FULL_04: &[u8] = include_bytes!("../recordings/trigger_lt_full_04.bin");
static TRIGGER_LT_FULL_05: &[u8] = include_bytes!("../recordings/trigger_lt_full_05.bin");

static TRIGGER_RT_FULL_01: &[u8] = include_bytes!("../recordings/trigger_rt_full_01.bin");
static TRIGGER_RT_FULL_02: &[u8] = include_bytes!("../recordings/trigger_rt_full_02.bin");
static TRIGGER_RT_FULL_03: &[u8] = include_bytes!("../recordings/trigger_rt_full_03.bin");
static TRIGGER_RT_FULL_04: &[u8] = include_bytes!("../recordings/trigger_rt_full_04.bin");
static TRIGGER_RT_FULL_05: &[u8] = include_bytes!("../recordings/trigger_rt_full_05.bin");

static TRIGGER_RT_HALF_01: &[u8] = include_bytes!("../recordings/trigger_rt_half_01.bin");
static TRIGGER_RT_HALF_02: &[u8] = include_bytes!("../recordings/trigger_rt_half_02.bin");
static TRIGGER_RT_HALF_03: &[u8] = include_bytes!("../recordings/trigger_rt_half_03.bin");
static TRIGGER_RT_HALF_04: &[u8] = include_bytes!("../recordings/trigger_rt_half_04.bin");
static TRIGGER_RT_HALF_05: &[u8] = include_bytes!("../recordings/trigger_rt_half_05.bin");

pub fn get_recordings(action_key: &str) -> Vec<&'static [u8]> {
    match action_key {
        "stick_left_x_neg_fast" => vec![STICK_LEFT_X_NEG_FAST_01, STICK_LEFT_X_NEG_FAST_02, STICK_LEFT_X_NEG_FAST_03, STICK_LEFT_X_NEG_FAST_04, STICK_LEFT_X_NEG_FAST_05],
        "stick_left_x_neg_slow" => vec![STICK_LEFT_X_NEG_SLOW_01, STICK_LEFT_X_NEG_SLOW_02, STICK_LEFT_X_NEG_SLOW_03, STICK_LEFT_X_NEG_SLOW_04, STICK_LEFT_X_NEG_SLOW_05],
        "stick_left_x_pos_fast" => vec![STICK_LEFT_X_POS_FAST_01, STICK_LEFT_X_POS_FAST_02, STICK_LEFT_X_POS_FAST_03, STICK_LEFT_X_POS_FAST_04, STICK_LEFT_X_POS_FAST_05],
        "stick_left_x_pos_slow" => vec![STICK_LEFT_X_POS_SLOW_01, STICK_LEFT_X_POS_SLOW_02, STICK_LEFT_X_POS_SLOW_03, STICK_LEFT_X_POS_SLOW_04, STICK_LEFT_X_POS_SLOW_05],
        "stick_left_y_neg_fast" => vec![STICK_LEFT_Y_NEG_FAST_01, STICK_LEFT_Y_NEG_FAST_02, STICK_LEFT_Y_NEG_FAST_03, STICK_LEFT_Y_NEG_FAST_04, STICK_LEFT_Y_NEG_FAST_05],
        "stick_left_y_pos_fast" => vec![STICK_LEFT_Y_POS_FAST_01, STICK_LEFT_Y_POS_FAST_02, STICK_LEFT_Y_POS_FAST_03, STICK_LEFT_Y_POS_FAST_04, STICK_LEFT_Y_POS_FAST_05],
        "stick_right_x_neg_fast" => vec![STICK_RIGHT_X_NEG_FAST_01, STICK_RIGHT_X_NEG_FAST_02, STICK_RIGHT_X_NEG_FAST_03, STICK_RIGHT_X_NEG_FAST_04, STICK_RIGHT_X_NEG_FAST_05],
        "stick_right_x_pos_fast" => vec![STICK_RIGHT_X_POS_FAST_01, STICK_RIGHT_X_POS_FAST_02, STICK_RIGHT_X_POS_FAST_03, STICK_RIGHT_X_POS_FAST_04, STICK_RIGHT_X_POS_FAST_05, STICK_RIGHT_X_POS_FAST_06],
        "stick_right_y_neg_fast" => vec![STICK_RIGHT_Y_NEG_FAST_01, STICK_RIGHT_Y_NEG_FAST_02, STICK_RIGHT_Y_NEG_FAST_03, STICK_RIGHT_Y_NEG_FAST_04, STICK_RIGHT_Y_NEG_FAST_05, STICK_RIGHT_Y_NEG_FAST_06],
        "stick_right_y_pos_fast" => vec![STICK_RIGHT_Y_POS_FAST_01, STICK_RIGHT_Y_POS_FAST_02, STICK_RIGHT_Y_POS_FAST_03, STICK_RIGHT_Y_POS_FAST_04, STICK_RIGHT_Y_POS_FAST_05],
        "trigger_lt_full" => vec![TRIGGER_LT_FULL_01, TRIGGER_LT_FULL_02, TRIGGER_LT_FULL_03, TRIGGER_LT_FULL_04, TRIGGER_LT_FULL_05],
        "trigger_rt_full" => vec![TRIGGER_RT_FULL_01, TRIGGER_RT_FULL_02, TRIGGER_RT_FULL_03, TRIGGER_RT_FULL_04, TRIGGER_RT_FULL_05],
        "trigger_rt_half" => vec![TRIGGER_RT_HALF_01, TRIGGER_RT_HALF_02, TRIGGER_RT_HALF_03, TRIGGER_RT_HALF_04, TRIGGER_RT_HALF_05],
        _ => vec![],
    }
}

pub fn has_recordings(action_key: &str) -> bool {
    !get_recordings(action_key).is_empty()
}

pub fn play_recorded(
    controller: &mut GamepadController,
    action_key: &str,
    speed: Option<f64>,
) -> bool {
    let raw_options = get_recordings(action_key);
    if raw_options.is_empty() {
        return false;
    }

    let mut rng = thread_rng();
    let chosen_raw = raw_options.choose(&mut rng).unwrap();
    let samples = parse_recording(chosen_raw);

    // Speed -> time_scale calculation
    let base_time_scale = match speed {
        Some(s) => 1.0 / s.max(0.1),
        None => 1.0,
    };

    let ts_jitter = HumanTiming::sample(0.10, 1.0, 50.0); // ±10% variation
    let time_scale = (base_time_scale * ts_jitter).clamp(0.5, 3.0);

    let amp_jitter = HumanTiming::sample(0.06, 1.0, 50.0); // ±6% variation
    let amp_scale = amp_jitter.clamp(0.88, 1.12);

    let noise_std = 0.007;

    for sample in samples {
        if controller.sleep_responsive(0.0) { // Quick responsive check
            // Apply scale
            let mut lx = sample.lx as f64 * amp_scale;
            let mut ly = sample.ly as f64 * amp_scale;
            let mut rx = sample.rx as f64 * amp_scale;
            let mut ry = sample.ry as f64 * amp_scale;
            let lt = (sample.lt as f64 * amp_scale).clamp(0.0, 1.0);
            let rt = (sample.rt as f64 * amp_scale).clamp(0.0, 1.0);

            // Add Simplex tremors
            // Let's get simplex tremor:
            let nx = controller.next_simplex_noise(noise_std, 0.0); // dt is advanced by noise_t
            // Wait, we have the simplex clock. We can just sample from controller simplex:
            let ny = controller.next_simplex_noise(noise_std, 0.0);
            let nrx = controller.next_simplex_noise(noise_std, 0.0);
            let nry = controller.next_simplex_noise(noise_std, 0.0);

            lx += nx;
            ly += ny;
            rx += nrx;
            ry += nry;

            // XInput Y axis inversion: pygame records Y as -1=up, XInput expects +32767=up.
            // Negate Y axis values:
            let lx_raw = (lx.clamp(-1.0, 1.0) * 32767.0) as i16;
            let ly_raw = ((-ly).clamp(-1.0, 1.0) * 32767.0) as i16; // Negated
            let rx_raw = (rx.clamp(-1.0, 1.0) * 32767.0) as i16;
            let ry_raw = ((-ry).clamp(-1.0, 1.0) * 32767.0) as i16; // Negated

            controller.pad.left_joystick(lx_raw, ly_raw);
            controller.pad.right_joystick(rx_raw, ry_raw);
            controller.pad.left_trigger_float(lt as f32);
            controller.pad.right_trigger_float(rt as f32);

            let delay_s = sample.dt as f64 * time_scale;
            if !controller.sleep_responsive(delay_s.max(0.001)) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

// Primitives for long holds
pub fn hold_trigger(
    controller: &mut GamepadController,
    side: &str,
    peak: f64,
    duration_s: f64,
) -> bool {
    let cycle_s = 0.010;
    let end = std::time::Instant::now() + std::time::Duration::from_secs_f64(duration_s);
    let tremor_std = 0.004;

    while std::time::Instant::now() < end {
        if controller.sleep_responsive(0.0) {
            let noise = controller.next_simplex_noise(tremor_std, cycle_s);
            let val = (peak + noise).clamp(0.0, 1.0);
            controller.apply_trigger(side, val);
            std::thread::sleep(std::time::Duration::from_secs_f64(cycle_s));
        } else {
            controller.apply_trigger(side, 0.0);
            return false;
        }
    }

    controller.apply_trigger(side, peak);
    true
}

pub fn hold_stick(
    controller: &mut GamepadController,
    axis: &str,
    value: f64,
    duration_s: f64,
) -> bool {
    let cycle_s = 0.010;
    let end = std::time::Instant::now() + std::time::Duration::from_secs_f64(duration_s);
    let tremor_std = 0.005;

    while std::time::Instant::now() < end {
        if controller.sleep_responsive(0.0) {
            let noise = controller.next_simplex_noise(tremor_std, cycle_s);
            let val = (value + noise).clamp(-1.0, 1.0);
            controller.apply_stick(axis, val);
            std::thread::sleep(std::time::Duration::from_secs_f64(cycle_s));
        } else {
            controller.apply_stick(axis, 0.0);
            return false;
        }
    }

    controller.apply_stick(axis, value);
    true
}
