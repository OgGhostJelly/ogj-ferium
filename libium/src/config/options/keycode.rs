use strum::{EnumString, FromRepr, IntoStaticStr};

/// A Minecraft keycode,stored in the `options.txt` file, should be compatible with most versions of Minecraft.
#[derive(EnumString, FromRepr, IntoStaticStr)]
pub enum Keycode {
    #[strum(serialize = "key.keyboard.unknown")]
    Unknown = 0,
    #[strum(serialize = "key.keyboard.escape")]
    Escape = 1,
    #[strum(serialize = "key.keyboard.1")]
    Key1 = 2,
    #[strum(serialize = "key.keyboard.2")]
    Key2 = 3,
    #[strum(serialize = "key.keyboard.3")]
    Key3 = 4,
    #[strum(serialize = "key.keyboard.4")]
    Key4 = 5,
    #[strum(serialize = "key.keyboard.5")]
    Key5 = 6,
    #[strum(serialize = "key.keyboard.6")]
    Key6 = 7,
    #[strum(serialize = "key.keyboard.7")]
    Key7 = 8,
    #[strum(serialize = "key.keyboard.8")]
    Key8 = 9,
    #[strum(serialize = "key.keyboard.9")]
    Key9 = 10,
    #[strum(serialize = "key.keyboard.0")]
    Key0 = 11,
    #[strum(serialize = "key.keyboard.minus")]
    Minus = 12,
    #[strum(serialize = "key.keyboard.equal")]
    Equal = 13,
    #[strum(serialize = "key.keyboard.backspace")]
    Backspace = 14,
    #[strum(serialize = "key.keyboard.tab")]
    Tab = 15,
    #[strum(serialize = "key.keyboard.q")]
    Q = 16,
    #[strum(serialize = "key.keyboard.w")]
    W = 17,
    #[strum(serialize = "key.keyboard.e")]
    E = 18,
    #[strum(serialize = "key.keyboard.r")]
    R = 19,
    #[strum(serialize = "key.keyboard.t")]
    T = 20,
    #[strum(serialize = "key.keyboard.y")]
    Y = 21,
    #[strum(serialize = "key.keyboard.u")]
    U = 22,
    #[strum(serialize = "key.keyboard.i")]
    I = 23,
    #[strum(serialize = "key.keyboard.o")]
    O = 24,
    #[strum(serialize = "key.keyboard.p")]
    P = 25,
    #[strum(serialize = "key.keyboard.left.bracket")]
    LeftBracket = 26,
    #[strum(serialize = "key.keyboard.right.bracket")]
    RightBracket = 27,
    #[strum(serialize = "key.keyboard.enter")]
    Enter = 28,
    #[strum(serialize = "key.keyboard.left.control")]
    LeftControl = 29,
    #[strum(serialize = "key.keyboard.a")]
    A = 30,
    #[strum(serialize = "key.keyboard.s")]
    S = 31,
    #[strum(serialize = "key.keyboard.d")]
    D = 32,
    #[strum(serialize = "key.keyboard.f")]
    F = 33,
    #[strum(serialize = "key.keyboard.g")]
    G = 34,
    #[strum(serialize = "key.keyboard.h")]
    H = 35,
    #[strum(serialize = "key.keyboard.j")]
    J = 36,
    #[strum(serialize = "key.keyboard.k")]
    K = 37,
    #[strum(serialize = "key.keyboard.l")]
    L = 38,
    #[strum(serialize = "key.keyboard.semicolon")]
    Semicolon = 39,
    #[strum(serialize = "key.keyboard.apostrophe")]
    Apostrophe = 40,
    #[strum(serialize = "key.keyboard.grave.accent")]
    Grave = 41,
    #[strum(serialize = "key.keyboard.left.shift")]
    LeftShift = 42,
    #[strum(serialize = "key.keyboard.backslash")]
    Backslash = 43,
    #[strum(serialize = "key.keyboard.z")]
    Z = 44,
    #[strum(serialize = "key.keyboard.x")]
    X = 45,
    #[strum(serialize = "key.keyboard.c")]
    C = 46,
    #[strum(serialize = "key.keyboard.v")]
    V = 47,
    #[strum(serialize = "key.keyboard.b")]
    B = 48,
    #[strum(serialize = "key.keyboard.n")]
    N = 49,
    #[strum(serialize = "key.keyboard.m")]
    M = 50,
    #[strum(serialize = "key.keyboard.comma")]
    Comma = 51,
    #[strum(serialize = "key.keyboard.period")]
    Period = 52,
    #[strum(serialize = "key.keyboard.slash")]
    Slash = 53,
    #[strum(serialize = "key.keyboard.right.shift")]
    RightShift = 54,
    #[strum(serialize = "key.keyboard.multiply")]
    Multiply = 55,
    #[strum(serialize = "key.keyboard.menu")]
    Menu = 56,
    #[strum(serialize = "key.keyboard.space")]
    Space = 57,
    #[strum(serialize = "key.keyboard.caps.lock")]
    CapsLock = 58,
    #[strum(serialize = "key.keyboard.f1")]
    F1 = 59,
    #[strum(serialize = "key.keyboard.f2")]
    F2 = 60,
    #[strum(serialize = "key.keyboard.f3")]
    F3 = 61,
    #[strum(serialize = "key.keyboard.f4")]
    F4 = 62,
    #[strum(serialize = "key.keyboard.f5")]
    F5 = 63,
    #[strum(serialize = "key.keyboard.f6")]
    F6 = 64,
    #[strum(serialize = "key.keyboard.f7")]
    F7 = 65,
    #[strum(serialize = "key.keyboard.f8")]
    F8 = 66,
    #[strum(serialize = "key.keyboard.f9")]
    F9 = 67,
    #[strum(serialize = "key.keyboard.f10")]
    F10 = 68,
    #[strum(serialize = "key.keyboard.num.lock")]
    NumLock = 69,
    #[strum(serialize = "key.keyboard.scroll.lock")]
    ScrollLock = 70,
    #[strum(serialize = "key.keyboard.keypad.7")]
    Keypad7 = 71,
    #[strum(serialize = "key.keyboard.keypad.8")]
    Keypad8 = 72,
    #[strum(serialize = "key.keyboard.keypad.9")]
    Keypad9 = 73,
    #[strum(serialize = "key.keyboard.keypad.subtract")]
    KeypadSubtract = 74,
    #[strum(serialize = "key.keyboard.keypad.4")]
    Keypad4 = 75,
    #[strum(serialize = "key.keyboard.keypad.5")]
    Keypad5 = 76,
    #[strum(serialize = "key.keyboard.keypad.6")]
    Keypad6 = 77,
    #[strum(serialize = "key.keyboard.keypad.add")]
    KeypadAdd = 78,
    #[strum(serialize = "key.keyboard.keypad.1")]
    Keypad1 = 79,
    #[strum(serialize = "key.keyboard.keypad.2")]
    Keypad2 = 80,
    #[strum(serialize = "key.keyboard.keypad.3")]
    Keypad3 = 81,
    #[strum(serialize = "key.keyboard.keypad.0")]
    Keypad0 = 82,
    #[strum(serialize = "key.keyboard.keypad.decimal")]
    KeypadDecimal = 83,
    #[strum(serialize = "key.keyboard.f11")]
    F11 = 84,
    #[strum(serialize = "key.keyboard.f12")]
    F12 = 85,
    #[strum(serialize = "key.keyboard.f13")]
    F13 = 86,
    #[strum(serialize = "key.keyboard.f14")]
    F14 = 87,
    #[strum(serialize = "key.keyboard.f15")]
    F15 = 88,
    // KANA
    // CONVERT
    // NOCONVERT
    // YEN
    #[strum(serialize = "key.keyboard.keypad.equal")]
    KeypadEqual = 141,
    // CIRCUMFLEX
    // AT
    // COLON
    // UNDERLINE
    // KANJI
    // STOP
    // AX
    // UNLABELED
    #[strum(serialize = "key.keyboard.keypad.enter")]
    KeypadEnter = 156,
    #[strum(serialize = "key.keyboard.right.control")]
    RightControl = 157,
    // NUMPADCOMMA
    #[strum(serialize = "key.keyboard.keypad.divide")]
    KeypadDivide = 181,
    // SYSRQ
    // RMENU
    #[strum(serialize = "key.keyboard.pause")]
    Pause = 197,
    #[strum(serialize = "key.keyboard.home")]
    Home = 199,
    #[strum(serialize = "key.keyboard.up")]
    Up = 200,
    #[strum(serialize = "key.keyboard.page.up")]
    PageUp = 201,
    #[strum(serialize = "key.keyboard.left")]
    Left = 203,
    #[strum(serialize = "key.keyboard.right")]
    Right = 205,
    #[strum(serialize = "key.keyboard.end")]
    End = 207,
    #[strum(serialize = "key.keyboard.down")]
    Down = 208,
    #[strum(serialize = "key.keyboard.page.down")]
    PageDown = 209,
    #[strum(serialize = "key.keyboard.insert")]
    Insert = 210,
    #[strum(serialize = "key.keyboard.delete")]
    Delete = 211,
    #[strum(serialize = "key.keyboard.left.win")]
    LeftSuper = 219,
    #[strum(serialize = "key.keyboard.right.win")]
    RightSuper = 220,
    // APPS
    // POWER
    // SLEEP
}

impl Keycode {
    pub fn id_post1_13(self) -> &'static str {
        self.into()
    }

    pub fn id_pre1_13(self) -> usize {
        self as usize
    }
}
