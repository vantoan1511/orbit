use std::collections::VecDeque;

pub const DEFAULT_MAX_LOG_LINES: usize = 50_000;

#[derive(Debug, Clone)]
pub struct LogSessionState {
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub available_containers: Vec<String>,
    pub lines: VecDeque<String>,
    pub max_lines: usize,
    pub search_query: String,
    pub auto_scroll: bool,
    pub is_paused: bool,
}

impl LogSessionState {
    pub fn new(namespace: String, pod: String, container: Option<String>) -> Self {
        Self {
            namespace,
            pod,
            container,
            available_containers: Vec::new(),
            lines: VecDeque::new(),
            max_lines: DEFAULT_MAX_LOG_LINES,
            search_query: String::new(),
            auto_scroll: true,
            is_paused: false,
        }
    }

    pub fn append_lines<I, S>(&mut self, new_lines: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for line in new_lines {
            self.lines.push_back(line.into());
        }

        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }

    pub fn filtered_lines(&self, filter: &str) -> Vec<usize> {
        let f = filter.trim().to_lowercase();
        if f.is_empty() {
            return (0..self.lines.len()).collect();
        }

        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&f))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn export_text(&self) -> String {
        self.lines.iter().cloned().collect::<Vec<_>>().join("\n")
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_enforces_max_capacity_and_filters_lines() {
        let mut session = LogSessionState::new("default".into(), "nginx".into(), None);
        session.max_lines = 1_000;

        let input_lines: Vec<String> = (0..1_200)
            .map(|i| {
                if i % 100 == 0 {
                    format!("[ERROR] line {}", i)
                } else {
                    format!("[INFO] line {}", i)
                }
            })
            .collect();

        session.append_lines(input_lines);

        assert_eq!(session.len(), 1_000);

        let error_indices = session.filtered_lines("ERROR");
        // Among the retained 1000 lines (indices 200..1200 of original):
        // 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100 -> exactly 10 ERROR lines
        assert_eq!(error_indices.len(), 10);
        for idx in error_indices {
            assert!(session.lines[idx].contains("ERROR"));
        }
    }
}
