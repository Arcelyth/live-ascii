use std::ptr;

use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};

pub fn allocate_aligned(size: usize, alignment: usize) -> *mut u8 {
    let mut ptr: *mut libc::c_void = ptr::null_mut();
    unsafe {
        libc::posix_memalign(&mut ptr, alignment, size);
    }
    ptr as *mut u8
}

pub fn default_fade_time() -> f32 {
    -1.0
}

pub fn key_code_to_str(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Null => "Null".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::ScrollLock => "ScrollLock".to_string(),
        KeyCode::NumLock => "NumLock".to_string(),
        KeyCode::PrintScreen => "PrintScreen".to_string(),
        KeyCode::Pause => "Pause".to_string(),
        KeyCode::Menu => "Menu".to_string(),
        KeyCode::KeypadBegin => "KeypadBegin".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Modifier(mkc) => match mkc {
            ModifierKeyCode::LeftShift => "LeftShift".to_string(),
            ModifierKeyCode::LeftControl => "LeftControl".to_string(),
            ModifierKeyCode::LeftAlt => "LeftAlt".to_string(),
            ModifierKeyCode::LeftSuper => "LeftSuper".to_string(),
            ModifierKeyCode::LeftHyper => "LeftHyper".to_string(),
            ModifierKeyCode::LeftMeta => "LeftMeta".to_string(),
            ModifierKeyCode::RightShift => "RightShift".to_string(),
            ModifierKeyCode::RightControl => "RightControl".to_string(),
            ModifierKeyCode::RightAlt => "RightAlt".to_string(),
            ModifierKeyCode::RightSuper => "RightSuper".to_string(),
            ModifierKeyCode::RightHyper => "RightHyper".to_string(),
            ModifierKeyCode::RightMeta => "RightMeta".to_string(),
            _ => "".to_string(),
        },
        _ => "".to_string(),
    }
}

pub fn modifiers_to_vec(modifiers: KeyModifiers) -> Vec<String> {
    let mut mods = Vec::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        mods.push("Control".to_string());
    }
    if modifiers.contains(KeyModifiers::ALT) {
        mods.push("Alt".to_string());
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        mods.push("Shift".to_string());
    }
    if modifiers.contains(KeyModifiers::SUPER) {
        mods.push("Super".to_string());
    }
    mods
}

pub fn get_file_name(filename: &str) -> &str {
    filename.split('.').next().unwrap_or(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
   
    #[test]
    fn get_file_name_test() {
        let pre = get_file_name("test.model3.json");
        assert_eq!(pre, "test");
    }
}
