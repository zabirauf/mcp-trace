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
    Messages,  // Request + Response only
    Errors,    // Error + Warning
    System,    // Info + Debug + connection/disconnection logs
}

pub struct App {
    pub proxies: HashMap<ProxyId, ProxyInfo>,
    pub logs: Vec<LogEntry>,
    pub scroll_offset: usize,
    pub selected_proxy: Option<ProxyId>,
    pub active_tab: TabType,
    pub tab_scroll_offsets: HashMap<TabType, usize>,
    pub selected_log_index: Option<usize>,
    pub show_detail_view: bool,
    pub detail_word_wrap: bool,
}

impl App {
    pub fn new() -> Self {
        let mut tab_scroll_offsets = HashMap::new();
        tab_scroll_offsets.insert(TabType::All, 0);
        tab_scroll_offsets.insert(TabType::Messages, 0);
        tab_scroll_offsets.insert(TabType::Errors, 0);
        tab_scroll_offsets.insert(TabType::System, 0);
        
        Self {
            proxies: HashMap::new(),
            logs: Vec::new(),
            scroll_offset: 0,
            selected_proxy: None,
            active_tab: TabType::Messages,  // Default to Messages tab
            tab_scroll_offsets,
            selected_log_index: None,
            show_detail_view: false,
            detail_word_wrap: true,
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
                // Store all logs without filtering
                self.logs.push(entry);
                
                // Limit log size
                const MAX_LOGS: usize = 10000;
                if self.logs.len() > MAX_LOGS {
                    self.logs.drain(0..self.logs.len() - MAX_LOGS);
                }

                // Auto-scroll to bottom if we're already at the bottom for the current tab
                let filtered_logs = self.get_filtered_logs();
                if self.scroll_offset + 20 >= filtered_logs.len() {
                    self.scroll_to_bottom();
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
        self.scroll_offset = 0;
    }

    pub fn refresh(&mut self) {
        // Force refresh - in a real implementation, this might 
        // send requests to proxies for updated stats
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
        }
    }

    pub fn scroll_down(&mut self) {
        let filtered_count = self.get_filtered_logs().len();
        if self.scroll_offset + 1 < filtered_count {
            self.scroll_offset += 1;
            self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
        }
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
        self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
    }

    pub fn page_down(&mut self) {
        let filtered_count = self.get_filtered_logs().len();
        if self.scroll_offset + 10 < filtered_count {
            self.scroll_offset += 10;
        } else {
            self.scroll_to_bottom();
        }
        self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
    }

    pub fn scroll_to_bottom(&mut self) {
        let filtered_logs = self.get_filtered_logs();
        if !filtered_logs.is_empty() {
            self.scroll_offset = filtered_logs.len().saturating_sub(1);
            self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
        }
    }

    pub fn tick(&mut self) {
        // Called periodically for any time-based updates
    }

    pub fn get_visible_logs(&self, height: usize) -> Vec<&LogEntry> {
        let filtered_logs = self.get_filtered_logs();
        let start = self.scroll_offset;
        let end = (start + height).min(filtered_logs.len());
        filtered_logs[start..end].to_vec()
    }
    
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs.iter().filter(|log| {
            // First apply proxy filter if any
            if let Some(ref selected_proxy) = self.selected_proxy {
                if &log.proxy_id != selected_proxy {
                    return false;
                }
            }
            
            // Then apply tab filter
            match self.active_tab {
                TabType::All => true,
                TabType::Messages => matches!(log.level, LogLevel::Request | LogLevel::Response),
                TabType::Errors => matches!(log.level, LogLevel::Error | LogLevel::Warning),
                TabType::System => matches!(log.level, LogLevel::Info | LogLevel::Debug),
            }
        }).collect()
    }
    
    pub fn switch_tab(&mut self, tab: TabType) {
        // Save current scroll position
        self.tab_scroll_offsets.insert(self.active_tab, self.scroll_offset);
        
        // Switch to new tab
        self.active_tab = tab;
        
        // Restore scroll position for new tab
        self.scroll_offset = *self.tab_scroll_offsets.get(&tab).unwrap_or(&0);
        
        // Ensure scroll offset is valid for the filtered logs
        let filtered_count = self.get_filtered_logs().len();
        if self.scroll_offset >= filtered_count {
            self.scroll_offset = filtered_count.saturating_sub(1);
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
        self.logs.iter().filter(|log| {
            // Apply proxy filter if any
            if let Some(ref selected_proxy) = self.selected_proxy {
                if &log.proxy_id != selected_proxy {
                    return false;
                }
            }
            
            // Apply tab filter
            match tab {
                TabType::All => true,
                TabType::Messages => matches!(log.level, LogLevel::Request | LogLevel::Response),
                TabType::Errors => matches!(log.level, LogLevel::Error | LogLevel::Warning),
                TabType::System => matches!(log.level, LogLevel::Info | LogLevel::Debug),
            }
        }).count()
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
        let filtered_logs = self.get_filtered_logs();
        if !filtered_logs.is_empty() && self.scroll_offset < filtered_logs.len() {
            // Find the index of the selected log in the full logs vector
            let selected_log = filtered_logs[self.scroll_offset];
            if let Some(index) = self.logs.iter().position(|log| std::ptr::eq(log, selected_log)) {
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
    }
    
    pub fn toggle_word_wrap(&mut self) {
        self.detail_word_wrap = !self.detail_word_wrap;
    }
    
    pub fn get_selected_log(&self) -> Option<&LogEntry> {
        if let Some(index) = self.selected_log_index {
            self.logs.get(index)
        } else {
            None
        }
    }
    
    pub fn format_log_content(&self, log: &LogEntry) -> String {
        // Try to format metadata as pretty JSON if available
        if let Some(ref metadata) = log.metadata {
            match serde_json::to_string_pretty(metadata) {
                Ok(formatted) => return formatted,
                Err(_) => {},
            }
        }
        
        // Try to parse the message as JSON and format it
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&log.message) {
            match serde_json::to_string_pretty(&json_value) {
                Ok(formatted) => return formatted,
                Err(_) => {},
            }
        }
        
        // Fallback to the raw message
        log.message.clone()
    }
}