﻿use std::collections::{HashMap};
use std::sync::{RwLock};

#[derive(Copy, Clone)]
pub enum ActionType {
    Press,
    Release,
    Hold,
}

pub type ActionMappingCallback = Box<dyn FnMut(&InputAction, ActionType)>;
pub type AxisMappingCallback = Box<dyn FnMut(&InputAxis)>;

pub struct InputAction {
    released_key_left: i8,
    key_mapping: HashMap<InputMapping, bool>,
    // (input, is_pressed)
    callbacks: RwLock<Vec<ActionMappingCallback>>,
}

impl InputAction {
    pub fn new() -> Self {
        Self {
            released_key_left: 0,
            key_mapping: Default::default(),
            callbacks: Default::default(),
        }
    }

    pub fn map(mut self, input: InputMapping) -> Self {
        self.key_mapping.insert(input, false);
        self.released_key_left += 1;
        self
    }
}

pub struct InputAxis {
    axis_mapping: HashMap<InputMapping, f32>,
    // (input, value)
    callbacks: RwLock<Vec<AxisMappingCallback>>,
}

impl InputAxis {
    pub fn map_axis(mut self, input: InputMapping) -> Self {
        self.axis_mapping.insert(input, 0.0);
        self
    }
}

pub struct InputManager {
    action_mapping: RwLock<HashMap<String, InputAction>>,
    axis_mapping: RwLock<HashMap<String, InputAxis>>,
}

impl InputManager {
    pub fn new() -> Self {
        Self { action_mapping: Default::default(), axis_mapping: Default::default() }
    }

    pub fn new_action(&self, name: &str, action: InputAction) {
        self.action_mapping.write().unwrap().insert(name.to_string(), action);
    }
    pub fn new_axis(&self, name: &str, axis: InputAxis) {
        self.axis_mapping.write().unwrap().insert(name.to_string(), axis);
    }

    pub fn bind_action(&self, name: &str, event: ActionMappingCallback) {
        match self.action_mapping.write().unwrap().get_mut(name) {
            None => { panic!("cannot find action {name}"); }
            Some(mapping) => { mapping.callbacks.write().unwrap().push(event); }
        }
    }
    pub fn bind_axis(&self, name: &str, event: AxisMappingCallback) {
        match self.axis_mapping.write().unwrap().get_mut(name) {
            None => { panic!("cannot find axis {name}"); }
            Some(mapping) => { mapping.callbacks.write().unwrap().push(event); }
        }
    }

    pub fn press_input(&self, pressed_key: InputMapping) {
        for (_, action) in &mut *self.action_mapping.write().unwrap() {
            let mut just_pressed = false;

            match action.key_mapping.get_mut(&pressed_key) {
                None => {}
                Some(pressed) => {
                    if *pressed != true {
                        action.released_key_left -= 1;
                        *pressed = true;
                        just_pressed = true;
                    }
                }
            }
            if action.released_key_left < 0 {
                panic!("press progress should never be under 0");
            }
            if action.released_key_left == 0 {
                let action_type = if just_pressed { ActionType::Press } else { ActionType::Hold };
                for binding in &mut *action.callbacks.write().unwrap() {
                    binding.as_mut()(action as &InputAction, action_type)
                }
            }
        }
    }

    pub fn release_input(&self, pressed_key: InputMapping) {
        for (_, action) in &mut *self.action_mapping.write().unwrap() {
            let mut just_released = false;
            match action.key_mapping.get_mut(&pressed_key) {
                None => {}
                Some(pressed) => {
                    if *pressed != false {
                        if action.released_key_left == 0 {
                            just_released = true;
                        }
                        action.released_key_left += 1;
                        *pressed = false;
                    }
                }
            }
            if just_released {
                for binding in &mut *action.callbacks.write().unwrap() {
                    binding.as_mut()(action as &InputAction, ActionType::Release)
                }
            }
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum InputMapping {
    Keyboard(KeyboardKey),
    MouseButton(MouseButton),
    MouseAxis(MouseAxis),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button1,
    Button2,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum MouseAxis {
    X,
    Y,
    ScrollX,
    ScrollY,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum KeyboardKey {
    Any(usize),
    Backspace,
    Tab,
    Enter,
    Pause,
    CapsLock,
    Escape,
    Alt,
    Space,
    PageUp,
    PageDown,
    End,
    Home,
    Left,
    Up,
    Right,
    Down,
    Print,
    PrintScreen,
    Insert,
    Delete,
    Help,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    LeftWin,
    RightWin,
    Sleep,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    NumMultiply,
    NumAdd,
    NumSubtract,
    NumDivide,
    NumDelete,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    NumLock,
    ScrollStop,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    Apps,
    LeftMenu,
    RightMenu,
    VolumeMute,
    VolumeUp,
    VolumeDown,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaPlayPause,
    Add,
    Tilde,
    Exclamation,
    Colon,
    Comma,
    Period,
    LeftBracket,
    BackSlash,
    RightBracket,
    Semicolon,
    Power,
}