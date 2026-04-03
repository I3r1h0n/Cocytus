pub mod styles;
pub mod json_highlight;
pub mod cpp_highlight;

use iced::Color;

// Background colors
/// Main window background (darkest)
pub const BG_BASE: Color = Color::from_rgb(0x0d as f32 / 255.0, 0x11 as f32 / 255.0, 0x17 as f32 / 255.0);
/// Card / panel background
pub const BG_SURFACE: Color = Color::from_rgb(0x16 as f32 / 255.0, 0x1b as f32 / 255.0, 0x22 as f32 / 255.0);
/// Button / input resting background
pub const BG_ELEMENT: Color = Color::from_rgb(0x21 as f32 / 255.0, 0x26 as f32 / 255.0, 0x2d as f32 / 255.0);
/// Button hover background
pub const BG_ELEMENT_HOVER: Color = Color::from_rgb(0x30 as f32 / 255.0, 0x36 as f32 / 255.0, 0x3d as f32 / 255.0);
/// Button pressed background
pub const BG_ELEMENT_PRESSED: Color = Color::from_rgb(0x28 as f32 / 255.0, 0x2e as f32 / 255.0, 0x36 as f32 / 255.0);

// Border colors
/// Default border for cards and inputs
pub const BORDER: Color = Color::from_rgb(0x30 as f32 / 255.0, 0x36 as f32 / 255.0, 0x3d as f32 / 255.0);
/// Separator / divider line
pub const DIVIDER: Color = Color::from_rgb(0x21 as f32 / 255.0, 0x26 as f32 / 255.0, 0x2d as f32 / 255.0);

// Text colors
/// Primary text (headings, main content)
pub const TEXT_PRIMARY: Color = Color::from_rgb(0xe6 as f32 / 255.0, 0xed as f32 / 255.0, 0xf3 as f32 / 255.0);
/// Secondary text (labels, descriptions)
pub const TEXT_SECONDARY: Color = Color::from_rgb(0xc9 as f32 / 255.0, 0xd1 as f32 / 255.0, 0xd9 as f32 / 255.0);
/// Muted text (hints, placeholders)
pub const TEXT_MUTED: Color = Color::from_rgb(0x8b as f32 / 255.0, 0x94 as f32 / 255.0, 0x9e as f32 / 255.0);
/// Disabled text
pub const TEXT_DISABLED: Color = Color::from_rgb(0x48 as f32 / 255.0, 0x4f as f32 / 255.0, 0x58 as f32 / 255.0);

// Accent colors
/// Accent / brand highlight (logo, links)
pub const ACCENT: Color = Color::from_rgb(0x58 as f32 / 255.0, 0xa6 as f32 / 255.0, 0xff as f32 / 255.0);
/// Selected item highlight in menus
pub const ACCENT_MUTED: Color = Color::from_rgb(0x1f as f32 / 255.0, 0x3d as f32 / 255.0, 0x5c as f32 / 255.0);
