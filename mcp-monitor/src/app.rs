use mcp_common::{LogEntry, LogLevel, ProxyId, ProxyInfo, ProxyStats};
use std::collections::HashMap;

#[derive(Debug)]
pub enum AppEvent {
    ProxyConnected(ProxyInfo),
    ProxyDisconnected(ProxyId),
    NewLogEntry(LogEntry),
    StatsUpdate(ProxyStats),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabType {
    All,
    Messages, // Request + Response only
    Errors,   // Error + Warning
    System,   // Info + Debug + connection/disconnection logs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMode {
    Follow,        // Automatically follow latest log
    Navigate,      // Manual navigation with selection
    Search,        // Search mode with filtering
    SearchResults, // Navigating search results (no dialog)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    ProxyList, // Focus on the proxy list (left panel)
    LogView,   // Focus on the log view (right panel)
}

pub struct App {
    pub proxies: HashMap<ProxyId, ProxyInfo>,
    pub logs: Vec<LogEntry>,
    pub selected_index: usize, // Currently selected item in the filtered list
    pub viewport_offset: usize, // First visible item in the viewport
    pub selected_proxy: Option<ProxyId>,
    pub proxy_selected_index: usize, // Currently selected proxy in the list
    pub focus_area: FocusArea,       // Which area has focus
    pub active_tab: TabType,
    pub tab_states: HashMap<TabType, ListState>, // Store selection and viewport for each tab
    pub selected_log_index: Option<usize>,
    pub show_detail_view: bool,
    pub detail_word_wrap: bool,
    pub detail_scroll_offset: u16, // Vertical scroll offset for detail view
    pub navigation_mode: NavigationMode,
    pub search_query: String,
    pub search_results: Vec<usize>, // Indices of matching logs in the main logs vector
    pub search_cursor: usize,       // Current cursor position in search input
    pub show_help_dialog: bool,     // Whether to show the help dialog
}

#[derive(Debug, Clone)]
pub struct ListState {
    pub selected_index: usize,
    pub viewport_offset: usize,
    pub navigation_mode: NavigationMode,
}

impl App {
    pub fn new() -> Self {
        let mut tab_states = HashMap::new();
        tab_states.insert(
            TabType::All,
            ListState {
                selected_index: 0,
                viewport_offset: 0,
                navigation_mode: NavigationMode::Follow,
            },
        );
        tab_states.insert(
            TabType::Messages,
            ListState {
                selected_index: 0,
                viewport_offset: 0,
                navigation_mode: NavigationMode::Follow,
            },
        );
        tab_states.insert(
            TabType::Errors,
            ListState {
                selected_index: 0,
                viewport_offset: 0,
                navigation_mode: NavigationMode::Follow,
            },
        );
        tab_states.insert(
            TabType::System,
            ListState {
                selected_index: 0,
                viewport_offset: 0,
                navigation_mode: NavigationMode::Follow,
            },
        );

        Self {
            proxies: HashMap::new(),
            logs: Vec::new(),
            selected_index: 0,
            viewport_offset: 0,
            selected_proxy: None,
            proxy_selected_index: 0,
            focus_area: FocusArea::LogView, // Default focus on logs
            active_tab: TabType::Messages,  // Default to Messages tab
            tab_states,
            selected_log_index: None,
            show_detail_view: false,
            detail_word_wrap: true,
            detail_scroll_offset: 0,
            navigation_mode: NavigationMode::Follow,
            search_query: String::new(),
            search_results: Vec::new(),
            search_cursor: 0,
            show_help_dialog: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ProxyConnected(info) => {
                self.proxies.insert(info.id.clone(), info);
            }
            AppEvent::ProxyDisconnected(id) => {
                self.proxies.remove(&id);
                if self.selected_proxy.as_ref() == Some(&id) {
                    self.selected_proxy = None;
                }
            }
            AppEvent::NewLogEntry(entry) => {
                // Store all logs without filtering (logs are added at the bottom)
                self.logs.push(entry);

                // Limit log size
                const MAX_LOGS: usize = 10000;
                if self.logs.len() > MAX_LOGS {
                    self.logs.drain(0..self.logs.len() - MAX_LOGS);

                    // Adjust selection if logs were removed
                    for state in self.tab_states.values_mut() {
                        if state.selected_index > 0 {
                            state.selected_index = state
                                .selected_index
                                .saturating_sub(self.logs.len() - MAX_LOGS);
                        }
                        if state.viewport_offset > 0 {
                            state.viewport_offset = state
                                .viewport_offset
                                .saturating_sub(self.logs.len() - MAX_LOGS);
                        }
                    }
                }

                // In follow mode, automatically select the latest log
                if self.navigation_mode == NavigationMode::Follow {
                    let filtered_logs = self.get_search_filtered_logs();
                    if !filtered_logs.is_empty() {
                        self.selected_index = filtered_logs.len() - 1;
                    }
                }
            }
            AppEvent::StatsUpdate(stats) => {
                if let Some(proxy) = self.proxies.get_mut(&stats.proxy_id) {
                    proxy.stats = stats;
                }
            }
        }
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.selected_index = 0;
        self.viewport_offset = 0;
        self.navigation_mode = NavigationMode::Follow;
        // Reset all tab states
        for state in self.tab_states.values_mut() {
            state.selected_index = 0;
            state.viewport_offset = 0;
            state.navigation_mode = NavigationMode::Follow;
        }
    }

    pub fn refresh(&mut self) {
        // Force refresh - in a real implementation, this might
        // send requests to proxies for updated stats
    }

    pub fn scroll_up(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_selection_visible();
            self.save_tab_state();
        }
    }

    pub fn scroll_down(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        let filtered_count = self.get_search_filtered_logs().len();
        if filtered_count > 0 && self.selected_index < filtered_count - 1 {
            self.selected_index += 1;
            self.ensure_selection_visible();
            self.save_tab_state();
        }
    }

    pub fn page_up(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        let page_size = 10;
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.ensure_selection_visible();
        self.save_tab_state();
    }

    pub fn page_down(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        let page_size = 10;
        let filtered_count = self.get_search_filtered_logs().len();
        if filtered_count > 0 {
            self.selected_index = (self.selected_index + page_size).min(filtered_count - 1);
            self.ensure_selection_visible();
            self.save_tab_state();
        }
    }

    pub fn scroll_to_top(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        self.selected_index = 0;
        self.viewport_offset = 0;
        self.save_tab_state();
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.navigation_mode == NavigationMode::Follow {
            self.navigation_mode = NavigationMode::Navigate;
        }
        let filtered_logs = self.get_search_filtered_logs();
        if !filtered_logs.is_empty() {
            self.selected_index = filtered_logs.len() - 1;
            self.ensure_selection_visible();
            self.save_tab_state();
        }
    }

    pub fn exit_navigation_mode(&mut self) {
        if self.navigation_mode == NavigationMode::Search
            || self.navigation_mode == NavigationMode::SearchResults
        {
            self.exit_search_mode();
        } else {
            self.navigation_mode = NavigationMode::Follow;
            // Go to the latest log
            let filtered_logs = self.get_search_filtered_logs();
            if !filtered_logs.is_empty() {
                self.selected_index = filtered_logs.len() - 1;
                self.ensure_selection_visible();
                self.save_tab_state();
            }
        }
    }

    fn ensure_selection_visible(&mut self) {
        // This will be called with the viewport height from the UI
        // For now, we'll just ensure the viewport follows the selection
        // The actual viewport adjustment will happen in get_visible_logs_with_selection
    }

    fn save_tab_state(&mut self) {
        if let Some(state) = self.tab_states.get_mut(&self.active_tab) {
            state.selected_index = self.selected_index;
            state.viewport_offset = self.viewport_offset;
            state.navigation_mode = self.navigation_mode;
        }
    }

    // Focus and proxy selection methods
    pub fn switch_focus_to_proxy_list(&mut self) {
        self.focus_area = FocusArea::ProxyList;
    }

    pub fn switch_focus_to_logs(&mut self) {
        self.focus_area = FocusArea::LogView;
    }

    pub fn proxy_scroll_up(&mut self) {
        if self.proxy_selected_index > 0 {
            self.proxy_selected_index -= 1;
        }
    }

    pub fn proxy_scroll_down(&mut self) {
        let proxy_count = self.get_proxy_list().len();
        if proxy_count > 0 && self.proxy_selected_index < proxy_count - 1 {
            self.proxy_selected_index += 1;
        }
    }

    pub fn select_current_proxy(&mut self) {
        let proxy_list = self.get_proxy_list();
        if self.proxy_selected_index < proxy_list.len() {
            let selected_proxy_id = proxy_list[self.proxy_selected_index].id.clone();
            self.selected_proxy = Some(selected_proxy_id);

            // Reset log selection to latest when changing proxy filter
            self.navigation_mode = NavigationMode::Follow;
            let filtered_logs = self.get_filtered_logs();
            if !filtered_logs.is_empty() {
                self.selected_index = filtered_logs.len() - 1;
            } else {
                self.selected_index = 0;
            }
            self.viewport_offset = 0;
            self.save_tab_state();
        }
    }

    pub fn clear_proxy_selection(&mut self) {
        self.selected_proxy = None;

        // Reset log selection to latest when clearing proxy filter
        self.navigation_mode = NavigationMode::Follow;
        let filtered_logs = self.get_filtered_logs();
        if !filtered_logs.is_empty() {
            self.selected_index = filtered_logs.len() - 1;
        } else {
            self.selected_index = 0;
        }
        self.viewport_offset = 0;
        self.save_tab_state();
    }

    pub fn tick(&mut self) {
        // Called periodically for any time-based updates
    }

    pub fn prepare_viewport(&mut self, height: usize) {
        let filtered_count = self.get_search_filtered_logs().len();

        if filtered_count == 0 {
            self.selected_index = 0;
            self.viewport_offset = 0;
            return;
        }

        // Ensure selected index is valid
        if self.selected_index >= filtered_count {
            self.selected_index = filtered_count.saturating_sub(1);
        }

        // Calculate viewport to keep selection visible
        if height > 0 {
            // If selection is above viewport, scroll up
            if self.selected_index < self.viewport_offset {
                self.viewport_offset = self.selected_index;
            }
            // If selection is below viewport, scroll down
            else if self.selected_index >= self.viewport_offset + height {
                self.viewport_offset = self.selected_index.saturating_sub(height - 1);
            }
        }
    }

    pub fn get_visible_logs(&self, height: usize) -> Vec<&LogEntry> {
        let filtered_logs = self.get_search_filtered_logs();

        if filtered_logs.is_empty() || height == 0 {
            return vec![];
        }

        // Ensure viewport_offset is valid
        let start = self
            .viewport_offset
            .min(filtered_logs.len().saturating_sub(1));

        // Get visible range, limited by height
        let end = (start + height).min(filtered_logs.len());
        filtered_logs[start..end].to_vec()
    }

    pub fn get_relative_selection(&self, height: usize) -> Option<usize> {
        let filtered_logs = self.get_search_filtered_logs();
        if filtered_logs.is_empty() {
            return None;
        }

        let end = (self.viewport_offset + height).min(filtered_logs.len());

        // Calculate relative selection position within viewport
        if self.selected_index >= self.viewport_offset && self.selected_index < end {
            Some(self.selected_index - self.viewport_offset)
        } else {
            None
        }
    }

    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|log| {
                // First apply proxy filter if any
                if let Some(ref selected_proxy) = self.selected_proxy {
                    if &log.proxy_id != selected_proxy {
                        return false;
                    }
                }

                // Then apply tab filter
                match self.active_tab {
                    TabType::All => true,
                    TabType::Messages => {
                        matches!(log.level, LogLevel::Request | LogLevel::Response)
                    }
                    TabType::Errors => matches!(log.level, LogLevel::Error | LogLevel::Warning),
                    TabType::System => matches!(log.level, LogLevel::Info | LogLevel::Debug),
                }
            })
            .collect()
    }

    pub fn switch_tab(&mut self, tab: TabType) {
        // Save current state
        self.save_tab_state();

        // Switch to new tab
        self.active_tab = tab;

        // Restore state for new tab
        if let Some(state) = self.tab_states.get(&tab) {
            self.selected_index = state.selected_index;
            self.viewport_offset = state.viewport_offset;
            self.navigation_mode = state.navigation_mode;
        }

        // Ensure indices are valid for the filtered logs
        let filtered_count = self.get_filtered_logs().len();
        if filtered_count == 0 {
            self.selected_index = 0;
            self.viewport_offset = 0;
        } else if self.selected_index >= filtered_count {
            self.selected_index = filtered_count - 1;
        }
    }

    pub fn next_tab(&mut self) {
        let next_tab = match self.active_tab {
            TabType::All => TabType::Messages,
            TabType::Messages => TabType::Errors,
            TabType::Errors => TabType::System,
            TabType::System => TabType::All,
        };
        self.switch_tab(next_tab);
    }

    pub fn prev_tab(&mut self) {
        let prev_tab = match self.active_tab {
            TabType::All => TabType::System,
            TabType::Messages => TabType::All,
            TabType::Errors => TabType::Messages,
            TabType::System => TabType::Errors,
        };
        self.switch_tab(prev_tab);
    }

    pub fn get_tab_log_count(&self, tab: TabType) -> usize {
        self.logs
            .iter()
            .filter(|log| {
                // Apply proxy filter if any
                if let Some(ref selected_proxy) = self.selected_proxy {
                    if &log.proxy_id != selected_proxy {
                        return false;
                    }
                }

                // Apply tab filter
                match tab {
                    TabType::All => true,
                    TabType::Messages => {
                        matches!(log.level, LogLevel::Request | LogLevel::Response)
                    }
                    TabType::Errors => matches!(log.level, LogLevel::Error | LogLevel::Warning),
                    TabType::System => matches!(log.level, LogLevel::Info | LogLevel::Debug),
                }
            })
            .count()
    }

    pub fn get_proxy_list(&self) -> Vec<&ProxyInfo> {
        let mut proxies: Vec<_> = self.proxies.values().collect();
        proxies.sort_by(|a, b| a.name.cmp(&b.name));
        proxies
    }

    pub fn total_stats(&self) -> ProxyStats {
        let mut total = ProxyStats::default();

        for proxy in self.proxies.values() {
            total.total_requests += proxy.stats.total_requests;
            total.successful_requests += proxy.stats.successful_requests;
            total.failed_requests += proxy.stats.failed_requests;
            total.active_connections += proxy.stats.active_connections;
            total.bytes_transferred += proxy.stats.bytes_transferred;
        }

        total
    }

    // Log selection methods
    pub fn select_log_at_cursor(&mut self) {
        let filtered_logs = self.get_search_filtered_logs();
        if !filtered_logs.is_empty() && self.selected_index < filtered_logs.len() {
            // Find the index of the selected log in the full logs vector
            let selected_log = filtered_logs[self.selected_index];
            if let Some(index) = self
                .logs
                .iter()
                .position(|log| std::ptr::eq(log, selected_log))
            {
                self.selected_log_index = Some(index);
            }
        }
    }

    pub fn show_selected_log_detail(&mut self) {
        if let Some(index) = self.selected_log_index {
            if index < self.logs.len() {
                let log = &self.logs[index];
                // Only show detail for Request/Response logs that have meaningful content
                if matches!(log.level, LogLevel::Request | LogLevel::Response) {
                    self.show_detail_view = true;
                }
            }
        }
    }

    pub fn hide_detail_view(&mut self) {
        self.show_detail_view = false;
        self.selected_log_index = None;
        self.detail_scroll_offset = 0; // Reset scroll when hiding
    }

    pub fn toggle_word_wrap(&mut self) {
        self.detail_word_wrap = !self.detail_word_wrap;
        self.detail_scroll_offset = 0; // Reset scroll when toggling wrap
    }

    pub fn detail_scroll_up(&mut self) {
        self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(3);
    }

    pub fn detail_scroll_down(&mut self) {
        self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(3);
    }

    pub fn get_selected_log(&self) -> Option<&LogEntry> {
        if let Some(index) = self.selected_log_index {
            self.logs.get(index)
        } else {
            None
        }
    }

    pub fn format_log_content(&self, log: &LogEntry) -> String {
        // First priority: format metadata as pretty JSON if available
        if let Some(ref metadata) = log.metadata {
            match serde_json::to_string_pretty(metadata) {
                Ok(formatted) => {
                    return format!(
                        "=== METADATA ===\n{}\n\n=== MESSAGE ===\n{}",
                        formatted,
                        self.format_message_content(&log.message)
                    );
                }
                Err(_) => {}
            }
        }

        // Second priority: just format the message
        self.format_message_content(&log.message)
    }

    fn format_message_content(&self, message: &str) -> String {
        let trimmed = message.trim();

        // Clean up the message by removing common prefixes and control characters
        let cleaned = self.clean_json_message(trimmed);

        // Try to parse the cleaned message as JSON and format it
        match serde_json::from_str::<serde_json::Value>(&cleaned) {
            Ok(json_value) => {
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => {
                        return formatted; // Return just the formatted JSON
                    }
                    Err(_) => {
                        return format!("JSON parse success but format failed:\n{}", cleaned);
                    }
                }
            }
            Err(_) => {
                // If cleaning didn't work, try parsing the original
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if let Ok(formatted) = serde_json::to_string_pretty(&json_value) {
                        return formatted;
                    }
                }
            }
        }

        // If it's not JSON, return the original message
        trimmed.to_string()
    }

    fn clean_json_message(&self, message: &str) -> String {
        let mut cleaned = message.to_string();

        // Remove common prefixes that might interfere with JSON parsing
        let prefixes_to_remove = ["<-", "->", "<<", ">>", "IN:", "OUT:", "REQ:", "RESP:"];

        for prefix in &prefixes_to_remove {
            if cleaned.trim_start().starts_with(prefix) {
                cleaned = cleaned
                    .trim_start()
                    .strip_prefix(prefix)
                    .unwrap_or(&cleaned)
                    .to_string();
                cleaned = cleaned.trim_start().to_string(); // Remove any remaining whitespace
                break;
            }
        }

        // Remove any leading/trailing whitespace and control characters
        cleaned = cleaned.trim().to_string();

        // Remove any remaining non-printable characters at the start
        while cleaned.chars().next().map_or(false, |c| {
            c.is_control() && c != '\n' && c != '\r' && c != '\t'
        }) {
            cleaned = cleaned.chars().skip(1).collect();
        }

        cleaned
    }

    // Search mode methods
    pub fn enter_search_mode(&mut self) {
        self.navigation_mode = NavigationMode::Search;
        self.search_query.clear();
        self.search_results.clear();
        self.search_cursor = 0;
        self.selected_index = 0;
        self.viewport_offset = 0;
    }

    pub fn exit_search_mode(&mut self) {
        self.navigation_mode = NavigationMode::Navigate;
        self.search_query.clear();
        self.search_results.clear();
        self.search_cursor = 0;

        // Return to regular filtered view
        let filtered_logs = self.get_filtered_logs();
        if !filtered_logs.is_empty() {
            self.selected_index = filtered_logs.len() - 1;
            self.ensure_selection_visible();
        }
        self.save_tab_state();
    }

    pub fn confirm_search_results(&mut self) {
        // Switch to SearchResults mode to keep the search results visible
        self.navigation_mode = NavigationMode::SearchResults;
        self.search_cursor = 0;

        // Keep the current selection and viewport
        self.save_tab_state();
    }

    pub fn search_input_char(&mut self, c: char) {
        if self.navigation_mode == NavigationMode::Search {
            self.search_query.insert(self.search_cursor, c);
            self.search_cursor += 1;
            self.update_search_results();
        }
    }

    pub fn search_backspace(&mut self) {
        if self.navigation_mode == NavigationMode::Search && self.search_cursor > 0 {
            self.search_cursor -= 1;
            self.search_query.remove(self.search_cursor);
            self.update_search_results();
        }
    }

    pub fn search_delete(&mut self) {
        if self.navigation_mode == NavigationMode::Search
            && self.search_cursor < self.search_query.len()
        {
            self.search_query.remove(self.search_cursor);
            self.update_search_results();
        }
    }

    pub fn search_cursor_left(&mut self) {
        if self.navigation_mode == NavigationMode::Search && self.search_cursor > 0 {
            self.search_cursor -= 1;
        }
    }

    pub fn search_cursor_right(&mut self) {
        if self.navigation_mode == NavigationMode::Search
            && self.search_cursor < self.search_query.len()
        {
            self.search_cursor += 1;
        }
    }

    pub fn search_cursor_home(&mut self) {
        if self.navigation_mode == NavigationMode::Search {
            self.search_cursor = 0;
        }
    }

    pub fn search_cursor_end(&mut self) {
        if self.navigation_mode == NavigationMode::Search {
            self.search_cursor = self.search_query.len();
        }
    }

    fn update_search_results(&mut self) {
        self.search_results.clear();

        if self.search_query.is_empty() {
            self.selected_index = 0;
            self.viewport_offset = 0;
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        // Find matching log indices
        for (index, log) in self.logs.iter().enumerate() {
            // Apply proxy filter if any
            if let Some(ref selected_proxy) = self.selected_proxy {
                if &log.proxy_id != selected_proxy {
                    continue;
                }
            }

            // Apply tab filter
            let matches_tab = match self.active_tab {
                TabType::All => true,
                TabType::Messages => matches!(log.level, LogLevel::Request | LogLevel::Response),
                TabType::Errors => matches!(log.level, LogLevel::Error | LogLevel::Warning),
                TabType::System => matches!(log.level, LogLevel::Info | LogLevel::Debug),
            };

            if !matches_tab {
                continue;
            }

            // Check if log matches search query (case-insensitive)
            let message_matches = log.message.to_lowercase().contains(&query_lower);
            let proxy_name_matches = self
                .proxies
                .get(&log.proxy_id)
                .map(|p| p.name.to_lowercase().contains(&query_lower))
                .unwrap_or(false);
            let level_matches = format!("{:?}", log.level)
                .to_lowercase()
                .contains(&query_lower);

            if message_matches || proxy_name_matches || level_matches {
                self.search_results.push(index);
            }
        }

        // Reset selection to first result
        self.selected_index = 0;
        self.viewport_offset = 0;
    }

    pub fn get_search_filtered_logs(&self) -> Vec<&LogEntry> {
        if self.navigation_mode == NavigationMode::Search
            || self.navigation_mode == NavigationMode::SearchResults
        {
            self.search_results
                .iter()
                .filter_map(|&index| self.logs.get(index))
                .collect()
        } else {
            self.get_filtered_logs()
        }
    }
}
