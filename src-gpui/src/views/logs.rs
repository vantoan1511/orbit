use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::logs::{LogExportRow, WorkloadPodTarget, parse_k8s_log_line};

use crate::icons::render_icon;
use crate::navigation::LogsTabTarget;
use crate::theme::{OrbitTheme, apply_text_step};

pub const LOG_LINE_LIMIT_OPTIONS: [i64; 5] = [50, 100, 200, 500, 1000];
pub const DEFAULT_LOG_TAIL_LINES: i64 = 50;
pub const LOG_BUFFER_CAP: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub pod: String,
    pub container: String,
    pub message: String,
    pub raw: String,
}

impl LogEntry {
    pub fn from_raw(pod: &str, container: &str, raw: String) -> Self {
        let (ts, msg) = parse_k8s_log_line(&raw);
        Self {
            timestamp: ts.map(ToString::to_string),
            pod: pod.to_string(),
            container: container.to_string(),
            message: msg.to_string(),
            raw,
        }
    }

    pub fn source_label(&self) -> String {
        if self.container.is_empty() {
            self.pod.clone()
        } else {
            format!("{}/{}", self.pod, self.container)
        }
    }

    pub fn format_line(&self) -> String {
        let source = self.source_label();
        match &self.timestamp {
            Some(ts) => format!("{ts}  {source}  {}", self.message),
            None => format!("{source}  {}", self.message),
        }
    }

    pub fn to_export_row(&self) -> LogExportRow {
        LogExportRow {
            timestamp: self.timestamp.clone(),
            pod: self.pod.clone(),
            container: self.container.clone(),
            message: self.message.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogViewerState {
    pub target: LogsTabTarget,
    pub stream_id: u64,
    pub is_streaming: bool,
    pub is_paused: bool,
    pub is_following: bool,
    pub wrap_lines: bool,
    pub include_unavailable_pods: bool,
    pub tail_lines: i64,
    pub pods: Vec<WorkloadPodTarget>,
    pub selected_pod: Option<String>,
    pub selected_container: Option<String>,
    pub excluded_unavailable_count: usize,
    pub entries: Vec<LogEntry>,
    pub paused_buffer: Vec<LogEntry>,
    pub status_message: Option<String>,
    pub status_is_error: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub active_match_pos: Option<usize>,
    pub selected_anchor: Option<usize>,
    pub selected_range: Option<(usize, usize)>,
    pub is_dragging_selection: bool,
    pub scroll_handle: ScrollHandle,
    pub pending_scroll_to_bottom: bool,
    pub pending_scroll_to_match: bool,
}

impl LogViewerState {
    pub fn new(target: LogsTabTarget, stream_id: u64) -> Self {
        Self {
            target,
            stream_id,
            is_streaming: true,
            is_paused: false,
            is_following: true,
            wrap_lines: true,
            include_unavailable_pods: false,
            tail_lines: DEFAULT_LOG_TAIL_LINES,
            pods: Vec::new(),
            selected_pod: None,
            selected_container: None,
            excluded_unavailable_count: 0,
            entries: Vec::new(),
            paused_buffer: Vec::new(),
            status_message: None,
            status_is_error: false,
            search_query: String::new(),
            search_focused: false,
            active_match_pos: None,
            selected_anchor: None,
            selected_range: None,
            is_dragging_selection: false,
            scroll_handle: ScrollHandle::new(),
            pending_scroll_to_bottom: false,
            pending_scroll_to_match: false,
        }
    }

    pub fn reset_for_restart(&mut self, new_stream_id: u64) {
        self.stream_id = new_stream_id;
        self.is_streaming = true;
        self.entries.clear();
        self.paused_buffer.clear();
        self.status_message = None;
        self.status_is_error = false;
        self.active_match_pos = None;
        self.clear_selection();
        self.pending_scroll_to_bottom = self.is_following && !self.is_paused;
    }

    pub fn push_lines(&mut self, pod: &str, container: &str, lines: Vec<String>) {
        let converted: Vec<LogEntry> = lines
            .into_iter()
            .map(|raw| LogEntry::from_raw(pod, container, raw))
            .collect();

        if self.is_paused {
            self.paused_buffer.extend(converted);
            if self.paused_buffer.len() > LOG_BUFFER_CAP {
                let overflow = self.paused_buffer.len() - LOG_BUFFER_CAP;
                self.paused_buffer.drain(0..overflow);
            }
            return;
        }

        self.append_entries_capped(converted);
        if self.is_following {
            self.pending_scroll_to_bottom = true;
        }
    }

    fn append_entries_capped(&mut self, items: Vec<LogEntry>) {
        self.entries.extend(items);
        if self.entries.len() > LOG_BUFFER_CAP {
            let overflow = self.entries.len() - LOG_BUFFER_CAP;
            self.entries.drain(0..overflow);
            self.clear_selection();
        }
        self.reconcile_match_pos();
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        if !self.is_paused && !self.paused_buffer.is_empty() {
            let buffered = std::mem::take(&mut self.paused_buffer);
            self.append_entries_capped(buffered);
            if self.is_following {
                self.pending_scroll_to_bottom = true;
            }
        }
    }

    pub fn toggle_follow(&mut self) {
        self.is_following = !self.is_following;
        if self.is_following && !self.is_paused {
            self.pending_scroll_to_bottom = true;
        }
    }

    pub fn toggle_wrap(&mut self) {
        self.wrap_lines = !self.wrap_lines;
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.clear_selection();
        let visible_len = self.visible_indices().len();
        if self.search_query.trim().is_empty() || visible_len == 0 {
            self.active_match_pos = None;
        } else {
            self.active_match_pos = Some(0);
            self.pending_scroll_to_match = true;
        }
    }

    fn reconcile_match_pos(&mut self) {
        if self.search_query.trim().is_empty() {
            self.active_match_pos = None;
            return;
        }
        let len = self.visible_indices().len();
        if len == 0 {
            self.active_match_pos = None;
        } else if self.active_match_pos.is_none_or(|p| p >= len) {
            self.active_match_pos = Some(0);
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let query = self.search_query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return (0..self.entries.len()).collect();
        }
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let matches = entry.message.to_ascii_lowercase().contains(&query)
                    || entry.pod.to_ascii_lowercase().contains(&query)
                    || entry.container.to_ascii_lowercase().contains(&query)
                    || entry
                        .timestamp
                        .as_deref()
                        .is_some_and(|ts| ts.to_ascii_lowercase().contains(&query));
                matches.then_some(idx)
            })
            .collect()
    }

    pub fn next_match(&mut self) {
        let len = self.visible_indices().len();
        if len == 0 || self.search_query.trim().is_empty() {
            self.active_match_pos = None;
            return;
        }
        let next = match self.active_match_pos {
            Some(pos) => (pos + 1) % len,
            None => 0,
        };
        self.active_match_pos = Some(next);
        self.pending_scroll_to_match = true;
    }

    pub fn prev_match(&mut self) {
        let len = self.visible_indices().len();
        if len == 0 || self.search_query.trim().is_empty() {
            self.active_match_pos = None;
            return;
        }
        let prev = match self.active_match_pos {
            Some(0) | None => len - 1,
            Some(pos) => pos - 1,
        };
        self.active_match_pos = Some(prev);
        self.pending_scroll_to_match = true;
    }

    pub fn select_line(&mut self, visible_idx: usize, shift: bool) {
        if shift {
            if let Some(anchor) = self.selected_anchor {
                let start = anchor.min(visible_idx);
                let end = anchor.max(visible_idx);
                self.selected_range = Some((start, end));
                return;
            }
        }
        self.selected_anchor = Some(visible_idx);
        self.selected_range = Some((visible_idx, visible_idx));
    }

    pub fn extend_drag_selection(&mut self, visible_idx: usize) {
        if !self.is_dragging_selection {
            return;
        }
        let anchor = self.selected_anchor.unwrap_or(visible_idx);
        let start = anchor.min(visible_idx);
        let end = anchor.max(visible_idx);
        self.selected_range = Some((start, end));
    }

    pub fn clear_selection(&mut self) {
        self.selected_anchor = None;
        self.selected_range = None;
        self.is_dragging_selection = false;
    }

    pub fn is_visible_index_selected(&self, visible_idx: usize) -> bool {
        self.selected_range
            .is_some_and(|(start, end)| visible_idx >= start && visible_idx <= end)
    }

    pub fn copy_text(&self) -> String {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return String::new();
        }
        let slice = match self.selected_range {
            Some((start, end)) if start <= end && start < visible.len() => {
                let clamped_end = end.min(visible.len() - 1);
                &visible[start..=clamped_end]
            }
            _ => &visible[..],
        };
        slice
            .iter()
            .filter_map(|&idx| self.entries.get(idx))
            .map(LogEntry::format_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn build_export_rows(&self) -> Vec<LogExportRow> {
        self.visible_indices()
            .into_iter()
            .filter_map(|idx| self.entries.get(idx))
            .map(LogEntry::to_export_row)
            .collect()
    }

    pub fn available_containers(&self) -> Vec<String> {
        let mut containers = Vec::new();
        for pod in &self.pods {
            if let Some(ref filter_pod) = self.selected_pod
                && &pod.name != filter_pod
            {
                continue;
            }
            for c in &pod.containers {
                if !containers.contains(c) {
                    containers.push(c.clone());
                }
            }
        }
        containers
    }

    pub fn cycle_pod(&mut self) {
        if self.pods.is_empty() {
            self.selected_pod = None;
            return;
        }
        let names: Vec<&str> = self.pods.iter().map(|p| p.name.as_str()).collect();
        self.selected_pod = match self.selected_pod.as_deref() {
            None => names.first().map(|s| (*s).to_string()),
            Some(current) => match names.iter().position(|n| *n == current) {
                Some(idx) if idx + 1 < names.len() => Some(names[idx + 1].to_string()),
                _ => None,
            },
        };
        if let Some(ref c) = self.selected_container
            && !self.available_containers().contains(c)
        {
            self.selected_container = None;
        }
    }

    pub fn cycle_container(&mut self) {
        let containers = self.available_containers();
        if containers.is_empty() {
            self.selected_container = None;
            return;
        }
        self.selected_container = match self.selected_container.as_deref() {
            None => containers.first().cloned(),
            Some(current) => match containers.iter().position(|c| c == current) {
                Some(idx) if idx + 1 < containers.len() => Some(containers[idx + 1].clone()),
                _ => None,
            },
        };
    }

    pub fn cycle_tail_lines(&mut self) {
        let idx = LOG_LINE_LIMIT_OPTIONS
            .iter()
            .position(|&v| v == self.tail_lines)
            .unwrap_or(0);
        self.tail_lines = LOG_LINE_LIMIT_OPTIONS[(idx + 1) % LOG_LINE_LIMIT_OPTIONS.len()];
    }
}

pub type LogActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type LogLineSelectCallback = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;
pub type LogLineHoverCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;
pub type LogExportCallback = Rc<dyn Fn(LogExportFormat, &mut Window, &mut App)>;

pub struct LogViewerCallbacks {
    pub on_cycle_pod: LogActionCallback,
    pub on_cycle_container: LogActionCallback,
    pub on_toggle_include_unavailable: LogActionCallback,
    pub on_cycle_tail_lines: LogActionCallback,
    pub on_focus_search: LogActionCallback,
    pub on_clear_search: LogActionCallback,
    pub on_prev_match: LogActionCallback,
    pub on_next_match: LogActionCallback,
    pub on_toggle_pause: LogActionCallback,
    pub on_toggle_follow: LogActionCallback,
    pub on_toggle_wrap: LogActionCallback,
    pub on_copy: LogActionCallback,
    pub on_export: LogExportCallback,
    pub on_line_mouse_down: LogLineSelectCallback,
    pub on_line_mouse_move: LogLineHoverCallback,
    pub on_scroll_wheel_up: LogActionCallback,
}

fn toolbar_pill_button(
    id: impl Into<ElementId>,
    icon: Icon,
    label: String,
    active: bool,
    theme: &OrbitTheme,
    on_click: LogActionCallback,
) -> AnyElement {
    let bg = if active {
        theme.accent.soft
    } else {
        theme.surface.bg_card
    };
    let border_col = if active {
        theme.accent.accent
    } else {
        theme.border.default
    };
    let text_col = if active {
        theme.accent.accent
    } else {
        theme.text.secondary
    };
    let hover_bg = theme.surface.bg_hover;

    div()
        .id(id)
        .h(px(26.0))
        .px_2()
        .flex()
        .items_center()
        .gap_1()
        .rounded(theme.style.radius_control)
        .bg(bg)
        .border_1()
        .border_color(border_col)
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_click(window, cx);
        })
        .child(render_icon(icon, px(12.0), text_col))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_col)
                .child(label),
        )
        .into_any_element()
}

pub fn render_logs_view(
    state: &mut LogViewerState,
    theme: &OrbitTheme,
    search_focus: &FocusHandle,
    callbacks: LogViewerCallbacks,
) -> AnyElement {
    let visible = state.visible_indices();

    if state.pending_scroll_to_bottom && !visible.is_empty() {
        state.scroll_handle.scroll_to_item(visible.len() - 1);
        state.pending_scroll_to_bottom = false;
    } else if state.pending_scroll_to_match {
        if let Some(pos) = state.active_match_pos
            && pos < visible.len()
        {
            state.scroll_handle.scroll_to_item(pos);
        }
        state.pending_scroll_to_match = false;
    }

    let pod_label = state
        .selected_pod
        .clone()
        .unwrap_or_else(|| "All Pods".to_string());
    let container_label = state
        .selected_container
        .clone()
        .unwrap_or_else(|| "All Containers".to_string());
    let containers = state.available_containers();

    let unavail_label = if state.excluded_unavailable_count > 0 && !state.include_unavailable_pods {
        format!("Include unavailable ({})", state.excluded_unavailable_count)
    } else {
        "Include unavailable".to_string()
    };

    let pause_icon = if state.is_paused {
        Icon::Play
    } else {
        Icon::Pause
    };
    let pause_label = if state.is_paused {
        if state.paused_buffer.is_empty() {
            "Resume".to_string()
        } else {
            format!("Resume (+{})", state.paused_buffer.len())
        }
    } else {
        "Pause".to_string()
    };

    let wrap_label = if state.wrap_lines {
        "Wrap".to_string()
    } else {
        "No Wrap".to_string()
    };

    let copy_label = match state.selected_range {
        Some((start, end)) if start <= end && start < visible.len() => {
            let count = end.min(visible.len() - 1) - start + 1;
            format!("Copy ({count})")
        }
        _ => "Copy".to_string(),
    };

    let search_border = if state.search_focused {
        theme.accent.accent
    } else {
        theme.border.default
    };
    let search_placeholder = state.search_query.is_empty();
    let search_display = if search_placeholder {
        "Filter logs...".to_string()
    } else {
        state.search_query.clone()
    };

    let match_badge = if state.search_query.trim().is_empty() {
        None
    } else if visible.is_empty() {
        Some("0/0".to_string())
    } else {
        let cur = state.active_match_pos.map_or(1, |p| p + 1);
        Some(format!("{cur}/{}", visible.len()))
    };

    let on_focus_search = callbacks.on_focus_search.clone();
    let on_clear_search = callbacks.on_clear_search.clone();
    let on_prev_match = callbacks.on_prev_match.clone();
    let on_next_match = callbacks.on_next_match.clone();
    let on_export_json = callbacks.on_export.clone();
    let on_export_csv = callbacks.on_export.clone();
    let hover_bg = theme.surface.bg_hover;

    let search_box = div()
        .id("logs-search-input")
        .track_focus(search_focus)
        .h(px(26.0))
        .w(px(220.0))
        .px_2()
        .flex()
        .items_center()
        .gap_1p5()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(search_border)
        .cursor_text()
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_focus_search(window, cx);
        })
        .child(render_icon(Icon::Search, px(12.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(if search_placeholder {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .child(search_display),
        )
        .when_some(match_badge, |el, badge| {
            el.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(badge),
            )
        })
        .when(!search_placeholder, |el| {
            el.child(
                div()
                    .id("logs-search-prev")
                    .p_0p5()
                    .rounded(px(3.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        cx.stop_propagation();
                        on_prev_match(window, cx);
                    })
                    .child(render_icon(Icon::ChevronUp, px(12.0), theme.text.secondary)),
            )
            .child(
                div()
                    .id("logs-search-next")
                    .p_0p5()
                    .rounded(px(3.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        cx.stop_propagation();
                        on_next_match(window, cx);
                    })
                    .child(render_icon(Icon::ChevronDown, px(12.0), theme.text.secondary)),
            )
            .child(
                div()
                    .id("logs-search-clear")
                    .p_0p5()
                    .rounded(px(3.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        cx.stop_propagation();
                        on_clear_search(window, cx);
                    })
                    .child(render_icon(Icon::X, px(11.0), theme.text.muted)),
            )
        });

    let toolbar = div()
        .id("logs-toolbar")
        .w_full()
        .px_3()
        .py_2()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_2()
        .bg(theme.surface.bg_panel)
        .border_b_1()
        .border_color(theme.border.default)
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .h(px(26.0))
                        .px_2()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_card)
                        .border_1()
                        .border_color(theme.border.default)
                        .child(render_icon(Icon::ScrollText, px(12.0), theme.text.secondary))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(format!("{}/{}", state.target.kind, state.target.name)),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.muted)
                                .child(state.target.namespace.clone()),
                        ),
                )
                .when(state.pods.len() > 1 || state.selected_pod.is_some(), |el| {
                    el.child(toolbar_pill_button(
                        "logs-pod-filter",
                        Icon::Box,
                        pod_label,
                        state.selected_pod.is_some(),
                        theme,
                        callbacks.on_cycle_pod.clone(),
                    ))
                })
                .when(containers.len() > 1 || state.selected_container.is_some(), |el| {
                    el.child(toolbar_pill_button(
                        "logs-container-filter",
                        Icon::Layers,
                        container_label,
                        state.selected_container.is_some(),
                        theme,
                        callbacks.on_cycle_container.clone(),
                    ))
                })
                .child(toolbar_pill_button(
                    "logs-include-unavailable",
                    Icon::Eye,
                    unavail_label,
                    state.include_unavailable_pods,
                    theme,
                    callbacks.on_toggle_include_unavailable.clone(),
                ))
                .child(toolbar_pill_button(
                    "logs-tail-lines",
                    Icon::ListOrdered,
                    format!("{} lines", state.tail_lines),
                    false,
                    theme,
                    callbacks.on_cycle_tail_lines.clone(),
                )),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_1p5()
                .child(search_box)
                .child(toolbar_pill_button(
                    "logs-pause-toggle",
                    pause_icon,
                    pause_label,
                    state.is_paused,
                    theme,
                    callbacks.on_toggle_pause.clone(),
                ))
                .child(toolbar_pill_button(
                    "logs-follow-toggle",
                    Icon::ArrowDownToLine,
                    "Follow".to_string(),
                    state.is_following,
                    theme,
                    callbacks.on_toggle_follow.clone(),
                ))
                .child(toolbar_pill_button(
                    "logs-wrap-toggle",
                    Icon::WrapText,
                    wrap_label,
                    state.wrap_lines,
                    theme,
                    callbacks.on_toggle_wrap.clone(),
                ))
                .child(toolbar_pill_button(
                    "logs-copy-btn",
                    Icon::Copy,
                    copy_label,
                    false,
                    theme,
                    callbacks.on_copy.clone(),
                ))
                .child(toolbar_pill_button(
                    "logs-export-json",
                    Icon::FileJson,
                    "JSON".to_string(),
                    false,
                    theme,
                    Rc::new(move |window, cx| on_export_json(LogExportFormat::Json, window, cx)),
                ))
                .child(toolbar_pill_button(
                    "logs-export-csv",
                    Icon::FileSpreadsheet,
                    "CSV".to_string(),
                    false,
                    theme,
                    Rc::new(move |window, cx| on_export_csv(LogExportFormat::Csv, window, cx)),
                )),
        );

    let status_banner = state.status_message.as_ref().map(|msg| {
        let (bg, fg) = if state.status_is_error {
            (theme.semantic.danger.soft, theme.semantic.danger.solid)
        } else {
            (theme.semantic.info.soft, theme.semantic.info.solid)
        };
        div()
            .w_full()
            .px_3()
            .py_1p5()
            .flex_shrink_0()
            .bg(bg)
            .border_b_1()
            .border_color(theme.border.default)
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(fg)
                    .child(msg.clone()),
            )
    });

    let scroll_handle = state.scroll_handle.clone();
    let on_scroll_wheel_up = callbacks.on_scroll_wheel_up.clone();
    let mut viewport = div()
        .id("logs-viewport")
        .flex_1()
        .min_h_0()
        .min_w_0()
        .p_2()
        .bg(theme.surface.bg_app)
        .font_family(theme.typography.mono_family.clone())
        .overflow_y_scroll()
        .when(!state.wrap_lines, |el| el.overflow_x_scroll())
        .track_scroll(&scroll_handle)
        .on_scroll(move |event, window, cx| {
            let delta = event.delta.pixel_delta(px(16.0));
            if delta.y > px(0.0) {
                on_scroll_wheel_up(window, cx);
            }
        });

    if visible.is_empty() {
        let empty_text = if state.is_streaming {
            "Waiting for log lines..."
        } else if !state.search_query.trim().is_empty() {
            "No log lines match the current search filter."
        } else {
            "No log lines available."
        };
        viewport = viewport.child(
            div()
                .w_full()
                .py_8()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child(empty_text),
                ),
        );
    } else {
        let mut lines_col = div().flex().flex_col().gap_px();
        if state.wrap_lines {
            lines_col = lines_col.w_full().min_w_0();
        } else {
            lines_col = lines_col.min_w_full();
        }

        for (visible_idx, &entry_idx) in visible.iter().enumerate() {
            let Some(entry) = state.entries.get(entry_idx) else {
                continue;
            };
            let is_selected = state.is_visible_index_selected(visible_idx);
            let is_active_match = state.active_match_pos == Some(visible_idx)
                && !state.search_query.trim().is_empty();

            let row_bg = if is_selected {
                theme.accent.soft
            } else if is_active_match {
                theme.semantic.warning.soft
            } else {
                gpui::rgba(0x0000_0000)
            };

            let on_down = callbacks.on_line_mouse_down.clone();
            let on_move = callbacks.on_line_mouse_move.clone();
            let line_id: SharedString = format!("log-line-{visible_idx}").into();

            let ts_text = entry.timestamp.clone().unwrap_or_default();
            let source_text = entry.source_label();
            let msg_text = entry.message.clone();

            let mut row = div()
                .id(line_id)
                .px_2()
                .py_0p5()
                .rounded(px(2.0))
                .bg(row_bg)
                .hover(move |s| s.bg(hover_bg))
                .flex()
                .items_start()
                .gap_2()
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    on_down(visible_idx, event.modifiers.shift, window, cx);
                })
                .on_mouse_move(move |_event, window, cx| {
                    on_move(visible_idx, window, cx);
                });

            if state.wrap_lines {
                row = row.w_full().min_w_0();
            } else {
                row = row.whitespace_nowrap();
            }

            if !ts_text.is_empty() {
                row = row.child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .flex_shrink_0()
                        .text_color(theme.text.muted)
                        .child(ts_text),
                );
            }

            row = row
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .flex_shrink_0()
                        .px_1()
                        .rounded(px(2.0))
                        .bg(theme.surface.bg_card)
                        .text_color(theme.text.secondary)
                        .child(source_text),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .when(state.wrap_lines, |el| el.flex_1().min_w_0())
                        .text_color(theme.text.primary)
                        .child(msg_text),
                );

            lines_col = lines_col.child(row);
        }
        viewport = viewport.child(lines_col);
    }

    div()
        .id("logs-view-root")
        .flex_1()
        .min_h_0()
        .min_w_0()
        .flex()
        .flex_col()
        .bg(theme.surface.bg_app)
        .child(toolbar)
        .when_some(status_banner, |el, banner| el.child(banner))
        .child(viewport)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_target() -> LogsTabTarget {
        LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api".to_string(),
        }
    }

    #[test]
    fn test_log_viewer_defaults() {
        let state = LogViewerState::new(sample_target(), 1);
        assert_eq!(state.tail_lines, 50);
        assert!(!state.include_unavailable_pods);
        assert!(state.is_following);
        assert!(!state.is_paused);
        assert!(state.wrap_lines);
        assert_eq!(LOG_LINE_LIMIT_OPTIONS, [50, 100, 200, 500, 1000]);
    }

    #[test]
    fn test_log_line_formatting_includes_timestamp_pod_container_and_message() {
        let entry = LogEntry::from_raw(
            "api-7f9b",
            "app",
            "2026-11-25T12:00:00Z started server".to_string(),
        );
        assert_eq!(entry.timestamp.as_deref(), Some("2026-11-25T12:00:00Z"));
        assert_eq!(entry.pod, "api-7f9b");
        assert_eq!(entry.container, "app");
        assert_eq!(entry.message, "started server");
        assert_eq!(
            entry.format_line(),
            "2026-11-25T12:00:00Z  api-7f9b/app  started server"
        );
    }

    #[test]
    fn test_pause_buffers_incoming_lines_and_resume_flushes() {
        let mut state = LogViewerState::new(sample_target(), 1);
        state.push_lines("pod-1", "app", vec!["line 1".to_string()]);
        assert_eq!(state.entries.len(), 1);

        state.toggle_pause();
        assert!(state.is_paused);
        state.push_lines("pod-1", "app", vec!["line 2".to_string(), "line 3".to_string()]);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.paused_buffer.len(), 2);

        state.toggle_pause();
        assert!(!state.is_paused);
        assert!(state.paused_buffer.is_empty());
        assert_eq!(state.entries.len(), 3);
        assert_eq!(state.entries[1].message, "line 2");
        assert_eq!(state.entries[2].message, "line 3");
    }

    #[test]
    fn test_search_filters_and_navigates_matches() {
        let mut state = LogViewerState::new(sample_target(), 1);
        state.push_lines(
            "pod-1",
            "app",
            vec![
                "INFO starting".to_string(),
                "ERROR connection refused".to_string(),
                "INFO healthy".to_string(),
                "error timeout".to_string(),
            ],
        );

        state.set_search_query("ERROR".to_string());
        let visible = state.visible_indices();
        assert_eq!(visible, vec![1, 3]);
        assert_eq!(state.active_match_pos, Some(0));

        state.next_match();
        assert_eq!(state.active_match_pos, Some(1));
        state.next_match();
        assert_eq!(state.active_match_pos, Some(0));

        state.prev_match();
        assert_eq!(state.active_match_pos, Some(1));
    }

    #[test]
    fn test_line_selection_click_shift_click_and_copy_selected_text() {
        let mut state = LogViewerState::new(sample_target(), 1);
        state.push_lines(
            "pod-1",
            "app",
            vec![
                "first".to_string(),
                "second".to_string(),
                "third".to_string(),
            ],
        );

        // No selection -> copies all visible lines
        assert_eq!(
            state.copy_text(),
            "pod-1/app  first\npod-1/app  second\npod-1/app  third"
        );

        // Single click on line 2
        state.select_line(2, false);
        assert_eq!(state.selected_range, Some((2, 2)));
        assert_eq!(state.copy_text(), "pod-1/app  third");

        // Shift+click backward to line 0
        state.select_line(0, true);
        assert_eq!(state.selected_range, Some((0, 2)));
        assert_eq!(
            state.copy_text(),
            "pod-1/app  first\npod-1/app  second\npod-1/app  third"
        );

        // Drag selection from line 1 to line 2
        state.select_line(1, false);
        state.is_dragging_selection = true;
        state.extend_drag_selection(2);
        state.is_dragging_selection = false;
        assert_eq!(state.selected_range, Some((1, 2)));
        assert_eq!(state.copy_text(), "pod-1/app  second\npod-1/app  third");
    }

    #[test]
    fn test_wrap_toggle_switches_between_wrap_and_no_wrap() {
        let mut state = LogViewerState::new(sample_target(), 1);
        assert!(state.wrap_lines);
        state.toggle_wrap();
        assert!(!state.wrap_lines);
        state.toggle_wrap();
        assert!(state.wrap_lines);
    }

    #[test]
    fn test_export_rows_reflect_visible_filtered_logs() {
        let mut state = LogViewerState::new(sample_target(), 1);
        state.push_lines(
            "pod-1",
            "app",
            vec![
                "2026-11-25T12:00:00Z ok".to_string(),
                "2026-11-25T12:00:01Z fail".to_string(),
            ],
        );
        state.set_search_query("fail".to_string());
        let rows = state.build_export_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].message, "fail");
        assert_eq!(rows[0].timestamp.as_deref(), Some("2026-11-25T12:00:01Z"));
    }
}
