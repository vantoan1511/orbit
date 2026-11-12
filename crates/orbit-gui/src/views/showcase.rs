#![cfg(debug_assertions)]

use iced::{
    widget::{
        button, column, container, horizontal_space, pick_list, row, scrollable, text,
        text_input, toggler,
    },
    Alignment, Color, Element, Length,
};
use orbit_core::command::ResourceKind;

use crate::components::{
    badge::{badge, resource_kind_badge, status_dot_pill, status_pill, BadgeSize, BadgeVariant},
    button::{
        custom_button, destructive_button, ghost_button, icon_button, outline_button,
        primary_button, secondary_button, ButtonSize, ButtonVariant,
    },
    card::{alert_banner, AlertLevel},
};
use crate::icons::{icon, Icon};
use crate::theme::{
    accent_color_from_name, color_to_hex, ghost_action_button_style, pick_list_style,
    text_input_style, toggler_style, OrbitPalette,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowcaseSection {
    All,
    Tokens,
    Buttons,
    Forms,
    Badges,
    Typography,
    Alerts,
}

impl ShowcaseSection {
    pub const ALL: &'static [Self] = &[
        Self::All,
        Self::Tokens,
        Self::Buttons,
        Self::Forms,
        Self::Badges,
        Self::Typography,
        Self::Alerts,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Tokens => "Tokens",
            Self::Buttons => "Buttons",
            Self::Forms => "Forms",
            Self::Badges => "Badges",
            Self::Typography => "Typography",
            Self::Alerts => "Alerts",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowcaseState {
    pub active_section: ShowcaseSection,
    pub sample_toggle_1: bool,
    pub sample_toggle_2: bool,
    pub sample_counter: i32,
    pub sample_text_input: String,
    pub sample_search_input: String,
    pub sample_selected_option: String,
    pub sample_selected_segment: String,
    pub sample_options: Vec<String>,
    pub active_accent: String,
    pub last_action: Option<String>,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            active_section: ShowcaseSection::All,
            sample_toggle_1: true,
            sample_toggle_2: false,
            sample_counter: 5,
            sample_text_input: "cluster-admin".to_string(),
            sample_search_input: String::new(),
            sample_selected_option: "production".to_string(),
            sample_selected_segment: "Overview".to_string(),
            sample_options: vec![
                "production".to_string(),
                "staging".to_string(),
                "development".to_string(),
            ],
            active_accent: "blue".to_string(),
            last_action: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShowcaseMessage {
    SelectSection(ShowcaseSection),
    SetAccentColor(String),
    ToggleTheme,
    ToggleSample1(bool),
    ToggleSample2(bool),
    IncrementCounter,
    DecrementCounter,
    TextInputChanged(String),
    SearchInputChanged(String),
    ClearSearchInput,
    OptionSelected(String),
    SegmentSelected(String),
    TriggerAction(String),
}

impl ShowcaseState {
    pub fn update(&mut self, msg: ShowcaseMessage) {
        match msg {
            ShowcaseMessage::SelectSection(sec) => self.active_section = sec,
            ShowcaseMessage::SetAccentColor(accent) => self.active_accent = accent,
            ShowcaseMessage::ToggleTheme => {}
            ShowcaseMessage::ToggleSample1(val) => self.sample_toggle_1 = val,
            ShowcaseMessage::ToggleSample2(val) => self.sample_toggle_2 = val,
            ShowcaseMessage::IncrementCounter => {
                self.sample_counter = self.sample_counter.saturating_add(1);
            }
            ShowcaseMessage::DecrementCounter => {
                self.sample_counter = self.sample_counter.saturating_sub(1);
            }
            ShowcaseMessage::TextInputChanged(text) => self.sample_text_input = text,
            ShowcaseMessage::SearchInputChanged(text) => self.sample_search_input = text,
            ShowcaseMessage::ClearSearchInput => self.sample_search_input.clear(),
            ShowcaseMessage::OptionSelected(opt) => self.sample_selected_option = opt,
            ShowcaseMessage::SegmentSelected(seg) => self.sample_selected_segment = seg,
            ShowcaseMessage::TriggerAction(act) => self.last_action = Some(act),
        }
    }
}

fn preview_box_card<'a, Message: 'a>(
    title: &'static str,
    subtitle: &'static str,
    content: impl Into<Element<'a, Message>>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let header = column![
        text(title).size(14).color(palette.text_primary),
        text(subtitle).size(12).color(palette.text_muted),
    ]
    .spacing(2);

    let divider = container(horizontal_space()).height(1).style(move |_| container::Style {
        background: Some(iced::Background::Color(palette.border)),
        ..Default::default()
    });

    let card_content = column![header, divider, content.into()]
        .spacing(12)
        .width(Length::Fill);

    container(card_content)
        .padding(16)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(palette.bg_panel)),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: iced::border::Radius::from(6.0),
            },
            ..Default::default()
        })
        .into()
}

fn token_chip<'a, Message: 'a>(
    name: &'static str,
    color: Color,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let hex = color_to_hex(color);
    let swatch = container(horizontal_space())
        .width(16)
        .height(16)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: iced::border::Radius::from(3.0),
            },
            ..Default::default()
        });

    row![
        swatch,
        text(name).size(12).color(palette.text_primary),
        horizontal_space(),
        text(hex).size(11).color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

pub fn showcase_view<'a, Message: 'a + Clone>(
    state: &'a ShowcaseState,
    palette: &'a OrbitPalette,
    on_showcase_msg: impl Fn(ShowcaseMessage) -> Message + 'a + Copy,
    on_theme_toggle: Message,
) -> Element<'a, Message> {
    // 1. Header Toolbar
    let mut header_actions = row![].spacing(8).align_y(Alignment::Center);

    if let Some(ref act) = state.last_action {
        let action_badge: Element<'_, Message> = badge(
            format!("Last Action: {}", act),
            BadgeVariant::Info,
            BadgeSize::Small,
            true,
            palette,
        );
        header_actions = header_actions.push(action_badge);
    }

    let header_bar = row![
        column![
            text("UI Component Library & Design System Catalog")
                .size(20)
                .color(palette.text_primary),
            text("Internal design tokens, atomic primitives, and interactive component showcase")
                .size(12)
                .color(palette.text_muted),
        ]
        .spacing(2),
        horizontal_space(),
        header_actions,
    ]
    .align_y(Alignment::Center);

    // 2. Interactive Live Playground Bar
    let theme_icon = if palette.is_dark { Icon::Sun } else { Icon::Moon };
    let theme_btn = button(
        row![
            icon(theme_icon, 14, palette.text_primary),
            text(if palette.is_dark { "Light Mode" } else { "Dark Mode" })
                .size(12)
                .color(palette.text_primary),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .style(ghost_action_button_style(palette))
    .padding([6, 12])
    .on_press(on_theme_toggle);

    let accent_names = ["blue", "emerald", "violet", "rose", "amber"];
    let mut swatches_row = row![].spacing(6).align_y(Alignment::Center);

    for name in accent_names {
        let is_active = state.active_accent.eq_ignore_ascii_case(name);
        let accent_color = accent_color_from_name(name, palette.is_dark);
        let name_owned = name.to_string();

        let inner_dot = container(horizontal_space())
            .width(16)
            .height(16)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(accent_color)),
                border: iced::Border {
                    color: if is_active { palette.text_primary } else { Color::TRANSPARENT },
                    width: if is_active { 2.0 } else { 0.0 },
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            });

        let swatch_btn = button(inner_dot)
            .padding(2)
            .style(move |_theme, _status| button::Style {
                background: None,
                border: iced::Border {
                    color: if is_active { palette.text_primary } else { palette.border },
                    width: 1.0,
                    radius: iced::border::Radius::from(10.0),
                },
                ..Default::default()
            })
            .on_press(on_showcase_msg(ShowcaseMessage::SetAccentColor(name_owned)));

        swatches_row = swatches_row.push(swatch_btn);
    }

    let playground_bar = container(
        row![
            row![
                text("Appearance:").size(12).color(palette.text_muted),
                theme_btn,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            horizontal_space(),
            row![
                text("Accent Swatches:").size(12).color(palette.text_muted),
                swatches_row,
                badge(
                    format!("Active: {}", state.active_accent),
                    BadgeVariant::Info,
                    BadgeSize::Small,
                    false,
                    palette,
                ),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding(12)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(palette.bg_card)),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: iced::border::Radius::from(6.0),
        },
        ..Default::default()
    });

    // 3. Section Filter Navigation Pills
    let mut filter_row = row![].spacing(6).align_y(Alignment::Center);
    for sec in ShowcaseSection::ALL {
        let is_active = state.active_section == *sec;
        let btn = custom_button(
            text(sec.label()).size(12),
            if is_active { ButtonVariant::Primary } else { ButtonVariant::Ghost },
            ButtonSize::Small,
            Some(on_showcase_msg(ShowcaseMessage::SelectSection(*sec))),
            palette,
        );
        filter_row = filter_row.push(btn);
    }

    // 4. Component Cards Definition

    // Card 1: Tokens (Surfaces & Semantic Colors)
    let tokens_content = column![
        text("Surfaces").size(12).color(palette.text_secondary),
        token_chip("bg_app", palette.bg_app, palette),
        token_chip("bg_sidebar", palette.bg_sidebar, palette),
        token_chip("bg_panel", palette.bg_panel, palette),
        token_chip("bg_card", palette.bg_card, palette),
        token_chip("bg_hover", palette.bg_hover, palette),
        token_chip("border", palette.border, palette),
        text("Semantic Colors").size(12).color(palette.text_secondary),
        token_chip("accent", palette.accent, palette),
        token_chip("success", palette.success, palette),
        token_chip("warning", palette.warning, palette),
        token_chip("danger", palette.danger, palette),
        token_chip("info", palette.info, palette),
    ]
    .spacing(8)
    .width(Length::Fill);

    let tokens_card = preview_box_card(
        "Tokens & Surfaces",
        "Noir monochrome surface hierarchy and semantic color palette",
        tokens_content,
        palette,
    );

    // Card 2: Buttons Matrix
    let variants_row = row![
        primary_button(
            "Primary",
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("primary".into()))),
            palette,
        ),
        secondary_button(
            "Secondary",
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("secondary".into()))),
            palette,
        ),
        outline_button(
            "Outline",
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("outline".into()))),
            palette,
        ),
        ghost_button(
            "Ghost",
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("ghost".into()))),
            palette,
        ),
        destructive_button(
            "Destructive",
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("destructive".into()))),
            palette,
        ),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let with_icons_row = row![
        custom_button(
            row![
                icon(Icon::RotateCcw, 13, palette.text_primary),
                text("Revert").size(ButtonSize::Medium.text_size()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            ButtonVariant::Secondary,
            ButtonSize::Medium,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("revert_icon".into()))),
            palette,
        ),
        custom_button(
            row![
                icon(Icon::Plus, 13, palette.text_primary),
                text("Add Resource").size(ButtonSize::Medium.text_size()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            ButtonVariant::Outline,
            ButtonSize::Medium,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("add_icon".into()))),
            palette,
        ),
        icon_button(
            icon(Icon::Search, 14, palette.text_secondary),
            ButtonSize::Medium,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("search_icon".into()))),
            palette,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let sizes_row = row![
        custom_button(
            text("Small (24px)").size(ButtonSize::Small.text_size()),
            ButtonVariant::Secondary,
            ButtonSize::Small,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("small".into()))),
            palette,
        ),
        custom_button(
            text("Medium (32px)").size(ButtonSize::Medium.text_size()),
            ButtonVariant::Secondary,
            ButtonSize::Medium,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("medium".into()))),
            palette,
        ),
        custom_button(
            text("Large (38px)").size(ButtonSize::Large.text_size()),
            ButtonVariant::Secondary,
            ButtonSize::Large,
            Some(on_showcase_msg(ShowcaseMessage::TriggerAction("large".into()))),
            palette,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let disabled_row = row![
        primary_button("Disabled", None, palette),
        secondary_button("Disabled", None, palette),
        outline_button("Disabled", None, palette),
        ghost_button("Disabled", None, palette),
        destructive_button("Disabled", None, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let buttons_content = column![
        text("Variants").size(12).color(palette.text_secondary),
        variants_row,
        text("With Icons").size(12).color(palette.text_secondary),
        with_icons_row,
        text("Size Scale").size(12).color(palette.text_secondary),
        sizes_row,
        text("Disabled States").size(12).color(palette.text_secondary),
        disabled_row,
    ]
    .spacing(10)
    .width(Length::Fill);

    let buttons_card = preview_box_card(
        "Buttons & Interactive Actions",
        "Primary, secondary, outline, ghost, destructive, and icon button controls",
        buttons_content,
        palette,
    );

    // Card 3: Inputs & Form Controls
    let mut search_row = row![
        icon(Icon::Search, 12, palette.text_muted),
        text_input("Search catalog or cluster...", &state.sample_search_input)
            .on_input(move |text| on_showcase_msg(ShowcaseMessage::SearchInputChanged(text)))
            .style(text_input_style(palette))
            .size(12)
            .padding([4, 6])
            .width(Length::Fill),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    if !state.sample_search_input.is_empty() {
        let clear_btn = button(icon(Icon::X, 12, palette.text_muted))
            .padding([2, 4])
            .style(ghost_action_button_style(palette))
            .on_press(on_showcase_msg(ShowcaseMessage::ClearSearchInput));
        search_row = search_row.push(clear_btn);
    }

    let search_container = container(search_row)
        .padding([2, 8])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(palette.bg_card)),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: iced::border::Radius::from(4.0),
            },
            ..Default::default()
        });

    let input_field = text_input("Enter namespace or role...", &state.sample_text_input)
        .on_input(move |text| on_showcase_msg(ShowcaseMessage::TextInputChanged(text)))
        .style(text_input_style(palette))
        .size(12)
        .padding([5, 8])
        .width(Length::Fill);

    let stepper = row![
        custom_button(
            text("-").size(12),
            ButtonVariant::Secondary,
            ButtonSize::Small,
            Some(on_showcase_msg(ShowcaseMessage::DecrementCounter)),
            palette,
        ),
        container(
            text(state.sample_counter.to_string())
                .size(12)
                .color(palette.text_primary),
        )
        .padding([3, 10]),
        custom_button(
            text("+").size(12),
            ButtonVariant::Secondary,
            ButtonSize::Small,
            Some(on_showcase_msg(ShowcaseMessage::IncrementCounter)),
            palette,
        ),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let tog1 = toggler(state.sample_toggle_1)
        .on_toggle(move |val| on_showcase_msg(ShowcaseMessage::ToggleSample1(val)))
        .style(toggler_style(palette))
        .size(18);

    let tog2 = toggler(state.sample_toggle_2)
        .on_toggle(move |val| on_showcase_msg(ShowcaseMessage::ToggleSample2(val)))
        .style(toggler_style(palette))
        .size(18);

    let selected_opt = state.sample_selected_option.clone();
    let picker = pick_list(
        state.sample_options.as_slice(),
        Some(selected_opt),
        move |opt| on_showcase_msg(ShowcaseMessage::OptionSelected(opt.to_string())),
    )
    .style(pick_list_style(palette))
    .text_size(12)
    .padding([4, 8])
    .width(Length::Fixed(160.0));

    // Segmented tab control
    let segments = ["Overview", "Workloads", "Metrics"];
    let mut segment_row = row![].spacing(2);
    for seg in segments {
        let is_selected = state.sample_selected_segment == seg;
        let seg_str = seg.to_string();
        let btn = button(
            text(seg)
                .size(11)
                .color(if is_selected { Color::WHITE } else { palette.text_secondary }),
        )
        .padding([4, 10])
        .style(move |_theme, _status| button::Style {
            background: if is_selected {
                Some(iced::Background::Color(palette.accent))
            } else {
                None
            },
            border: iced::Border {
                radius: iced::border::Radius::from(4.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press(on_showcase_msg(ShowcaseMessage::SegmentSelected(seg_str)));
        segment_row = segment_row.push(btn);
    }

    let segmented_control = container(segment_row)
        .padding(2)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(palette.bg_hover)),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: iced::border::Radius::from(6.0),
            },
            ..Default::default()
        });

    let forms_content = column![
        text("Search Box with Clear Button").size(12).color(palette.text_secondary),
        search_container,
        text("Text Input with Focus Ring").size(12).color(palette.text_secondary),
        input_field,
        row![
            column![
                text("Stepper").size(12).color(palette.text_secondary),
                stepper,
            ]
            .spacing(4),
            horizontal_space(),
            column![
                text("Dropdown Picker").size(12).color(palette.text_secondary),
                picker,
            ]
            .spacing(4),
        ]
        .align_y(Alignment::Center),
        text("Segmented Tab Selector").size(12).color(palette.text_secondary),
        segmented_control,
        row![
            row![tog1, text("Telemetry Stream").size(12).color(palette.text_primary)]
                .spacing(8)
                .align_y(Alignment::Center),
            horizontal_space(),
            row![tog2, text("Compact Density").size(12).color(palette.text_primary)]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(12)
    .width(Length::Fill);

    let forms_card = preview_box_card(
        "Interactive Inputs & Form Controls",
        "Search input, text fields, stepper counter, togglers, and segmented tabs",
        forms_content,
        palette,
    );

    // Card 4: Badges & Kubernetes Statuses
    let badge_variants_row = row![
        badge("Success", BadgeVariant::Success, BadgeSize::Small, false, palette),
        badge("Warning", BadgeVariant::Warning, BadgeSize::Small, false, palette),
        badge("Danger", BadgeVariant::Danger, BadgeSize::Small, false, palette),
        badge("Info", BadgeVariant::Info, BadgeSize::Small, false, palette),
        badge("Neutral", BadgeVariant::Neutral, BadgeSize::Small, false, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let dot_pills_row = row![
        status_dot_pill("Ready (running)", "running", BadgeSize::Small, palette),
        status_dot_pill("Degraded (crashloop)", "crashloopbackoff", BadgeSize::Small, palette),
        status_dot_pill("Pending (waiting)", "pending", BadgeSize::Small, palette),
        status_pill("Active", BadgeSize::Small, palette),
        status_pill("Completed", BadgeSize::Small, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let resource_badges_row_1 = row![
        resource_kind_badge(ResourceKind::Pod, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::Deployment, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::StatefulSet, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::DaemonSet, BadgeSize::Small, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let resource_badges_row_2 = row![
        resource_kind_badge(ResourceKind::Service, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::Ingress, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::ConfigMap, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::Secret, BadgeSize::Small, palette),
        resource_kind_badge(ResourceKind::Node, BadgeSize::Small, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let badges_content = column![
        text("Semantic Variants").size(12).color(palette.text_secondary),
        badge_variants_row,
        text("Status Indicators").size(12).color(palette.text_secondary),
        dot_pills_row,
        text("Kubernetes Resource Kinds").size(12).color(palette.text_secondary),
        resource_badges_row_1,
        resource_badges_row_2,
    ]
    .spacing(10)
    .width(Length::Fill);

    let badges_card = preview_box_card(
        "Badges & Kubernetes Statuses",
        "Semantic color tints, pod health status pills, and resource kind badges",
        badges_content,
        palette,
    );

    // Card 5: Typography & Font Scale
    let code_block = container(
        row![
            text("kubectl get pods -n kube-system -o wide")
                .size(12)
                .color(palette.text_primary),
            horizontal_space(),
            badge("BASH", BadgeVariant::Neutral, BadgeSize::Small, false, palette),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(if palette.is_dark {
            Color::from_rgb8(0x12, 0x12, 0x15)
        } else {
            Color::from_rgb8(0xf4, 0xf4, 0xf5)
        })),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: iced::border::Radius::from(4.0),
        },
        ..Default::default()
    });

    let typography_content = column![
        column![
            text("H1 / Display Title (24px)").size(24).color(palette.text_primary),
            text("Cluster Overview & Control Plane").size(11).color(palette.text_muted),
        ]
        .spacing(2),
        column![
            text("H2 / Section Title (18px)").size(18).color(palette.text_primary),
            text("Workload Controller Status").size(11).color(palette.text_muted),
        ]
        .spacing(2),
        column![
            text("H3 / Card Header (14px)").size(14).color(palette.text_primary),
            text("Replication Controller Target").size(11).color(palette.text_muted),
        ]
        .spacing(2),
        column![
            text("H4 / Subsection (13px)").size(13).color(palette.text_secondary),
            text("Environment Variables").size(11).color(palette.text_muted),
        ]
        .spacing(2),
        column![
            text("Body Text (13px) - High performance native desktop dashboard for Kubernetes clusters.")
                .size(13)
                .color(palette.text_primary),
            text("Regular body text with high legibility").size(11).color(palette.text_muted),
        ]
        .spacing(2),
        column![
            text("Code Block (Monospace)").size(12).color(palette.text_secondary),
            code_block,
        ]
        .spacing(4),
        text("Caption / Metadata (11px) - Last reconciled: 2 mins ago by operator")
            .size(11)
            .color(palette.text_muted),
    ]
    .spacing(12)
    .width(Length::Fill);

    let typography_card = preview_box_card(
        "Typography & Font Hierarchy",
        "Proportional type scale from 24px display headings down to 11px captions",
        typography_content,
        palette,
    );

    // Card 6: Alerts & Callouts
    let alerts_content = column![
        alert_banner(
            AlertLevel::Info,
            "Cluster Connected",
            "Authenticated to production-east-1 via token (context: admin@prod)",
            palette,
        ),
        alert_banner(
            AlertLevel::Success,
            "Replication Complete",
            "Workload deployment scaled successfully to 5/5 ready replicas",
            palette,
        ),
        alert_banner(
            AlertLevel::Warning,
            "Memory Pressure Warning",
            "Node worker-node-04 memory utilization is at 88% of allocatable capacity",
            palette,
        ),
        alert_banner(
            AlertLevel::Danger,
            "CrashLoopBackOff Detected",
            "Pod payments-api-7b89f-2mql1 failed readiness probe (exit code 137 OOMKilled)",
            palette,
        ),
    ]
    .spacing(10)
    .width(Length::Fill);

    let alerts_card = preview_box_card(
        "Alert Banners & Callouts",
        "High-contrast system messages and contextual operational notifications",
        alerts_content,
        palette,
    );

    // 5. Layout Assembly (2-Column Grid for All, Single Card for Section Filters)
    let mut sections = column![header_bar, playground_bar, filter_row].spacing(16);

    match state.active_section {
        ShowcaseSection::All => {
            let col_left = column![tokens_card, buttons_card, alerts_card]
                .spacing(16)
                .width(Length::FillPortion(1));

            let col_right = column![forms_card, badges_card, typography_card]
                .spacing(16)
                .width(Length::FillPortion(1));

            let grid = row![col_left, col_right].spacing(16).width(Length::Fill);
            sections = sections.push(grid);
        }
        ShowcaseSection::Tokens => sections = sections.push(tokens_card),
        ShowcaseSection::Buttons => sections = sections.push(buttons_card),
        ShowcaseSection::Forms => sections = sections.push(forms_card),
        ShowcaseSection::Badges => sections = sections.push(badges_card),
        ShowcaseSection::Typography => sections = sections.push(typography_card),
        ShowcaseSection::Alerts => sections = sections.push(alerts_card),
    }

    scrollable(container(sections).padding(20).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_state_transitions() {
        let mut state = ShowcaseState::default();
        assert_eq!(state.sample_counter, 5);
        assert_eq!(state.sample_toggle_1, true);

        state.update(ShowcaseMessage::SelectSection(ShowcaseSection::Buttons));
        assert_eq!(state.active_section, ShowcaseSection::Buttons);

        state.update(ShowcaseMessage::ToggleSample1(false));
        assert_eq!(state.sample_toggle_1, false);

        state.update(ShowcaseMessage::ToggleSample2(true));
        assert_eq!(state.sample_toggle_2, true);

        state.update(ShowcaseMessage::IncrementCounter);
        assert_eq!(state.sample_counter, 6);

        state.update(ShowcaseMessage::DecrementCounter);
        assert_eq!(state.sample_counter, 5);

        state.update(ShowcaseMessage::TextInputChanged("kube-system".into()));
        assert_eq!(state.sample_text_input, "kube-system");

        state.update(ShowcaseMessage::OptionSelected("staging".into()));
        assert_eq!(state.sample_selected_option, "staging");

        state.update(ShowcaseMessage::TriggerAction("tested".into()));
        assert_eq!(state.last_action.as_deref(), Some("tested"));
    }

    #[test]
    fn test_showcase_extended_state_and_messages() {
        assert_eq!(ShowcaseSection::ALL.len(), 7);

        let mut state = ShowcaseState::default();
        assert_eq!(state.active_accent, "blue");
        assert_eq!(state.sample_search_input, "");
        assert_eq!(state.sample_selected_segment, "Overview");

        state.update(ShowcaseMessage::SearchInputChanged("test".into()));
        assert_eq!(state.sample_search_input, "test");

        state.update(ShowcaseMessage::ClearSearchInput);
        assert_eq!(state.sample_search_input, "");

        state.update(ShowcaseMessage::SegmentSelected("Metrics".into()));
        assert_eq!(state.sample_selected_segment, "Metrics");

        state.update(ShowcaseMessage::SetAccentColor("emerald".into()));
        assert_eq!(state.active_accent, "emerald");
    }

    #[test]
    fn test_showcase_catalog_view_renders_all_sections_both_themes() {
        let state = ShowcaseState::default();
        let dark_palette = OrbitPalette::dark();
        let light_palette = OrbitPalette::light();

        for sec in ShowcaseSection::ALL {
            let mut s = state.clone();
            s.active_section = *sec;
            let _dark_el: Element<'_, ()> = showcase_view(&s, &dark_palette, |_| (), ());
            let _light_el: Element<'_, ()> = showcase_view(&s, &light_palette, |_| (), ());
        }
    }
}
