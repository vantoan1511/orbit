use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::component::{
    Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    form::{field, v_form},
    input::{
        AutoClosingPair, BracketPair, Editor, EditorState, IndentationRules, Input, InputState,
        SyntaxContext, TabSize, language_config::LanguageConfig, set_language_config,
    },
    switch::Switch,
};
use gpui::{
    AnyElement, App, Entity, Focusable as _, FontWeight, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::edit_forms::{
    EditFormModel, EditMode, KeyValueEntry, apply_edit_form, extract_edit_form,
    format_editable_resource_yaml, merge_preserved_server_fields, transition_edit_mode,
    validate_edit_form,
};
use orbit_core::kubernetes::raw::parse_resource_yaml;
use serde_json::Value;

use crate::icons::render_icon;
use crate::navigation::{EditTabTarget, WorkspaceTab};
use crate::theme::{OrbitTheme, apply_text_step};

const SECRET_TYPE_PRESETS: &[&str] = &[
    "Opaque",
    "kubernetes.io/tls",
    "kubernetes.io/dockerconfigjson",
    "kubernetes.io/basic-auth",
];

const PV_RECLAIM_POLICIES: &[&str] = &["Retain", "Delete", "Recycle"];

const PV_ACCESS_MODES: &[&str] = &[
    "ReadWriteOnce",
    "ReadOnlyMany",
    "ReadWriteMany",
    "ReadWriteOncePod",
];

pub fn yaml_language_config() -> LanguageConfig {
    let mut config = LanguageConfig::default()
        .brackets([BracketPair::new("{", "}"), BracketPair::new("[", "]")])
        .auto_closing_pairs(
            [
                AutoClosingPair::new("{", "}"),
                AutoClosingPair::new("[", "]"),
                AutoClosingPair::new("\"", "\""),
                AutoClosingPair::new("'", "'"),
            ]
            .into_iter()
            .map(|pair| pair.not_in([SyntaxContext::String, SyntaxContext::Comment])),
        );

    if let (Ok(increase), Ok(decrease)) = (
        regex::Regex::new(r"(?::|\|[+-]?|>[+-]?|[\{\[])\s*$"),
        regex::Regex::new(r"^\s*[\}\]]"),
    ) {
        config = config.indentation_rules(IndentationRules::new(increase, decrease));
    }

    config
}

pub fn register_yaml_language_config(cx: &mut App) {
    set_language_config("yaml", yaml_language_config(), cx);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditTabStatus {
    Loading,
    Ready,
    Saving,
    LoadError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditFormSubTab {
    #[default]
    Data,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingCloseTabsAction {
    CloseSingle(WorkspaceTab),
    CloseOthers(WorkspaceTab),
    CloseRight(WorkspaceTab),
    CloseAll,
}

#[derive(Debug, Clone)]
pub struct EditSessionState {
    pub target: EditTabTarget,
    pub mode: EditMode,
    pub status: EditTabStatus,
    pub active_sub_tab: EditFormSubTab,
    pub load_request_id: u64,
    pub save_request_id: Option<u64>,
    pub initial_yaml: String,
    pub yaml_content: String,
    pub raw_data: Option<Value>,
    pub form_model: Option<EditFormModel>,
    pub yaml_parse_error: Option<String>,
    pub inline_error: Option<String>,
    pub form_validation_error: Option<String>,
    pub form_structure_revision: u64,
}

impl EditSessionState {
    pub fn new(target: EditTabTarget, load_request_id: u64) -> Self {
        Self {
            target,
            mode: EditMode::Form,
            status: EditTabStatus::Loading,
            active_sub_tab: EditFormSubTab::Data,
            load_request_id,
            save_request_id: None,
            initial_yaml: String::new(),
            yaml_content: String::new(),
            raw_data: None,
            form_model: None,
            yaml_parse_error: None,
            inline_error: None,
            form_validation_error: None,
            form_structure_revision: 0,
        }
    }

    pub fn begin_reload(&mut self, request_id: u64) {
        self.load_request_id = request_id;
        self.save_request_id = None;
        self.status = EditTabStatus::Loading;
        self.inline_error = None;
        self.yaml_parse_error = None;
        self.form_validation_error = None;
    }

    pub fn apply_loaded(&mut self, request_id: u64, raw_json: Value, yaml: String) -> bool {
        if self.load_request_id != request_id {
            return false;
        }
        let editable_yaml = format_editable_resource_yaml(&raw_json).unwrap_or(yaml);
        let form = extract_edit_form(&self.target.kind, &raw_json);
        self.form_validation_error = validate_edit_form(&form);
        self.raw_data = Some(raw_json);
        self.form_model = Some(form);
        self.initial_yaml = editable_yaml.clone();
        self.yaml_content = editable_yaml;
        self.status = EditTabStatus::Ready;
        self.yaml_parse_error = None;
        self.inline_error = None;
        self.form_structure_revision = self.form_structure_revision.wrapping_add(1);
        true
    }

    pub fn apply_load_failed(&mut self, request_id: u64, message: String) -> bool {
        if self.load_request_id != request_id {
            return false;
        }
        self.status = EditTabStatus::LoadError(message);
        true
    }

    pub fn begin_save(&mut self, save_request_id: u64) -> Result<Value, String> {
        if self.status != EditTabStatus::Ready {
            return Err("Resource is not ready to save.".to_string());
        }

        let payload = match self.mode {
            EditMode::Yaml => match parse_resource_yaml(&self.yaml_content) {
                Ok(parsed) => {
                    let merged = match self.raw_data.as_ref() {
                        Some(snapshot) => merge_preserved_server_fields(&parsed, snapshot),
                        None => parsed,
                    };
                    self.raw_data = Some(merged.clone());
                    self.yaml_parse_error = None;
                    merged
                }
                Err(err) => {
                    self.yaml_parse_error = Some(err.clone());
                    self.inline_error = Some(err.clone());
                    return Err(err);
                }
            },
            EditMode::Form => {
                if let Some(form) = self.form_model.as_ref() {
                    if let Some(err) = validate_edit_form(form) {
                        self.form_validation_error = Some(err.clone());
                        self.inline_error = Some(err.clone());
                        return Err(err);
                    }
                    let base = self.raw_data.as_ref().ok_or_else(|| {
                        "No resource manifest loaded for editing.".to_string()
                    })?;
                    let updated = apply_edit_form(base, form);
                    self.raw_data = Some(updated.clone());
                    if let Ok(yaml) = format_editable_resource_yaml(&updated) {
                        self.yaml_content = yaml;
                    }
                    updated
                } else {
                    self.raw_data.clone().ok_or_else(|| {
                        "No resource manifest loaded for editing.".to_string()
                    })?
                }
            }
        };

        self.status = EditTabStatus::Saving;
        self.save_request_id = Some(save_request_id);
        self.inline_error = None;
        Ok(payload)
    }

    pub fn apply_saved(&mut self, request_id: u64, raw_json: Value, yaml: String) -> bool {
        if self.save_request_id != Some(request_id) {
            return false;
        }
        let editable_yaml = format_editable_resource_yaml(&raw_json).unwrap_or(yaml);
        let form = extract_edit_form(&self.target.kind, &raw_json);
        self.form_validation_error = validate_edit_form(&form);
        self.raw_data = Some(raw_json);
        self.form_model = Some(form);
        self.initial_yaml = editable_yaml.clone();
        self.yaml_content = editable_yaml;
        self.save_request_id = None;
        self.status = EditTabStatus::Ready;
        self.yaml_parse_error = None;
        self.inline_error = None;
        self.form_structure_revision = self.form_structure_revision.wrapping_add(1);
        true
    }

    pub fn apply_save_failed(&mut self, request_id: u64, message: String, is_conflict: bool) -> bool {
        if self.save_request_id != Some(request_id) {
            return false;
        }
        self.save_request_id = None;
        self.status = EditTabStatus::Ready;
        self.inline_error = Some(if is_conflict {
            format!("Conflict (HTTP 409): {message}")
        } else {
            message
        });
        true
    }

    pub fn switch_mode(&mut self, target_mode: EditMode) -> Result<(), String> {
        if self.mode == target_mode {
            return Ok(());
        }
        let base = self
            .raw_data
            .as_ref()
            .ok_or_else(|| "Resource is still loading.".to_string())?;
        match transition_edit_mode(target_mode, &self.target.kind, base, &self.yaml_content) {
            Ok(transition) => {
                self.mode = transition.mode;
                self.raw_data = Some(transition.raw_data);
                self.yaml_content = transition.yaml_content;
                if let Some(form) = transition.form_model {
                    self.form_validation_error = validate_edit_form(&form);
                    self.form_model = Some(form);
                    self.form_structure_revision = self.form_structure_revision.wrapping_add(1);
                }
                self.yaml_parse_error = None;
                self.inline_error = None;
                Ok(())
            }
            Err(err) => {
                self.mode = EditMode::Yaml;
                self.yaml_parse_error = Some(err.clone());
                self.inline_error = Some(err.clone());
                Err(err)
            }
        }
    }

    pub fn update_yaml_from_editor(&mut self, new_yaml: String) {
        self.yaml_content = new_yaml;
        match parse_resource_yaml(&self.yaml_content) {
            Ok(parsed) => {
                let merged = match self.raw_data.as_ref() {
                    Some(snapshot) => merge_preserved_server_fields(&parsed, snapshot),
                    None => parsed,
                };
                self.raw_data = Some(merged);
                self.yaml_parse_error = None;
                self.inline_error = None;
            }
            Err(err) => {
                self.yaml_parse_error = Some(err);
            }
        }
    }

    pub fn update_form_model<F>(&mut self, mutator: F, structural_change: bool)
    where
        F: FnOnce(&mut EditFormModel),
    {
        let Some(form) = self.form_model.as_mut() else {
            return;
        };
        mutator(form);
        self.form_validation_error = validate_edit_form(form);
        if let Some(base) = self.raw_data.as_ref() {
            let updated = apply_edit_form(base, form);
            if let Ok(yaml) = format_editable_resource_yaml(&updated) {
                self.yaml_content = yaml;
            }
            self.raw_data = Some(updated);
        }
        self.yaml_parse_error = None;
        self.inline_error = None;
        if structural_change {
            self.form_structure_revision = self.form_structure_revision.wrapping_add(1);
        }
    }

    pub fn is_dirty(&self) -> bool {
        matches!(self.status, EditTabStatus::Ready | EditTabStatus::Saving)
            && self.yaml_content != self.initial_yaml
    }

    pub fn can_save(&self) -> bool {
        if self.status != EditTabStatus::Ready || !self.is_dirty() || self.raw_data.is_none() {
            return false;
        }
        match self.mode {
            EditMode::Yaml => self.yaml_parse_error.is_none(),
            EditMode::Form => self.form_validation_error.is_none(),
        }
    }

    pub fn active_error_banner(&self) -> Option<&str> {
        self.inline_error
            .as_deref()
            .or(self.yaml_parse_error.as_deref())
            .or(self.form_validation_error.as_deref())
    }
}

pub struct EditTabEntities {
    pub yaml_editor: Entity<EditorState>,
    pub form_inputs: HashMap<String, Entity<InputState>>,
    pub subscriptions: Vec<Subscription>,
    pub suppress_events: Rc<Cell<bool>>,
    pub synced_editor_yaml: String,
    pub synced_form_revision: u64,
}

impl EditTabEntities {
    pub fn new(initial_yaml: &str, window: &mut Window, cx: &mut App) -> Self {
        let yaml_editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("yaml")
                .tab_size(TabSize {
                    tab_size: 2,
                    hard_tabs: false,
                })
                .line_number(true)
                .searchable(true)
                .default_value(initial_yaml)
        });
        Self {
            yaml_editor,
            form_inputs: HashMap::new(),
            subscriptions: Vec::new(),
            suppress_events: Rc::new(Cell::new(false)),
            synced_editor_yaml: initial_yaml.to_string(),
            synced_form_revision: u64::MAX,
        }
    }

    pub fn sync_yaml_editor(&mut self, yaml: &str, window: &mut Window, cx: &mut App) {
        if self.synced_editor_yaml == yaml {
            return;
        }
        self.suppress_events.set(true);
        self.yaml_editor.update(cx, |state, cx| {
            state.set_value(yaml, window, cx);
        });
        self.suppress_events.set(false);
        self.synced_editor_yaml = yaml.to_string();
    }

    pub fn is_any_input_focused(&self, window: &Window, cx: &App) -> bool {
        if self.yaml_editor.read(cx).focus_handle(cx).is_focused(window) {
            return true;
        }
        self.form_inputs
            .values()
            .any(|input| input.read(cx).focus_handle(cx).is_focused(window))
    }
}

pub struct EditTabEntry {
    pub session: EditSessionState,
    pub entities: Option<EditTabEntities>,
}

impl EditTabEntry {
    pub fn new(target: EditTabTarget, load_request_id: u64) -> Self {
        Self {
            session: EditSessionState::new(target, load_request_id),
            entities: None,
        }
    }
}

pub type EditActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type EditModeToggleCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub type EditSubTabCallback = Rc<dyn Fn(EditFormSubTab, &mut Window, &mut App)>;
pub type EditFormMutatorCallback =
    Rc<dyn Fn(Box<dyn FnOnce(&mut EditFormModel)>, bool, &mut Window, &mut App)>;

pub struct EditViewCallbacks {
    pub on_cancel: EditActionCallback,
    pub on_save: EditActionCallback,
    pub on_retry_load: EditActionCallback,
    pub on_toggle_yaml_mode: EditModeToggleCallback,
    pub on_select_sub_tab: EditSubTabCallback,
    pub on_mutate_form: EditFormMutatorCallback,
}

pub fn collect_form_input_specs(form: &EditFormModel) -> Vec<(String, String, &'static str)> {
    let mut specs = Vec::new();
    match form {
        EditFormModel::ConfigMap(cm) => {
            push_kv_specs(&mut specs, "data", &cm.data, "Key", "Value");
            push_kv_specs(&mut specs, "labels", &cm.labels, "Label key", "Label value");
            push_kv_specs(
                &mut specs,
                "annotations",
                &cm.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::Secret(secret) => {
            specs.push((
                "secret:type".to_string(),
                secret.secret_type.clone(),
                "e.g. Opaque",
            ));
            push_kv_specs(&mut specs, "data", &secret.data, "Key", "Plain text value");
            push_kv_specs(
                &mut specs,
                "labels",
                &secret.labels,
                "Label key",
                "Label value",
            );
            push_kv_specs(
                &mut specs,
                "annotations",
                &secret.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::Namespace(ns) => {
            push_list_specs(&mut specs, "finalizers", &ns.finalizers, "e.g. kubernetes");
            push_kv_specs(&mut specs, "labels", &ns.labels, "Label key", "Label value");
            push_kv_specs(
                &mut specs,
                "annotations",
                &ns.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::PersistentVolume(pv) => {
            specs.push((
                "pv:capacity".to_string(),
                pv.capacity.clone(),
                "e.g. 10Gi, 500Mi",
            ));
            specs.push((
                "pv:storage_class_name".to_string(),
                pv.storage_class_name.clone(),
                "StorageClass name",
            ));
            push_list_specs(
                &mut specs,
                "mount_options",
                &pv.mount_options,
                "e.g. hard, nfsvers=4.1",
            );
            push_list_specs(&mut specs, "finalizers", &pv.finalizers, "e.g. kubernetes.io/pv-protection");
            push_kv_specs(&mut specs, "labels", &pv.labels, "Label key", "Label value");
            push_kv_specs(
                &mut specs,
                "annotations",
                &pv.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::PersistentVolumeClaim(pvc) => {
            specs.push((
                "pvc:storage_request".to_string(),
                pvc.storage_request.clone(),
                "e.g. 20Gi (expansion only)",
            ));
            push_kv_specs(
                &mut specs,
                "labels",
                &pvc.labels,
                "Label key",
                "Label value",
            );
            push_kv_specs(
                &mut specs,
                "annotations",
                &pvc.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::StorageClass(sc) => {
            push_list_specs(
                &mut specs,
                "mount_options",
                &sc.mount_options,
                "e.g. debug, noatime",
            );
            push_kv_specs(&mut specs, "labels", &sc.labels, "Label key", "Label value");
            push_kv_specs(
                &mut specs,
                "annotations",
                &sc.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
        EditFormModel::MetadataFallback(meta) => {
            push_kv_specs(
                &mut specs,
                "labels",
                &meta.labels,
                "Label key",
                "Label value",
            );
            push_kv_specs(
                &mut specs,
                "annotations",
                &meta.annotations,
                "Annotation key",
                "Annotation value",
            );
        }
    }
    specs
}

fn push_kv_specs(
    specs: &mut Vec<(String, String, &'static str)>,
    group: &str,
    entries: &[KeyValueEntry],
    key_placeholder: &'static str,
    val_placeholder: &'static str,
) {
    for (idx, entry) in entries.iter().enumerate() {
        specs.push((format!("kv:{group}:{idx}:key"), entry.key.clone(), key_placeholder));
        specs.push((
            format!("kv:{group}:{idx}:value"),
            entry.value.clone(),
            val_placeholder,
        ));
    }
}

fn push_list_specs(
    specs: &mut Vec<(String, String, &'static str)>,
    group: &str,
    items: &[String],
    placeholder: &'static str,
) {
    for (idx, item) in items.iter().enumerate() {
        specs.push((format!("list:{group}:{idx}"), item.clone(), placeholder));
    }
}

pub fn apply_input_value_to_form(form: &mut EditFormModel, field_id: &str, value: String) {
    if let Some(rest) = field_id.strip_prefix("kv:") {
        let mut parts = rest.split(':');
        let Some(group) = parts.next() else { return };
        let Some(idx) = parts.next().and_then(|s| s.parse::<usize>().ok()) else {
            return;
        };
        let Some(part) = parts.next() else { return };
        if let Some(entries) = kv_vec_mut(form, group)
            && let Some(entry) = entries.get_mut(idx)
        {
            if part == "key" {
                entry.key = value;
            } else if part == "value" {
                entry.value = value;
            }
        }
        return;
    }

    if let Some(rest) = field_id.strip_prefix("list:") {
        let mut parts = rest.split(':');
        let Some(group) = parts.next() else { return };
        let Some(idx) = parts.next().and_then(|s| s.parse::<usize>().ok()) else {
            return;
        };
        if let Some(items) = list_vec_mut(form, group)
            && let Some(slot) = items.get_mut(idx)
        {
            *slot = value;
        }
        return;
    }

    match (form, field_id) {
        (EditFormModel::Secret(s), "secret:type") => s.secret_type = value,
        (EditFormModel::PersistentVolume(pv), "pv:capacity") => pv.capacity = value,
        (EditFormModel::PersistentVolume(pv), "pv:storage_class_name") => {
            pv.storage_class_name = value;
        }
        (EditFormModel::PersistentVolumeClaim(pvc), "pvc:storage_request") => {
            pvc.storage_request = value;
        }
        _ => {}
    }
}

pub fn render_edit_view(
    session: &EditSessionState,
    entities: Option<&EditTabEntities>,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let is_saving = session.status == EditTabStatus::Saving;
    let is_yaml = session.mode == EditMode::Yaml;
    let is_dirty = session.is_dirty();
    let can_save = session.can_save();
    let kind = session.target.kind.clone();
    let subtitle = session.target.subtitle();

    let header = div()
        .w_full()
        .px_4()
        .py_2p5()
        .flex_shrink_0()
        .border_b_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .min_w(px(0.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(format!("Edit {kind}")),
                        )
                        .when(is_dirty, |el| {
                            el.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_1p5()
                                    .py(px(1.0))
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.semantic.warning.soft)
                                    .child(
                                        div()
                                            .w(px(6.0))
                                            .h(px(6.0))
                                            .rounded_full()
                                            .bg(theme.semantic.warning.solid),
                                    )
                                    .child(
                                        apply_text_step(div(), theme.typography.size_3xs)
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.semantic.warning.solid)
                                            .child("Unsaved"),
                                    ),
                            )
                        }),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font(theme.typography.mono_font(FontWeight::NORMAL))
                        .text_color(theme.text.muted)
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .flex_shrink_0()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(if !is_yaml {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if !is_yaml {
                                    theme.text.primary
                                } else {
                                    theme.text.muted
                                })
                                .child("Form"),
                        )
                        .child(
                            Switch::new("edit-mode-yaml-switch")
                                .checked(is_yaml)
                                .disabled(session.status != EditTabStatus::Ready)
                                .on_click({
                                    let cb = Rc::clone(&callbacks.on_toggle_yaml_mode);
                                    move |checked, window, cx| cb(*checked, window, cx)
                                }),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(if is_yaml {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if is_yaml {
                                    theme.text.primary
                                } else {
                                    theme.text.muted
                                })
                                .child("YAML"),
                        ),
                )
                .child(
                    Button::new("edit-cancel-btn")
                        .outline()
                        .small()
                        .disabled(is_saving)
                        .on_click({
                            let cb = Rc::clone(&callbacks.on_cancel);
                            move |_, window, cx| cb(window, cx)
                        })
                        .child("Cancel"),
                )
                .child(
                    Button::new("edit-save-btn")
                        .primary()
                        .small()
                        .disabled(!can_save)
                        .on_click({
                            let cb = Rc::clone(&callbacks.on_save);
                            move |_, window, cx| cb(window, cx)
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(render_icon(
                                    if is_saving {
                                        Icon::LoaderCircle
                                    } else {
                                        Icon::Save
                                    },
                                    px(13.0),
                                    theme.text.primary,
                                ))
                                .child(if is_saving {
                                    "Saving..."
                                } else {
                                    "Save Changes"
                                }),
                        ),
                ),
        );

    let error_banner = session.active_error_banner().map(|message| {
        div()
            .w_full()
            .px_4()
            .pt_3()
            .flex_shrink_0()
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.danger.soft)
                    .border_1()
                    .border_color(theme.semantic.danger.solid)
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_icon(
                        Icon::AlertTriangle,
                        px(14.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.primary)
                            .child(message.to_string()),
                    ),
            )
    });

    let body = match &session.status {
        EditTabStatus::Loading => div()
            .flex_1()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .child(render_icon(Icon::LoaderCircle, px(16.0), theme.text.muted))
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.secondary)
                    .child(format!("Loading {}...", session.target.kind)),
            )
            .into_any_element(),
        EditTabStatus::LoadError(message) => div()
            .flex_1()
            .w_full()
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_3()
            .child(
                apply_text_step(div(), theme.typography.size_sm)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.semantic.danger.solid)
                    .child(format!("Failed to load {}", session.target.kind)),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.secondary)
                    .child(message.clone()),
            )
            .child(
                Button::new("edit-retry-load-btn")
                    .outline()
                    .small()
                    .on_click({
                        let cb = Rc::clone(&callbacks.on_retry_load);
                        move |_, window, cx| cb(window, cx)
                    })
                    .child("Retry"),
            )
            .into_any_element(),
        EditTabStatus::Ready | EditTabStatus::Saving => match (session.mode, entities) {
            (EditMode::Yaml, Some(ent)) => div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .p_3()
                .child(
                    Editor::new(&ent.yaml_editor)
                        .readonly(is_saving)
                        .size_full(),
                )
                .into_any_element(),
            (EditMode::Form, Some(ent)) => match session.form_model.as_ref() {
                Some(form_model) => render_form_mode_body(
                    session,
                    form_model,
                    &ent.form_inputs,
                    is_saving,
                    theme,
                    callbacks,
                ),
                None => div().flex_1().into_any_element(),
            },
            _ => div().flex_1().into_any_element(),
        },
    };

    div()
        .id(SharedString::from(format!(
            "edit-view-{}",
            session.target.tab_id()
        )))
        .flex_1()
        .min_h(px(0.0))
        .w_full()
        .bg(theme.surface.bg_app)
        .flex()
        .flex_col()
        .child(header)
        .when_some(error_banner, |el, banner| el.child(banner))
        .child(body)
        .into_any_element()
}

fn render_form_mode_body(
    session: &EditSessionState,
    form_model: &EditFormModel,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let content = match form_model {
        EditFormModel::ConfigMap(cm) => {
            let tab_bar = render_sub_tab_bar(
                session.active_sub_tab,
                "Data",
                "Metadata",
                theme,
                &callbacks.on_select_sub_tab,
            );
            let tab_body = match session.active_sub_tab {
                EditFormSubTab::Data => render_two_col_section(
                    "Configuration Data",
                    "Add and edit configuration key-value entries.",
                    theme,
                    render_kv_editor_section(
                        "data",
                        "ConfigMap Data",
                        "Add Entry",
                        &cm.data,
                        inputs,
                        disabled,
                        theme,
                        &callbacks.on_mutate_form,
                    ),
                ),
                EditFormSubTab::Metadata => render_metadata_kv_sections(
                    &cm.labels,
                    &cm.annotations,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ),
            };
            div()
                .flex()
                .flex_col()
                .gap_6()
                .child(tab_bar)
                .child(tab_body)
                .into_any_element()
        }
        EditFormModel::Secret(secret) => {
            let tab_bar = render_sub_tab_bar(
                session.active_sub_tab,
                "Data (Decoded)",
                "Metadata & Settings",
                theme,
                &callbacks.on_select_sub_tab,
            );
            let tab_body = match session.active_sub_tab {
                EditFormSubTab::Data => render_two_col_section(
                    "Secret Payload",
                    "Values are decoded for editing and re-encoded in Base64 on save.",
                    theme,
                    render_kv_editor_section(
                        "data",
                        "Secret Data (Plain Text)",
                        "Add Entry",
                        &secret.data,
                        inputs,
                        disabled,
                        theme,
                        &callbacks.on_mutate_form,
                    ),
                ),
                EditFormSubTab::Metadata => {
                    let type_control = div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(v_form().child(field().label("Secret Type").required(true).child(
                            render_input_or_placeholder(inputs, "secret:type", disabled, theme),
                        )))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_1p5()
                                .children(SECRET_TYPE_PRESETS.iter().map(|preset| {
                                    let preset_str = (*preset).to_string();
                                    let selected = secret.secret_type == *preset;
                                    let mutate = Rc::clone(&callbacks.on_mutate_form);
                                    let mut btn = Button::new(SharedString::from(format!(
                                        "secret-type-preset-{preset}"
                                    )))
                                    .small()
                                    .disabled(disabled)
                                    .on_click(move |_, window, cx| {
                                        let next = preset_str.clone();
                                        mutate(
                                            Box::new(move |model| {
                                                if let EditFormModel::Secret(s) = model {
                                                    s.secret_type = next;
                                                }
                                            }),
                                            true,
                                            window,
                                            cx,
                                        );
                                    })
                                    .child(*preset);
                                    if selected {
                                        btn = btn.primary();
                                    } else {
                                        btn = btn.outline();
                                    }
                                    btn
                                })),
                        )
                        .into_any_element();

                    div()
                        .flex()
                        .flex_col()
                        .gap_8()
                        .child(render_two_col_section(
                            "Secret Configuration",
                            "Secret type determines how Kubernetes validates and mounts credentials.",
                            theme,
                            type_control,
                        ))
                        .child(render_metadata_kv_sections(
                            &secret.labels,
                            &secret.annotations,
                            inputs,
                            disabled,
                            theme,
                            &callbacks.on_mutate_form,
                        ))
                        .into_any_element()
                }
            };
            div()
                .flex()
                .flex_col()
                .gap_6()
                .child(tab_bar)
                .child(tab_body)
                .into_any_element()
        }
        EditFormModel::Namespace(ns) => div()
            .flex()
            .flex_col()
            .gap_8()
            .child(render_two_col_section(
                "Resource Identity",
                "Core identity and lifecycle state. These fields are immutable.",
                theme,
                render_readonly_fields(
                    &[
                        ("Namespace Name", &ns.name),
                        ("Status Phase", &ns.status_phase),
                    ],
                    theme,
                ),
            ))
            .child(render_two_col_section(
                "Configuration",
                "Namespace spec finalizers required before deletion completes.",
                theme,
                render_string_list_section(
                    "finalizers",
                    "Finalizers",
                    "Add Finalizer",
                    &ns.finalizers,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ),
            ))
            .child(render_metadata_kv_sections(
                &ns.labels,
                &ns.annotations,
                inputs,
                disabled,
                theme,
                &callbacks.on_mutate_form,
            ))
            .into_any_element(),
        EditFormModel::PersistentVolume(pv) => {
            let reclaim_buttons = div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(PV_RECLAIM_POLICIES.iter().map(|policy| {
                    let policy_str = (*policy).to_string();
                    let selected = pv.reclaim_policy == *policy;
                    let mutate = Rc::clone(&callbacks.on_mutate_form);
                    let mut btn =
                        Button::new(SharedString::from(format!("pv-reclaim-{policy}")))
                            .small()
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                let next = policy_str.clone();
                                mutate(
                                    Box::new(move |model| {
                                        if let EditFormModel::PersistentVolume(v) = model {
                                            v.reclaim_policy = next;
                                        }
                                    }),
                                    false,
                                    window,
                                    cx,
                                );
                            })
                            .child(*policy);
                    if selected {
                        btn = btn.primary();
                    } else {
                        btn = btn.outline();
                    }
                    btn
                }));

            let access_mode_buttons = div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(PV_ACCESS_MODES.iter().map(|mode| {
                    let mode_str = (*mode).to_string();
                    let selected = pv.access_modes.iter().any(|m| m == mode);
                    let mutate = Rc::clone(&callbacks.on_mutate_form);
                    let mut btn = Button::new(SharedString::from(format!("pv-access-{mode}")))
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            let target = mode_str.clone();
                            mutate(
                                Box::new(move |model| {
                                    if let EditFormModel::PersistentVolume(v) = model {
                                        if let Some(pos) =
                                            v.access_modes.iter().position(|m| *m == target)
                                        {
                                            v.access_modes.remove(pos);
                                        } else {
                                            v.access_modes.push(target);
                                        }
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        })
                        .child(*mode);
                    if selected {
                        btn = btn.primary();
                    } else {
                        btn = btn.outline();
                    }
                    btn
                }));

            let spec_form = div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    v_form()
                        .child(
                            field()
                                .label("Storage Capacity")
                                .description("Positive Kubernetes quantity, e.g. 10Gi or 500Mi.")
                                .required(true)
                                .child(render_input_or_placeholder(
                                    inputs,
                                    "pv:capacity",
                                    disabled,
                                    theme,
                                )),
                        )
                        .child(
                            field()
                                .label("Reclaim Policy")
                                .child(reclaim_buttons),
                        )
                        .child(
                            field()
                                .label("Access Modes")
                                .child(access_mode_buttons),
                        )
                        .child(
                            field()
                                .label("Storage Class Name")
                                .child(render_input_or_placeholder(
                                    inputs,
                                    "pv:storage_class_name",
                                    disabled,
                                    theme,
                                )),
                        ),
                )
                .child(render_string_list_section(
                    "mount_options",
                    "Mount Options",
                    "Add Mount Option",
                    &pv.mount_options,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .child(render_string_list_section(
                    "finalizers",
                    "Finalizers",
                    "Add Finalizer",
                    &pv.finalizers,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element();

            div()
                .flex()
                .flex_col()
                .gap_8()
                .child(render_two_col_section(
                    "Volume Identity (Immutable)",
                    "Cluster-scoped PersistentVolume identity and backing volume source.",
                    theme,
                    render_readonly_fields(
                        &[
                            ("Name", &pv.name),
                            ("Volume Mode", &pv.volume_mode),
                            ("Volume Source", &pv.source_summary),
                            ("Bound Claim", &pv.claim_ref_summary),
                        ],
                        theme,
                    ),
                ))
                .child(render_two_col_section(
                    "Mutable Volume Spec",
                    "Update capacity, reclaim policy, access modes, and mount options.",
                    theme,
                    spec_form,
                ))
                .child(render_metadata_kv_sections(
                    &pv.labels,
                    &pv.annotations,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element()
        }
        EditFormModel::PersistentVolumeClaim(pvc) => {
            let access_modes_joined = pvc.access_modes.join(", ");
            let sc_display = if pvc.storage_class_name.is_empty() {
                "-"
            } else {
                &pvc.storage_class_name
            };
            let vol_display = if pvc.volume_name.is_empty() {
                "-"
            } else {
                &pvc.volume_name
            };
            let expansion_form = v_form()
                .child(
                    field()
                        .label("Storage Request")
                        .description(format!(
                            "Kubernetes allows expanding PVC capacity only (current: {}).",
                            pvc.initial_storage_request
                        ))
                        .required(true)
                        .child(render_input_or_placeholder(
                            inputs,
                            "pvc:storage_request",
                            disabled,
                            theme,
                        )),
                )
                .into_any_element();

            div()
                .flex()
                .flex_col()
                .gap_8()
                .child(render_two_col_section(
                    "Claim Identity (Immutable)",
                    "Bound PVC spec fields cannot be modified after creation.",
                    theme,
                    render_readonly_fields(
                        &[
                            ("Name", &pvc.name),
                            ("Namespace", &pvc.namespace),
                            ("Storage Class", sc_display),
                            ("Bound Volume", vol_display),
                            ("Volume Mode", &pvc.volume_mode),
                            ("Access Modes", &access_modes_joined),
                        ],
                        theme,
                    ),
                ))
                .child(render_two_col_section(
                    "Volume Expansion",
                    "Increase the requested storage size for this claim.",
                    theme,
                    expansion_form,
                ))
                .child(render_metadata_kv_sections(
                    &pvc.labels,
                    &pvc.annotations,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element()
        }
        EditFormModel::StorageClass(sc) => {
            let settings_form = div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    v_form()
                        .child(
                            field()
                                .label("Default StorageClass")
                                .description(
                                    "Sets the storageclass.kubernetes.io/is-default-class annotation.",
                                )
                                .child(
                                    Switch::new("sc-is-default-switch")
                                        .checked(sc.is_default_class)
                                        .disabled(disabled)
                                        .on_click({
                                            let mutate = Rc::clone(&callbacks.on_mutate_form);
                                            move |checked, window, cx| {
                                                let val = *checked;
                                                mutate(
                                                    Box::new(move |model| {
                                                        if let EditFormModel::StorageClass(v) =
                                                            model
                                                        {
                                                            v.is_default_class = val;
                                                        }
                                                    }),
                                                    false,
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }),
                                ),
                        )
                        .child(
                            field()
                                .label("Allow Volume Expansion")
                                .description(
                                    "Allows PersistentVolumeClaims using this class to expand.",
                                )
                                .child(
                                    Switch::new("sc-allow-expansion-switch")
                                        .checked(sc.allow_volume_expansion)
                                        .disabled(disabled)
                                        .on_click({
                                            let mutate = Rc::clone(&callbacks.on_mutate_form);
                                            move |checked, window, cx| {
                                                let val = *checked;
                                                mutate(
                                                    Box::new(move |model| {
                                                        if let EditFormModel::StorageClass(v) =
                                                            model
                                                        {
                                                            v.allow_volume_expansion = val;
                                                        }
                                                    }),
                                                    false,
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }),
                                ),
                        ),
                )
                .child(render_string_list_section(
                    "mount_options",
                    "Mount Options",
                    "Add Mount Option",
                    &sc.mount_options,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element();

            div()
                .flex()
                .flex_col()
                .gap_8()
                .child(render_two_col_section(
                    "StorageClass Identity (Immutable)",
                    "Provisioner, reclaim policy, and binding mode are immutable.",
                    theme,
                    render_readonly_fields(
                        &[
                            ("Name", &sc.name),
                            ("Provisioner", &sc.provisioner),
                            ("Reclaim Policy", &sc.reclaim_policy),
                            ("Volume Binding Mode", &sc.volume_binding_mode),
                        ],
                        theme,
                    ),
                ))
                .child(render_two_col_section(
                    "Mutable StorageClass Settings",
                    "Top-level StorageClass options and default-class annotation.",
                    theme,
                    settings_form,
                ))
                .child(render_metadata_kv_sections(
                    &sc.labels,
                    &sc.annotations,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element()
        }
        EditFormModel::MetadataFallback(meta) => {
            let mut identity_rows: Vec<(&str, &str)> = vec![("Name", &meta.name)];
            if let Some(ns) = meta.namespace.as_deref() {
                identity_rows.push(("Namespace", ns));
            }
            div()
                .flex()
                .flex_col()
                .gap_8()
                .child(
                    div()
                        .w_full()
                        .px_3()
                        .py_2()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(Icon::Info, px(14.0), theme.text.muted))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.secondary)
                                .child(
                                    "Edit metadata labels and annotations below, or switch to YAML mode to edit the full specification.",
                                ),
                        ),
                )
                .child(render_two_col_section(
                    "Resource Identity",
                    "Immutable resource metadata identifiers.",
                    theme,
                    render_readonly_fields(&identity_rows, theme),
                ))
                .child(render_metadata_kv_sections(
                    &meta.labels,
                    &meta.annotations,
                    inputs,
                    disabled,
                    theme,
                    &callbacks.on_mutate_form,
                ))
                .into_any_element()
        }
    };

    div()
        .id("edit-form-scroll-body")
        .flex_1()
        .min_h(px(0.0))
        .w_full()
        .overflow_y_scroll()
        .p_6()
        .child(div().max_w(px(960.0)).w_full().child(content))
        .into_any_element()
}

fn render_sub_tab_bar(
    active: EditFormSubTab,
    data_label: &'static str,
    meta_label: &'static str,
    theme: &OrbitTheme,
    on_select: &EditSubTabCallback,
) -> AnyElement {
    let make_tab = |tab: EditFormSubTab, label: &'static str, id: &'static str| {
        let is_active = active == tab;
        let cb = Rc::clone(on_select);
        let mut btn = Button::new(id)
            .small()
            .on_click(move |_, window, cx| cb(tab, window, cx))
            .child(label);
        if is_active {
            btn = btn.primary();
        } else {
            btn = btn.ghost();
        }
        btn
    };

    div()
        .flex()
        .items_center()
        .gap_2()
        .pb_2()
        .border_b_1()
        .border_color(theme.border.default)
        .child(make_tab(EditFormSubTab::Data, data_label, "edit-subtab-data"))
        .child(make_tab(
            EditFormSubTab::Metadata,
            meta_label,
            "edit-subtab-metadata",
        ))
        .into_any_element()
}

fn render_two_col_section(
    title: &'static str,
    description: &'static str,
    theme: &OrbitTheme,
    right_content: AnyElement,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_start()
        .gap_6()
        .child(
            div()
                .w(px(240.0))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.muted)
                        .child(title.to_uppercase()),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child(description),
                ),
        )
        .child(div().flex_1().min_w(px(0.0)).child(right_content))
        .into_any_element()
}

fn render_readonly_fields(rows: &[(&str, &str)], theme: &OrbitTheme) -> AnyElement {
    let mut form = v_form();
    for (label, value) in rows {
        let display = if value.is_empty() { "-" } else { value };
        form = form.child(
            field().label((*label).to_string()).child(
                apply_text_step(div(), theme.typography.size_xs)
                    .w_full()
                    .px_2p5()
                    .py_1p5()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_panel)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(theme.text.secondary)
                    .child(display.to_string()),
            ),
        );
    }
    form.into_any_element()
}

fn render_input_or_placeholder(
    inputs: &HashMap<String, Entity<InputState>>,
    field_id: &str,
    disabled: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    if let Some(state) = inputs.get(field_id) {
        Input::new(state)
            .small()
            .disabled(disabled)
            .into_any_element()
    } else {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("...")
            .into_any_element()
    }
}

fn render_metadata_kv_sections(
    labels: &[KeyValueEntry],
    annotations: &[KeyValueEntry],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    on_mutate_form: &EditFormMutatorCallback,
) -> AnyElement {
    render_two_col_section(
        "Metadata & Tags",
        "Labels and annotations attached to this Kubernetes resource.",
        theme,
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(render_kv_editor_section(
                "labels",
                "Labels",
                "Add Label",
                labels,
                inputs,
                disabled,
                theme,
                on_mutate_form,
            ))
            .child(render_kv_editor_section(
                "annotations",
                "Annotations",
                "Add Annotation",
                annotations,
                inputs,
                disabled,
                theme,
                on_mutate_form,
            ))
            .into_any_element(),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_kv_editor_section(
    group: &'static str,
    title: &'static str,
    add_label: &'static str,
    entries: &[KeyValueEntry],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    on_mutate_form: &EditFormMutatorCallback,
) -> AnyElement {
    let mut rows = div().flex().flex_col().gap_2();
    if entries.is_empty() {
        rows = rows.child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No entries configured."),
        );
    } else {
        for idx in 0..entries.len() {
            let key_id = format!("kv:{group}:{idx}:key");
            let val_id = format!("kv:{group}:{idx}:value");
            let mutate = Rc::clone(on_mutate_form);
            rows = rows.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .child(render_input_or_placeholder(inputs, &key_id, disabled, theme)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .child(render_input_or_placeholder(inputs, &val_id, disabled, theme)),
                    )
                    .child(
                        Button::new(SharedString::from(format!("kv-remove-{group}-{idx}")))
                            .ghost()
                            .small()
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                mutate(
                                    Box::new(move |model| {
                                        remove_kv_entry(model, group, idx);
                                    }),
                                    true,
                                    window,
                                    cx,
                                );
                            })
                            .child(render_icon(Icon::Trash2, px(13.0), theme.text.muted)),
                    ),
            );
        }
    }

    let add_mutate = Rc::clone(on_mutate_form);
    div()
        .w_full()
        .p_4()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(title),
                )
                .child(
                    Button::new(SharedString::from(format!("kv-add-{group}")))
                        .outline()
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            add_mutate(
                                Box::new(move |model| {
                                    push_kv_entry(model, group);
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(render_icon(Icon::Plus, px(12.0), theme.text.secondary))
                                .child(add_label),
                        ),
                ),
        )
        .child(rows)
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_string_list_section(
    group: &'static str,
    title: &'static str,
    add_label: &'static str,
    items: &[String],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    on_mutate_form: &EditFormMutatorCallback,
) -> AnyElement {
    let mut rows = div().flex().flex_col().gap_2();
    if items.is_empty() {
        rows = rows.child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No items configured."),
        );
    } else {
        for idx in 0..items.len() {
            let item_id = format!("list:{group}:{idx}");
            let mutate = Rc::clone(on_mutate_form);
            rows = rows.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .child(render_input_or_placeholder(inputs, &item_id, disabled, theme)),
                    )
                    .child(
                        Button::new(SharedString::from(format!("list-remove-{group}-{idx}")))
                            .ghost()
                            .small()
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                mutate(
                                    Box::new(move |model| {
                                        remove_list_entry(model, group, idx);
                                    }),
                                    true,
                                    window,
                                    cx,
                                );
                            })
                            .child(render_icon(Icon::Trash2, px(13.0), theme.text.muted)),
                    ),
            );
        }
    }

    let add_mutate = Rc::clone(on_mutate_form);
    div()
        .w_full()
        .p_4()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(title),
                )
                .child(
                    Button::new(SharedString::from(format!("list-add-{group}")))
                        .outline()
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            add_mutate(
                                Box::new(move |model| {
                                    push_list_entry(model, group);
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(render_icon(Icon::Plus, px(12.0), theme.text.secondary))
                                .child(add_label),
                        ),
                ),
        )
        .child(rows)
        .into_any_element()
}

fn kv_vec_mut<'a>(model: &'a mut EditFormModel, group: &str) -> Option<&'a mut Vec<KeyValueEntry>> {
    match (model, group) {
        (EditFormModel::ConfigMap(cm), "data") => Some(&mut cm.data),
        (EditFormModel::ConfigMap(cm), "labels") => Some(&mut cm.labels),
        (EditFormModel::ConfigMap(cm), "annotations") => Some(&mut cm.annotations),
        (EditFormModel::Secret(s), "data") => Some(&mut s.data),
        (EditFormModel::Secret(s), "labels") => Some(&mut s.labels),
        (EditFormModel::Secret(s), "annotations") => Some(&mut s.annotations),
        (EditFormModel::Namespace(n), "labels") => Some(&mut n.labels),
        (EditFormModel::Namespace(n), "annotations") => Some(&mut n.annotations),
        (EditFormModel::PersistentVolume(pv), "labels") => Some(&mut pv.labels),
        (EditFormModel::PersistentVolume(pv), "annotations") => Some(&mut pv.annotations),
        (EditFormModel::PersistentVolumeClaim(pvc), "labels") => Some(&mut pvc.labels),
        (EditFormModel::PersistentVolumeClaim(pvc), "annotations") => Some(&mut pvc.annotations),
        (EditFormModel::StorageClass(sc), "labels") => Some(&mut sc.labels),
        (EditFormModel::StorageClass(sc), "annotations") => Some(&mut sc.annotations),
        (EditFormModel::MetadataFallback(m), "labels") => Some(&mut m.labels),
        (EditFormModel::MetadataFallback(m), "annotations") => Some(&mut m.annotations),
        _ => None,
    }
}

fn push_kv_entry(model: &mut EditFormModel, group: &str) {
    if let Some(vec) = kv_vec_mut(model, group) {
        vec.push(KeyValueEntry {
            key: String::new(),
            value: String::new(),
        });
    }
}

fn remove_kv_entry(model: &mut EditFormModel, group: &str, idx: usize) {
    if let Some(vec) = kv_vec_mut(model, group)
        && idx < vec.len()
    {
        vec.remove(idx);
    }
}

fn list_vec_mut<'a>(model: &'a mut EditFormModel, group: &str) -> Option<&'a mut Vec<String>> {
    match (model, group) {
        (EditFormModel::Namespace(n), "finalizers") => Some(&mut n.finalizers),
        (EditFormModel::PersistentVolume(pv), "mount_options") => Some(&mut pv.mount_options),
        (EditFormModel::PersistentVolume(pv), "finalizers") => Some(&mut pv.finalizers),
        (EditFormModel::StorageClass(sc), "mount_options") => Some(&mut sc.mount_options),
        _ => None,
    }
}

fn push_list_entry(model: &mut EditFormModel, group: &str) {
    if let Some(vec) = list_vec_mut(model, group) {
        vec.push(String::new());
    }
}

fn remove_list_entry(model: &mut EditFormModel, group: &str, idx: usize) {
    if let Some(vec) = list_vec_mut(model, group)
        && idx < vec.len()
    {
        vec.remove(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::raw::format_raw_resource_yaml;
    use serde_json::json;

    #[test]
    fn test_edit_session_yaml_mode_hides_generated_fields_and_preserves_resource_version_on_save() {
        let mut session =
            EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 1);
        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "cfg",
                "namespace": "default",
                "uid": "u-1",
                "resourceVersion": "77",
                "creationTimestamp": "2026-01-01T00:00:00Z",
                "managedFields": [{ "manager": "kubectl" }]
            },
            "data": { "k": "v1" },
            "status": { "phase": "Active" }
        });
        let raw_yaml = format_raw_resource_yaml(&raw).expect("raw yaml");
        assert!(session.apply_loaded(1, raw, raw_yaml));
        assert!(!session.yaml_content.contains("resourceVersion:"));
        assert!(!session.yaml_content.contains("managedFields:"));
        assert!(!session.yaml_content.contains("uid:"));
        assert!(!session.yaml_content.contains("status:"));
        assert!(!session.is_dirty());

        session.switch_mode(EditMode::Yaml).expect("yaml mode");
        assert!(!session.yaml_content.contains("resourceVersion:"));
        assert!(!session.is_dirty());

        let edited = session.yaml_content.replace("k: v1", "k: v2");
        session.update_yaml_from_editor(edited);
        assert!(session.is_dirty());
        let payload = session.begin_save(10).expect("begin save");
        assert_eq!(payload["data"]["k"], "v2");
        assert_eq!(payload["metadata"]["resourceVersion"], "77");
        assert_eq!(payload["metadata"]["uid"], "u-1");
    }

    #[test]
    fn test_yaml_language_config_and_tree_sitter_yaml_highlighter_are_configured() {
        let config = yaml_language_config();
        assert_eq!(config.brackets.len(), 2);
        assert_eq!(
            config
                .auto_closing_pairs
                .as_ref()
                .map(Vec::len),
            Some(4)
        );
        let rules = config
            .indentation_rules
            .as_ref()
            .expect("YAML indentation rules should be configured");
        let inc = rules
            .increase_indent_pattern
            .as_ref()
            .expect("increase indent pattern");
        assert!(inc.is_match("metadata:"));
        assert!(inc.is_match("  script: |"));
        assert!(inc.is_match("  items: ["));
        assert!(!inc.is_match("  name: app"));

        assert!(
            gpui::component::highlighter::LanguageRegistry::singleton()
                .language("yaml")
                .is_some_and(|cfg| cfg.language.is_some()),
            "tree-sitter-yaml grammar must be registered for YAML syntax highlighting"
        );
    }

    fn sample_target(kind: &str, ns: Option<&str>, name: &str) -> EditTabTarget {
        EditTabTarget {
            kind: kind.to_string(),
            namespace: ns.map(str::to_string),
            name: name.to_string(),
        }
    }

    #[test]
    fn test_edit_session_defaults_to_form_mode_and_tracks_dirty_state() {
        let mut session = EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 1);
        assert_eq!(session.mode, EditMode::Form);
        assert_eq!(session.status, EditTabStatus::Loading);
        assert!(!session.is_dirty());
        assert!(!session.can_save());

        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default", "resourceVersion": "1" },
            "data": { "k": "v1" }
        });
        let yaml = format_raw_resource_yaml(&raw).expect("format yaml");
        assert!(session.apply_loaded(1, raw, yaml));
        assert_eq!(session.status, EditTabStatus::Ready);
        assert!(!session.is_dirty());
        assert!(!session.can_save());

        session.update_form_model(
            |form| {
                if let EditFormModel::ConfigMap(cm) = form {
                    cm.data[0].value = "v2".to_string();
                }
            },
            false,
        );
        assert!(session.is_dirty());
        assert!(session.can_save());
        assert!(session.yaml_content.contains("v2"));
    }

    #[test]
    fn test_edit_session_blocks_switch_to_form_on_invalid_yaml_without_losing_edits() {
        let mut session = EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 1);
        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default" },
            "data": { "k": "v1" }
        });
        let yaml = format_raw_resource_yaml(&raw).expect("format yaml");
        session.apply_loaded(1, raw, yaml);

        session.switch_mode(EditMode::Yaml).expect("switch to yaml");
        assert_eq!(session.mode, EditMode::Yaml);

        let broken_yaml = "apiVersion: v1\nkind: ConfigMap\ndata: [broken".to_string();
        session.update_yaml_from_editor(broken_yaml.clone());
        assert!(session.yaml_parse_error.is_some());
        assert!(!session.can_save());

        let err = session.switch_mode(EditMode::Form);
        assert!(err.is_err());
        assert_eq!(session.mode, EditMode::Yaml);
        assert_eq!(session.yaml_content, broken_yaml);
    }

    #[test]
    fn test_edit_session_save_success_reloads_resource_version_and_resets_dirty() {
        let mut session = EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 1);
        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default", "resourceVersion": "10" },
            "data": { "k": "v1" }
        });
        let yaml = format_raw_resource_yaml(&raw).expect("format yaml");
        session.apply_loaded(1, raw, yaml);

        session.update_form_model(
            |form| {
                if let EditFormModel::ConfigMap(cm) = form {
                    cm.data[0].value = "v2".to_string();
                }
            },
            false,
        );
        let payload = session.begin_save(99).expect("begin save");
        assert_eq!(payload["data"]["k"], "v2");
        assert_eq!(session.status, EditTabStatus::Saving);
        assert!(!session.can_save());

        let server_updated = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default", "resourceVersion": "11" },
            "data": { "k": "v2" }
        });
        let updated_yaml = format_raw_resource_yaml(&server_updated).expect("format yaml");
        assert!(session.apply_saved(99, server_updated, updated_yaml));
        assert_eq!(session.status, EditTabStatus::Ready);
        assert!(!session.is_dirty());
        assert!(!session.can_save());
        assert_eq!(
            session.raw_data.as_ref().expect("raw")["metadata"]["resourceVersion"],
            "11"
        );
    }

    #[test]
    fn test_edit_session_save_failure_and_conflict_preserve_unsaved_edits() {
        let mut session = EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 1);
        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default", "resourceVersion": "10" },
            "data": { "k": "v1" }
        });
        let yaml = format_raw_resource_yaml(&raw).expect("format yaml");
        session.apply_loaded(1, raw, yaml);

        session.update_form_model(
            |form| {
                if let EditFormModel::ConfigMap(cm) = form {
                    cm.data[0].value = "edited-value".to_string();
                }
            },
            false,
        );
        session.begin_save(5).expect("begin save");
        assert!(session.apply_save_failed(5, "object has been modified".to_string(), true));
        assert_eq!(session.status, EditTabStatus::Ready);
        assert!(session.is_dirty());
        assert!(session.can_save());
        assert!(session.yaml_content.contains("edited-value"));
        assert!(
            session
                .inline_error
                .as_deref()
                .is_some_and(|m| m.contains("409"))
        );
    }

    #[test]
    fn test_edit_session_pvc_expansion_and_pv_capacity_validation_disable_save() {
        let mut pvc_session =
            EditSessionState::new(sample_target("PersistentVolumeClaim", Some("default"), "data"), 1);
        let pvc_raw = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": { "name": "data", "namespace": "default" },
            "spec": { "resources": { "requests": { "storage": "10Gi" } } }
        });
        let pvc_yaml = format_raw_resource_yaml(&pvc_raw).expect("format yaml");
        pvc_session.apply_loaded(1, pvc_raw, pvc_yaml);

        pvc_session.update_form_model(
            |form| {
                if let EditFormModel::PersistentVolumeClaim(pvc) = form {
                    pvc.storage_request = "5Gi".to_string();
                }
            },
            false,
        );
        assert!(pvc_session.form_validation_error.is_some());
        assert!(!pvc_session.can_save());

        pvc_session.update_form_model(
            |form| {
                if let EditFormModel::PersistentVolumeClaim(pvc) = form {
                    pvc.storage_request = "20Gi".to_string();
                }
            },
            false,
        );
        assert!(pvc_session.form_validation_error.is_none());
        assert!(pvc_session.can_save());

        let mut pv_session =
            EditSessionState::new(sample_target("PersistentVolume", None, "pv-1"), 2);
        let pv_raw = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolume",
            "metadata": { "name": "pv-1" },
            "spec": {
                "capacity": { "storage": "10Gi" },
                "accessModes": ["ReadWriteOnce"]
            }
        });
        let pv_yaml = format_raw_resource_yaml(&pv_raw).expect("format yaml");
        pv_session.apply_loaded(2, pv_raw, pv_yaml);

        pv_session.update_form_model(
            |form| {
                if let EditFormModel::PersistentVolume(pv) = form {
                    pv.capacity = "not-a-size".to_string();
                }
            },
            false,
        );
        assert!(pv_session.form_validation_error.is_some());
        assert!(!pv_session.can_save());
    }

    #[test]
    fn test_edit_session_ignores_stale_load_and_save_request_ids() {
        let mut session = EditSessionState::new(sample_target("ConfigMap", Some("default"), "cfg"), 10);
        let raw = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default" }
        });
        let yaml = format_raw_resource_yaml(&raw).expect("format yaml");

        assert!(!session.apply_loaded(9, raw.clone(), yaml.clone()));
        assert_eq!(session.status, EditTabStatus::Loading);
        assert!(!session.apply_load_failed(9, "stale error".to_string()));
        assert_eq!(session.status, EditTabStatus::Loading);

        assert!(session.apply_loaded(10, raw.clone(), yaml.clone()));
        session.update_yaml_from_editor(format!("{yaml}# change\n"));
        session.begin_save(20).expect("begin save");

        assert!(!session.apply_saved(19, raw, yaml));
        assert_eq!(session.status, EditTabStatus::Saving);
        assert!(!session.apply_save_failed(19, "stale fail".to_string(), false));
        assert_eq!(session.status, EditTabStatus::Saving);
    }

    #[test]
    fn test_context_menu_edit_entry_for_remaining_editable_views_and_excluded_views() {
        use crate::components::context_menu::ContextMenuEntry;
        use orbit_core::kubernetes::models::{EventInfo, HpaInfo, PodInfo, PolicyInfo};
        use orbit_core::kubernetes::port_forward::PersistedPortForward;

        let has_edit = |entries: &[ContextMenuEntry]| {
            entries.iter().any(|e| match e {
                ContextMenuEntry::Item(item) => {
                    item.id.as_ref() == "edit"
                        && item.label.as_ref() == "Edit"
                        && !item.disabled
                        && item.on_select.is_some()
                }
                ContextMenuEntry::Separator => false,
            })
        };

        // Verify the 6 editable views without standalone menu tests include Edit
        let pod = PodInfo {
            name: "web-0".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            restarts: 0,
            cpu: None,
            memory: None,
            cpu_cores: None,
            memory_bytes: None,
            cpu_pct: None,
            memory_pct: None,
            node: None,
            age: "1m".to_string(),
            images: Vec::new(),
            ip: None,
            node_ip: None,
            controlled_by: None,
            qos_class: None,
            containers: Vec::new(),
            labels: Default::default(),
            annotations: Default::default(),
        };
        assert!(has_edit(&crate::views::pods::build_pod_context_menu_entries(
            &pod,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        )));

        let hpa = HpaInfo {
            kind: "HorizontalPodAutoscaler".to_string(),
            name: "web-hpa".to_string(),
            namespace: "default".to_string(),
            scale_target_ref: orbit_core::kubernetes::models::HpaTargetRef {
                kind: "Deployment".to_string(),
                name: "web".to_string(),
                api_version: Some("apps/v1".to_string()),
            },
            min_replicas: 1,
            max_replicas: 5,
            current_replicas: 2,
            desired_replicas: 2,
            metrics: Vec::new(),
            metrics_summary: Vec::new(),
            conditions: Vec::new(),
            last_scale_time: None,
            status: "Healthy".to_string(),
            age: "1m".to_string(),
            created: String::new(),
            labels: Default::default(),
            annotations: Default::default(),
        };
        assert!(has_edit(&crate::views::hpas::build_hpa_context_menu_entries(
            &hpa,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        )));

        assert!(has_edit(
            &crate::views::storage_classes::build_storage_class_context_menu_entries(
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));

        assert!(has_edit(
            &crate::views::admission_webhooks::build_admission_webhook_context_menu_entries(
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));

        let policy = PolicyInfo {
            uid: "uid-1".to_string(),
            name: "pol".to_string(),
            r#type: "Network".to_string(),
            scope: "Namespaced".to_string(),
            namespace: "default".to_string(),
            kind: "NetworkPolicy".to_string(),
            status: "Active".to_string(),
            mode: "Enforce".to_string(),
            violations: 0,
            last_updated: "1m".to_string(),
            description: String::new(),
            rules: String::new(),
        };
        assert!(has_edit(
            &crate::views::security_overview::build_security_policy_context_menu_entries(
                &policy,
                |_, _| {},
                Some(|_: &mut Window, _: &mut App| {}),
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));
        assert!(has_edit(
            &crate::views::pod_security::build_pod_security_context_menu_entries(
                &policy,
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));

        // Verify Events and PortForward omit Edit
        let ev = EventInfo {
            uid: "u1".to_string(),
            name: "e1".to_string(),
            time: "1m".to_string(),
            r#type: "Normal".to_string(),
            reason: "Started".to_string(),
            object_name: "web-0".to_string(),
            object_kind: "Pod".to_string(),
            message: "Started".to_string(),
            namespace: "default".to_string(),
            source: "kubelet".to_string(),
            first_seen: String::new(),
            last_seen: String::new(),
            count: 1,
            labels: Default::default(),
            annotations: Default::default(),
        };
        assert!(!has_edit(
            &crate::views::events::build_event_context_menu_entries(
                &ev,
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));

        let pf = PersistedPortForward {
            id: "pf1".to_string(),
            cluster_id: "prod".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "web".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        assert!(!has_edit(
            &crate::views::port_forward::build_port_forward_context_menu_entries(
                &pf,
                |_, _| {},
                |_, _| {},
                |_, _| {},
            )
        ));
    }
}
