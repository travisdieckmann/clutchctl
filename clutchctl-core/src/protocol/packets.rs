//! Binary packet structures for iKKEGOL USB protocol
//! These structures must maintain exact binary compatibility with the C++ implementation

use bitflags::bitflags;

/// Configuration type identifiers
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Unconfigured = 0x00,
    Keyboard = 0x01,
    KeyboardOnce = 0x81,
    Mouse = 0x02,
    Text = 0x04,
    KeyboardMulti = 0x06,
    KeyboardMultiOnce = 0x86,
    Media = 0x07,
    Game = 0x08,
}

impl ConfigType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Unconfigured),
            0x01 => Some(Self::Keyboard),
            0x81 => Some(Self::KeyboardOnce),
            0x02 => Some(Self::Mouse),
            0x04 => Some(Self::Text),
            0x06 => Some(Self::KeyboardMulti),
            0x86 => Some(Self::KeyboardMultiOnce),
            0x07 => Some(Self::Media),
            0x08 => Some(Self::Game),
            _ => None,
        }
    }
}

bitflags! {
    /// Keyboard modifier keys
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ModifierKeys: u8 {
        const LEFT_CONTROL = 0x01;
        const LEFT_SHIFT = 0x02;
        const LEFT_ALT = 0x04;
        const LEFT_SUPER = 0x08;
        const RIGHT_CONTROL = 0x10;
        const RIGHT_SHIFT = 0x20;
        const RIGHT_ALT = 0x40;
        const RIGHT_SUPER = 0x80;
    }
}

bitflags! {
    /// Mouse button flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProtocolMouseButton: u8 {
        const LEFT = 0x01;
        const RIGHT = 0x02;
        const MIDDLE = 0x04;
        const BACK = 0x08;
        const FORWARD = 0x10;
    }
}

/// Media button codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaButton {
    VolumeMinus = 1,
    VolumePlus = 2,
    Mute = 3,
    Play = 4,
    Forward = 5,
    Next = 6,
    Stop = 7,
    OpenPlayer = 8,
    OpenHomepage = 9,
    StopWebpage = 10,
    BackBrowse = 11,
    ForwardBrowse = 12,
    Refresh = 13,
    OpenMyComputer = 14,
    OpenMail = 15,
    OpenCalc = 16,
    OpenSearch = 17,
    Shutdown = 18,
    Sleep = 19,
}

impl MediaButton {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::VolumeMinus),
            2 => Some(Self::VolumePlus),
            3 => Some(Self::Mute),
            4 => Some(Self::Play),
            5 => Some(Self::Forward),
            6 => Some(Self::Next),
            7 => Some(Self::Stop),
            8 => Some(Self::OpenPlayer),
            9 => Some(Self::OpenHomepage),
            10 => Some(Self::StopWebpage),
            11 => Some(Self::BackBrowse),
            12 => Some(Self::ForwardBrowse),
            13 => Some(Self::Refresh),
            14 => Some(Self::OpenMyComputer),
            15 => Some(Self::OpenMail),
            16 => Some(Self::OpenCalc),
            17 => Some(Self::OpenSearch),
            18 => Some(Self::Shutdown),
            19 => Some(Self::Sleep),
            _ => None,
        }
    }
}

/// Game button codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKey {
    Left = 1,
    Right = 2,
    Up = 3,
    Down = 4,
    Button1 = 5,
    Button2 = 6,
    Button3 = 7,
    Button4 = 8,
    Button5 = 9,
    Button6 = 10,
    Button7 = 11,
    Button8 = 12,
}

impl GameKey {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Left),
            2 => Some(Self::Right),
            3 => Some(Self::Up),
            4 => Some(Self::Down),
            5 => Some(Self::Button1),
            6 => Some(Self::Button2),
            7 => Some(Self::Button3),
            8 => Some(Self::Button4),
            9 => Some(Self::Button5),
            10 => Some(Self::Button6),
            11 => Some(Self::Button7),
            12 => Some(Self::Button8),
            _ => None,
        }
    }
}

/// Trigger mode for pedal activation
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    Release = 0,
    Press = 1,
}

impl TriggerMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Release),
            1 => Some(Self::Press),
            _ => None,
        }
    }
}

/// Keyboard configuration data
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct KeyboardData {
    pub modifiers: u8,
    pub keys: [u8; 6],
}

/// Mouse configuration data
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MouseData {
    pub unknown: [u8; 2],
    pub buttons: u8,
    pub mouse_x: i8,
    pub mouse_y: i8,
    pub mouse_wheel: i8,
}

/// Media configuration data
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MediaData {
    pub key: u8,
}

/// Game configuration data
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GameData {
    pub key: u8,
}

/// Text configuration data
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TextData {
    pub string: [u8; 38],
}

/// Configuration data union
/// Note: Rust doesn't have unions in safe code, so we use an enum
#[derive(Debug, Clone, Copy)]
pub enum ConfigData {
    Keyboard(KeyboardData),
    Mouse(MouseData),
    Media(MediaData),
    Game(GameData),
    Text(TextData),
    Raw([u8; 38]),
}

/// Main configuration packet structure
/// This must maintain binary compatibility with the C++ implementation
#[repr(C, packed)]
pub struct ConfigPacket {
    pub size: u8,
    pub config_type: u8,  // We use u8 instead of ConfigType for binary compatibility
    pub data: [u8; 38],    // Raw data that will be interpreted based on config_type
}

impl ConfigPacket {
    pub const PACKET_SIZE: usize = 40;

    /// Create an empty/unconfigured packet
    pub fn unconfigured() -> Self {
        Self {
            size: 0,
            config_type: ConfigType::Unconfigured as u8,
            data: [0; 38],
        }
    }

    /// Get the configuration type
    pub fn get_config_type(&self) -> Option<ConfigType> {
        ConfigType::from_u8(self.config_type)
    }

    /// Parse the data field based on the configuration type
    pub fn parse_data(&self) -> ConfigData {
        match self.get_config_type() {
            Some(ConfigType::Keyboard) | Some(ConfigType::KeyboardOnce) |
            Some(ConfigType::KeyboardMulti) | Some(ConfigType::KeyboardMultiOnce) => {
                let keyboard = unsafe {
                    std::ptr::read_unaligned(self.data.as_ptr() as *const KeyboardData)
                };
                ConfigData::Keyboard(keyboard)
            }
            Some(ConfigType::Mouse) => {
                let mouse = unsafe {
                    std::ptr::read_unaligned(self.data.as_ptr() as *const MouseData)
                };
                ConfigData::Mouse(mouse)
            }
            Some(ConfigType::Media) => {
                let media = unsafe {
                    std::ptr::read_unaligned(self.data.as_ptr() as *const MediaData)
                };
                ConfigData::Media(media)
            }
            Some(ConfigType::Game) => {
                let game = unsafe {
                    std::ptr::read_unaligned(self.data.as_ptr() as *const GameData)
                };
                ConfigData::Game(game)
            }
            Some(ConfigType::Text) => {
                let text = unsafe {
                    std::ptr::read_unaligned(self.data.as_ptr() as *const TextData)
                };
                ConfigData::Text(text)
            }
            _ => ConfigData::Raw(self.data),
        }
    }

    /// Convert to bytes for USB transmission
    pub fn to_bytes(&self) -> [u8; Self::PACKET_SIZE] {
        unsafe {
            std::mem::transmute_copy(self)
        }
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; Self::PACKET_SIZE]) -> Self {
        unsafe {
            std::mem::transmute_copy(bytes)
        }
    }
}

/// USB command codes
pub mod commands {
    pub const BEGIN_WRITE: [u8; 8] = [0x01, 0x80, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00];
    pub const READ_MODEL: [u8; 8] = [0x01, 0x83, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00];
    pub const READ_TRIGGER_MODES: [u8; 8] = [0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    pub fn read_config(pedal_index: u8) -> [u8; 8] {
        [0x01, 0x82, 0x08, pedal_index + 1, 0x00, 0x00, 0x00, 0x00]
    }

    pub fn write_config_header(size: u8, pedal_index: u8) -> [u8; 8] {
        [0x01, 0x81, size, pedal_index + 1, 0x00, 0x00, 0x00, 0x00]
    }

    pub fn write_trigger_modes(payload_size: u8) -> [u8; 8] {
        [0x01, 0x85, payload_size, 0x00, 0x00, 0x00, 0x00, 0x00]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_packet_size() {
        assert_eq!(std::mem::size_of::<ConfigPacket>(), ConfigPacket::PACKET_SIZE);
    }

    #[test]
    fn test_config_type_conversion() {
        assert_eq!(ConfigType::from_u8(0x01), Some(ConfigType::Keyboard));
        assert_eq!(ConfigType::from_u8(0x02), Some(ConfigType::Mouse));
        assert_eq!(ConfigType::from_u8(0xFF), None);
    }

    #[test]
    fn test_packet_round_trip() {
        let packet = ConfigPacket::unconfigured();
        let bytes = packet.to_bytes();
        let restored = ConfigPacket::from_bytes(&bytes);

        assert_eq!(packet.size, restored.size);
        assert_eq!(packet.config_type, restored.config_type);
    }
}