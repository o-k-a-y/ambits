use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::symbols::{ProjectTree, SymbolNode};
use crate::tracking::ReadDepth;
use crate::tracking::ContextLedger;
use crate::ingest::AgentToolCall;

/// A flattened row in the tree view, ready for rendering.
#[derive(Debug, Clone)]
pub struct TreeRow {
    pub symbol_id: String,
    pub display_name: String,
    pub kind_label: String,
    pub depth: usize,         // nesting depth for indentation
    pub is_file: bool,        // true for file headers
    pub is_expanded: bool,
    pub has_children: bool,
    pub line_range: String,
    pub token_count: usize,
    pub read_depth: ReadDepth,
}

/// Which panel is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Tree,
    Stats,
    Activity,
}

pub struct App {
    pub project_tree: ProjectTree,
    pub project_root: PathBuf,
    pub ledger: ContextLedger,
    pub should_quit: bool,

    // Tree view state.
    pub tree_rows: Vec<TreeRow>,
    pub selected_index: usize,
    pub collapsed: std::collections::HashSet<String>,

    // Activity feed.
    pub activity: Vec<AgentToolCall>,

    // Agents seen.
    pub agents_seen: Vec<String>,

    // Agent filter: if Some, only show coverage from this agent.
    pub agent_filter: Option<String>,

    // Focus.
    pub focus: FocusPanel,

    // Search.
    pub search_mode: bool,
    pub search_query: String,

    // Session info for display.
    pub session_id: Option<String>,
}

impl App {
    pub fn new(project_tree: ProjectTree, project_root: PathBuf) -> Self {
        // Start with all files collapsed.
        let collapsed: std::collections::HashSet<String> = project_tree
            .files
            .iter()
            .map(|f| f.file_path.to_string_lossy().to_string())
            .collect();

        let mut app = Self {
            project_tree,
            project_root,
            ledger: ContextLedger::new(),
            should_quit: false,
            tree_rows: Vec::new(),
            selected_index: 0,
            collapsed,
            activity: Vec::new(),
            agents_seen: Vec::new(),
            agent_filter: None,
            focus: FocusPanel::Tree,
            search_mode: false,
            search_query: String::new(),
            session_id: None,
        };
        app.rebuild_tree_rows();
        app
    }

    /// Rebuild the flattened tree rows from the project tree + collapsed state.
    pub fn rebuild_tree_rows(&mut self) {
        let mut rows = Vec::new();

        for file in &self.project_tree.files {
            let file_path = file.file_path.to_string_lossy().to_string();
            let file_id = file_path.clone();
            let is_expanded = !self.collapsed.contains(&file_id);

            rows.push(TreeRow {
                symbol_id: file_id.clone(),
                display_name: file_path.clone(),
                kind_label: String::new(),
                depth: 0,
                is_file: true,
                is_expanded,
                has_children: !file.symbols.is_empty(),
                line_range: format!("{} lines", file.total_lines),
                token_count: 0,
                read_depth: ReadDepth::Unseen,
            });

            if is_expanded {
                for sym in &file.symbols {
                    flatten_symbol(sym, 1, &self.collapsed, &self.ledger, &mut rows);
                }
            }
        }

        self.tree_rows = rows;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.search_mode {
            self.handle_search_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => self.toggle_expand(),
            KeyCode::Char('h') | KeyCode::Left => self.collapse_current(),
            KeyCode::Char('G') => self.select_last(),
            KeyCode::Char('g') => self.select_first(),
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
            }
            KeyCode::Char('a') => self.cycle_agent_filter(),
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::PageDown => self.move_selection(20),
            KeyCode::PageUp => self.move_selection(-20),
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.search_mode = false;
                self.jump_to_search_match();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.tree_rows.is_empty() {
            return;
        }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, self.tree_rows.len() as i32 - 1) as usize;
    }

    fn select_first(&mut self) {
        self.selected_index = 0;
    }

    fn select_last(&mut self) {
        if !self.tree_rows.is_empty() {
            self.selected_index = self.tree_rows.len() - 1;
        }
    }

    fn toggle_expand(&mut self) {
        if let Some(row) = self.tree_rows.get(self.selected_index) {
            if row.has_children {
                let id = row.symbol_id.clone();
                if self.collapsed.contains(&id) {
                    self.collapsed.remove(&id);
                } else {
                    self.collapsed.insert(id);
                }
                self.rebuild_tree_rows();
            }
        }
    }

    fn collapse_current(&mut self) {
        if let Some(row) = self.tree_rows.get(self.selected_index) {
            let id = row.symbol_id.clone();
            if row.has_children && !self.collapsed.contains(&id) {
                self.collapsed.insert(id);
                self.rebuild_tree_rows();
            }
        }
    }

    fn cycle_agent_filter(&mut self) {
        if self.agents_seen.is_empty() {
            self.agent_filter = None;
            return;
        }
        match &self.agent_filter {
            None => {
                self.agent_filter = Some(self.agents_seen[0].clone());
            }
            Some(current) => {
                let idx = self.agents_seen.iter().position(|a| a == current);
                match idx {
                    Some(i) if i + 1 < self.agents_seen.len() => {
                        self.agent_filter = Some(self.agents_seen[i + 1].clone());
                    }
                    _ => {
                        self.agent_filter = None;
                    }
                }
            }
        }
        self.rebuild_tree_rows();
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Tree => FocusPanel::Stats,
            FocusPanel::Stats => FocusPanel::Activity,
            FocusPanel::Activity => FocusPanel::Tree,
        };
    }

    fn jump_to_search_match(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        // Search forward from current position.
        let start = (self.selected_index + 1) % self.tree_rows.len();
        for i in 0..self.tree_rows.len() {
            let idx = (start + i) % self.tree_rows.len();
            if self.tree_rows[idx]
                .display_name
                .to_lowercase()
                .contains(&query)
            {
                self.selected_index = idx;
                return;
            }
        }
    }

    /// Process an agent tool call event and update the ledger.
    pub fn process_agent_event(&mut self, event: AgentToolCall) {
        // Track unique agents.
        if !self.agents_seen.contains(&event.agent_id) {
            self.agents_seen.push(event.agent_id.clone());
        }

        if let Some(ref file_path) = event.file_path {
            // Normalize the tool call path: strip the project root to get a relative path.
            let tool_rel = normalize_tool_path(file_path, &self.project_root);

            for file in &self.project_tree.files {
                if file.file_path == tool_rel {
                    mark_file_symbols(&file.symbols, &event, &mut self.ledger);
                }
            }
        }
        self.activity.push(event);
        if self.activity.len() > 200 {
            self.activity.drain(0..100);
        }
        self.rebuild_tree_rows();
    }
}

fn flatten_symbol(
    sym: &SymbolNode,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    ledger: &ContextLedger,
    rows: &mut Vec<TreeRow>,
) {
    let is_expanded = !collapsed.contains(&sym.id);
    let read_depth = ledger.depth_of(&sym.id);

    rows.push(TreeRow {
        symbol_id: sym.id.clone(),
        display_name: sym.name.clone(),
        kind_label: sym.kind.to_string(),
        depth,
        is_file: false,
        is_expanded,
        has_children: !sym.children.is_empty(),
        line_range: format!("L{}-{}", sym.line_range.start, sym.line_range.end),
        token_count: sym.estimated_tokens,
        read_depth,
    });

    if is_expanded {
        for child in &sym.children {
            flatten_symbol(child, depth + 1, collapsed, ledger, rows);
        }
    }
}

/// Convert a tool call file path (usually absolute) to a relative path matching
/// the project tree's convention. Strips the project root prefix if present.
fn normalize_tool_path(tool_path: &Path, project_root: &Path) -> PathBuf {
    if tool_path.is_absolute() {
        tool_path
            .strip_prefix(project_root)
            .unwrap_or(tool_path)
            .to_path_buf()
    } else {
        tool_path.to_path_buf()
    }
}

fn mark_file_symbols(
    symbols: &[SymbolNode],
    event: &AgentToolCall,
    ledger: &mut ContextLedger,
) {
    for sym in symbols {
        ledger.record(
            sym.id.clone(),
            event.read_depth,
            sym.content_hash,
            event.agent_id.clone(),
            sym.estimated_tokens,
        );
        mark_file_symbols(&sym.children, event, ledger);
    }
}
