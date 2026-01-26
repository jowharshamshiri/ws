use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::io::{stdout, Write};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{MoveTo, Show, Hide},
    style::{Color, SetForegroundColor, ResetColor, Print},
};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<TreeNode>,
    pub depth: usize,
}

pub struct InteractiveTree {
    root: TreeNode,
    selected_nodes: HashSet<PathBuf>,
    cursor_position: usize,
    scroll_offset: usize,
    flattened_nodes: Vec<TreeNode>,
    max_depth: Option<usize>,
    show_hidden: bool,
    callback: Option<Box<dyn Fn(&HashSet<PathBuf>) -> Result<()> + Send + Sync>>,
}

impl TreeNode {
    pub fn new(path: PathBuf, depth: usize) -> Self {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        
        let is_dir = path.is_dir();
        
        Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            children: Vec::new(),
            depth,
        }
    }

    pub fn load_children(&mut self, max_depth: Option<usize>, show_hidden: bool) -> Result<()> {
        if !self.is_dir || (max_depth.is_some() && self.depth >= max_depth.unwrap()) {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.path)?;
        let mut children = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if !show_hidden && path.file_name()
                .map(|name| name.to_string_lossy().starts_with('.'))
                .unwrap_or(false) {
                continue;
            }

            let mut child = TreeNode::new(path, self.depth + 1);
            if child.is_dir {
                child.load_children(max_depth, show_hidden)?;
            }
            children.push(child);
        }

        children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.children = children;
        Ok(())
    }

    pub fn toggle_expand(&mut self) {
        if self.is_dir {
            self.is_expanded = !self.is_expanded;
        }
    }

    pub fn expand_all(&mut self) {
        if self.is_dir {
            self.is_expanded = true;
            for child in &mut self.children {
                child.expand_all();
            }
        }
    }

    pub fn collapse_all(&mut self) {
        if self.is_dir {
            self.is_expanded = false;
            for child in &mut self.children {
                child.collapse_all();
            }
        }
    }
}

impl InteractiveTree {
    pub fn new(root_path: &Path, max_depth: Option<usize>, show_hidden: bool) -> Result<Self> {
        let mut root = TreeNode::new(root_path.to_path_buf(), 0);
        root.load_children(max_depth, show_hidden)?;
        root.is_expanded = true;

        let mut tree = Self {
            root,
            selected_nodes: HashSet::new(),
            cursor_position: 0,
            scroll_offset: 0,
            flattened_nodes: Vec::new(),
            max_depth,
            show_hidden,
            callback: None,
        };

        tree.update_flattened_nodes();
        Ok(tree)
    }

    pub fn set_callback<F>(&mut self, callback: F) 
    where 
        F: Fn(&HashSet<PathBuf>) -> Result<()> + Send + Sync + 'static 
    {
        self.callback = Some(Box::new(callback));
    }

    fn update_flattened_nodes(&mut self) {
        self.flattened_nodes.clear();
        self.flatten_node(&self.root.clone());
    }

    fn flatten_node(&mut self, node: &TreeNode) {
        self.flattened_nodes.push(node.clone());
        
        if node.is_expanded {
            for child in &node.children {
                self.flatten_node(child);
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Use alternate screen to avoid disturbing terminal history
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
        enable_raw_mode()?;

        let result = self.main_loop();

        // Clean exit - restore original terminal state
        disable_raw_mode()?;
        execute!(stdout(), Show, DisableMouseCapture, LeaveAlternateScreen)?;

        result
    }

    fn main_loop(&mut self) -> Result<()> {
        // Initial draw - only once
        self.draw()?;
        
        loop {
            // Wait for events - only redraw when something actually changes
            match event::read()? {
                Event::Key(key_event) => {
                    let old_cursor = self.cursor_position;
                    let old_scroll = self.scroll_offset;
                    let old_selected_count = self.selected_nodes.len();
                    let old_flattened_count = self.flattened_nodes.len();
                    
                    if self.handle_key_event(key_event)? {
                        break;
                    }
                    
                    // Redraw if cursor, scroll, selection, or tree structure changed
                    if self.cursor_position != old_cursor || 
                       self.scroll_offset != old_scroll || 
                       self.selected_nodes.len() != old_selected_count ||
                       self.flattened_nodes.len() != old_flattened_count {
                        self.draw()?;
                    }
                }
                Event::Mouse(mouse_event) => {
                    let old_cursor = self.cursor_position;
                    let old_selected_count = self.selected_nodes.len();
                    let old_flattened_count = self.flattened_nodes.len();
                    
                    self.handle_mouse_event(mouse_event)?;
                    
                    // Redraw if cursor, selection, or tree structure changed
                    if self.cursor_position != old_cursor || 
                       self.selected_nodes.len() != old_selected_count ||
                       self.flattened_nodes.len() != old_flattened_count {
                        self.draw()?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                self.update_scroll();
            }
            
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor_position < self.flattened_nodes.len().saturating_sub(1) {
                    self.cursor_position += 1;
                }
                self.update_scroll();
            }
            
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(node) = self.flattened_nodes.get(self.cursor_position) {
                    if node.is_dir {
                        self.toggle_node_at_cursor();
                    }
                }
            }
            
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(node) = self.flattened_nodes.get(self.cursor_position) {
                    if node.is_dir {
                        self.toggle_node_at_cursor();
                    }
                }
            }
            
            KeyCode::Enter => {
                if let Some(node) = self.flattened_nodes.get(self.cursor_position) {
                    if node.is_dir {
                        self.toggle_node_at_cursor();
                    }
                } else if let Some(callback) = &self.callback {
                    callback(&self.selected_nodes)?;
                }
            }
            
            KeyCode::Char(' ') | KeyCode::Char('s') => {
                self.toggle_selection_at_cursor();
            }
            
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_all();
            }
            
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.deselect_all();
            }
            
            KeyCode::Char('e') => {
                self.expand_all();
            }
            
            KeyCode::Char('c') => {
                self.collapse_all();
            }
            
            _ => {}
        }
        Ok(false)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let row = mouse.row as usize;
                // Adjust for header (3 lines) and calculate visible range
                if row >= 3 {
                    let visible_row = row - 3;
                    let node_index = visible_row + self.scroll_offset;
                    if node_index < self.flattened_nodes.len() {
                        self.cursor_position = node_index;
                        self.update_scroll();
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                let row = mouse.row as usize;
                if row >= 3 {
                    let visible_row = row - 3;
                    let node_index = visible_row + self.scroll_offset;
                    if node_index < self.flattened_nodes.len() {
                        self.cursor_position = node_index;
                        self.toggle_node_at_cursor();
                        self.update_scroll();
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    if self.cursor_position >= self.scroll_offset + self.get_visible_lines() {
                        self.cursor_position = self.scroll_offset + self.get_visible_lines() - 1;
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                let max_scroll = self.flattened_nodes.len().saturating_sub(self.get_visible_lines());
                if self.scroll_offset < max_scroll {
                    self.scroll_offset += 1;
                    if self.cursor_position < self.scroll_offset {
                        self.cursor_position = self.scroll_offset;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn toggle_node_at_cursor(&mut self) {
        if let Some(node) = self.flattened_nodes.get(self.cursor_position).cloned() {
            Self::toggle_node_recursive(&mut self.root, &node.path);
            self.update_flattened_nodes();
        }
    }

    fn toggle_node_recursive(current: &mut TreeNode, target_path: &Path) -> bool {
        if current.path == target_path {
            current.toggle_expand();
            return true;
        }

        for child in &mut current.children {
            if Self::toggle_node_recursive(child, target_path) {
                return true;
            }
        }
        false
    }

    fn toggle_selection_at_cursor(&mut self) {
        if let Some(node) = self.flattened_nodes.get(self.cursor_position) {
            if self.selected_nodes.contains(&node.path) {
                self.selected_nodes.remove(&node.path);
            } else {
                self.selected_nodes.insert(node.path.clone());
            }
        }
    }

    fn select_all(&mut self) {
        for node in &self.flattened_nodes {
            self.selected_nodes.insert(node.path.clone());
        }
    }

    fn deselect_all(&mut self) {
        self.selected_nodes.clear();
    }

    fn expand_all(&mut self) {
        self.root.expand_all();
        self.update_flattened_nodes();
    }

    fn collapse_all(&mut self) {
        self.root.collapse_all();
        self.update_flattened_nodes();
    }

    fn get_visible_lines(&self) -> usize {
        let terminal_height = crossterm::terminal::size().unwrap_or((80, 24)).1 as usize;
        terminal_height.saturating_sub(4) // Header (3 lines) + footer (1 line)
    }

    fn update_scroll(&mut self) {
        let visible_lines = self.get_visible_lines();

        if self.cursor_position < self.scroll_offset {
            self.scroll_offset = self.cursor_position;
        } else if self.cursor_position >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_position + 1 - visible_lines;
        }
    }

    fn draw(&mut self) -> Result<()> {
        // Clear screen and go to top
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        // Header
        execute!(
            stdout(),
            SetForegroundColor(Color::Cyan),
            Print("Interactive Tree Navigator"),
            ResetColor,
            MoveTo(0, 1),
            SetForegroundColor(Color::DarkGrey),
            Print("↑↓/jk: navigate | →←/hl: expand/collapse | Space/s: select | Enter: callback | Mouse: click/scroll | q: quit"),
            ResetColor
        )?;

        let visible_lines = self.get_visible_lines();

        // Draw visible tree nodes
        for (i, node) in self.flattened_nodes
            .iter()
            .skip(self.scroll_offset)
            .take(visible_lines)
            .enumerate()
        {
            let absolute_index = i + self.scroll_offset;
            let is_cursor = absolute_index == self.cursor_position;
            let is_selected = self.selected_nodes.contains(&node.path);

            // Move to the correct line
            execute!(stdout(), MoveTo(0, (i + 3) as u16))?;

            // Build the complete line first
            let mut line = String::new();

            // Cursor indicator
            if is_cursor {
                line.push_str("▶ ");
            } else {
                line.push_str("  ");
            }

            // Selection indicator
            if is_selected {
                line.push_str("✓ ");
            } else {
                line.push_str("  ");
            }

            // Indentation
            for _ in 0..node.depth {
                line.push_str("  ");
            }

            // Expand/collapse indicator
            if node.is_dir {
                if node.is_expanded {
                    line.push_str("📂 ");
                } else {
                    line.push_str("📁 ");
                }
            } else {
                line.push_str("📄 ");
            }

            // Node name
            line.push_str(&node.name);

            // Set color and print the complete line
            if is_cursor {
                execute!(stdout(), SetForegroundColor(Color::Yellow))?;
            } else if is_selected {
                execute!(stdout(), SetForegroundColor(Color::Green))?;
            } else if node.is_dir {
                execute!(stdout(), SetForegroundColor(Color::Blue))?;
            } else {
                execute!(stdout(), SetForegroundColor(Color::White))?;
            }

            execute!(stdout(), Print(&line), ResetColor)?;
        }

        // Footer
        let terminal_height = crossterm::terminal::size().unwrap_or((80, 24)).1;
        let footer_y = terminal_height - 1;
        execute!(
            stdout(),
            MoveTo(0, footer_y),
            SetForegroundColor(Color::DarkGrey),
            Print(&format!(
                "Selected: {} | {}/{} | e/c: expand/collapse all | Ctrl+A/D: select/deselect all",
                self.selected_nodes.len(),
                self.cursor_position + 1,
                self.flattened_nodes.len()
            )),
            ResetColor
        )?;

        stdout().flush()?;
        Ok(())
    }
}