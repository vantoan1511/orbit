use gpui::{Rgba, rgb};

pub const BG_APP_HEX: u32 = 0x09090b;
pub const BG_SIDEBAR_HEX: u32 = 0x111114;
pub const BG_CARD_HEX: u32 = 0x18181b;
pub const BORDER_HEX: u32 = 0x27272a;
pub const BG_HOVER_HEX: u32 = 0x27272a;
pub const TEXT_PRIMARY_HEX: u32 = 0xf4f4f5;
pub const TEXT_MUTED_HEX: u32 = 0xa1a1aa;
pub const ACCENT_HEX: u32 = 0x6aa8ff;
pub const STATUS_SUCCESS_HEX: u32 = 0x22c55e;
pub const STATUS_WARNING_HEX: u32 = 0xeab308;
pub const STATUS_DANGER_HEX: u32 = 0xef4444;

pub fn bg_app() -> Rgba {
    rgb(BG_APP_HEX)
}

pub fn bg_sidebar() -> Rgba {
    rgb(BG_SIDEBAR_HEX)
}

pub fn bg_card() -> Rgba {
    rgb(BG_CARD_HEX)
}

pub fn border() -> Rgba {
    rgb(BORDER_HEX)
}

pub fn bg_hover() -> Rgba {
    rgb(BG_HOVER_HEX)
}

pub fn text_primary() -> Rgba {
    rgb(TEXT_PRIMARY_HEX)
}

pub fn text_muted() -> Rgba {
    rgb(TEXT_MUTED_HEX)
}

pub fn accent() -> Rgba {
    rgb(ACCENT_HEX)
}

pub fn status_success() -> Rgba {
    rgb(STATUS_SUCCESS_HEX)
}

pub fn status_warning() -> Rgba {
    rgb(STATUS_WARNING_HEX)
}

pub fn status_danger() -> Rgba {
    rgb(STATUS_DANGER_HEX)
}

pub fn status_color_hex(status: &str) -> u32 {
    match status {
        "Running" | "Succeeded" | "Completed" | "Ready" | "Active" => STATUS_SUCCESS_HEX,
        "Pending" | "ContainerCreating" | "Terminating" => STATUS_WARNING_HEX,
        "CrashLoopBackOff" | "Error" | "Failed" | "ImagePullBackOff" | "ErrImagePull" => {
            STATUS_DANGER_HEX
        }
        _ => TEXT_MUTED_HEX,
    }
}

pub fn status_color(status: &str) -> Rgba {
    rgb(status_color_hex(status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_constants_match_zed_dark_spec() {
        assert_eq!(BG_APP_HEX, 0x09090b);
        assert_eq!(BG_CARD_HEX, 0x18181b);
        assert_eq!(BORDER_HEX, 0x27272a);
    }

    #[test]
    fn test_status_color_mapping() {
        assert_eq!(status_color_hex("Running"), STATUS_SUCCESS_HEX);
        assert_eq!(status_color_hex("Pending"), STATUS_WARNING_HEX);
        assert_eq!(status_color_hex("CrashLoopBackOff"), STATUS_DANGER_HEX);
        assert_eq!(status_color_hex("Unknown"), TEXT_MUTED_HEX);
    }
}
