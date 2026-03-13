// SEE INTEGRATION_PLAN.md FOR ROADMAP FROM SIMULATION TO PRODUCTION

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, BorderType, Gauge, List, ListItem, Paragraph, Sparkline,
    },
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sysinfo::System;
use std::{io, time::Duration, collections::VecDeque, net::UdpSocket};
use chrono::Local;
use rand::Rng;
use regex;

// Shared data structure matching ghost-node (32-byte aligned)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct SemanticAtom {
    timestamp: u32,        // Bytes 0-3   (4 bytes) - Unix timestamp
    node_id: u16,         // Bytes 4-5   (2 bytes) - Source node identifier
    atom_type: u16,       // Bytes 6-7   (2 bytes) - Type of atom/data
    energy_micro_j: u32,  // Bytes 8-11  (4 bytes) - Energy cost in microjoules
    payload: [u8; 20],    // Bytes 12-31 (20 bytes) - Data payload
}

// Energy analytics constants
const SAMS_ATOM_SIZE: u64 = 32; // bytes
const JSON_ATOM_SIZE: u64 = 512; // bytes
const SAMS_ENERGY_PER_ATOM: u64 = 12; // microjoules
const JSON_ENERGY_PER_ATOM: u64 = 180; // microjoules
const ATOMS_PER_SECOND: u64 = 100;
const MAX_STORED_ATOMS: usize = 1000;

// Query system with Natural Language support
#[derive(Debug, Clone)]
enum QueryFilter {
    EnergyGreaterThan(u32),
    EnergyLessThan(u32),
    NodeId(u16),
    AtomType(u16),
}

#[derive(Debug, Clone)]
struct Query {
    filters: Vec<QueryFilter>,
    original_input: String,
    normalized_input: String,
}

// Natural Language normalization
fn normalize_query(input: &str) -> String {
    let mut normalized = input.to_lowercase();
    
    // Energy synonyms mapping
    let energy_terms = [
        ("power", "energy"),
        ("consumption", "energy"),
        ("usage", "energy"),
        ("hungry", "energy"),
        ("expensive", "energy"),
        ("spotreba", "energy"), // Slovak
        ("energia", "energy"), // Slovak
    ];
    
    // Node synonyms mapping
    let node_terms = [
        ("sensor", "node"),
        ("machine", "node"),
        ("unit", "node"),
        ("source", "node"),
        ("id", "node"),
        ("senzor", "node"), // Slovak
        ("zdroj", "node"), // Slovak
    ];
    
    // High/Low threshold mapping
    let threshold_terms = [
        ("high", "80"),
        ("very high", "90"),
        ("low", "20"),
        ("efficient", "20"),
        ("very low", "10"),
        ("nizke", "20"), // Slovak
        ("vysoke", "80"), // Slovak
    ];
    
    // Apply energy term replacements
    for (synonym, target) in &energy_terms {
        normalized = normalized.replace(synonym, target);
    }
    
    // Apply node term replacements
    for (synonym, target) in &node_terms {
        normalized = normalized.replace(synonym, target);
    }
    
    // Apply threshold replacements (with word boundaries)
    for (synonym, target) in &threshold_terms {
        let pattern = format!(r"\b{}\b", regex::escape(synonym));
        if let Ok(re) = regex::Regex::new(&pattern) {
            normalized = re.replace_all(&normalized, *target).to_string();
        }
    }
    
    normalized
}

impl Query {
    fn parse(input: &str) -> Result<Self, String> {
        let mut filters = Vec::new();
        let original_input = input.to_string();
        let normalized = normalize_query(input);
        let parts: Vec<&str> = normalized.split_whitespace().collect();
        
        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "energy" | "consumption" | "power" | "usage" if i + 2 < parts.len() => {
                    match parts[i + 1] {
                        ">" | ">=" => {
                            if let Ok(value) = parts[i + 2].parse::<u32>() {
                                filters.push(QueryFilter::EnergyGreaterThan(value));
                            }
                        }
                        "<" | "<=" => {
                            if let Ok(value) = parts[i + 2].parse::<u32>() {
                                filters.push(QueryFilter::EnergyLessThan(value));
                            }
                        }
                        _ => {}
                    }
                    i += 3;
                }
                "node" | "sensor" | "machine" | "unit" | "source" | "id" if i + 2 < parts.len() => {
                    if parts[i + 1] == "=" {
                        if let Ok(value) = parts[i + 2].parse::<u16>() {
                            filters.push(QueryFilter::NodeId(value));
                        }
                    }
                    i += 3;
                }
                "type" if i + 2 < parts.len() => {
                    if parts[i + 1] == "=" {
                        if let Ok(value) = parts[i + 2].parse::<u16>() {
                            filters.push(QueryFilter::AtomType(value));
                        }
                    }
                    i += 3;
                }
                _ => i += 1,
            }
        }
        
        Ok(Query { 
            filters, 
            original_input,
            normalized_input: normalized,
        })
    }
    
    fn matches(&self, atom: &SemanticAtom) -> bool {
        self.filters.iter().all(|filter| {
            match filter {
                QueryFilter::EnergyGreaterThan(threshold) => atom.energy_micro_j > *threshold,
                QueryFilter::EnergyLessThan(threshold) => atom.energy_micro_j < *threshold,
                QueryFilter::NodeId(node_id) => atom.node_id == *node_id,
                QueryFilter::AtomType(atom_type) => atom.atom_type == *atom_type,
            }
        })
    }
}

// Application state structure
struct App {
    sys: System,
    cpu_history: VecDeque<f64>,
    memory_history: VecDeque<u64>,
    log_entries: VecDeque<String>,
    encryption_status: bool,
    scan_mode: bool,
    // Energy analytics
    total_sams_atoms_processed: u64,
    total_energy_saved: u64, // in microjoules
    green_efficiency: f64,
    // UDP listener for real SAMS data
    udp_socket: UdpSocket,
    use_live_data: bool,
    // Semantic Query System
    atom_storage: VecDeque<SemanticAtom>,
    query_mode: bool,
    query_input: String,
    current_query: Option<Query>,
    query_matches: Vec<usize>,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        // Setup UDP listener for real SAMS data
        let udp_socket = UdpSocket::bind("127.0.0.1:5555")?;
        udp_socket.set_nonblocking(true)?;
        
        Ok(Self {
            sys,
            cpu_history: VecDeque::with_capacity(50),
            memory_history: VecDeque::with_capacity(50),
            log_entries: VecDeque::with_capacity(100),
            encryption_status: true,
            scan_mode: false,
            total_sams_atoms_processed: 0,
            total_energy_saved: 0,
            green_efficiency: 0.0,
            udp_socket,
            use_live_data: true, // Start with live data mode
            // Semantic Query System
            atom_storage: VecDeque::with_capacity(MAX_STORED_ATOMS),
            query_mode: false,
            query_input: String::new(),
            current_query: None,
            query_matches: Vec::new(),
        })
    }

    fn receive_udp_atoms(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; std::mem::size_of::<SemanticAtom>()];
        
        match self.udp_socket.recv_from(&mut buffer) {
            Ok((bytes_received, _)) if bytes_received == std::mem::size_of::<SemanticAtom>() => {
                // Parse the received atom (32-byte aligned)
                let atom = unsafe {
                    std::ptr::read(buffer.as_ptr() as *const SemanticAtom)
                };
                
                // Store atom for querying
                self.atom_storage.push_back(atom);
                if self.atom_storage.len() > MAX_STORED_ATOMS {
                    self.atom_storage.pop_front();
                }
                
                // Update energy analytics with real data
                self.total_sams_atoms_processed += 1;
                let sams_energy = atom.energy_micro_j as u64;
                let json_energy = JSON_ENERGY_PER_ATOM;
                let savings = json_energy - sams_energy;
                self.total_energy_saved += savings;
                
                // Calculate green efficiency
                let total_sams_energy = self.total_sams_atoms_processed * sams_energy;
                let total_json_energy = self.total_sams_atoms_processed * json_energy;
                self.green_efficiency = if total_json_energy > 0 {
                    (total_sams_energy as f64 / total_json_energy as f64) * 100.0
                } else {
                    0.0
                };
                
                // Add log entry for received atom
                let node_id = atom.node_id;
                let energy = atom.energy_micro_j;
                let atom_type = atom.atom_type;
                let message = format!("[ATOM] ID:{} from Node:{} Energy:{}μJ Type:{}", 
                    atom_type, node_id, energy, atom_type);
                self.add_log(&message, "OK");
                
                // Update query matches if filter is active
                if self.current_query.is_some() {
                    if let Some(ref query) = self.current_query.clone() {
                        self.update_query_matches(query);
                    }
                }
            }
            Ok(_) => {
                // No data received or invalid packet size
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available (normal for non-blocking)
            }
            Err(e) => {
                // Real error
                self.add_log(&format!("[UDP] Error receiving atom: {}", e), "WARN");
            }
        }
        Ok(())
    }

    fn update_metrics(&mut self) {
        self.sys.refresh_all();
        
        // Try to receive real UDP atoms first
        if self.use_live_data {
            if let Err(_) = self.receive_udp_atoms() {
                // If UDP fails, fall back to simulation temporarily
                self.use_live_data = false;
                self.add_log("[SYSTEM] UDP listener failed, switching to simulation mode", "WARN");
            }
        }
        
        // Update system metrics (always do this)
        for cpu in self.sys.cpus() {
            self.cpu_history.push_back(cpu.cpu_usage() as f64);
            if self.cpu_history.len() > 50 {
                self.cpu_history.pop_front();
            }
        }

        let used_memory = self.sys.used_memory();
        self.memory_history.push_back(used_memory);
        if self.memory_history.len() > 50 {
            self.memory_history.pop_front();
        }

        // Only add simulated log entries if not receiving live data
        if !self.use_live_data {
            // Update energy analytics (simulate atoms processed)
            let atoms_this_cycle = ATOMS_PER_SECOND / 4; // 250ms cycle
            self.total_sams_atoms_processed += atoms_this_cycle;
            
            let sams_energy = atoms_this_cycle * SAMS_ENERGY_PER_ATOM;
            let json_energy = atoms_this_cycle * JSON_ENERGY_PER_ATOM;
            let savings = json_energy - sams_energy;
            self.total_energy_saved += savings;
            
            // Calculate green efficiency
            let total_sams_energy = self.total_sams_atoms_processed * SAMS_ENERGY_PER_ATOM;
            let total_json_energy = self.total_sams_atoms_processed * JSON_ENERGY_PER_ATOM;
            self.green_efficiency = if total_json_energy > 0 {
                (total_sams_energy as f64 / total_json_energy as f64) * 100.0
            } else {
                0.0
            };

            // Add periodic log entries
            if self.scan_mode {
                self.add_log("[SCAN] Deep security scan in progress...", "WARN");
                self.add_log("[SCAN] Analyzing quantum encryption layers...", "INFO");
                self.add_log("[SCAN] PQC tunnel integrity verified", "OK");
                self.scan_mode = false;
            } else if rand::thread_rng().gen::<f32>() < 0.15 {
                let messages = vec![
                    ("[OK] PQC Tunnel established", "OK"),
                    ("[WARN] Unauthorized node ping detected", "WARN"),
                    ("[INFO] Syncing 32-byte atoms...", "INFO"),
                    ("[OK] Neural network sync complete", "OK"),
                    ("[INFO] Quantum entanglement stable", "INFO"),
                    ("[WARN] Anomaly detected in data stream", "WARN"),
                    ("[OK] Firewall rules updated", "OK"),
                    // Energy-specific messages
                    ("[ENERGY] Atom routed: 12μJ cost", "INFO"),
                    ("[SAVED] 168μJ vs Legacy protocol", "OK"),
                    ("[GREEN] Efficiency target achieved", "OK"),
                    ("[ATOM] Batch processed: 100 atoms", "INFO"),
                ];
                let (msg, level) = messages[rand::thread_rng().gen_range(0..messages.len())];
                self.add_log(msg, level);
            }
        }
    }

    fn add_log(&mut self, message: &str, level: &str) {
        let timestamp = Local::now().format("%H:%M:%S");
        let level_icon = match level {
            "OK" => "✓",
            "WARN" => "⚠",
            "INFO" => "ℹ",
            _ => "?",
        };
        let entry = format!("{} {} {}", timestamp, level_icon, message);
        self.log_entries.push_back(entry);
        if self.log_entries.len() > 100 {
            self.log_entries.pop_front();
        }
    }

    fn toggle_encryption_status(&mut self) {
        self.encryption_status = !self.encryption_status;
    }

    fn start_security_scan(&mut self) {
        self.scan_mode = true;
    }
    
    fn update_query_matches(&mut self, query: &Query) {
        self.query_matches.clear();
        for (i, atom) in self.atom_storage.iter().enumerate() {
            let node_id = atom.node_id;
            let energy = atom.energy_micro_j;
            let atom_type = atom.atom_type;
            let atom_copy = SemanticAtom {
                timestamp: atom.timestamp,
                node_id,
                atom_type,
                energy_micro_j: energy,
                payload: atom.payload,
            };
            if query.matches(&atom_copy) {
                self.query_matches.push(i);
            }
        }
    }
    
    fn execute_query(&mut self) {
        match Query::parse(&self.query_input) {
            Ok(query) => {
                self.current_query = Some(query.clone());
                self.update_query_matches(&query);
                if query.original_input != query.normalized_input {
                    self.add_log(&format!("[QUERY] NL interpreted as: '{}'", query.normalized_input), "OK");
                }
                self.add_log(&format!("[QUERY] Found {} matches for '{}'", self.query_matches.len(), query.original_input), "OK");
            }
            Err(e) => {
                self.add_log(&format!("[QUERY] Parse error: {}", e), "WARN");
            }
        }
    }
    
    fn clear_query(&mut self) {
        self.current_query = None;
        self.query_matches.clear();
        self.query_input.clear();
        self.query_mode = false;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application state
    let mut app = App::new()?;
    app.add_log("[SYSTEM] SAMS-CORE OS initialized", "OK");
    app.add_log("[SYSTEM] Quantum encryption enabled", "OK");
    app.add_log("[SYSTEM] Neural network online", "INFO");
    app.add_log("[SYSTEM] UDP listener ready on port 5555", "OK");

    // Main application loop
    let mut last_update = std::time::Instant::now();
    let mut encryption_blink = false;
    loop {
        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.query_mode {
                            app.clear_query();
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('s') => app.start_security_scan(),
                    KeyCode::Char('e') => app.toggle_encryption_status(),
                    KeyCode::Char('l') => {
                        app.use_live_data = !app.use_live_data;
                        let mode = if app.use_live_data { "LIVE" } else { "SIMULATION" };
                        app.add_log(&format!("[SYSTEM] Switched to {} mode", mode), "INFO");
                    }
                    KeyCode::Char('/') => {
                        app.query_mode = true;
                        app.query_input.clear();
                    }
                    KeyCode::Enter if app.query_mode => {
                        app.execute_query();
                    }
                    KeyCode::Char(c) if app.query_mode => {
                        app.query_input.push(c);
                    }
                    KeyCode::Backspace if app.query_mode => {
                        app.query_input.pop();
                    }
                    _ => {}
                }
            }
        }

        // Update metrics every 250ms
        if last_update.elapsed() >= Duration::from_millis(250) {
            app.update_metrics();
            encryption_blink = !encryption_blink;
            last_update = std::time::Instant::now();
        }

        // Render UI
        terminal.draw(|f| {
            let size = f.size();
            render_ui(f, &app, size, encryption_blink);
        })?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn render_ui(f: &mut Frame, app: &App, size: Rect, encryption_blink: bool) {
    // Main layout - split into header, search, main content, and status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Search Bar
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Header section
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(chunks[0]);

    // Main title
    let title = Paragraph::new("SAMS-CORE OS // SECURE TERMINAL v1.0.4")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
    f.render_widget(title, header_chunks[0]);

    // Encryption status (blinking)
    let status_text = if app.encryption_status && encryption_blink {
        "● SYSTEM ENCRYPTED"
    } else if app.encryption_status {
        "○ SYSTEM ENCRYPTED"
    } else {
        "○ SYSTEM DECRYPTED"
    };
    let status_color = if app.encryption_status { Color::Green } else { Color::Red };
    let encryption_status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
    f.render_widget(encryption_status, header_chunks[1]);

    // Search Bar
    render_search_bar(f, app, chunks[1]);

    // Main content area - split into top (metrics), middle (energy), and bottom (log)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35), // CPU & Memory
            Constraint::Percentage(25), // Energy Analytics
            Constraint::Percentage(40), // Log
        ])
        .split(chunks[2]);

    // Top section - split into CPU and Memory
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    // CPU Load section
    let cpu_block = Block::default()
        .title(" CORE LOAD ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::Green));
    f.render_widget(cpu_block, top_chunks[0]);

    // CPU gauges for each core
    let cpu_inner = top_chunks[0].inner(&Margin::new(1, 1));
    let cpu_count = app.sys.cpus().len();
    if cpu_count > 0 {
        let cpu_height = cpu_inner.height / cpu_count as u16;
        let cpu_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(cpu_height); cpu_count])
            .split(cpu_inner);

        for (i, cpu) in app.sys.cpus().iter().enumerate() {
            if i < cpu_chunks.len() {
                let cpu_gauge = Gauge::default()
                    .block(Block::default().title(format!("CORE{}", i)))
                    .gauge_style(Style::default().fg(Color::Green))
                    .percent(cpu.cpu_usage() as u16);
                f.render_widget(cpu_gauge, cpu_chunks[i]);
            }
        }
    }

    // Memory Flux section
    let memory_block = Block::default()
        .title(" MEMORY FLUX ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::Green));
    f.render_widget(memory_block, top_chunks[1]);
    let memory_inner = top_chunks[1].inner(&Margin::new(1, 1));

    // Memory usage sparkline
    let memory_data: Vec<u64> = app.memory_history.iter().cloned().collect();
    let max_memory = *memory_data.iter().max().unwrap_or(&1);
    let sparkline = Sparkline::default()
        .data(&memory_data)
        .max(max_memory)
        .style(Style::default().fg(Color::Green));
    f.render_widget(sparkline, memory_inner);

    // Energy Analytics section
    let energy_block = Block::default()
        .title(" 🔋 ENERGY SAVINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::Green));
    f.render_widget(energy_block, main_chunks[1]);
    let energy_inner = main_chunks[1].inner(&Margin::new(1, 1));

    // Energy metrics layout
    let energy_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Cumulative savings
            Constraint::Length(3), // Comparison
            Constraint::Length(3), // Efficiency
        ])
        .split(energy_inner);

    // Cumulative Microjoules saved
    let savings_text = format!("CUMULATIVE: {} μJ saved", app.total_energy_saved);
    let savings_para = Paragraph::new(savings_text)
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    f.render_widget(savings_para, energy_chunks[0]);

    // Size comparison
    let comparison_text = format!("SAMS: {}B vs JSON: {}B per atom", SAMS_ATOM_SIZE, JSON_ATOM_SIZE);
    let comparison_para = Paragraph::new(comparison_text)
        .style(Style::default().fg(Color::Green));
    f.render_widget(comparison_para, energy_chunks[1]);

    // Green Efficiency percentage
    let efficiency_color = if app.green_efficiency >= 90.0 { Color::Green } else { Color::Yellow };
    let efficiency_text = format!("GREEN EFFICIENCY: {:.1}%", app.green_efficiency);
    let efficiency_para = Paragraph::new(efficiency_text)
        .style(Style::default().fg(efficiency_color).add_modifier(Modifier::BOLD));
    f.render_widget(efficiency_para, energy_chunks[2]);

    // Bottom section - Encrypted Security Log (filtered if query active)
    let log_block = Block::default()
        .title(if app.current_query.is_some() {
            format!(" FILTERED LOG ({} matches) ", app.query_matches.len())
        } else {
            " ENCRYPTED SECURITY LOG ".to_string()
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::Green));
    f.render_widget(log_block, main_chunks[2]);
    let log_inner = main_chunks[2].inner(&Margin::new(1, 1));

    // Log entries (filtered or all)
    let log_items: Vec<ListItem> = if app.current_query.is_some() {
        // Show only matching atoms
        app.query_matches
            .iter()
            .rev()
            .take(log_inner.height as usize - 2)
            .filter_map(|&index| {
                app.atom_storage.get(index).map(|atom| {
                    let node_id = atom.node_id;
                    let energy = atom.energy_micro_j;
                    let atom_type = atom.atom_type;
                    let timestamp = chrono::DateTime::from_timestamp(atom.timestamp as i64, 0)
                        .map(|dt| dt.format("%H:%M:%S").to_string())
                        .unwrap_or_else(|| "UNKNOWN".to_string());
                    let message = format!("[{}] [ATOM] Node:{} Energy:{}μJ Type:{}", 
                        timestamp, node_id, energy, atom_type);
                    let style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                    ListItem::new(message).style(style)
                })
            })
            .collect()
    } else {
        // Show all log entries
        app.log_entries
            .iter()
            .rev()
            .take(log_inner.height as usize - 2)
            .map(|entry| {
                let style = if entry.contains("✓") {
                    Style::default().fg(Color::Green)
                } else if entry.contains("⚠") {
                    Style::default().fg(Color::Yellow)
                } else if entry.contains("[ENERGY]") || entry.contains("[SAVED]") || entry.contains("[GREEN]") {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if entry.contains("[QUERY]") {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(entry.as_str()).style(style)
            })
            .collect()
    };

    let log_list = List::new(log_items)
        .style(Style::default().fg(Color::Green));
    f.render_widget(log_list, log_inner);

    // Status bar
    let mode_text = if app.use_live_data { "LIVE" } else { "SIM" };
    let query_status = if app.current_query.is_some() {
        format!(" | Found {} matches for '{}'", app.query_matches.len(), app.query_input)
    } else {
        String::new()
    };
    let status_text = format!(" [q] Quit | [s] Scan | [e] Encrypt | [l] Mode: {} [/] Search{} ", mode_text, query_status);
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
    f.render_widget(status, chunks[3]);
}

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let search_text = if app.query_mode {
        format!("Search: {}_", app.query_input)
    } else if let Some(ref query) = app.current_query {
        if query.original_input != query.normalized_input {
            format!("Filter: '{}' -> '{}'", query.original_input, query.normalized_input)
        } else {
            format!("Filter active: '{}' (Esc to clear)", query.original_input)
        }
    } else {
        "Press '/' to search atoms".to_string()
    };
    
    let search_color = if app.query_mode { Color::Yellow } else { Color::Cyan };
    let search_widget = Paragraph::new(search_text)
        .style(Style::default().fg(search_color))
        .block(Block::default()
            .title(" SEMANTIC QUERY (NL) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    
    f.render_widget(search_widget, area);
}
