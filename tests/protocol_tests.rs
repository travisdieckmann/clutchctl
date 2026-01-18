//! Protocol encoding/decoding tests

use clutchctl_core::configuration::{
    Configuration, GamepadConfiguration, KeyboardConfiguration, MediaConfiguration,
    MouseConfiguration, TextConfiguration, KeyMode, Trigger,
};
use clutchctl_core::protocol::{
    self, ConfigPacket, ConfigType, GameKey, MediaButton, ModifierKeys, ProtocolMouseButton,
};
use std::collections::HashSet;

#[test]
fn test_packet_size() {
    assert_eq!(std::mem::size_of::<ConfigPacket>(), 40);
}

#[test]
fn test_unconfigured_encoding() {
    let config = Configuration::Unconfigured;
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Unconfigured));
    assert_eq!(packet.size, 0);
}

#[test]
fn test_keyboard_encoding() {
    let mut kbd = KeyboardConfiguration::new(
        KeyMode::Standard,
        vec!["a".to_string()],
    );
    kbd.modifiers = ModifierKeys::LEFT_CONTROL | ModifierKeys::LEFT_SHIFT;

    let config = Configuration::Keyboard(kbd);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Keyboard));
    assert_eq!(packet.size, 40);
}

#[test]
fn test_mouse_button_encoding() {
    let mut buttons = HashSet::new();
    buttons.insert(MouseButton::Left);
    buttons.insert(MouseButton::Right);

    let mouse = MouseConfiguration::buttons(buttons);
    let config = Configuration::Mouse(mouse);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Mouse));

    // Check that the buttons are encoded correctly
    let data = packet.parse_data();
    if let protocol::ConfigData::Mouse(mouse_data) = data {
        let proto_buttons = ProtocolMouseButton::from_bits_truncate(mouse_data.buttons);
        assert!(proto_buttons.contains(ProtocolMouseButton::LEFT));
        assert!(proto_buttons.contains(ProtocolMouseButton::RIGHT));
    } else {
        panic!("Expected mouse data");
    }
}

#[test]
fn test_mouse_axis_encoding() {
    let mouse = MouseConfiguration::axis(10, -20, 5);
    let config = Configuration::Mouse(mouse);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Mouse));

    let data = packet.parse_data();
    if let protocol::ConfigData::Mouse(mouse_data) = data {
        assert_eq!(mouse_data.mouse_x, 10);
        assert_eq!(mouse_data.mouse_y, -20);
        assert_eq!(mouse_data.mouse_wheel, 5);
    } else {
        panic!("Expected mouse data");
    }
}

#[test]
fn test_text_encoding() {
    let text = TextConfiguration::new("Hello, World!".to_string());
    let config = Configuration::Text(text);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Text));
    assert_eq!(packet.size, 40);

    // Decode and verify
    let decoded = protocol::ikkegol::parse_config(&packet).unwrap();
    if let Configuration::Text(text_config) = decoded {
        assert_eq!(text_config.text, "Hello, World!");
    } else {
        panic!("Expected text configuration");
    }
}

#[test]
fn test_media_encoding() {
    let media = MediaConfiguration::new(MediaButton::Play);
    let config = Configuration::Media(media);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Media));

    let data = packet.parse_data();
    if let protocol::ConfigData::Media(media_data) = data {
        assert_eq!(media_data.key, MediaButton::Play as u8);
    } else {
        panic!("Expected media data");
    }
}

#[test]
fn test_gamepad_encoding() {
    let gamepad = GamepadConfiguration::new(GameKey::Button1);
    let config = Configuration::Gamepad(gamepad);
    let packet = protocol::ikkegol::encode_config(&config).unwrap();

    assert_eq!(packet.get_config_type(), Some(ConfigType::Game));

    let data = packet.parse_data();
    if let protocol::ConfigData::Game(game_data) = data {
        assert_eq!(game_data.key, GameKey::Button1 as u8);
    } else {
        panic!("Expected game data");
    }
}

#[test]
fn test_trigger_mode_conversion() {
    use clutchctl_core::protocol::TriggerMode;

    let press = Trigger::OnPress;
    let mode: TriggerMode = press.into();
    assert_eq!(mode, TriggerMode::Press);

    let release = Trigger::OnRelease;
    let mode: TriggerMode = release.into();
    assert_eq!(mode, TriggerMode::Release);
}