use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn default_stage1_car() -> String {
    "toyota_tacoma_fe".to_string()
}

fn default_stage2_car() -> String {
    "subaru_impreza_22b".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotConfig {
    pub stages_enabled: HashMap<String, bool>,
    pub loop_count: i32,
    pub credits_budget: i32,
    pub stage1_duration: i32,
    pub stage2_iterations: i32,
    pub stage3_iterations: i32,
    pub stage4_iterations: i32,
    pub smart_settings: bool,
    pub baseline_resolution: (u32, u32),
    #[serde(default = "default_stage1_car")]
    pub stage1_car: String,
    #[serde(default = "default_stage2_car")]
    pub stage2_car: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        let mut stages_enabled = HashMap::new();
        stages_enabled.insert("stage1".to_string(), true);
        stages_enabled.insert("stage2".to_string(), true);
        stages_enabled.insert("stage3".to_string(), true);
        stages_enabled.insert("stage4".to_string(), true);

        Self {
            stages_enabled,
            loop_count: 0,
            credits_budget: 0,
            stage1_duration: 6,
            stage2_iterations: 2,
            stage3_iterations: 2,
            stage4_iterations: 2,
            smart_settings: true,
            baseline_resolution: (2560, 1440),
            stage1_car: default_stage1_car(),
            stage2_car: default_stage2_car(),
        }
    }
}

impl BotConfig {
    pub fn config_path() -> PathBuf {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| {
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
                format!("{}\\AppData\\Roaming", home)
            });
        let dir = PathBuf::from(appdata).join("com.forzawsfb.app");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<Self>(&content) {
                    config.resolve_conflicts();
                    return config;
                }
            }
        }
        let mut def = Self::default();
        def.resolve_conflicts();
        def
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn resolve_conflicts(&mut self) {
        if !self.smart_settings {
            return;
        }

        // S2 estimation limit (99 SP limit)
        if *self.stages_enabled.get("stage2").unwrap_or(&false) {
            let max_s2 = 99;
            if self.stage2_iterations > max_s2 {
                self.stage2_iterations = max_s2;
            }
        }

        // 00. S1 + S2 + S3 + S4 Quad-link resolution
        let s1 = *self.stages_enabled.get("stage1").unwrap_or(&false);
        let s2 = *self.stages_enabled.get("stage2").unwrap_or(&false);
        let s3 = *self.stages_enabled.get("stage3").unwrap_or(&false);
        let s4 = *self.stages_enabled.get("stage4").unwrap_or(&false);

        if s1 && s2 && s3 && s4 {
            let max_s4 = 33;
            if self.stage4_iterations > max_s4 {
                self.stage4_iterations = max_s4;
            }
            self.stage3_iterations = self.stage4_iterations;
            self.stage2_iterations = self.stage2_iterations.max(self.stage4_iterations * 3);
            let required_s1 = ((self.stage4_iterations as f32 * 86000.0) / 160000.0).ceil() as i32 * 6;
            self.stage1_duration = self.stage1_duration.max(required_s1);
        } else if s2 && s3 && s4 {
            let max_s4 = 33;
            if self.stage4_iterations > max_s4 {
                self.stage4_iterations = max_s4;
            }
            self.stage3_iterations = self.stage4_iterations;
            self.stage2_iterations = self.stage2_iterations.max(self.stage4_iterations * 3);
        } else {
            // 1. S2 + S4 Conflict resolution (SP check with 99 SP limit for S2 / 33 limit for S4)
            if s2 && s4 {
                let max_s4 = 33;
                if self.stage4_iterations > max_s4 {
                    self.stage4_iterations = max_s4;
                }
                self.stage2_iterations = self.stage2_iterations.max(self.stage4_iterations * 3);
            }

            // 2. S3 + S4 Conflict resolution (S3 >= S4 check)
            if s3 && s4 {
                if self.stage3_iterations < self.stage4_iterations {
                    self.stage3_iterations = self.stage4_iterations;
                }
            }
        }

        // 3. S1 + S3 Conflict resolution (Credits check)
        if s1 && s3 {
            let laps = self.stage1_duration / 6;
            let credits_earned = laps * 160000;
            let spent = self.stage3_iterations * 86000;
            if credits_earned < spent {
                let required_laps = ((self.stage3_iterations as f32 * 86000.0) / 160000.0).ceil() as i32;
                self.stage1_duration = required_laps * 6;
            }
        }
    }
}
