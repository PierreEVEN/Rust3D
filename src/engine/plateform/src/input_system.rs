use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone)]
pub enum ActionType {
    Press,
    Release,
    Hold,
}

pub type ActionMappingCallback = Box<dyn FnMut(&InputAction, ActionType)>;
pub type AxisMappingCallback = Box<dyn FnMut(&InputAxis)>;

pub struct InputManager {
    action_mapping: RwLock<HashMap<String, Arc<InputAction>>>,
    axis_mapping: RwLock<HashMap<String, Arc<InputAxis>>>,
}

pub struct InputAction {
    press_progress: i8,
    actions: HashSet<(InputMapping, bool)>,
    bindings: RwLock<Vec<ActionMappingCallback>>,
}

impl InputAction {
    pub fn map(mut self, input: InputMapping) -> Self {
        self.actions.insert((input, false));
        self.press_progress += 1;
        self
    }
}

pub struct InputAxis {
    name: String,
    axis: HashSet<InputMapping>,
    bindings: RwLock<Vec<AxisMappingCallback>>,
}

impl InputAxis {
    pub fn map_axis(mut self, input: InputMapping) -> Self {
        self.axis.insert(input);
        self
    }
}


impl InputManager {
    pub fn new() -> Self {
        Self { action_mapping: Default::default(), axis_mapping: Default::default() }
    }

    pub fn new_action(self, name: &str) -> Arc<InputAction> {
        let input_action = Arc::new(InputAction {
            press_progress: 0,
            actions: HashSet::default(),
            bindings: Default::default(),
        });
        self.action_mapping.write().unwrap().insert(name.to_string(), input_action.clone());
        input_action
    }
    pub fn new_axis(&self, name: &str) -> InputAxis {
        InputAxis {
            name: name.to_string(),
            axis: HashSet::default(),
            bindings: Default::default(),
        }
    }

    pub fn bind_action(&self, name: &str, event: ActionMappingCallback) {
        match self.action_mapping.write().unwrap().get_mut(name) {
            None => { panic!("cannot find action {name}"); }
            Some(mapping) => { mapping.bindings.write().unwrap().push(event); }
        }
    }
    pub fn bind_axis(&self, name: &str, event: AxisMappingCallback) {
        match self.axis_mapping.write().unwrap().get_mut(name) {
            None => { panic!("cannot find axis {name}"); }
            Some(mapping) => { mapping.bindings.write().unwrap().push(event); }
        }
    }

    pub fn press_input(&mut self, pressed_key: InputMapping) {
        for (_, action) in &mut *self.action_mapping.write().unwrap() {
            let mut just_pressed = false;
            for (key, mut status) in &action.actions {
                if *key == pressed_key && status != true {
                    action.press_progress -= 1;
                    status = true;
                    just_pressed = true;
                }
            }
            if action.press_progress < 0 {
                panic!("press progress should never be under 0");
            }
            if action.press_progress == 0 {
                let action_type = if just_pressed { ActionType::Press } else { ActionType::Hold };                
                for binding in &mut *action.bindings.write().unwrap() {
                    binding.as_mut()(action as &InputAction, action_type)                    
                }
            }
        }
    }
    
    pub fn release_input(&mut self, pressed_key: InputMapping) {
        for (_, action) in &mut *self.action_mapping.write().unwrap() {
            let mut just_released = false;
            for (key, mut status) in &action.actions {
                if *key == pressed_key && status != false {
                    action.press_progress += 1;
                    status = false;
                    just_released = true;
                }
            }
            if just_released {
                for binding in &mut *action.bindings.write().unwrap() {
                    binding.as_mut()(action as &InputAction, ActionType::Release)
                }
            }
        }
    }
}

pub fn test() {
    let mut input_manager = InputManager::new();

    input_manager.new_action("MoveForward")
        .map(InputMapping::Keyboard(KeyboardKey::Key0))
        .map(InputMapping::Keyboard(KeyboardKey::KeyA));

    input_manager.bind_action("MoveForward", Box::new(move |action, action_type| {
        match action_type {
            ActionType::Press => {}
            _ => {}
        }
    }));

    input_manager.press_input(InputMapping::Keyboard(KeyboardKey::Key0));
    input_manager.press_input(InputMapping::Keyboard(KeyboardKey::KeyA));
    input_manager.release_input(InputMapping::Keyboard(KeyboardKey::Key0));
    input_manager.release_input(InputMapping::Keyboard(KeyboardKey::KeyA));
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
    KeyKeyX,
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