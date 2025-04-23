use std::fmt;

/// A `key_key` field in `options.txt`.
/// Used to store the keycode of an action in Minecraft,
/// e.g `key_key.attack` or `key_key.jump`.
///
/// Used in conjunction with [super::Options::set_keybind].
pub enum KeyKey {
    Attack,
    Use,
    Forward,
    Left,
    Back,
    Right,
    Jump,
    Sneak,
    Sprint,
    Drop,
    Inventory,
    Chat,
    Playerlist,
    PickItem,
    Command,
    SocialInteractions,
    Screenshot,
    TogglePerspective,
    SmoothCamera,
    Fullscreen,
    SpectatorOutlines,
    SwapOffhand,
    SaveToolbarActivator,
    LoadToolbarActivator,
    Advancements,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
}

impl KeyKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Attack => "key_key.attack",
            Self::Use => "key_key.use",
            Self::Forward => "key_key.forward",
            Self::Left => "key_key.left",
            Self::Back => "key_key.back",
            Self::Right => "key_key.right",
            Self::Jump => "key_key.jump",
            Self::Sneak => "key_key.sneak",
            Self::Sprint => "key_key.sprint",
            Self::Drop => "key_key.drop",
            Self::Inventory => "key_key.inventory",
            Self::Chat => "key_key.chat",
            Self::Playerlist => "key_key.playerlist",
            Self::PickItem => "key_key.pickItem",
            Self::Command => "key_key.command",
            Self::SocialInteractions => "key_key.socialInteractions",
            Self::Screenshot => "key_key.screenshot",
            Self::TogglePerspective => "key_key.togglePerspective",
            Self::SmoothCamera => "key_key.smoothCamera",
            Self::Fullscreen => "key_key.fullscreen",
            Self::SpectatorOutlines => "key_key.spectatorOutlines",
            Self::SwapOffhand => "key_key.swapOffhand",
            Self::SaveToolbarActivator => "key_key.saveToolbarActivator",
            Self::LoadToolbarActivator => "key_key.loadToolbarActivator",
            Self::Advancements => "key_key.advancements",
            Self::Hotbar1 => "key_key.hotbar.1",
            Self::Hotbar2 => "key_key.hotbar.2",
            Self::Hotbar3 => "key_key.hotbar.3",
            Self::Hotbar4 => "key_key.hotbar.4",
            Self::Hotbar5 => "key_key.hotbar.5",
            Self::Hotbar6 => "key_key.hotbar.6",
            Self::Hotbar7 => "key_key.hotbar.7",
            Self::Hotbar8 => "key_key.hotbar.8",
            Self::Hotbar9 => "key_key.hotbar.9",
        }
    }
}

impl fmt::Display for KeyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
