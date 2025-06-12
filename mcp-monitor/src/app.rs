use mcp_common::{LogEntry, LogLevel, ProxyId, ProxyInfo, ProxyStats};
use std::collections::HashMap;

#[derive(Debug)]
pub enum AppEvent {
    ProxyConnected(ProxyInfo),
    ProxyDisconnected(ProxyId),
    NewLogEntry(LogEntry),
    StatsUpdate(ProxyStats),
}

pub struct App {
    pub proxies: HashMap<ProxyId, ProxyInfo>,
    pub logs: Vec<LogEntry>,
    pub scroll_offset: usize,
    pub selected_proxy: Option<ProxyId>,
    pub show_stats: bool,
    pub filter_level: Option<LogLevel>,
}

impl App {
    pub fn new() -> Self {
        Self {
            proxies: HashMap::new(),
            logs: Vec::new(),
            scroll_offset: 0,
            selected_proxy: None,
            show_stats: true,
            filter_level: None,
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
                // Apply filters
                if let Some(ref filter_level) = self.filter_level {
                    if !self.matches_filter_level(&entry.level, filter_level) {
                        return;
                    }
                }

                if let Some(ref selected_proxy) = self.selected_proxy {
                    if &entry.proxy_id != selected_proxy {
                        return;
                    }
                }

                self.logs.push(entry);
                
                // Limit log size
                const MAX_LOGS: usize = 10000;
                if self.logs.len() > MAX_LOGS {
                    self.logs.drain(0..self.logs.len() - MAX_LOGS);
                }

                // Auto-scroll to bottom if we're already at the bottom
                if self.scroll_offset + 20 >= self.logs.len() {
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

    fn matches_filter_level(&self, entry_level: &LogLevel, filter_level: &LogLevel) -> bool {
        use LogLevel::*;
        match filter_level {
            Error => matches!(entry_level, Error),
            Warning => matches!(entry_level, Error | Warning),
            Info => matches!(entry_level, Error | Warning | Info),
            Debug => true, // Show all
            Request => matches!(entry_level, Request),
            Response => matches!(entry_level, Response),
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
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset + 1 < self.logs.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    pub fn page_down(&mut self) {
        if self.scroll_offset + 10 < self.logs.len() {
            self.scroll_offset += 10;
        } else {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        if !self.logs.is_empty() {
            self.scroll_offset = self.logs.len().saturating_sub(1);
        }
    }

    pub fn tick(&mut self) {
        // Called periodically for any time-based updates
    }

    pub fn get_visible_logs(&self, height: usize) -> &[LogEntry] {
        let start = self.scroll_offset;
        let end = (start + height).min(self.logs.len());
        &self.logs[start..end]
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
}