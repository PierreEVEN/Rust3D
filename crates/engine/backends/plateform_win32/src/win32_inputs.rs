use crate::utils::{hiword, loword};
use plateform::input_system::{InputManager, InputMapping, KeyboardKey, MouseButton};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut LAST_MOUSE_STATE: u16 = 0;

fn win32_mouse_button(mouse_state: usize) -> Option<(MouseButton, bool)> {
    unsafe {
        if mouse_state & (VK_LBUTTON.0 as usize) != 0 {
            return Some((MouseButton::Left, true));
        }
        if !(mouse_state & (VK_LBUTTON).0 as usize) != 0 {
            return Some((MouseButton::Left, false));
        }
        if mouse_state & (VK_RBUTTON.0 as usize) != 0 {
            return Some((MouseButton::Right, true));
        }
        if !(mouse_state & (VK_RBUTTON).0 as usize) != 0 {
            return Some((MouseButton::Right, false));
        }
        if mouse_state & (VK_MBUTTON.0 as usize) != 0 {
            return Some((MouseButton::Middle, true));
        }
        if !(mouse_state & (VK_MBUTTON).0 as usize) != 0 {
            return Some((MouseButton::Middle, false));
        }
        if mouse_state & (VK_XBUTTON1.0 as usize) != 0 {
            return Some((MouseButton::Button1, true));
        }
        if !(mouse_state & (VK_XBUTTON1).0 as usize) != 0 {
            return Some((MouseButton::Button1, false));
        }
        if mouse_state & (VK_XBUTTON2.0 as usize) != 0 {
            return Some((MouseButton::Button2, true));
        }
        if !(mouse_state & (VK_XBUTTON2).0 as usize) != 0 {
            return Some((MouseButton::Button2, false));
        }
        LAST_MOUSE_STATE = mouse_state as u16;
    }

    None
}

fn win32_keyboard(wp: WPARAM, scan_code: u32) -> Option<KeyboardKey> {
    match wp.0 {
        0x00 | 0xFF => {
            return None;
        }
        0x30 => {
            return Some(KeyboardKey::Key0);
        }
        0x31 => {
            return Some(KeyboardKey::Key1);
        }
        0x32 => {
            return Some(KeyboardKey::Key2);
        }
        0x33 => {
            return Some(KeyboardKey::Key3);
        }
        0x34 => {
            return Some(KeyboardKey::Key4);
        }
        0x35 => {
            return Some(KeyboardKey::Key5);
        }
        0x36 => {
            return Some(KeyboardKey::Key6);
        }
        0x37 => {
            return Some(KeyboardKey::Key7);
        }
        0x38 => {
            return Some(KeyboardKey::Key8);
        }
        0x39 => {
            return Some(KeyboardKey::Key9);
        }
        0x41 => {
            return Some(KeyboardKey::KeyA);
        }
        0x42 => {
            return Some(KeyboardKey::KeyB);
        }
        0x43 => {
            return Some(KeyboardKey::KeyC);
        }
        0x44 => {
            return Some(KeyboardKey::KeyD);
        }
        0x45 => {
            return Some(KeyboardKey::KeyE);
        }
        0x46 => {
            return Some(KeyboardKey::KeyF);
        }
        0x47 => {
            return Some(KeyboardKey::KeyG);
        }
        0x48 => {
            return Some(KeyboardKey::KeyH);
        }
        0x49 => {
            return Some(KeyboardKey::KeyI);
        }
        0x4A => {
            return Some(KeyboardKey::KeyJ);
        }
        0x4B => {
            return Some(KeyboardKey::KeyK);
        }
        0x4C => {
            return Some(KeyboardKey::KeyL);
        }
        0x4D => {
            return Some(KeyboardKey::KeyM);
        }
        0x4E => {
            return Some(KeyboardKey::KeyN);
        }
        0x4F => {
            return Some(KeyboardKey::KeyO);
        }
        0x50 => {
            return Some(KeyboardKey::KeyP);
        }
        0x51 => {
            return Some(KeyboardKey::KeyQ);
        }
        0x52 => {
            return Some(KeyboardKey::KeyR);
        }
        0x53 => {
            return Some(KeyboardKey::KeyS);
        }
        0x54 => {
            return Some(KeyboardKey::KeyT);
        }
        0x55 => {
            return Some(KeyboardKey::KeyU);
        }
        0x56 => {
            return Some(KeyboardKey::KeyV);
        }
        0x57 => {
            return Some(KeyboardKey::KeyW);
        }
        0x58 => {
            return Some(KeyboardKey::KeyX);
        }
        0x59 => {
            return Some(KeyboardKey::KeyY);
        }
        0x5A => {
            return Some(KeyboardKey::KeyZ);
        }
        _ => {}
    }

    match VIRTUAL_KEY(wp.0 as u16) {
        VK_BACK => {
            return Some(KeyboardKey::Backspace);
        } // VK_BACK
        VK_TAB => {
            return Some(KeyboardKey::Tab);
        } // VK_TAB
        VK_PAUSE => {
            return Some(KeyboardKey::Pause);
        } // VK_PAUSE
        VK_RETURN => {
            return Some(KeyboardKey::Enter);
        } // VK_RETURN
        VK_SHIFT => unsafe {
            return if MapVirtualKeyW(scan_code, MAPVK_VSC_TO_VK_EX) == VK_RSHIFT.0 as u32 {
                Some(KeyboardKey::RightShift)
            } else {
                Some(KeyboardKey::LeftShift)
            };
        }, // VK_SHIFT
        VK_ESCAPE => {
            return Some(KeyboardKey::Escape);
        } // VK_ESCAPE
        VK_SPACE => {
            return Some(KeyboardKey::Space);
        } // VK_SPACE
        VK_PRIOR => {
            return Some(KeyboardKey::PageUp);
        } // VK_PRIOR
        VK_NEXT => {
            return Some(KeyboardKey::PageDown);
        } // VK_NEXT
        VK_END => {
            return Some(KeyboardKey::End);
        } // VK_END
        VK_HOME => {
            return Some(KeyboardKey::Home);
        } // VK_HOME
        VK_LEFT => {
            return Some(KeyboardKey::Left);
        } // VK_LEFT
        VK_RIGHT => {
            return Some(KeyboardKey::Right);
        } // VK_RIGHT
        VK_UP => {
            return Some(KeyboardKey::Up);
        } // VK_UP
        VK_DOWN => {
            return Some(KeyboardKey::Down);
        } // VK_DOWN
        VK_PRINT => {
            return Some(KeyboardKey::Print);
        } // VK_PRINT
        VK_SNAPSHOT => {
            return Some(KeyboardKey::PrintScreen);
        } // VK_SNAPSHOT
        VK_INSERT => {
            return Some(KeyboardKey::Insert);
        } // VK_INSERT
        VK_DELETE => {
            return Some(KeyboardKey::Delete);
        } // VK_DELETE
        VK_HELP => {
            return Some(KeyboardKey::Help);
        } // VK_HELP
        VK_LWIN => {
            return Some(KeyboardKey::LeftWin);
        } // VK_LWIN
        VK_RWIN => {
            return Some(KeyboardKey::RightWin);
        } // VK_RWIN
        VK_SLEEP => {
            return Some(KeyboardKey::Sleep);
        } // VK_SLEEP
        VK_NUMPAD0 => {
            return Some(KeyboardKey::Num0);
        } // VK_NUMPAD0
        VK_NUMPAD1 => {
            return Some(KeyboardKey::Num1);
        } // VK_NUMPAD1
        VK_NUMPAD2 => {
            return Some(KeyboardKey::Num2);
        } // VK_NUMPAD2
        VK_NUMPAD3 => {
            return Some(KeyboardKey::Num3);
        } // VK_NUMPAD3
        VK_NUMPAD4 => {
            return Some(KeyboardKey::Num4);
        } // VK_NUMPAD4
        VK_NUMPAD5 => {
            return Some(KeyboardKey::Num5);
        } // VK_NUMPAD5
        VK_NUMPAD6 => {
            return Some(KeyboardKey::Num6);
        } // VK_NUMPAD6
        VK_NUMPAD7 => {
            return Some(KeyboardKey::Num7);
        } // VK_NUMPAD7
        VK_NUMPAD8 => {
            return Some(KeyboardKey::Num8);
        } // VK_NUMPAD8
        VK_NUMPAD9 => {
            return Some(KeyboardKey::Num9);
        } // VK_NUMPAD9
        VK_MULTIPLY => {
            return Some(KeyboardKey::NumMultiply);
        } // VK_MULTIPLY
        VK_DECIMAL => {
            return Some(KeyboardKey::NumDelete);
        } // VK_DECIMAL
        VK_ADD => {
            return Some(KeyboardKey::NumAdd);
        } // VK_ADD
        VK_SUBTRACT => {
            return Some(KeyboardKey::NumSubtract);
        } // VK_SUBTRACT
        VK_DIVIDE => {
            return Some(KeyboardKey::NumDivide);
        } // VK_DIVIDE
        VK_F1 => {
            return Some(KeyboardKey::F1);
        } // VK_F1
        VK_F2 => {
            return Some(KeyboardKey::F2);
        } // VK_F2
        VK_F3 => {
            return Some(KeyboardKey::F3);
        } // VK_F3
        VK_F4 => {
            return Some(KeyboardKey::F4);
        } // VK_F4
        VK_F5 => {
            return Some(KeyboardKey::F5);
        } // VK_F5
        VK_F6 => {
            return Some(KeyboardKey::F6);
        } // VK_F6
        VK_F7 => {
            return Some(KeyboardKey::F7);
        } // VK_F7
        VK_F8 => {
            return Some(KeyboardKey::F8);
        } // VK_F8
        VK_F9 => {
            return Some(KeyboardKey::F9);
        } // VK_F9
        VK_F10 => {
            return Some(KeyboardKey::F10);
        } // VK_F10
        VK_F11 => {
            return Some(KeyboardKey::F11);
        } // VK_F11
        VK_F12 => {
            return Some(KeyboardKey::F12);
        } // VK_F12
        VK_F13 => {
            return Some(KeyboardKey::F13);
        } // VK_F13
        VK_F14 => {
            return Some(KeyboardKey::F14);
        } // VK_F14
        VK_F15 => {
            return Some(KeyboardKey::F15);
        } // VK_F15
        VK_F16 => {
            return Some(KeyboardKey::F16);
        } // VK_F16
        VK_F17 => {
            return Some(KeyboardKey::F17);
        } // VK_F17
        VK_F18 => {
            return Some(KeyboardKey::F18);
        } // VK_F18
        VK_F19 => {
            return Some(KeyboardKey::F19);
        } // VK_F19
        VK_F20 => {
            return Some(KeyboardKey::F20);
        } // VK_F20
        VK_F21 => {
            return Some(KeyboardKey::F21);
        } // VK_F21
        VK_F22 => {
            return Some(KeyboardKey::F22);
        } // VK_F22
        VK_F23 => {
            return Some(KeyboardKey::F23);
        } // VK_F23
        VK_F24 => {
            return Some(KeyboardKey::F24);
        } // VK_F24
        VK_NUMLOCK => {
            return Some(KeyboardKey::NumLock);
        } // VK_NUMLOCK
        VK_SCROLL => {
            return Some(KeyboardKey::ScrollStop);
        } // VK_SCROLL
        VK_LSHIFT => {
            return Some(KeyboardKey::LeftShift);
        } // VK_LSHIFT
        VK_RSHIFT => {
            return Some(KeyboardKey::RightShift);
        } // VK_RSHIFT
        VK_LCONTROL => {
            return Some(KeyboardKey::LeftControl);
        } // VK_LCONTROL
        VK_RCONTROL => {
            return Some(KeyboardKey::RightControl);
        } // VK_RCONTROL
        VK_APPS => {
            return Some(KeyboardKey::Apps);
        } // VK_APPS
        VK_LMENU => {
            return Some(KeyboardKey::LeftMenu);
        } // VK_LMENU
        VK_RMENU => {
            return Some(KeyboardKey::RightMenu);
        } // VK_RMENU
        VK_VOLUME_MUTE => {
            return Some(KeyboardKey::VolumeMute);
        } // VK_VOLUME_MUTE
        VK_VOLUME_UP => {
            return Some(KeyboardKey::VolumeUp);
        } // VK_VOLUME_UP
        VK_VOLUME_DOWN => {
            return Some(KeyboardKey::VolumeDown);
        } // VK_VOLUME_DOWN
        VK_MEDIA_NEXT_TRACK => {
            return Some(KeyboardKey::MediaNextTrack);
        } // VK_MEDIA_NEXT_TRACK
        VK_MEDIA_PREV_TRACK => {
            return Some(KeyboardKey::MediaPrevTrack);
        } // VK_MEDIA_PREV_TRACK
        VK_MEDIA_PLAY_PAUSE => {
            return Some(KeyboardKey::MediaPlayPause);
        } // VK_MEDIA_PLAY_PAUSE
        VK_MEDIA_STOP => {
            return Some(KeyboardKey::MediaStop);
        } // VK_MEDIA_STOP
        VK_OEM_PLUS => {
            return Some(KeyboardKey::Add);
        } // VK_OEM_PLUS
        VK_OEM_7 => {
            return Some(KeyboardKey::Power);
        } // VK_OEM_7
        VK_CAPITAL => {
            return Some(KeyboardKey::CapsLock);
        } // VK_CAPITAL
        VK_CONTROL => unsafe {
            return if MapVirtualKeyW(scan_code, MAPVK_VSC_TO_VK_EX) == VK_RCONTROL.0 as u32 {
                Some(KeyboardKey::RightControl)
            } else {
                Some(KeyboardKey::LeftControl)
            };
        }, // VK_CONTROL
        VK_MENU => {
            return Some(KeyboardKey::Alt);
        } // VK_MENU
        VK_OEM_8 => {
            return Some(KeyboardKey::Exclamation);
        } // VK_OEM_8
        VK_OEM_2 => {
            return Some(KeyboardKey::Colon);
        } // VK_OEM_2
        VK_OEM_PERIOD => {
            return Some(KeyboardKey::Period);
        } // VK_OEM_PERIOD
        VK_OEM_COMMA => {
            return Some(KeyboardKey::Comma);
        } // VK_OEM_COMMA
        VK_OEM_4 => {
            return Some(KeyboardKey::LeftBracket);
        } // VK_OEM_4
        VK_OEM_3 => {
            return Some(KeyboardKey::Tilde);
        } // VK_OEM_3
        VK_OEM_1 => {
            return Some(KeyboardKey::Semicolon);
        } // VK_OEM_1
        VK_OEM_6 => {
            return Some(KeyboardKey::RightBracket);
        } // VK_OEM_6
        VK_OEM_5 => {
            return Some(KeyboardKey::LeftBracket);
        } // VK_OEM_5²
        VK_OEM_102 => {
            return Some(KeyboardKey::BackSlash);
        } // VK_OEM_102
        _ => {}
    }

    Some(KeyboardKey::Any(wp.0))
}

pub fn win32_input(msg: u32, wparam: WPARAM, lparam: LPARAM, input_manager: &InputManager) {
    let _extended: bool = lparam.0 & 0x01000000 != 0;
    let scancode: u32 = ((lparam.0 & 0x00ff0000) >> 16) as u32;

    match msg {
        WM_KEYDOWN => match win32_keyboard(wparam, scancode) {
            None => {}
            Some(keyboard_key) => {
                input_manager._press_input(InputMapping::Keyboard(keyboard_key));
            }
        },
        WM_KEYUP => match win32_keyboard(wparam, scancode) {
            None => {}
            Some(keyboard_key) => {
                input_manager._release_input(InputMapping::Keyboard(keyboard_key));
            }
        },
        WM_MOUSEMOVE => {
            input_manager._set_mouse_pos(loword(lparam.0) as f32, hiword(lparam.0) as f32);
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN | WM_LBUTTONUP
        | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => match win32_mouse_button(wparam.0) {
            None => {}
            Some((input, pressed)) => {
                if pressed {
                    input_manager._press_input(InputMapping::MouseButton(input));
                } else {
                    input_manager._release_input(InputMapping::MouseButton(input));
                }
            }
        },
        _ => {}
    }
}
