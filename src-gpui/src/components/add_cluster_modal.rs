use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, MouseButton, Rgba, SharedString, Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::kubeconfig_import::ManualAuthType;

use crate::add_cluster_state::{
    AddClusterFieldId, AddClusterModalState, AddClusterTab, CLUSTER_COLOR_PRESETS,
};
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, rgb_hex, rgba_hex};

pub type ModalActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type ModalTabCallback = Rc<dyn Fn(AddClusterTab, &mut Window, &mut App)>;
pub type ModalFieldCallback = Rc<dyn Fn(AddClusterFieldId, &mut Window, &mut App)>;
pub type ModalAuthTypeCallback = Rc<dyn Fn(ManualAuthType, &mut Window, &mut App)>;
pub type ModalStringCallback = Rc<dyn Fn(String, &mut Window, &mut App)>;
pub type ModalOptStringCallback = Rc<dyn Fn(Option<String>, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct AddClusterModalCallbacks {
    pub on_close: ModalActionCallback,
    pub on_select_tab: ModalTabCallback,
    pub on_browse_file: ModalActionCallback,
    pub on_paste_yaml: ModalActionCallback,
    pub on_clear_yaml: ModalActionCallback,
    pub on_focus_field: ModalFieldCallback,
    pub on_select_auth_type: ModalAuthTypeCallback,
    pub on_toggle_insecure_tls: ModalActionCallback,
    pub on_toggle_context: ModalStringCallback,
    pub on_select_all_contexts: ModalActionCallback,
    pub on_deselect_all_contexts: ModalActionCallback,
    pub on_toggle_customization: ModalActionCallback,
    pub on_select_color: ModalOptStringCallback,
    pub on_add_tag: ModalActionCallback,
    pub on_remove_tag: ModalStringCallback,
    pub on_toggle_switch_context: ModalActionCallback,
    pub on_submit: ModalActionCallback,
}

pub fn parse_hex_color(hex: &str) -> Option<Rgba> {
    let stripped = hex.strip_prefix('#').unwrap_or(hex);
    if stripped.len() != 6 {
        return None;
    }
    u32::from_str_radix(stripped, 16).ok().map(rgb_hex)
}

pub fn render_add_cluster_modal(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let on_close_backdrop = callbacks.on_close.clone();
    let on_close_btn = callbacks.on_close.clone();

    let modal_card = theme
        .style_card(div())
        .id("add-cluster-modal-card")
        .w(px(640.0))
        .max_h(px(760.0))
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(theme.border.strong)
        .shadow_lg()
        .flex()
        .flex_col()
        .overflow_hidden()
        .on_mouse_down(MouseButton::Left, |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        // Header
        .child(
            div()
                .px_5()
                .py_3()
                .border_b_1()
                .border_color(theme.border.default)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(Icon::Server, px(16.0), theme.accent.accent))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child("Add Cluster"),
                        ),
                )
                .child(
                    div()
                        .id("add-cluster-close-btn")
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded(theme.style.radius_control)
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface.bg_hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_close_btn(window, cx);
                        })
                        .child(render_icon(Icon::X, px(14.0), theme.text.muted)),
                ),
        )
        // Tab switcher
        .child(render_tab_switcher(state, theme, callbacks))
        // Scrollable body
        .child(
            div()
                .id("add-cluster-modal-body")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px_5()
                .py_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(match state.active_tab {
                    AddClusterTab::File => render_file_tab(state, theme, callbacks),
                    AddClusterTab::Yaml => render_yaml_tab(state, theme, callbacks),
                    AddClusterTab::Manual => render_manual_tab(state, theme, callbacks),
                })
                .when(!state.discovered_contexts.is_empty(), |el| {
                    el.child(render_discovered_contexts(state, theme, callbacks))
                })
                .child(render_customization_section(state, theme, callbacks)),
        )
        // Footer
        .child(render_modal_footer(state, theme, callbacks));

    div()
        .id("add-cluster-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(theme.surface.overlay_bg)
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_close_backdrop(window, cx);
        })
        .child(modal_card)
        .into_any_element()
}

fn render_tab_switcher(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let tabs = [
        (AddClusterTab::File, "Kubeconfig File", Icon::FolderOpen),
        (AddClusterTab::Yaml, "Paste YAML", Icon::FileCode),
        (AddClusterTab::Manual, "Manual Endpoint", Icon::Globe),
    ];

    let mut row = div()
        .px_5()
        .pt_2()
        .border_b_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .gap_1();

    for (tab, label, icon) in tabs {
        let active = state.active_tab == tab;
        let on_select = callbacks.on_select_tab.clone();
        row = row.child(
            div()
                .id(SharedString::from(format!("add-cluster-tab-{label}")))
                .px_3()
                .py_2()
                .border_b_2()
                .border_color(if active {
                    theme.accent.accent
                } else {
                    rgba_hex(0x000000, 0.0)
                })
                .flex()
                .items_center()
                .gap_1p5()
                .cursor_pointer()
                .text_xs()
                .font_weight(if active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if active {
                    theme.text.primary
                } else {
                    theme.text.muted
                })
                .hover(|s| s.text_color(theme.text.primary))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_select(tab, window, cx);
                })
                .child(render_icon(
                    icon,
                    px(13.0),
                    if active {
                        theme.accent.accent
                    } else {
                        theme.text.muted
                    },
                ))
                .child(label),
        );
    }

    row.into_any_element()
}

fn render_file_tab(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let on_browse = callbacks.on_browse_file.clone();
    let file_label = state
        .file_path
        .as_deref()
        .unwrap_or("No kubeconfig file selected");

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child("Select a kubeconfig file from disk to import its contexts."),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    theme
                        .style_control(div())
                        .id("add-cluster-browse-file-btn")
                        .px_3()
                        .py_1p5()
                        .bg(theme.surface.bg_hover)
                        .border_1()
                        .border_color(theme.border.strong)
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .hover(|s| s.bg(theme.surface.bg_active))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_browse(window, cx);
                        })
                        .child(render_icon(Icon::FolderOpen, px(13.0), theme.accent.accent))
                        .child("Browse Kubeconfig..."),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_xs()
                        .text_color(if state.file_path.is_some() {
                            theme.text.primary
                        } else {
                            theme.text.muted
                        })
                        .truncate()
                        .child(file_label.to_string()),
                ),
        )
        .when_some(state.file_error.clone(), |el, err| {
            el.child(render_inline_error(&err, theme))
        })
        .into_any_element()
}

fn render_yaml_tab(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let is_focused = state.focused_field == Some(AddClusterFieldId::YamlContent);
    let on_focus = callbacks.on_focus_field.clone();
    let on_paste = callbacks.on_paste_yaml.clone();
    let on_clear = callbacks.on_clear_yaml.clone();
    let is_empty = state.yaml_content.is_empty();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child("Kubeconfig YAML"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            theme
                                .style_control(div())
                                .id("add-cluster-paste-yaml-btn")
                                .px_2()
                                .py_1()
                                .bg(theme.surface.bg_hover)
                                .border_1()
                                .border_color(theme.border.default)
                                .flex()
                                .items_center()
                                .gap_1()
                                .cursor_pointer()
                                .text_xs()
                                .text_color(theme.text.primary)
                                .hover(|s| s.bg(theme.surface.bg_active))
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_paste(window, cx);
                                })
                                .child(render_icon(Icon::ClipboardPaste, px(12.0), theme.text.secondary))
                                .child("Paste from Clipboard"),
                        )
                        .when(!is_empty, |el| {
                            el.child(
                                theme
                                    .style_control(div())
                                    .id("add-cluster-clear-yaml-btn")
                                    .px_2()
                                    .py_1()
                                    .bg(theme.surface.bg_hover)
                                    .border_1()
                                    .border_color(theme.border.default)
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(theme.text.muted)
                                    .hover(|s| s.text_color(theme.text.primary))
                                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                        window.prevent_default();
                                        cx.stop_propagation();
                                        on_clear(window, cx);
                                    })
                                    .child("Clear"),
                            )
                        }),
                ),
        )
        .child(
            theme
                .style_control(div())
                .id("add-cluster-yaml-editor")
                .h(px(160.0))
                .p_2p5()
                .bg(theme.surface.bg_app)
                .border_1()
                .border_color(if is_focused {
                    theme.accent.accent
                } else {
                    theme.border.default
                })
                .overflow_y_scroll()
                .cursor_text()
                .font_family(theme.typography.mono_family.clone())
                .text_xs()
                .text_color(if is_empty {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_focus(AddClusterFieldId::YamlContent, window, cx);
                })
                .child(
                    div()
                        .flex()
                        .items_start()
                        .child(if is_empty {
                            "Paste kubeconfig YAML here (apiVersion: v1 ...)".to_string()
                        } else {
                            state.yaml_content.clone()
                        })
                        .when(is_focused, |el| el.child(render_caret(theme))),
                ),
        )
        .when_some(state.yaml_error.clone(), |el, err| {
            el.child(render_inline_error(&err, theme))
        })
        .into_any_element()
}

fn render_manual_tab(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let auth_options = [
        (ManualAuthType::Token, "Token"),
        (ManualAuthType::Certificate, "Certificate"),
        (ManualAuthType::Basic, "Basic"),
        (ManualAuthType::None, "None"),
    ];
    let on_toggle_tls = callbacks.on_toggle_insecure_tls.clone();

    let mut container = div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .gap_3()
                .child(div().flex_1().child(render_text_input(
                    "add-cluster-field-cluster-name",
                    "Cluster Name *",
                    "e.g. prod-eks",
                    AddClusterFieldId::ClusterName,
                    &state.manual.cluster_name,
                    state.focused_field,
                    theme,
                    callbacks,
                )))
                .child(div().flex_1().child(render_text_input(
                    "add-cluster-field-context-name",
                    "Context Name",
                    "Defaults to cluster name",
                    AddClusterFieldId::ContextName,
                    &state.manual.context_name,
                    state.focused_field,
                    theme,
                    callbacks,
                ))),
        )
        .child(render_text_input(
            "add-cluster-field-server-url",
            "API Server URL *",
            "https://api.cluster.example.com:6443",
            AddClusterFieldId::ServerUrl,
            &state.manual.server_url,
            state.focused_field,
            theme,
            callbacks,
        ))
        // Auth type selector
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child("Authentication"),
                )
                .child({
                    let mut row = div().flex().items_center().gap_1p5();
                    for (auth, label) in auth_options {
                        let active = state.manual.auth_type == auth;
                        let on_auth = callbacks.on_select_auth_type.clone();
                        row = row.child(
                            theme
                                .style_control(div())
                                .id(SharedString::from(format!("add-cluster-auth-{label}")))
                                .px_2p5()
                                .py_1()
                                .border_1()
                                .border_color(if active {
                                    theme.accent.accent
                                } else {
                                    theme.border.default
                                })
                                .bg(if active {
                                    theme.accent.soft
                                } else {
                                    theme.surface.bg_app
                                })
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(if active {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if active {
                                    theme.text.primary
                                } else {
                                    theme.text.secondary
                                })
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_auth(auth, window, cx);
                                })
                                .child(label),
                        );
                    }
                    row
                }),
        );

    match state.manual.auth_type {
        ManualAuthType::Token => {
            container = container.child(render_text_input(
                "add-cluster-field-token",
                "Bearer Token *",
                "eyJhbGciOi...",
                AddClusterFieldId::Token,
                state.manual.token.as_deref().unwrap_or(""),
                state.focused_field,
                theme,
                callbacks,
            ));
        }
        ManualAuthType::Certificate => {
            container = container
                .child(render_text_input(
                    "add-cluster-field-client-cert",
                    "Client Certificate (PEM or Base64) *",
                    "-----BEGIN CERTIFICATE-----",
                    AddClusterFieldId::ClientCert,
                    state.manual.client_cert_data.as_deref().unwrap_or(""),
                    state.focused_field,
                    theme,
                    callbacks,
                ))
                .child(render_text_input(
                    "add-cluster-field-client-key",
                    "Client Key (PEM or Base64) *",
                    "Paste PEM or base64 client key",
                    AddClusterFieldId::ClientKey,
                    state.manual.client_key_data.as_deref().unwrap_or(""),
                    state.focused_field,
                    theme,
                    callbacks,
                ));
        }
        ManualAuthType::Basic => {
            container = container.child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().flex_1().child(render_text_input(
                        "add-cluster-field-username",
                        "Username *",
                        "admin",
                        AddClusterFieldId::Username,
                        state.manual.username.as_deref().unwrap_or(""),
                        state.focused_field,
                        theme,
                        callbacks,
                    )))
                    .child(div().flex_1().child(render_text_input(
                        "add-cluster-field-password",
                        "Password *",
                        "••••••••",
                        AddClusterFieldId::Password,
                        state.manual.password.as_deref().unwrap_or(""),
                        state.focused_field,
                        theme,
                        callbacks,
                    ))),
            );
        }
        ManualAuthType::None => {}
    }

    container = container
        .child(
            div()
                .id("add-cluster-tls-toggle")
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_toggle_tls(window, cx);
                })
                .child(render_checkbox(state.manual.insecure_skip_tls_verify, theme))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text.secondary)
                        .child("Skip TLS certificate verification (insecure)"),
                ),
        )
        .when(!state.manual.insecure_skip_tls_verify, |el| {
            el.child(render_text_input(
                "add-cluster-field-ca-cert",
                "CA Certificate (Optional PEM or Base64)",
                "-----BEGIN CERTIFICATE-----",
                AddClusterFieldId::CaCert,
                state.manual.ca_cert_data.as_deref().unwrap_or(""),
                state.focused_field,
                theme,
                callbacks,
            ))
        })
        .when_some(state.manual_error.clone(), |el, err| {
            el.child(render_inline_error(&err, theme))
        });

    container.into_any_element()
}

fn render_discovered_contexts(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let on_select_all = callbacks.on_select_all_contexts.clone();
    let on_deselect_all = callbacks.on_deselect_all_contexts.clone();

    let mut list = div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.secondary)
                        .child(format!(
                            "Discovered Contexts ({})",
                            state.discovered_contexts.len()
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .id("add-cluster-select-all-btn")
                                .text_xs()
                                .text_color(theme.accent.accent)
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_select_all(window, cx);
                                })
                                .child("Select All"),
                        )
                        .child(
                            div()
                                .id("add-cluster-deselect-all-btn")
                                .text_xs()
                                .text_color(theme.text.muted)
                                .cursor_pointer()
                                .hover(|s| s.text_color(theme.text.primary))
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_deselect_all(window, cx);
                                })
                                .child("Deselect All"),
                        ),
                ),
        );

    for ctx in &state.discovered_contexts {
        let existing = state.is_context_existing(&ctx.name);
        let selected = state.selected_contexts.iter().any(|c| c == &ctx.name);
        let ctx_name = ctx.name.clone();
        let on_toggle = callbacks.on_toggle_context.clone();

        list = list.child(
            theme
                .style_control(div())
                .id(SharedString::from(format!("add-cluster-ctx-{}", ctx.name)))
                .px_3()
                .py_2()
                .border_1()
                .border_color(if selected {
                    theme.accent.accent
                } else {
                    theme.border.default
                })
                .bg(if selected {
                    theme.accent.soft
                } else {
                    theme.surface.bg_app
                })
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .when(!existing, |el| {
                    el.cursor_pointer().on_mouse_down(
                        MouseButton::Left,
                        move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_toggle(ctx_name.clone(), window, cx);
                        },
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .flex_1()
                        .child(render_checkbox(selected, theme))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if existing {
                                                    theme.text.muted
                                                } else {
                                                    theme.text.primary
                                                })
                                                .truncate()
                                                .child(ctx.name.clone()),
                                        )
                                        .when(ctx.is_current_context, |el| {
                                            el.child(render_mini_badge(
                                                "Current",
                                                theme.accent.accent,
                                                theme.accent.soft,
                                            ))
                                        })
                                        .when(existing, |el| {
                                            el.child(render_mini_badge(
                                                "Already exists",
                                                theme.semantic.warning.solid,
                                                theme.semantic.warning.soft,
                                            ))
                                        }),
                                )
                                .when_some(ctx.server_url.clone(), |el, url| {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text.muted)
                                            .truncate()
                                            .child(url),
                                    )
                                }),
                        ),
                )
                .child(render_mini_badge(
                    &ctx.auth_type,
                    theme.text.secondary,
                    theme.surface.bg_hover,
                )),
        );
    }

    list.into_any_element()
}

fn render_customization_section(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let on_toggle = callbacks.on_toggle_customization.clone();
    let on_add_tag = callbacks.on_add_tag.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .id("add-cluster-customization-toggle")
                .flex()
                .items_center()
                .gap_1p5()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_toggle(window, cx);
                })
                .child(render_icon(
                    if state.customization_expanded {
                        Icon::ChevronDown
                    } else {
                        Icon::ChevronRight
                    },
                    px(13.0),
                    theme.text.muted,
                ))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.secondary)
                        .child("Cluster Customization (Optional)"),
                ),
        )
        .when(state.customization_expanded, |el| {
            let mut swatches = div().flex().items_center().gap_1p5();
            for (name, hex) in CLUSTER_COLOR_PRESETS {
                let selected = state.color.as_deref() == Some(*hex);
                let on_color = callbacks.on_select_color.clone();
                let hex_str = (*hex).to_string();
                let color = parse_hex_color(hex).unwrap_or(theme.accent.accent);
                swatches = swatches.child(
                    div()
                        .id(SharedString::from(format!("add-cluster-color-{name}")))
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .bg(color)
                        .border_2()
                        .border_color(if selected {
                            theme.text.primary
                        } else {
                            rgba_hex(0x000000, 0.0)
                        })
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_color(Some(hex_str.clone()), window, cx);
                        }),
                );
            }
            let on_clear_color = callbacks.on_select_color.clone();
            swatches = swatches.when(state.color.is_some(), |s| {
                s.child(
                    div()
                        .id("add-cluster-color-clear")
                        .px_2()
                        .py_0p5()
                        .text_xs()
                        .text_color(theme.text.muted)
                        .cursor_pointer()
                        .hover(|h| h.text_color(theme.text.primary))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_clear_color(None, window, cx);
                        })
                        .child("Clear"),
                )
            });

            let mut tag_chips = div().flex().flex_wrap().items_center().gap_1p5();
            for tag in &state.tags {
                let tag_val = tag.clone();
                let on_rm = callbacks.on_remove_tag.clone();
                tag_chips = tag_chips.child(
                    theme
                        .style_control(div())
                        .px_2()
                        .py_0p5()
                        .bg(theme.surface.bg_hover)
                        .border_1()
                        .border_color(theme.border.default)
                        .flex()
                        .items_center()
                        .gap_1()
                        .text_xs()
                        .text_color(theme.text.primary)
                        .child(tag.clone())
                        .child(
                            div()
                                .id(SharedString::from(format!("add-cluster-rm-tag-{tag}")))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_rm(tag_val.clone(), window, cx);
                                })
                                .child(render_icon(Icon::X, px(11.0), theme.text.muted)),
                        ),
                );
            }

            el.child(
                div()
                    .pl_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(render_text_input(
                        "add-cluster-field-alias",
                        "Display Alias",
                        "e.g. Production US-East",
                        AddClusterFieldId::Alias,
                        &state.alias,
                        state.focused_field,
                        theme,
                        callbacks,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.secondary)
                                    .child("Environment Color"),
                            )
                            .child(swatches),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .flex()
                                    .items_end()
                                    .gap_2()
                                    .child(div().flex_1().child(render_text_input(
                                        "add-cluster-field-tag-input",
                                        "Tags",
                                        "Add tag and press Enter",
                                        AddClusterFieldId::TagInput,
                                        &state.tag_input,
                                        state.focused_field,
                                        theme,
                                        callbacks,
                                    )))
                                    .child(
                                        theme
                                            .style_control(div())
                                            .id("add-cluster-add-tag-btn")
                                            .px_2p5()
                                            .py_1p5()
                                            .bg(theme.surface.bg_hover)
                                            .border_1()
                                            .border_color(theme.border.default)
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(theme.text.primary)
                                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                                window.prevent_default();
                                                cx.stop_propagation();
                                                on_add_tag(window, cx);
                                            })
                                            .child("Add"),
                                    ),
                            )
                            .when(!state.tags.is_empty(), |t| t.child(tag_chips)),
                    ),
            )
        })
        .into_any_element()
}

fn render_modal_footer(
    state: &AddClusterModalState,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let on_toggle_switch = callbacks.on_toggle_switch_context.clone();
    let on_cancel = callbacks.on_close.clone();
    let on_submit = callbacks.on_submit.clone();
    let can_submit = state.can_submit();

    div()
        .px_5()
        .py_3()
        .border_t_1()
        .border_color(theme.border.default)
        .flex()
        .flex_col()
        .gap_2p5()
        .when_some(state.submit_error.clone(), |el, err| {
            el.child(render_inline_error(&err, theme))
        })
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .id("add-cluster-switch-context-toggle")
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_toggle_switch(window, cx);
                        })
                        .child(render_checkbox(state.switch_context, theme))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text.secondary)
                                .child("Connect to cluster after adding"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            theme
                                .style_control(div())
                                .id("add-cluster-cancel-btn")
                                .px_3()
                                .py_1p5()
                                .bg(theme.surface.bg_hover)
                                .border_1()
                                .border_color(theme.border.default)
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.secondary)
                                .hover(|s| s.text_color(theme.text.primary))
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_cancel(window, cx);
                                })
                                .child("Cancel"),
                        )
                        .child(
                            theme
                                .style_control(div())
                                .id("add-cluster-submit-btn")
                                .px_3p5()
                                .py_1p5()
                                .bg(if can_submit {
                                    theme.accent.accent
                                } else {
                                    theme.surface.bg_hover
                                })
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(if can_submit {
                                    rgb_hex(0xffffff)
                                } else {
                                    theme.text.disabled
                                })
                                .when(can_submit, |el| {
                                    el.cursor_pointer()
                                        .hover(|s| s.bg(theme.accent.hover))
                                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                            window.prevent_default();
                                            cx.stop_propagation();
                                            on_submit(window, cx);
                                        })
                                })
                                .child(state.submit_label()),
                        ),
                ),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_text_input(
    id: &'static str,
    label: &'static str,
    placeholder: &'static str,
    field: AddClusterFieldId,
    value: &str,
    focused_field: Option<AddClusterFieldId>,
    theme: &OrbitTheme,
    callbacks: &AddClusterModalCallbacks,
) -> AnyElement {
    let is_focused = focused_field == Some(field);
    let on_focus = callbacks.on_focus_field.clone();
    let is_empty = value.is_empty();
    let display = if field == AddClusterFieldId::Password && !is_empty {
        "•".repeat(value.chars().count())
    } else if is_empty {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(label),
        )
        .child(
            theme
                .style_control(div())
                .id(id)
                .px_2p5()
                .py_1p5()
                .bg(theme.surface.bg_app)
                .border_1()
                .border_color(if is_focused {
                    theme.accent.accent
                } else {
                    theme.border.default
                })
                .cursor_text()
                .flex()
                .items_center()
                .text_xs()
                .text_color(if is_empty {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_focus(field, window, cx);
                })
                .child(div().truncate().child(display))
                .when(is_focused, |el| el.child(render_caret(theme))),
        )
        .into_any_element()
}

fn render_checkbox(checked: bool, theme: &OrbitTheme) -> AnyElement {
    div()
        .w(px(14.0))
        .h(px(14.0))
        .rounded(px(3.0))
        .border_1()
        .border_color(if checked {
            theme.accent.accent
        } else {
            theme.border.strong
        })
        .bg(if checked {
            theme.accent.accent
        } else {
            theme.surface.bg_app
        })
        .flex()
        .items_center()
        .justify_center()
        .when(checked, |el| {
            el.child(render_icon(Icon::Check, px(10.0), rgb_hex(0xffffff)))
        })
        .into_any_element()
}

fn render_mini_badge(label: &str, text_color: Rgba, bg_color: Rgba) -> AnyElement {
    div()
        .px_1p5()
        .py_0p5()
        .rounded(px(4.0))
        .bg(bg_color)
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_color)
        .child(label.to_string())
        .into_any_element()
}

fn render_inline_error(message: &str, theme: &OrbitTheme) -> AnyElement {
    theme
        .style_control(div())
        .px_3()
        .py_2()
        .bg(theme.semantic.danger.soft)
        .border_1()
        .border_color(theme.semantic.danger.solid)
        .flex()
        .items_center()
        .gap_2()
        .text_xs()
        .text_color(theme.semantic.danger.solid)
        .child(render_icon(
            Icon::CircleAlert,
            px(13.0),
            theme.semantic.danger.solid,
        ))
        .child(div().flex_1().child(message.to_string()))
        .into_any_element()
}

fn render_caret(theme: &OrbitTheme) -> AnyElement {
    div()
        .w(px(1.5))
        .h(px(13.0))
        .ml_0p5()
        .bg(theme.accent.accent)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_cluster_color_presets_match_valid_hex_swatches() {
        assert_eq!(CLUSTER_COLOR_PRESETS.len(), 8);
        for (name, hex) in CLUSTER_COLOR_PRESETS {
            assert!(!name.is_empty());
            assert!(hex.starts_with('#') && hex.len() == 7, "invalid hex: {hex}");
            assert!(
                parse_hex_color(hex).is_some(),
                "failed to parse preset {name}: {hex}"
            );
        }
    }

    #[test]
    fn test_add_cluster_submit_label_and_can_submit_presentation() {
        let mut state = AddClusterModalState::default();
        state.open(&[]);
        assert!(!state.can_submit());
        assert_eq!(state.submit_label(), "Add Cluster");

        state.set_tab(AddClusterTab::Yaml);
        state.set_yaml_content(
            r#"
apiVersion: v1
kind: Config
clusters:
  - name: c1
    cluster:
      server: https://10.0.0.1:6443
  - name: c2
    cluster:
      server: https://10.0.0.2:6443
contexts:
  - name: ctx-1
    context:
      cluster: c1
      user: u1
  - name: ctx-2
    context:
      cluster: c2
      user: u2
users:
  - name: u1
    user:
      token: t1
  - name: u2
    user:
      token: t2
"#
            .to_string(),
        );
        assert!(state.can_submit());
        assert_eq!(state.submit_label(), "Add 2 Clusters");

        state.is_submitting = true;
        assert!(!state.can_submit());
        assert_eq!(state.submit_label(), "Adding...");
    }
}
