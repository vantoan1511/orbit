#![cfg(debug_assertions)]

use iced::{
    widget::{
        button, column, container, horizontal_space, pick_list, row, scrollable, text,
        text_input, toggler,
    },
    Alignment, Element, Length,
};
use orbit_core::command::ResourceKind;

use crate::components::{
    badge::{badge, resource_kind_badge, status_dot_pill, status_pill, BadgeSize, BadgeVariant},
    button::{
        custom_button, destructive_button, ghost_button, icon_button, primary_button,
        secondary_button, ButtonSize, ButtonVariant,
    },
    card::{container_card, metric_card, section_card},
    form_row::form_row,
};
use crate::icons::{icon, Icon};
use crate::theme::{
    ghost_action_button_style, pick_list_style, text_input_style,
    toggler_style, OrbitPalette,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowcaseSection {
    All,
    Badges,
    Buttons,
    Cards,
    Forms,
}

impl ShowcaseSection {
    pub const ALL: &'static [Self] = &[
        Self::All,
        Self::Badges,
        Self::Buttons,
        Self::Cards,
        Self::Forms,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Components",
            Self::Badges => "Badges & Pills",
            Self::Buttons => "Buttons",
            Self::Cards => "Cards",
            Self::Forms => "Forms",
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
    pub sample_selected_option: String,
    pub sample_options: Vec<String>,
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
            sample_selected_option: "production".to_string(),
            sample_options: vec![
                "production".to_string(),
                "staging".to_string(),
                "development".to_string(),
            ],
            last_action: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShowcaseMessage {
    SelectSection(ShowcaseSection),
    ToggleSample1(bool),
    ToggleSample2(bool),
    IncrementCounter,
    DecrementCounter,
    TextInputChanged(String),
    OptionSelected(String),
    TriggerAction(String),
}

impl ShowcaseState {
    pub fn update(&mut self, msg: ShowcaseMessage) {
        match msg {
            ShowcaseMessage::SelectSection(sec) => self.active_section = sec,
            ShowcaseMessage::ToggleSample1(val) => self.sample_toggle_1 = val,
            ShowcaseMessage::ToggleSample2(val) => self.sample_toggle_2 = val,
            ShowcaseMessage::IncrementCounter => {
                self.sample_counter = self.sample_counter.saturating_add(1);
            }
            ShowcaseMessage::DecrementCounter => {
                self.sample_counter = self.sample_counter.saturating_sub(1);
            }
            ShowcaseMessage::TextInputChanged(text) => self.sample_text_input = text,
            ShowcaseMessage::OptionSelected(opt) => self.sample_selected_option = opt,
            ShowcaseMessage::TriggerAction(act) => self.last_action = Some(act),
        }
    }
}

pub fn showcase_view<'a, Message: 'a + Clone>(
    state: &'a ShowcaseState,
    palette: &'a OrbitPalette,
    on_showcase_msg: impl Fn(ShowcaseMessage) -> Message + 'a + Copy,
    on_theme_toggle: Message,
) -> Element<'a, Message> {
    // 1. Header toolbar
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

    let mut header_actions = row![theme_btn].spacing(8).align_y(Alignment::Center);

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
            text("UI Component Library").size(20).color(palette.text_primary),
            text("Internal design tokens, atomic primitives, and component showcase")
                .size(12)
                .color(palette.text_muted),
        ]
        .spacing(2),
        horizontal_space(),
        header_actions,
    ]
    .align_y(Alignment::Center);

    // Filter pills
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

    let mut sections = column![header_bar, filter_row].spacing(20);

    // 2. Badges & Pills Section
    if matches!(state.active_section, ShowcaseSection::All | ShowcaseSection::Badges) {
        let badge_variants_row = row![
            badge("Success", BadgeVariant::Success, BadgeSize::Small, false, palette),
            badge("Warning", BadgeVariant::Warning, BadgeSize::Small, false, palette),
            badge("Danger", BadgeVariant::Danger, BadgeSize::Small, false, palette),
            badge("Info", BadgeVariant::Info, BadgeSize::Small, false, palette),
            badge("Neutral", BadgeVariant::Neutral, BadgeSize::Small, false, palette),
            badge("Medium Success", BadgeVariant::Success, BadgeSize::Medium, false, palette),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let dot_pills_row = row![
            status_dot_pill("Ready (running)", "running", BadgeSize::Small, palette),
            status_dot_pill("Degraded (crashloop)", "crashloopbackoff", BadgeSize::Small, palette),
            status_dot_pill("Pending (waiting)", "pending", BadgeSize::Small, palette),
            status_pill("Active", BadgeSize::Small, palette),
            status_pill("Completed", BadgeSize::Small, palette),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let resource_badges_row = row![
            resource_kind_badge(ResourceKind::Pod, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::Deployment, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::StatefulSet, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::Service, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::ConfigMap, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::Secret, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::Node, BadgeSize::Small, palette),
            resource_kind_badge(ResourceKind::Ingress, BadgeSize::Small, palette),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let badge_card = section_card(
            "Badges & Metadata Pills",
            "Color tokens, status indicators, and Kubernetes resource badges",
            vec![
                form_row("Semantic Variants", "Standard semantic color tints with borders", badge_variants_row.into(), palette),
                form_row("Status Dot Pills", "Live pod and controller health indicators", dot_pills_row.into(), palette),
                form_row("Kubernetes Resources", "Standard color tokens per resource kind", resource_badges_row.into(), palette),
            ],
            palette,
        );
        sections = sections.push(badge_card);
    }

    // 3. Buttons Section
    if matches!(state.active_section, ShowcaseSection::All | ShowcaseSection::Buttons) {
        let variants_row = row![
            primary_button(
                "Primary Action",
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("primary_clicked".into()))),
                palette,
            ),
            secondary_button(
                "Secondary Action",
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("secondary_clicked".into()))),
                palette,
            ),
            ghost_button(
                "Ghost Button",
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("ghost_clicked".into()))),
                palette,
            ),
            destructive_button(
                "Destructive Action",
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("destructive_clicked".into()))),
                palette,
            ),
            icon_button(
                icon(Icon::RotateCcw, 14, palette.text_secondary),
                ButtonSize::Medium,
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("icon_clicked".into()))),
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
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("small_clicked".into()))),
                palette,
            ),
            custom_button(
                text("Medium (32px)").size(ButtonSize::Medium.text_size()),
                ButtonVariant::Secondary,
                ButtonSize::Medium,
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("medium_clicked".into()))),
                palette,
            ),
            custom_button(
                text("Large (38px)").size(ButtonSize::Large.text_size()),
                ButtonVariant::Secondary,
                ButtonSize::Large,
                Some(on_showcase_msg(ShowcaseMessage::TriggerAction("large_clicked".into()))),
                palette,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let disabled_row = row![
            primary_button("Disabled Primary", None, palette),
            secondary_button("Disabled Secondary", None, palette),
            ghost_button("Disabled Ghost", None, palette),
            destructive_button("Disabled Destructive", None, palette),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let button_card = section_card(
            "Buttons & Actions",
            "Consistent interactive controls with hover, active, and disabled states",
            vec![
                form_row("Button Variants", "Primary, secondary, ghost, destructive, and icon buttons", variants_row.into(), palette),
                form_row("Size Scale", "Small (dense tables), Medium (dialogs/forms), Large (welcome/onboarding)", sizes_row.into(), palette),
                form_row("Disabled States", "Dimmed opacity, muted text, non-interactive cursor", disabled_row.into(), palette),
            ],
            palette,
        );
        sections = sections.push(button_card);
    }

    // 4. Cards & Containers Section
    if matches!(state.active_section, ShowcaseSection::All | ShowcaseSection::Cards) {
        let metrics_row = row![
            metric_card("NODES", "3/3", "Ready / Total", Some(Icon::Server), palette),
            metric_card("PODS", "48/52", "4 degraded", Some(Icon::Box), palette),
            metric_card("DEPLOYMENTS", "14", "Active controllers", Some(Icon::Layers), palette),
            metric_card("WARNINGS", "3", "Cluster events", Some(Icon::Bell), palette),
        ]
        .spacing(12);

        let container_demo = container_card(
            column![
                text("Basic Container Card").size(14).color(palette.text_primary),
                text("Encapsulates complex view widgets with 1px border and 16px padding")
                    .size(12)
                    .color(palette.text_muted),
            ]
            .spacing(4),
            palette,
        );

        let card_section = column![
            text("Cards & Metric Summaries").size(16).color(palette.text_primary),
            metrics_row,
            container_demo,
        ]
        .spacing(12);

        sections = sections.push(card_section);
    }

    // 5. Form Controls Section
    if matches!(state.active_section, ShowcaseSection::All | ShowcaseSection::Forms) {
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

        let input_field = text_input("Enter role or namespace...", &state.sample_text_input)
            .on_input(move |text| on_showcase_msg(ShowcaseMessage::TextInputChanged(text)))
            .style(text_input_style(palette))
            .size(12)
            .padding([5, 8])
            .width(Length::Fixed(180.0));

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

        let form_card = section_card(
            "Interactive Form Controls",
            "Standardized input controls aligned with Orbit's two-column form specification",
            vec![
                form_row("Real-time Telemetry", "Stream live metric batch updates into memory", tog1.into(), palette),
                form_row("Compact Layout", "Reduce table row height and padding", tog2.into(), palette),
                form_row("Environment Profile", "Target Kubernetes deployment environment", picker.into(), palette),
                form_row("Service Account Role", "Configured cluster role or identity", input_field.into(), palette),
                form_row("Replica Count Stepper", "Adjust active workload scale target", stepper.into(), palette),
            ],
            palette,
        );

        sections = sections.push(form_card);
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
    fn test_showcase_view_renders_without_panic() {
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
