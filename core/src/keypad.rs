#[derive(Debug, Primitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Keys {
    ButtonA = 0,
    ButtonB = 1,
    Select = 2,
    Start = 3,
    Right = 4,
    Left = 5,
    Up = 6,
    Down = 7,
    ButtonR = 8,
    ButtonL = 9,
}

pub const NUM_KEYS: usize = 10;
pub const KEYINPUT_ALL_RELEASED: u16 = 0b1111111111;

#[derive(Debug, Primitive, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyState {
    Pressed = 0,
    Released = 1,
}

impl From<KeyState> for bool {
    fn from(val: KeyState) -> Self {
        match val {
            KeyState::Pressed => false,
            KeyState::Released => true,
        }
    }
}
