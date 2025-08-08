use juste::io::{Key, Mouse};
use winit::{event::MouseButton, keyboard::KeyCode};

pub fn filter_mouse(button: MouseButton) -> Mouse {
    use winit::event::MouseButton::*;

    match button {
        Left => Mouse::Left,
        Right => Mouse::Right,
        Middle => Mouse::Middle,
        _ => Mouse::Null,
    }
}
pub fn filter_keyboard(code: KeyCode) -> Key {
    use winit::keyboard::KeyCode::*;
    match code {
        KeyA => Key::A,
        KeyB => Key::B,
        KeyC => Key::C,
        KeyD => Key::D,
        KeyE => Key::E,
        KeyF => Key::F,
        KeyG => Key::G,
        KeyH => Key::H,
        KeyI => Key::I,
        KeyJ => Key::J,
        KeyK => Key::K,
        KeyL => Key::L,
        KeyM => Key::M,
        KeyN => Key::N,
        KeyO => Key::O,
        KeyP => Key::P,
        KeyQ => Key::Q,
        KeyR => Key::R,
        KeyS => Key::S,
        KeyT => Key::T,
        KeyU => Key::U,
        KeyV => Key::V,
        KeyW => Key::W,
        KeyX => Key::X,
        KeyY => Key::Y,
        KeyZ => Key::Z,

        Digit0 | Numpad0 => Key::Num0,
        Digit1 | Numpad1 => Key::Num1,
        Digit2 | Numpad2 => Key::Num2,
        Digit3 | Numpad3 => Key::Num3,
        Digit4 | Numpad4 => Key::Num4,
        Digit5 | Numpad5 => Key::Num5,
        Digit6 | Numpad6 => Key::Num6,
        Digit7 | Numpad7 => Key::Num7,
        Digit8 | Numpad8 => Key::Num8,
        Digit9 | Numpad9 => Key::Num9,

        F1 => Key::F1,
        F2 => Key::F2,
        F3 => Key::F3,
        F4 => Key::F4,
        F5 => Key::F5,
        F6 => Key::F6,
        F7 => Key::F7,
        F8 => Key::F8,
        F9 => Key::F9,
        F10 => Key::F10,
        F11 => Key::F11,
        F12 => Key::F12,

        ShiftLeft | ShiftRight => Key::Shift,
        ControlLeft | ControlRight => Key::Control,
        AltLeft | AltRight => Key::Alt,
        SuperLeft | SuperRight => Key::Meta,
        CapsLock => Key::CapsLock,
        NumLock => Key::NumLock,
        ScrollLock => Key::ScrollLock,

        Home => Key::Home,
        End => Key::End,
        PageUp => Key::PageUp,
        PageDown => Key::PageDown,
        Insert => Key::Insert,
        Delete => Key::Delete,
        ArrowLeft => Key::LeftArrow,
        ArrowRight => Key::RightArrow,
        ArrowUp => Key::UpArrow,
        ArrowDown => Key::DownArrow,

        Escape => Key::Escape,
        Space => Key::Space,
        Enter | NumpadEnter => Key::Enter,
        Backspace => Key::Backspace,
        Tab => Key::Tab,
        Pause => Key::Pause,
        PrintScreen => Key::PrintScreen,
        // Menu => juste::Key::Menu,
        Power => Key::Power,
        Sleep => Key::Sleep,
        WakeUp => Key::Wake,

        _ => Key::Null,
    }
}
