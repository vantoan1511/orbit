use gpui::{Rgba, WindowAppearance};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::Dark, ThemeMode::Light, ThemeMode::System];

    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "light" => Self::Light,
            "system" => Self::System,
            "dark" => Self::Dark,
            _ => Self::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::System => "System",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::System,
            Self::System => Self::Dark,
        }
    }

    pub fn resolve_is_dark(self, window_appearance: WindowAppearance) -> bool {
        match self {
            Self::Dark => true,
            Self::Light => false,
            Self::System => matches!(
                window_appearance,
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiStyle {
    #[default]
    Flat,
    Glass,
    Aero,
    LiquidGlass,
    Ios,
    Paper,
}

impl UiStyle {
    pub const ALL: [UiStyle; 6] = [
        UiStyle::Flat,
        UiStyle::Glass,
        UiStyle::Aero,
        UiStyle::LiquidGlass,
        UiStyle::Ios,
        UiStyle::Paper,
    ];

    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "glass" => Self::Glass,
            "aero" => Self::Aero,
            "liquidglass" => Self::LiquidGlass,
            "ios" => Self::Ios,
            "paper" => Self::Paper,
            "flat" => Self::Flat,
            _ => Self::Flat,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Glass => "glass",
            Self::Aero => "aero",
            Self::LiquidGlass => "liquidglass",
            Self::Ios => "ios",
            Self::Paper => "paper",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Flat => "Flat",
            Self::Glass => "Modern Glass",
            Self::Aero => "Frutiger Aero",
            Self::LiquidGlass => "Liquid Glass",
            Self::Ios => "iOS Cupertino",
            Self::Paper => "Editorial Paper",
        }
    }

    pub fn is_translucent(self) -> bool {
        !matches!(self, Self::Flat | Self::Paper)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentColor {
    #[default]
    Blue,
    Emerald,
    Violet,
    Cyan,
    Amber,
    Rose,
    Slate,
}

impl AccentColor {
    pub const ALL: [AccentColor; 7] = [
        AccentColor::Blue,
        AccentColor::Emerald,
        AccentColor::Violet,
        AccentColor::Cyan,
        AccentColor::Amber,
        AccentColor::Rose,
        AccentColor::Slate,
    ];

    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "emerald" => Self::Emerald,
            "violet" => Self::Violet,
            "cyan" => Self::Cyan,
            "amber" => Self::Amber,
            "rose" => Self::Rose,
            "slate" => Self::Slate,
            "blue" => Self::Blue,
            _ => Self::Blue,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRole {
    Success,
    Warning,
    Danger,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticPreset {
    #[default]
    Default,
    Colorblind,
    HighContrast,
    Custom,
}

impl SemanticPreset {
    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "colorblind" => Self::Colorblind,
            "high-contrast" => Self::HighContrast,
            "custom" => Self::Custom,
            "default" => Self::Default,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticColorsPreference {
    pub preset: SemanticPreset,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub info: String,
}

impl Default for SemanticColorsPreference {
    fn default() -> Self {
        Self {
            preset: SemanticPreset::Default,
            success: "emerald".to_string(),
            warning: "amber".to_string(),
            danger: "rose".to_string(),
            info: "blue".to_string(),
        }
    }
}

fn is_valid_role_option(role: SemanticRole, id: &str) -> bool {
    match role {
        SemanticRole::Success => matches!(id, "emerald" | "forest" | "mint" | "teal"),
        SemanticRole::Warning => matches!(id, "amber" | "orange" | "yellow" | "bronze"),
        SemanticRole::Danger => matches!(id, "rose" | "crimson" | "ruby" | "coral"),
        SemanticRole::Info => matches!(id, "blue" | "cyan" | "sky" | "indigo"),
    }
}

fn default_role_option(role: SemanticRole) -> &'static str {
    match role {
        SemanticRole::Success => "emerald",
        SemanticRole::Warning => "amber",
        SemanticRole::Danger => "rose",
        SemanticRole::Info => "blue",
    }
}

fn sanitize_role_from_json(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    role: SemanticRole,
    preset_default: &str,
) -> String {
    match obj.get(key).and_then(|v| v.as_str()) {
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            if is_valid_role_option(role, &normalized) {
                normalized
            } else {
                default_role_option(role).to_string()
            }
        }
        None => preset_default.to_string(),
    }
}

impl SemanticColorsPreference {
    pub fn from_json(value: &serde_json::Value) -> Self {
        let Some(obj) = value.as_object() else {
            return Self::default();
        };

        let preset = obj
            .get("preset")
            .and_then(|v| v.as_str())
            .map(SemanticPreset::from_str_lossy)
            .unwrap_or(SemanticPreset::Default);

        let (def_success, def_warning, def_danger, def_info) = match preset {
            SemanticPreset::Colorblind => ("teal", "amber", "crimson", "sky"),
            SemanticPreset::HighContrast => ("forest", "orange", "ruby", "indigo"),
            SemanticPreset::Default | SemanticPreset::Custom => {
                ("emerald", "amber", "rose", "blue")
            }
        };

        Self {
            preset,
            success: sanitize_role_from_json(obj, "success", SemanticRole::Success, def_success),
            warning: sanitize_role_from_json(obj, "warning", SemanticRole::Warning, def_warning),
            danger: sanitize_role_from_json(obj, "danger", SemanticRole::Danger, def_danger),
            info: sanitize_role_from_json(obj, "info", SemanticRole::Info, def_info),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum K8sResourceKind {
    Deployment,
    DaemonSet,
    StatefulSet,
    Job,
    Pod,
    ReplicaSet,
    Node,
    Secret,
    ConfigMap,
    Service,
    Ingress,
    Hpa,
}

impl K8sResourceKind {
    pub const ALL: [K8sResourceKind; 12] = [
        K8sResourceKind::Deployment,
        K8sResourceKind::DaemonSet,
        K8sResourceKind::StatefulSet,
        K8sResourceKind::Job,
        K8sResourceKind::Pod,
        K8sResourceKind::ReplicaSet,
        K8sResourceKind::Node,
        K8sResourceKind::Secret,
        K8sResourceKind::ConfigMap,
        K8sResourceKind::Service,
        K8sResourceKind::Ingress,
        K8sResourceKind::Hpa,
    ];
}

pub const fn rgba_u8(r: u8, g: u8, b: u8, a: f32) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a,
    }
}

pub const fn rgb_hex(hex: u32) -> Rgba {
    rgba_hex(hex, 1.0)
}

pub const fn rgba_hex(hex: u32, a: f32) -> Rgba {
    Rgba {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_enum_parsing_and_semantic_json_deserialization() {
        assert_eq!(ThemeMode::from_str_lossy("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str_lossy("LIGHT"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str_lossy("system"), ThemeMode::System);
        assert_eq!(ThemeMode::from_str_lossy("unknown"), ThemeMode::Dark);

        assert!(ThemeMode::Dark.resolve_is_dark(WindowAppearance::Light));
        assert!(!ThemeMode::Light.resolve_is_dark(WindowAppearance::Dark));
        assert!(ThemeMode::System.resolve_is_dark(WindowAppearance::Dark));
        assert!(ThemeMode::System.resolve_is_dark(WindowAppearance::VibrantDark));
        assert!(!ThemeMode::System.resolve_is_dark(WindowAppearance::Light));
        assert!(!ThemeMode::System.resolve_is_dark(WindowAppearance::VibrantLight));

        assert_eq!(UiStyle::from_str_lossy("flat"), UiStyle::Flat);
        assert_eq!(UiStyle::from_str_lossy("GLASS"), UiStyle::Glass);
        assert_eq!(UiStyle::from_str_lossy("aero"), UiStyle::Aero);
        assert_eq!(UiStyle::from_str_lossy("liquidglass"), UiStyle::LiquidGlass);
        assert_eq!(UiStyle::from_str_lossy("ios"), UiStyle::Ios);
        assert_eq!(UiStyle::from_str_lossy("paper"), UiStyle::Paper);
        assert_eq!(UiStyle::from_str_lossy("unknown"), UiStyle::Flat);

        assert!(!UiStyle::Flat.is_translucent());
        assert!(!UiStyle::Paper.is_translucent());
        assert!(UiStyle::Glass.is_translucent());
        assert!(UiStyle::Aero.is_translucent());
        assert!(UiStyle::LiquidGlass.is_translucent());
        assert!(UiStyle::Ios.is_translucent());

        assert_eq!(AccentColor::from_str_lossy("blue"), AccentColor::Blue);
        assert_eq!(AccentColor::from_str_lossy("emerald"), AccentColor::Emerald);
        assert_eq!(AccentColor::from_str_lossy("violet"), AccentColor::Violet);
        assert_eq!(AccentColor::from_str_lossy("cyan"), AccentColor::Cyan);
        assert_eq!(AccentColor::from_str_lossy("amber"), AccentColor::Amber);
        assert_eq!(AccentColor::from_str_lossy("rose"), AccentColor::Rose);
        assert_eq!(AccentColor::from_str_lossy("slate"), AccentColor::Slate);
        assert_eq!(AccentColor::from_str_lossy("unknown"), AccentColor::Blue);

        let colorblind = SemanticColorsPreference::from_json(&serde_json::json!({
            "preset": "colorblind"
        }));
        assert_eq!(
            colorblind,
            SemanticColorsPreference {
                preset: SemanticPreset::Colorblind,
                success: "teal".to_string(),
                warning: "amber".to_string(),
                danger: "crimson".to_string(),
                info: "sky".to_string(),
            }
        );

        let custom = SemanticColorsPreference::from_json(&serde_json::json!({
            "preset": "custom",
            "success": "mint",
            "danger": "invalid"
        }));
        assert_eq!(
            custom,
            SemanticColorsPreference {
                preset: SemanticPreset::Custom,
                success: "mint".to_string(),
                warning: "amber".to_string(),
                danger: "rose".to_string(),
                info: "blue".to_string(),
            }
        );
    }
}
