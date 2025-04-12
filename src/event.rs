//! all Event

pub enum Event {
    Key,
    Mouse,
    Unsupported(Vec<u8>),
}

pub enum Key {
    /// Backspace.
    Backspace,
    /// Left arrow.
    Left,
    /// Shift Left arrow.
    ShiftLeft,
    /// Alt Left arrow.
    AltLeft,
    /// Ctrl Left arrow.
    CtrlLeft,
    /// Right arrow.
    Right,
    /// Shift Right arrow.
    ShiftRight,
    /// Alt Right arrow.
    AltRight,
    /// Ctrl Right arrow.
    CtrlRight,
    /// Up arrow.
    Up,
    /// Shift Up arrow.
    ShiftUp,
    /// Alt Up arrow.
    AltUp,
    /// Ctrl Up arrow.
    CtrlUp,
    /// Down arrow.
    Down,
    /// Shift Down arrow.
    ShiftDown,
    /// Alt Down arrow.
    AltDown,
    /// Ctrl Down arrow
    CtrlDown,
    /// Home key.
    Home,
    /// Ctrl Home key.
    CtrlHome,
    /// End key.
    End,
    /// Ctrl End key.
    CtrlEnd,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Backward Tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Function keys.
    ///
    /// Only function keys 1 through 12 are supported.
    F(u8),
    /// Normal character.
    Char(char),
    /// Alt modified character.
    Alt(char),
    /// Ctrl modified character.
    ///
    /// Note that certain keys may not be modifiable with `ctrl`, due to limitations of terminals.
    Ctrl(char),
    /// Null byte.
    Null,
    /// Esc key.
    Esc,
}