use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Live {
    pub version: usize,
    pub name: String,
    #[serde(default)]
    pub hotkeys: Vec<Hotkey>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum HotkeyAction {
    #[serde(rename = "Set/UnSet Expression")]
    SetUnsetExpression,
    #[serde(rename = "Open/Close Motion Panel")]
    OpenCloseMotionPanel,
    #[serde(rename = "Open/Close Debug Panel")]
    OpenCloseDebugPanel,
    #[serde(rename = "Enable/Disable Physics")]
    EnableDisablePhysics,
    #[serde(rename = "Open/Close Camera")]
    OpenCloseCamera,
    #[serde(rename = "Next Shader")]
    NextShader,
    #[serde(rename = "Previous Shader")]
    PrevShader,
    #[serde(rename = "Open/Close Receiver")]
    OpenCloseReceiver,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Action {
    SetUnsetExpression(String),
    OpenCloseMotionPanel,
    OpenCloseDebugPanel,
    EnableDisablePhysics,
    OpenCloseCamera,
    NextShader,
    PrevShader,
    OpenCloseReceiver(Option<usize>),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HotkeyTriggers {
    pub trigger1: String,
    pub trigger2: String,
    pub trigger3: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Hotkey {
    pub action: HotkeyAction,
    #[serde(default)]
    pub file: String,
    pub triggers: HotkeyTriggers,
    #[serde(default = "default_fade_seconds")]
    pub fade_seconds: f32,
    #[serde(default = "default_after_seconds")]
    pub stop_after_seconds: f32,
    #[serde(default)]
    pub stop_when_release_key: bool,
    pub port: Option<usize>,
}

impl Hotkey {
    pub fn is_trigger(&self, pressed_keys: &HashSet<String>) -> bool {
        let mut required = Vec::with_capacity(3);
        if !self.triggers.trigger1.is_empty() {
            required.push(&self.triggers.trigger1);
        }
        if !self.triggers.trigger2.is_empty() {
            required.push(&self.triggers.trigger2);
        }
        if !self.triggers.trigger3.is_empty() {
            required.push(&self.triggers.trigger3);
        }

        if required.is_empty() {
            return false;
        }

        required.iter().all(|&req| pressed_keys.contains(req))
    }

    pub fn apply(&self, action_queue: &mut Vec<Action>) {
        match self.action {
            HotkeyAction::SetUnsetExpression => {
                let a = Action::SetUnsetExpression(self.file.clone());
                if !action_queue.contains(&a) {
                    action_queue.push(a);
                }
            }
            HotkeyAction::OpenCloseMotionPanel => {
                if !action_queue.contains(&Action::OpenCloseMotionPanel) {
                    action_queue.push(Action::OpenCloseMotionPanel)
                }
            }
            HotkeyAction::OpenCloseDebugPanel => {
                if !action_queue.contains(&Action::OpenCloseDebugPanel) {
                    action_queue.push(Action::OpenCloseDebugPanel)
                }
            }
            HotkeyAction::EnableDisablePhysics => {
                if !action_queue.contains(&Action::EnableDisablePhysics) {
                    action_queue.push(Action::EnableDisablePhysics)
                }
            }
            HotkeyAction::OpenCloseCamera => {
                if !action_queue.contains(&Action::OpenCloseCamera) {
                    action_queue.push(Action::OpenCloseCamera)
                }
            }
            HotkeyAction::NextShader => {
                if !action_queue.contains(&Action::NextShader) {
                    action_queue.push(Action::NextShader)
                }
            }
            HotkeyAction::PrevShader => {
                if !action_queue.contains(&Action::PrevShader) {
                    action_queue.push(Action::PrevShader)
                }
            }
            HotkeyAction::OpenCloseReceiver => {
                if !action_queue.contains(&Action::OpenCloseReceiver(self.port)) {
                    action_queue.push(Action::OpenCloseReceiver(self.port))
                }
            }
        }
    }
}

fn default_after_seconds() -> f32 {
    -1.
}

fn default_fade_seconds() -> f32 {
    0.5
}

impl Live {
    pub fn from_path(base_dir: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let full_path = Path::new(base_dir).join(path);
        let data = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))?;

        let live: Live = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse JSON ({:?}): {}", full_path, e))?;

        Ok(live)
    }

    pub fn from_data(data: String) -> Result<Self, Box<dyn Error>> {
        let live: Live = serde_json::from_str(&data)?;
        Ok(live)
    }

    pub fn handle_hotkeys(
        &self,
        key: String,
        modifiers: Vec<String>,
        action_queue: &mut Vec<Action>,
    ) {
        let mut current_pressed = std::collections::HashSet::with_capacity(modifiers.len() + 1);
        if !key.is_empty() {
            current_pressed.insert(key);
        }
        for m in modifiers {
            current_pressed.insert(m);
        }

        for hotkey in self.hotkeys.iter() {
            if hotkey.is_trigger(&current_pressed) {
                hotkey.apply(action_queue);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_live_json() {
        let live = Live::from_path("./test_file", "test.live.json").unwrap();
        let expected = Live {
            version: 1,
            name: "test".to_string(),
            hotkeys: vec![Hotkey {
                action: HotkeyAction::SetUnsetExpression,
                file: "exp_02.exp3.json".to_string(),
                triggers: HotkeyTriggers {
                    trigger1: "A".to_string(),
                    trigger2: "".to_string(),
                    trigger3: "".to_string(),
                },
                fade_seconds: 0.5,
                stop_after_seconds: 3.,
                stop_when_release_key: false,
                port: None,
            }],
        };
        assert_eq!(live, expected);
    }
}
