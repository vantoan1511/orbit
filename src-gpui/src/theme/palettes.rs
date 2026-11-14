use gpui::{BoxShadow, Pixels, Rgba, WindowBackgroundAppearance, point, px};

use crate::theme::tokens::{
    AccentColor, K8sResourceKind, SemanticColorsPreference, SemanticRole, UiStyle, rgb_hex,
    rgba_hex, rgba_u8,
};

#[derive(Debug, Clone)]
pub struct SurfaceTokens {
    pub bg_app: Rgba,
    pub bg_sidebar: Rgba,
    pub bg_panel: Rgba,
    pub bg_card: Rgba,
    pub bg_hover: Rgba,
    pub bg_active: Rgba,
    pub control_bg: Rgba,
    pub control_hover: Rgba,
    pub overlay_bg: Rgba,
    pub dialog_bg: Rgba,
    pub mask_bg: Rgba,
    pub input_bg: Rgba,
    pub selection: Rgba,
}

#[derive(Debug, Clone)]
pub struct BorderTokens {
    pub default: Rgba,
    pub strong: Rgba,
    pub subtle: Rgba,
    pub input: Rgba,
    pub card: Rgba,
}

#[derive(Debug, Clone)]
pub struct TextTokens {
    pub primary: Rgba,
    pub secondary: Rgba,
    pub muted: Rgba,
    pub disabled: Rgba,
    pub on_accent: Rgba,
}

#[derive(Debug, Clone)]
pub struct AccentTokens {
    pub accent: Rgba,
    pub hover: Rgba,
    pub active: Rgba,
    pub soft: Rgba,
    pub selection: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticShade {
    pub solid: Rgba,
    pub soft: Rgba,
}

#[derive(Debug, Clone)]
pub struct SemanticTokens {
    pub success: SemanticShade,
    pub warning: SemanticShade,
    pub danger: SemanticShade,
    pub info: SemanticShade,
}

#[derive(Debug, Clone)]
pub struct K8sResourceColors {
    pub deployment: Rgba,
    pub daemonset: Rgba,
    pub statefulset: Rgba,
    pub job: Rgba,
    pub pod: Rgba,
    pub replicaset: Rgba,
    pub node: Rgba,
    pub secret: Rgba,
    pub configmap: Rgba,
    pub service: Rgba,
    pub ingress: Rgba,
    pub hpa: Rgba,
}

impl K8sResourceColors {
    pub fn get(&self, kind: K8sResourceKind) -> Rgba {
        match kind {
            K8sResourceKind::Deployment => self.deployment,
            K8sResourceKind::DaemonSet => self.daemonset,
            K8sResourceKind::StatefulSet => self.statefulset,
            K8sResourceKind::Job => self.job,
            K8sResourceKind::Pod => self.pod,
            K8sResourceKind::ReplicaSet => self.replicaset,
            K8sResourceKind::Node => self.node,
            K8sResourceKind::Secret => self.secret,
            K8sResourceKind::ConfigMap => self.configmap,
            K8sResourceKind::Service => self.service,
            K8sResourceKind::Ingress => self.ingress,
            K8sResourceKind::Hpa => self.hpa,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyleTokens {
    pub ui_style: UiStyle,
    pub is_translucent: bool,
    pub window_background: WindowBackgroundAppearance,
    pub radius_card: Pixels,
    pub radius_control: Pixels,
    pub blur_px: f32,
    pub saturate_pct: u16,
    pub sheen_top: Option<Rgba>,
    pub sheen_bottom: Option<Rgba>,
    pub card_shadows: Vec<BoxShadow>,
    pub overlay_shadows: Vec<BoxShadow>,
    pub control_shadows: Vec<BoxShadow>,
}

pub struct StyleBundle {
    pub surface: SurfaceTokens,
    pub border: BorderTokens,
    pub text: TextTokens,
    pub style: StyleTokens,
}

fn box_shadow(color: Rgba, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> BoxShadow {
    BoxShadow {
        color: color.into(),
        offset: point(px(offset_x), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
    }
}

pub fn resolve_noir_tokens(is_dark: bool) -> (SurfaceTokens, BorderTokens, TextTokens) {
    if is_dark {
        (
            SurfaceTokens {
                bg_app: rgb_hex(0x0b0d0f),
                bg_sidebar: rgb_hex(0x0d1012),
                bg_panel: rgb_hex(0x0b0d0f),
                bg_card: rgb_hex(0x0e1012),
                bg_hover: rgb_hex(0x1e2228),
                bg_active: rgb_hex(0x15181c),
                control_bg: rgb_hex(0x1a1d21),
                control_hover: rgb_hex(0x1e2228),
                overlay_bg: rgb_hex(0x0e1012),
                dialog_bg: rgb_hex(0x0e1012),
                mask_bg: rgba_u8(0, 0, 0, 0.6),
                input_bg: rgb_hex(0x1a1d21),
                selection: rgba_u8(106, 168, 255, 0.18),
            },
            BorderTokens {
                default: rgb_hex(0x1f2226),
                strong: rgb_hex(0x2a2d31),
                subtle: rgb_hex(0x16181b),
                input: rgba_u8(0, 0, 0, 0.0),
                card: rgb_hex(0x1f2226),
            },
            TextTokens {
                primary: rgb_hex(0xeceef2),
                secondary: rgb_hex(0xb5bbc7),
                muted: rgb_hex(0x878d98),
                disabled: rgb_hex(0x626975),
                on_accent: rgb_hex(0x0b0d0f),
            },
        )
    } else {
        (
            SurfaceTokens {
                bg_app: rgb_hex(0xf8fafc),
                bg_sidebar: rgb_hex(0xffffff),
                bg_panel: rgb_hex(0xffffff),
                bg_card: rgb_hex(0xffffff),
                bg_hover: rgb_hex(0xe8ebef),
                bg_active: rgb_hex(0xdce1e8),
                control_bg: rgb_hex(0xffffff),
                control_hover: rgb_hex(0xe8ebef),
                overlay_bg: rgb_hex(0xffffff),
                dialog_bg: rgb_hex(0xffffff),
                mask_bg: rgba_u8(0, 0, 0, 0.4),
                input_bg: rgb_hex(0xffffff),
                selection: rgba_u8(79, 140, 255, 0.15),
            },
            BorderTokens {
                default: rgb_hex(0xd5d9e1),
                strong: rgb_hex(0xc7cdd7),
                subtle: rgb_hex(0xe6e9ef),
                input: rgba_u8(0, 0, 0, 0.0),
                card: rgb_hex(0xd5d9e1),
            },
            TextTokens {
                primary: rgb_hex(0x181a1f),
                secondary: rgb_hex(0x525866),
                muted: rgb_hex(0x7b8191),
                disabled: rgb_hex(0xa6abb8),
                on_accent: rgb_hex(0xffffff),
            },
        )
    }
}

pub fn resolve_style_bundle(
    ui_style: UiStyle,
    is_dark: bool,
    accent: &AccentTokens,
) -> StyleBundle {
    let (mut surface, mut border, mut text) = resolve_noir_tokens(is_dark);
    surface.selection = accent.selection;

    let style = match ui_style {
        UiStyle::Flat => resolve_flat_style(is_dark),
        UiStyle::Glass => resolve_glass_style(is_dark, &mut surface, &mut border),
        UiStyle::Aero => resolve_aero_style(is_dark, &mut surface, &mut border),
        UiStyle::LiquidGlass => resolve_liquid_glass_style(is_dark, &mut surface, &mut border),
        UiStyle::Ios => resolve_ios_style(is_dark, &mut surface, &mut border),
        UiStyle::Paper => resolve_paper_style(is_dark, &mut surface, &mut border, &mut text),
    };

    StyleBundle {
        surface,
        border,
        text,
        style,
    }
}

fn resolve_flat_style(is_dark: bool) -> StyleTokens {
    StyleTokens {
        ui_style: UiStyle::Flat,
        is_translucent: false,
        window_background: WindowBackgroundAppearance::Opaque,
        radius_card: px(8.0),
        radius_control: px(6.0),
        blur_px: 0.0,
        saturate_pct: 100,
        sheen_top: None,
        sheen_bottom: None,
        card_shadows: vec![],
        overlay_shadows: if is_dark {
            vec![box_shadow(rgba_u8(0, 0, 0, 0.45), 0.0, 8.0, 20.0, 0.0)]
        } else {
            vec![box_shadow(rgba_u8(0, 0, 0, 0.08), 0.0, 4.0, 12.0, 0.0)]
        },
        control_shadows: vec![],
    }
}

fn resolve_glass_style(
    is_dark: bool,
    surface: &mut SurfaceTokens,
    border: &mut BorderTokens,
) -> StyleTokens {
    if is_dark {
        surface.bg_app = rgba_u8(11, 13, 15, 0.82);
        surface.bg_sidebar = rgba_u8(13, 16, 18, 0.72);
        surface.bg_panel = rgba_u8(11, 13, 15, 0.65);
        surface.bg_card = rgba_u8(14, 16, 18, 0.65);
        surface.control_bg = rgba_u8(26, 29, 33, 0.72);
        surface.overlay_bg = rgba_u8(14, 16, 18, 0.78);
        surface.dialog_bg = rgba_u8(16, 20, 24, 0.80);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.55);
        border.default = rgba_u8(255, 255, 255, 0.08);
        border.subtle = rgba_u8(255, 255, 255, 0.04);
        border.input = rgba_u8(255, 255, 255, 0.04);
        border.card = rgba_u8(255, 255, 255, 0.12);
    } else {
        surface.bg_app = rgba_u8(248, 250, 252, 0.82);
        surface.bg_sidebar = rgba_u8(255, 255, 255, 0.72);
        surface.bg_panel = rgba_u8(248, 250, 252, 0.65);
        surface.bg_card = rgba_u8(255, 255, 255, 0.65);
        surface.control_bg = rgba_u8(255, 255, 255, 0.72);
        surface.overlay_bg = rgba_u8(255, 255, 255, 0.78);
        surface.dialog_bg = rgba_u8(255, 255, 255, 0.80);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.25);
        border.default = rgba_u8(0, 0, 0, 0.08);
        border.subtle = rgba_u8(0, 0, 0, 0.04);
        border.input = rgba_u8(0, 0, 0, 0.04);
        border.card = rgba_u8(0, 0, 0, 0.08);
    }
    let shadow_color = if is_dark {
        rgba_u8(0, 0, 0, 0.4)
    } else {
        rgba_u8(0, 0, 0, 0.08)
    };
    let shadows = vec![box_shadow(shadow_color, 0.0, 8.0, 32.0, 0.0)];
    StyleTokens {
        ui_style: UiStyle::Glass,
        is_translucent: true,
        window_background: WindowBackgroundAppearance::Blurred,
        radius_card: px(12.0),
        radius_control: px(6.0),
        blur_px: 14.0,
        saturate_pct: 160,
        sheen_top: Some(if is_dark {
            rgba_u8(255, 255, 255, 0.1)
        } else {
            rgba_u8(255, 255, 255, 0.15)
        }),
        sheen_bottom: None,
        card_shadows: shadows.clone(),
        overlay_shadows: shadows,
        control_shadows: vec![],
    }
}

fn resolve_aero_style(
    is_dark: bool,
    surface: &mut SurfaceTokens,
    border: &mut BorderTokens,
) -> StyleTokens {
    if is_dark {
        surface.bg_app = rgba_u8(11, 15, 20, 0.82);
        surface.bg_sidebar = rgba_u8(15, 20, 26, 0.82);
        surface.bg_panel = rgba_u8(11, 15, 20, 0.75);
        surface.bg_card = rgba_u8(18, 24, 32, 0.78);
        surface.control_bg = rgba_u8(24, 30, 38, 0.78);
        surface.overlay_bg = rgba_u8(18, 24, 32, 0.80);
        surface.dialog_bg = rgba_u8(20, 26, 36, 0.82);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.55);
        border.default = rgba_u8(255, 255, 255, 0.12);
        border.subtle = rgba_u8(255, 255, 255, 0.05);
        border.input = rgba_u8(255, 255, 255, 0.06);
        border.card = rgba_u8(255, 255, 255, 0.16);
    } else {
        surface.bg_app = rgba_u8(245, 248, 255, 0.82);
        surface.bg_sidebar = rgba_u8(255, 255, 255, 0.85);
        surface.bg_panel = rgba_u8(245, 248, 255, 0.75);
        surface.bg_card = rgba_u8(255, 255, 255, 0.8);
        surface.control_bg = rgba_u8(255, 255, 255, 0.82);
        surface.overlay_bg = rgba_u8(255, 255, 255, 0.82);
        surface.dialog_bg = rgba_u8(255, 255, 255, 0.85);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.25);
        border.default = rgba_u8(200, 215, 235, 0.6);
        border.subtle = rgba_u8(200, 215, 235, 0.3);
        border.input = rgba_u8(200, 215, 235, 0.4);
        border.card = rgba_u8(200, 215, 235, 0.8);
    }
    let shadows = if is_dark {
        vec![box_shadow(rgba_u8(0, 0, 0, 0.35), 0.0, 2.0, 8.0, 0.0)]
    } else {
        vec![box_shadow(rgba_u8(0, 0, 0, 0.06), 0.0, 2.0, 6.0, 0.0)]
    };
    StyleTokens {
        ui_style: UiStyle::Aero,
        is_translucent: true,
        window_background: WindowBackgroundAppearance::Blurred,
        radius_card: px(12.0),
        radius_control: px(8.0),
        blur_px: 14.0,
        saturate_pct: 150,
        sheen_top: Some(rgba_u8(255, 255, 255, 0.08)),
        sheen_bottom: Some(rgba_u8(255, 255, 255, 0.01)),
        card_shadows: shadows.clone(),
        overlay_shadows: shadows.clone(),
        control_shadows: shadows,
    }
}

fn resolve_liquid_glass_style(
    is_dark: bool,
    surface: &mut SurfaceTokens,
    border: &mut BorderTokens,
) -> StyleTokens {
    if is_dark {
        surface.bg_app = rgba_u8(8, 10, 14, 0.82);
        surface.bg_sidebar = rgba_u8(10, 13, 18, 0.75);
        surface.bg_panel = rgba_u8(8, 10, 14, 0.70);
        surface.bg_card = rgba_u8(15, 20, 28, 0.68);
        surface.control_bg = rgba_u8(20, 26, 36, 0.68);
        surface.overlay_bg = rgba_u8(15, 20, 28, 0.76);
        surface.dialog_bg = rgba_u8(18, 24, 34, 0.78);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.60);
        border.default = rgba_u8(255, 255, 255, 0.12);
        border.subtle = rgba_u8(255, 255, 255, 0.05);
        border.input = rgba_u8(255, 255, 255, 0.06);
        border.card = rgba_u8(255, 255, 255, 0.20);
    } else {
        surface.bg_app = rgba_u8(240, 244, 250, 0.78);
        surface.bg_sidebar = rgba_u8(255, 255, 255, 0.68);
        surface.bg_panel = rgba_u8(240, 244, 250, 0.60);
        surface.bg_card = rgba_u8(255, 255, 255, 0.58);
        surface.control_bg = rgba_u8(255, 255, 255, 0.62);
        surface.overlay_bg = rgba_u8(255, 255, 255, 0.72);
        surface.dialog_bg = rgba_u8(255, 255, 255, 0.74);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.30);
        border.default = rgba_u8(255, 255, 255, 0.35);
        border.subtle = rgba_u8(255, 255, 255, 0.18);
        border.input = rgba_u8(255, 255, 255, 0.20);
        border.card = rgba_u8(255, 255, 255, 0.50);
    }
    let shadows = if is_dark {
        vec![box_shadow(rgba_u8(0, 0, 0, 0.45), 0.0, 8.0, 32.0, 0.0)]
    } else {
        vec![box_shadow(rgba_u8(31, 38, 135, 0.12), 0.0, 8.0, 32.0, 0.0)]
    };
    StyleTokens {
        ui_style: UiStyle::LiquidGlass,
        is_translucent: true,
        window_background: WindowBackgroundAppearance::Blurred,
        radius_card: px(12.0),
        radius_control: px(8.0),
        blur_px: 20.0,
        saturate_pct: 180,
        sheen_top: Some(rgba_u8(255, 255, 255, 0.1)),
        sheen_bottom: Some(rgba_u8(255, 255, 255, 0.02)),
        card_shadows: shadows.clone(),
        overlay_shadows: shadows,
        control_shadows: vec![],
    }
}

fn resolve_ios_style(
    is_dark: bool,
    surface: &mut SurfaceTokens,
    border: &mut BorderTokens,
) -> StyleTokens {
    if is_dark {
        surface.bg_app = rgba_u8(18, 18, 18, 0.85);
        surface.bg_sidebar = rgba_u8(28, 28, 30, 0.82);
        surface.bg_panel = rgba_u8(18, 18, 18, 0.82);
        surface.bg_card = rgba_u8(30, 30, 32, 0.75);
        surface.control_bg = rgba_u8(44, 44, 46, 0.82);
        surface.overlay_bg = rgba_u8(30, 30, 32, 0.80);
        surface.dialog_bg = rgba_u8(32, 32, 36, 0.82);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.55);
        border.default = rgba_u8(255, 255, 255, 0.08);
        border.subtle = rgba_u8(255, 255, 255, 0.04);
        border.input = rgba_u8(255, 255, 255, 0.04);
        border.card = rgba_u8(255, 255, 255, 0.10);
    } else {
        surface.bg_app = rgba_u8(245, 245, 247, 0.85);
        surface.bg_sidebar = rgba_u8(245, 245, 247, 0.82);
        surface.bg_panel = rgba_u8(255, 255, 255, 0.85);
        surface.bg_card = rgba_u8(255, 255, 255, 0.75);
        surface.control_bg = rgba_u8(242, 242, 247, 0.82);
        surface.overlay_bg = rgba_u8(255, 255, 255, 0.80);
        surface.dialog_bg = rgba_u8(255, 255, 255, 0.82);
        surface.mask_bg = rgba_u8(0, 0, 0, 0.25);
        border.default = rgba_u8(0, 0, 0, 0.08);
        border.subtle = rgba_u8(0, 0, 0, 0.04);
        border.input = rgba_u8(0, 0, 0, 0.04);
        border.card = rgba_u8(0, 0, 0, 0.08);
    }
    let shadows = if is_dark {
        vec![
            box_shadow(rgba_u8(0, 0, 0, 0.5), 0.0, 10.0, 25.0, -5.0),
            box_shadow(rgba_u8(0, 0, 0, 0.5), 0.0, 8.0, 10.0, -6.0),
        ]
    } else {
        vec![
            box_shadow(rgba_u8(0, 0, 0, 0.08), 0.0, 10.0, 25.0, -5.0),
            box_shadow(rgba_u8(0, 0, 0, 0.04), 0.0, 8.0, 10.0, -6.0),
        ]
    };
    StyleTokens {
        ui_style: UiStyle::Ios,
        is_translucent: true,
        window_background: WindowBackgroundAppearance::Blurred,
        radius_card: px(16.0),
        radius_control: px(10.0),
        blur_px: 24.0,
        saturate_pct: 190,
        sheen_top: None,
        sheen_bottom: None,
        card_shadows: shadows.clone(),
        overlay_shadows: shadows,
        control_shadows: vec![],
    }
}

fn resolve_paper_style(
    is_dark: bool,
    surface: &mut SurfaceTokens,
    border: &mut BorderTokens,
    text: &mut TextTokens,
) -> StyleTokens {
    if is_dark {
        surface.bg_app = rgb_hex(0x141311);
        surface.bg_sidebar = rgb_hex(0x181614);
        surface.bg_panel = rgb_hex(0x161513);
        surface.bg_card = rgb_hex(0x1c1a17);
        surface.bg_hover = rgb_hex(0x292521);
        surface.bg_active = rgb_hex(0x221f1b);
        surface.input_bg = rgb_hex(0x24211d);
        surface.control_bg = rgb_hex(0x24211d);
        surface.control_hover = rgb_hex(0x292521);
        surface.overlay_bg = rgb_hex(0x1c1a17);
        surface.dialog_bg = rgb_hex(0x1f1d1a);
        surface.mask_bg = rgba_u8(10, 9, 8, 0.65);
        border.default = rgb_hex(0x2e2a25);
        border.strong = rgb_hex(0x3d3831);
        border.subtle = rgb_hex(0x24211d);
        border.input = rgb_hex(0x332e29);
        border.card = rgb_hex(0x2e2a25);
        text.primary = rgb_hex(0xf0ece1);
        text.secondary = rgb_hex(0xb8b2a6);
        text.muted = rgb_hex(0x8a8377);
        text.disabled = rgb_hex(0x635d53);
    } else {
        surface.bg_app = rgb_hex(0xf6f3ec);
        surface.bg_sidebar = rgb_hex(0xf0ece1);
        surface.bg_panel = rgb_hex(0xf9f6f0);
        surface.bg_card = rgb_hex(0xfcfaf5);
        surface.bg_hover = rgb_hex(0xe8e2d5);
        surface.bg_active = rgb_hex(0xded7c8);
        surface.input_bg = rgb_hex(0xfffdf9);
        surface.control_bg = rgb_hex(0xfffdf9);
        surface.control_hover = rgb_hex(0xe8e2d5);
        surface.overlay_bg = rgb_hex(0xfcfaf5);
        surface.dialog_bg = rgb_hex(0xfcfaf5);
        surface.mask_bg = rgba_u8(28, 25, 23, 0.35);
        border.default = rgb_hex(0xdcd5c7);
        border.strong = rgb_hex(0xc7beae);
        border.subtle = rgb_hex(0xe8e2d5);
        border.input = rgb_hex(0xdcd5c7);
        border.card = rgb_hex(0xdcd5c7);
        text.primary = rgb_hex(0x1c1917);
        text.secondary = rgb_hex(0x57534e);
        text.muted = rgb_hex(0x78716c);
        text.disabled = rgb_hex(0xa8a29e);
    }
    let shadows = if is_dark {
        vec![
            box_shadow(rgba_u8(0, 0, 0, 0.45), 0.0, 1.0, 3.0, 0.0),
            box_shadow(rgba_u8(0, 0, 0, 0.35), 0.0, 1.0, 2.0, 0.0),
        ]
    } else {
        vec![
            box_shadow(rgba_u8(28, 25, 23, 0.06), 0.0, 1.0, 3.0, 0.0),
            box_shadow(rgba_u8(28, 25, 23, 0.04), 0.0, 1.0, 2.0, 0.0),
        ]
    };
    StyleTokens {
        ui_style: UiStyle::Paper,
        is_translucent: false,
        window_background: WindowBackgroundAppearance::Opaque,
        radius_card: px(4.0),
        radius_control: px(3.0),
        blur_px: 0.0,
        saturate_pct: 100,
        sheen_top: Some(if is_dark {
            rgba_u8(255, 248, 235, 0.03)
        } else {
            rgba_u8(255, 253, 248, 0.65)
        }),
        sheen_bottom: Some(if is_dark {
            rgba_u8(255, 248, 235, 0.005)
        } else {
            rgba_u8(244, 239, 228, 0.15)
        }),
        card_shadows: shadows.clone(),
        overlay_shadows: shadows,
        control_shadows: vec![],
    }
}

pub fn resolve_accent_tokens(accent: AccentColor, is_dark: bool) -> AccentTokens {
    let (accent_hex, hover_hex, active_hex) = match (accent, is_dark) {
        (AccentColor::Blue, false) => (0x4f8cff, 0x6ca2ff, 0x3c78e8),
        (AccentColor::Blue, true) => (0x6aa8ff, 0x84b8ff, 0x4e95ff),
        (AccentColor::Emerald, false) => (0x10b981, 0x34d399, 0x059669),
        (AccentColor::Emerald, true) => (0x34d399, 0x6ee7b7, 0x10b981),
        (AccentColor::Violet, false) => (0x8b5cf6, 0xa78bfa, 0x7c3aed),
        (AccentColor::Violet, true) => (0xa78bfa, 0xc4b5fd, 0x8b5cf6),
        (AccentColor::Cyan, false) => (0x06b6d4, 0x22d3ee, 0x0891b2),
        (AccentColor::Cyan, true) => (0x22d3ee, 0x67e8f9, 0x06b6d4),
        (AccentColor::Amber, false) => (0xf59e0b, 0xfbbf24, 0xd97706),
        (AccentColor::Amber, true) => (0xfbbf24, 0xfcd34d, 0xf59e0b),
        (AccentColor::Rose, false) => (0xf43f5e, 0xfb7185, 0xe11d48),
        (AccentColor::Rose, true) => (0xfb7185, 0xfda4af, 0xf43f5e),
        (AccentColor::Slate, false) => (0x475569, 0x64748b, 0x334155),
        (AccentColor::Slate, true) => (0x94a3b8, 0xcbd5e1, 0x64748b),
    };

    AccentTokens {
        accent: rgb_hex(accent_hex),
        hover: rgb_hex(hover_hex),
        active: rgb_hex(active_hex),
        soft: rgba_hex(accent_hex, if is_dark { 0.14 } else { 0.12 }),
        selection: rgba_hex(accent_hex, if is_dark { 0.18 } else { 0.15 }),
    }
}

fn resolve_role_shade(role: SemanticRole, option_id: &str, is_dark: bool) -> SemanticShade {
    let solid_hex = match (role, option_id, is_dark) {
        (SemanticRole::Success, "forest", false) => 0x15803d,
        (SemanticRole::Success, "forest", true) => 0x22c55e,
        (SemanticRole::Success, "mint", false) => 0x059669,
        (SemanticRole::Success, "mint", true) => 0x10b981,
        (SemanticRole::Success, "teal", false) => 0x0f766e,
        (SemanticRole::Success, "teal", true) => 0x14b8a6,
        (SemanticRole::Success, _, false) => 0x28a745,
        (SemanticRole::Success, _, true) => 0x46d16e,

        (SemanticRole::Warning, "orange", false) => 0xea580c,
        (SemanticRole::Warning, "orange", true) => 0xfb923c,
        (SemanticRole::Warning, "yellow", false) => 0xca8a04,
        (SemanticRole::Warning, "yellow", true) => 0xfacc15,
        (SemanticRole::Warning, "bronze", false) => 0xb45309,
        (SemanticRole::Warning, "bronze", true) => 0xf59e0b,
        (SemanticRole::Warning, _, false) => 0xf4a100,
        (SemanticRole::Warning, _, true) => 0xffc54d,

        (SemanticRole::Danger, "crimson", false) => 0xdc2626,
        (SemanticRole::Danger, "crimson", true) => 0xf87171,
        (SemanticRole::Danger, "ruby", false) => 0xe11d48,
        (SemanticRole::Danger, "ruby", true) => 0xfb7185,
        (SemanticRole::Danger, "coral", false) => 0xbe123c,
        (SemanticRole::Danger, "coral", true) => 0xf43f5e,
        (SemanticRole::Danger, _, false) => 0xd64545,
        (SemanticRole::Danger, _, true) => 0xff6b6b,

        (SemanticRole::Info, "cyan", false) => 0x0891b2,
        (SemanticRole::Info, "cyan", true) => 0x22d3ee,
        (SemanticRole::Info, "sky", false) => 0x0284c7,
        (SemanticRole::Info, "sky", true) => 0x38bdf8,
        (SemanticRole::Info, "indigo", false) => 0x4f46e5,
        (SemanticRole::Info, "indigo", true) => 0x818cf8,
        (SemanticRole::Info, _, false) => 0x3d8bff,
        (SemanticRole::Info, _, true) => 0x6aa8ff,
    };

    SemanticShade {
        solid: rgb_hex(solid_hex),
        soft: rgba_hex(solid_hex, if is_dark { 0.14 } else { 0.12 }),
    }
}

pub fn resolve_semantic_tokens(
    pref: &SemanticColorsPreference,
    is_dark: bool,
) -> SemanticTokens {
    SemanticTokens {
        success: resolve_role_shade(SemanticRole::Success, &pref.success, is_dark),
        warning: resolve_role_shade(SemanticRole::Warning, &pref.warning, is_dark),
        danger: resolve_role_shade(SemanticRole::Danger, &pref.danger, is_dark),
        info: resolve_role_shade(SemanticRole::Info, &pref.info, is_dark),
    }
}

pub fn resolve_k8s_colors(is_dark: bool) -> K8sResourceColors {
    if is_dark {
        K8sResourceColors {
            deployment: rgb_hex(0x60a5fa),
            daemonset: rgb_hex(0xfb923c),
            statefulset: rgb_hex(0x22d3ee),
            job: rgb_hex(0xc084fc),
            pod: rgb_hex(0x34d399),
            replicaset: rgb_hex(0xa78bfa),
            node: rgb_hex(0x818cf8),
            secret: rgb_hex(0xf472b6),
            configmap: rgb_hex(0xfacc15),
            service: rgb_hex(0x2dd4bf),
            ingress: rgb_hex(0xfb7185),
            hpa: rgb_hex(0x38bdf8),
        }
    } else {
        K8sResourceColors {
            deployment: rgb_hex(0x3b82f6),
            daemonset: rgb_hex(0xf97316),
            statefulset: rgb_hex(0x06b6d4),
            job: rgb_hex(0xa855f7),
            pod: rgb_hex(0x10b981),
            replicaset: rgb_hex(0x8b5cf6),
            node: rgb_hex(0x6366f1),
            secret: rgb_hex(0xec4899),
            configmap: rgb_hex(0xeab308),
            service: rgb_hex(0x14b8a6),
            ingress: rgb_hex(0xf43f5e),
            hpa: rgb_hex(0x0284c7),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::SemanticPreset;

    #[test]
    fn test_all_six_ui_styles_radii_translucency_and_palette_parity() {
        let expected_radii = [
            (UiStyle::Flat, 8.0, 6.0, WindowBackgroundAppearance::Opaque),
            (UiStyle::Glass, 12.0, 6.0, WindowBackgroundAppearance::Blurred),
            (UiStyle::Aero, 12.0, 8.0, WindowBackgroundAppearance::Blurred),
            (UiStyle::LiquidGlass, 12.0, 8.0, WindowBackgroundAppearance::Blurred),
            (UiStyle::Ios, 16.0, 10.0, WindowBackgroundAppearance::Blurred),
            (UiStyle::Paper, 4.0, 3.0, WindowBackgroundAppearance::Opaque),
        ];

        for is_dark in [false, true] {
            let accent = resolve_accent_tokens(AccentColor::Blue, is_dark);
            for (style, card_r, control_r, win_bg) in expected_radii {
                let bundle = resolve_style_bundle(style, is_dark, &accent);
                assert_eq!(bundle.style.ui_style, style);
                assert_eq!(bundle.style.radius_card, px(card_r));
                assert_eq!(bundle.style.radius_control, px(control_r));
                assert_eq!(bundle.style.window_background, win_bg);
                assert_eq!(bundle.style.is_translucent, style.is_translucent());
            }
        }

        let (light_noir, _, _) = resolve_noir_tokens(false);
        assert_eq!(light_noir.bg_app, rgb_hex(0xf8fafc));
        let (dark_noir, _, _) = resolve_noir_tokens(true);
        assert_eq!(dark_noir.bg_app, rgb_hex(0x0b0d0f));

        let light_accent = resolve_accent_tokens(AccentColor::Blue, false);
        let dark_accent = resolve_accent_tokens(AccentColor::Blue, true);
        let paper_light = resolve_style_bundle(UiStyle::Paper, false, &light_accent);
        assert_eq!(paper_light.surface.bg_app, rgb_hex(0xf6f3ec));
        let paper_dark = resolve_style_bundle(UiStyle::Paper, true, &dark_accent);
        assert_eq!(paper_dark.surface.bg_app, rgb_hex(0x141311));

        for accent_kind in AccentColor::ALL {
            let light = resolve_accent_tokens(accent_kind, false);
            let dark = resolve_accent_tokens(accent_kind, true);
            assert_ne!(light.accent, dark.accent);
        }

        let colorblind = SemanticColorsPreference {
            preset: SemanticPreset::Colorblind,
            success: "teal".to_string(),
            warning: "amber".to_string(),
            danger: "crimson".to_string(),
            info: "sky".to_string(),
        };
        let sem_dark = resolve_semantic_tokens(&colorblind, true);
        assert_eq!(sem_dark.success.solid, rgb_hex(0x14b8a6));
        assert_eq!(sem_dark.danger.solid, rgb_hex(0xf87171));

        let k8s_light = resolve_k8s_colors(false);
        let k8s_dark = resolve_k8s_colors(true);
        assert_eq!(k8s_light.deployment, rgb_hex(0x3b82f6));
        assert_eq!(k8s_dark.deployment, rgb_hex(0x60a5fa));
        for kind in K8sResourceKind::ALL {
            assert_ne!(k8s_light.get(kind), k8s_dark.get(kind));
        }
    }
}
