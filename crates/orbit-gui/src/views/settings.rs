use std::collections::HashMap;
use std::fmt;
use iced::{
    widget::{
        column, container, horizontal_space, pick_list, row, scrollable, text,
        text_input, toggler,
    },
    Element, Length,
};
use crate::components::{
    button::ghost_button,
    card::section_card,
    form_row::form_row,
};
use crate::theme::{
    pick_list_style, text_input_style, toggler_style, OrbitPalette,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeModeOption {
    Dark,
    Light,
    System,
}

impl ThemeModeOption {
    pub const ALL: &'static [Self] = &[Self::Dark, Self::Light, Self::System];
}

impl fmt::Display for ThemeModeOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dark => write!(f, "Dark"),
            Self::Light => write!(f, "Light"),
            Self::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentColorOption {
    Blue,
    Emerald,
    Violet,
    Rose,
    Amber,
}

impl AccentColorOption {
    pub const ALL: &'static [Self] = &[
        Self::Blue,
        Self::Emerald,
        Self::Violet,
        Self::Rose,
        Self::Amber,
    ];
}

impl fmt::Display for AccentColorOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blue => write!(f, "Blue"),
            Self::Emerald => write!(f, "Emerald"),
            Self::Violet => write!(f, "Violet"),
            Self::Rose => write!(f, "Rose"),
            Self::Amber => write!(f, "Amber"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRetentionOption {
    Files5,
    Files10,
    Files20,
    Files50,
    Files100,
}

impl LogRetentionOption {
    pub const ALL: &'static [Self] = &[
        Self::Files5,
        Self::Files10,
        Self::Files20,
        Self::Files50,
        Self::Files100,
    ];
}

impl fmt::Display for LogRetentionOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Files5 => write!(f, "5 files"),
            Self::Files10 => write!(f, "10 files"),
            Self::Files20 => write!(f, "20 files"),
            Self::Files50 => write!(f, "50 files"),
            Self::Files100 => write!(f, "100 files"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsViewState {
    pub terminal_shell_input: Option<String>,
    pub feedback_message: Option<String>,
}

pub fn parse_theme_mode(val: Option<&serde_json::Value>) -> ThemeModeOption {
    match val.and_then(|v| v.as_str()) {
        Some("light") => ThemeModeOption::Light,
        Some("system") => ThemeModeOption::System,
        _ => ThemeModeOption::Dark,
    }
}

pub fn theme_mode_to_json(opt: ThemeModeOption) -> serde_json::Value {
    match opt {
        ThemeModeOption::Dark => serde_json::json!("dark"),
        ThemeModeOption::Light => serde_json::json!("light"),
        ThemeModeOption::System => serde_json::json!("system"),
    }
}

pub fn parse_accent_color(val: Option<&serde_json::Value>) -> AccentColorOption {
    match val.and_then(|v| v.as_str()) {
        Some("emerald") => AccentColorOption::Emerald,
        Some("violet") => AccentColorOption::Violet,
        Some("rose") => AccentColorOption::Rose,
        Some("amber") => AccentColorOption::Amber,
        _ => AccentColorOption::Blue,
    }
}

pub fn accent_color_to_json(opt: AccentColorOption) -> serde_json::Value {
    match opt {
        AccentColorOption::Blue => serde_json::json!("blue"),
        AccentColorOption::Emerald => serde_json::json!("emerald"),
        AccentColorOption::Violet => serde_json::json!("violet"),
        AccentColorOption::Rose => serde_json::json!("rose"),
        AccentColorOption::Amber => serde_json::json!("amber"),
    }
}

pub fn parse_log_retention(val: Option<&serde_json::Value>) -> LogRetentionOption {
    let num = val
        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(10);
    match num {
        5 => LogRetentionOption::Files5,
        10 => LogRetentionOption::Files10,
        20 => LogRetentionOption::Files20,
        50 => LogRetentionOption::Files50,
        100 => LogRetentionOption::Files100,
        _ => LogRetentionOption::Files10,
    }
}

pub fn log_retention_to_u64(opt: LogRetentionOption) -> u64 {
    match opt {
        LogRetentionOption::Files5 => 5,
        LogRetentionOption::Files10 => 10,
        LogRetentionOption::Files20 => 20,
        LogRetentionOption::Files50 => 50,
        LogRetentionOption::Files100 => 100,
    }
}

pub fn parse_bool_setting(val: Option<&serde_json::Value>, default: bool) -> bool {
    val.and_then(|v| v.as_bool()).unwrap_or(default)
}

pub fn parse_notification_pref(prefs: Option<&serde_json::Value>, key: &str, default: bool) -> bool {
    prefs
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

pub fn update_notification_pref(
    existing: Option<&serde_json::Value>,
    key: &str,
    val: bool,
) -> serde_json::Value {
    let mut map = match existing.and_then(|v| v.as_object()) {
        Some(obj) => obj.clone(),
        None => {
            let mut m = serde_json::Map::new();
            m.insert("enableToasts".to_string(), serde_json::json!(true));
            m.insert("notifyOnSuccess".to_string(), serde_json::json!(true));
            m.insert("notifyOnError".to_string(), serde_json::json!(true));
            m.insert("notifyOnUpdates".to_string(), serde_json::json!(true));
            m
        }
    };
    map.insert(key.to_string(), serde_json::json!(val));
    serde_json::Value::Object(map)
}

pub fn settings_view<'a, Message: 'a + Clone>(
    values: &'a HashMap<String, serde_json::Value>,
    state: &'a SettingsViewState,
    palette: &'a OrbitPalette,
    on_update: impl Fn(String, serde_json::Value) -> Message + 'a + Copy,
    on_shell_change: impl Fn(String) -> Message + 'a,
    on_reset: Message,
) -> Element<'a, Message> {
    // Card 1: Appearance & Theme
    let selected_theme = parse_theme_mode(values.get("themeMode"));
    let theme_picker = pick_list(
        ThemeModeOption::ALL,
        Some(selected_theme),
        move |opt| on_update("themeMode".to_string(), theme_mode_to_json(opt)),
    )
    .style(pick_list_style(palette))
    .text_size(12)
    .padding([4, 8])
    .width(Length::Fixed(140.0));

    let selected_accent = parse_accent_color(values.get("accentColor"));
    let accent_picker = pick_list(
        AccentColorOption::ALL,
        Some(selected_accent),
        move |opt| on_update("accentColor".to_string(), accent_color_to_json(opt)),
    )
    .style(pick_list_style(palette))
    .text_size(12)
    .padding([4, 8])
    .width(Length::Fixed(140.0));

    let card_appearance = section_card(
        "Appearance & Theme",
        "Customize interface theme and primary accent colors",
        vec![
            form_row(
                "Theme Mode",
                "Choose between Dark, Light, or System appearance",
                theme_picker.into(),
                palette,
            ),
            form_row(
                "Accent Color",
                "Select highlight color for tabs, active indicators, and focus outlines",
                accent_picker.into(),
                palette,
            ),
        ],
        palette,
    );

    // Card 2: System & Window Behavior
    let launch_on_startup = parse_bool_setting(values.get("launchOnStartup"), false);
    let launch_tog = toggler(launch_on_startup)
        .on_toggle(move |val| on_update("launchOnStartup".to_string(), serde_json::json!(val)))
        .style(toggler_style(palette))
        .size(18);

    let close_to_tray = parse_bool_setting(values.get("closeToTray"), true);
    let tray_tog = toggler(close_to_tray)
        .on_toggle(move |val| on_update("closeToTray".to_string(), serde_json::json!(val)))
        .style(toggler_style(palette))
        .size(18);

    let auto_check_updates = parse_bool_setting(values.get("autoCheckUpdates"), true);
    let updates_tog = toggler(auto_check_updates)
        .on_toggle(move |val| on_update("autoCheckUpdates".to_string(), serde_json::json!(val)))
        .style(toggler_style(palette))
        .size(18);

    let card_system = section_card(
        "System & Window Behavior",
        "Configure startup, window closing, and update checks",
        vec![
            form_row(
                "Launch on Startup",
                "Automatically start Orbit minimized to the system tray when logging in",
                launch_tog.into(),
                palette,
            ),
            form_row(
                "Close to System Tray",
                "Keep Orbit running in the background tray when window is closed",
                tray_tog.into(),
                palette,
            ),
            form_row(
                "Automatically Check for Updates",
                "Periodically query for new releases and security patches",
                updates_tog.into(),
                palette,
            ),
        ],
        palette,
    );

    // Card 3: Workloads & Cluster
    let selected_retention = parse_log_retention(values.get("maxLogFiles"));
    let retention_picker = pick_list(
        LogRetentionOption::ALL,
        Some(selected_retention),
        move |opt| on_update("maxLogFiles".to_string(), serde_json::json!(log_retention_to_u64(opt))),
    )
    .style(pick_list_style(palette))
    .text_size(12)
    .padding([4, 8])
    .width(Length::Fixed(140.0));

    let shell_val = match &state.terminal_shell_input {
        Some(val) => val.as_str(),
        None => values
            .get("defaultTerminalShell")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    };

    let shell_input = text_input("e.g. pwsh.exe, /bin/zsh", shell_val)
        .on_input(on_shell_change)
        .on_submit(on_update(
            "defaultTerminalShell".to_string(),
            serde_json::json!(shell_val),
        ))
        .style(text_input_style(palette))
        .size(12)
        .padding([6, 8])
        .width(Length::Fixed(220.0));

    let card_workloads = section_card(
        "Workloads & Cluster",
        "Manage application logging capacity and default shell preferences",
        vec![
            form_row(
                "Log Retention Limit",
                "Maximum number of rotating log files retained per session",
                retention_picker.into(),
                palette,
            ),
            form_row(
                "Default Terminal Shell",
                "Custom executable path for container exec and terminal sessions",
                shell_input.into(),
                palette,
            ),
        ],
        palette,
    );

    // Card 4: Notifications
    let notif_prefs = values.get("notificationPreferences");
    let enable_toasts = parse_notification_pref(notif_prefs, "enableToasts", true);
    let notify_on_error = parse_notification_pref(notif_prefs, "notifyOnError", true);
    let notify_on_updates = parse_notification_pref(notif_prefs, "notifyOnUpdates", true);

    let toasts_tog = toggler(enable_toasts)
        .on_toggle(move |val| {
            on_update(
                "notificationPreferences".to_string(),
                update_notification_pref(values.get("notificationPreferences"), "enableToasts", val),
            )
        })
        .style(toggler_style(palette))
        .size(18);

    let errors_tog = toggler(notify_on_error)
        .on_toggle(move |val| {
            on_update(
                "notificationPreferences".to_string(),
                update_notification_pref(values.get("notificationPreferences"), "notifyOnError", val),
            )
        })
        .style(toggler_style(palette))
        .size(18);

    let updates_notif_tog = toggler(notify_on_updates)
        .on_toggle(move |val| {
            on_update(
                "notificationPreferences".to_string(),
                update_notification_pref(values.get("notificationPreferences"), "notifyOnUpdates", val),
            )
        })
        .style(toggler_style(palette))
        .size(18);

    let card_notifications = section_card(
        "Notifications",
        "Configure toast alerts and event notification popups",
        vec![
            form_row(
                "Enable Toast Popups",
                "Display non-intrusive notification bubbles in the bottom-right corner",
                toasts_tog.into(),
                palette,
            ),
            form_row(
                "Notify on Errors & Crashes",
                "Surface alerts when pod crashes or cluster connection failures occur",
                errors_tog.into(),
                palette,
            ),
            form_row(
                "Notify on Updates",
                "Surface banner notifications when a new version of Orbit is discovered",
                updates_notif_tog.into(),
                palette,
            ),
        ],
        palette,
    );

    // Action Footer
    let feedback_el: Element<'a, Message> = if let Some(msg) = &state.feedback_message {
        row![
            text("●").size(10).color(palette.success),
            text(msg.clone()).size(12).color(palette.success),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![].into()
    };

    let reset_btn = ghost_button("Reset to Defaults", Some(on_reset), palette);

    let footer = row![feedback_el, horizontal_space(), reset_btn]
        .align_y(iced::Alignment::Center)
        .padding([8, 0]);

    // Page Header
    let page_header = column![
        text("Preferences").size(20).color(palette.text_primary),
        text("Configure your Orbit desktop environment and cluster preferences")
            .size(12)
            .color(palette.text_secondary),
    ]
    .spacing(4);

    let content = column![
        page_header,
        card_appearance,
        card_system,
        card_workloads,
        card_notifications,
        footer,
    ]
    .spacing(16)
    .max_width(800.0);

    container(scrollable(content).width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([24, 32])
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_option_parsers_and_notification_helpers() {
        assert_eq!(parse_theme_mode(Some(&serde_json::json!("dark"))), ThemeModeOption::Dark);
        assert_eq!(parse_theme_mode(Some(&serde_json::json!("light"))), ThemeModeOption::Light);
        assert_eq!(parse_theme_mode(Some(&serde_json::json!("system"))), ThemeModeOption::System);
        assert_eq!(parse_theme_mode(None), ThemeModeOption::Dark);
        assert_eq!(parse_theme_mode(Some(&serde_json::json!("unknown"))), ThemeModeOption::Dark);

        assert_eq!(theme_mode_to_json(ThemeModeOption::Dark), serde_json::json!("dark"));
        assert_eq!(theme_mode_to_json(ThemeModeOption::Light), serde_json::json!("light"));
        assert_eq!(theme_mode_to_json(ThemeModeOption::System), serde_json::json!("system"));

        assert_eq!(parse_accent_color(Some(&serde_json::json!("emerald"))), AccentColorOption::Emerald);
        assert_eq!(parse_accent_color(Some(&serde_json::json!("violet"))), AccentColorOption::Violet);
        assert_eq!(parse_accent_color(Some(&serde_json::json!("rose"))), AccentColorOption::Rose);
        assert_eq!(parse_accent_color(Some(&serde_json::json!("amber"))), AccentColorOption::Amber);
        assert_eq!(parse_accent_color(Some(&serde_json::json!("blue"))), AccentColorOption::Blue);
        assert_eq!(parse_accent_color(None), AccentColorOption::Blue);

        assert_eq!(accent_color_to_json(AccentColorOption::Blue), serde_json::json!("blue"));
        assert_eq!(accent_color_to_json(AccentColorOption::Emerald), serde_json::json!("emerald"));
        assert_eq!(accent_color_to_json(AccentColorOption::Violet), serde_json::json!("violet"));
        assert_eq!(accent_color_to_json(AccentColorOption::Rose), serde_json::json!("rose"));
        assert_eq!(accent_color_to_json(AccentColorOption::Amber), serde_json::json!("amber"));

        assert_eq!(parse_log_retention(Some(&serde_json::json!(5))), LogRetentionOption::Files5);
        assert_eq!(parse_log_retention(Some(&serde_json::json!(10))), LogRetentionOption::Files10);
        assert_eq!(parse_log_retention(Some(&serde_json::json!(20))), LogRetentionOption::Files20);
        assert_eq!(parse_log_retention(Some(&serde_json::json!(50))), LogRetentionOption::Files50);
        assert_eq!(parse_log_retention(Some(&serde_json::json!(100))), LogRetentionOption::Files100);
        assert_eq!(parse_log_retention(None), LogRetentionOption::Files10);

        assert_eq!(log_retention_to_u64(LogRetentionOption::Files5), 5);
        assert_eq!(log_retention_to_u64(LogRetentionOption::Files10), 10);
        assert_eq!(log_retention_to_u64(LogRetentionOption::Files20), 20);
        assert_eq!(log_retention_to_u64(LogRetentionOption::Files50), 50);
        assert_eq!(log_retention_to_u64(LogRetentionOption::Files100), 100);

        assert_eq!(parse_bool_setting(Some(&serde_json::json!(true)), false), true);
        assert_eq!(parse_bool_setting(Some(&serde_json::json!(false)), true), false);
        assert_eq!(parse_bool_setting(None, true), true);

        let initial_notifications = serde_json::json!({
            "enableToasts": true,
            "notifyOnSuccess": false,
            "notifyOnError": true
        });
        assert_eq!(parse_notification_pref(Some(&initial_notifications), "enableToasts", false), true);
        assert_eq!(parse_notification_pref(Some(&initial_notifications), "notifyOnSuccess", true), false);
        assert_eq!(parse_notification_pref(Some(&initial_notifications), "notifyOnUpdates", true), true);

        let updated = update_notification_pref(Some(&initial_notifications), "notifyOnSuccess", true);
        assert_eq!(updated["notifyOnSuccess"], serde_json::json!(true));
        assert_eq!(updated["enableToasts"], serde_json::json!(true));
        assert_eq!(updated["notifyOnError"], serde_json::json!(true));

        let from_none = update_notification_pref(None, "enableToasts", false);
        assert_eq!(from_none["enableToasts"], serde_json::json!(false));
        assert_eq!(from_none["notifyOnError"], serde_json::json!(true));
    }

    #[test]
    fn test_settings_view_renders_cards_and_footer() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        #[derive(Debug, Clone, PartialEq)]
        enum TestMsg {
            Update(String, serde_json::Value),
            Shell(String),
            Reset,
        }

        let empty_values = HashMap::new();
        let state = SettingsViewState::default();

        {
            let dark_view: Element<'_, TestMsg> = settings_view(
                &empty_values,
                &state,
                &dark,
                |k, v| TestMsg::Update(k, v),
                |s| TestMsg::Shell(s),
                TestMsg::Reset,
            );
            let _ = dark_view;
        }

        let mut populated_values = HashMap::new();
        populated_values.insert("themeMode".to_string(), serde_json::json!("light"));
        populated_values.insert("accentColor".to_string(), serde_json::json!("emerald"));
        populated_values.insert("maxLogFiles".to_string(), serde_json::json!(50));
        populated_values.insert("launchOnStartup".to_string(), serde_json::json!(true));
        populated_values.insert("closeToTray".to_string(), serde_json::json!(false));
        populated_values.insert("autoCheckUpdates".to_string(), serde_json::json!(true));
        populated_values.insert("defaultTerminalShell".to_string(), serde_json::json!("pwsh.exe"));
        populated_values.insert(
            "notificationPreferences".to_string(),
            serde_json::json!({
                "enableToasts": false,
                "notifyOnError": true,
                "notifyOnUpdates": false
            }),
        );

        let mut populated_state = SettingsViewState::default();
        populated_state.feedback_message = Some("Settings saved".to_string());
        populated_state.terminal_shell_input = Some("pwsh.exe".to_string());

        {
            let light_view: Element<'_, TestMsg> = settings_view(
                &populated_values,
                &populated_state,
                &light,
                |k, v| TestMsg::Update(k, v),
                |s| TestMsg::Shell(s),
                TestMsg::Reset,
            );
            let _ = light_view;
        }
    }
}

