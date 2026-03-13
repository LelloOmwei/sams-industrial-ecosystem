use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Paragraph, Gauge},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use clap::Parser;
use std::{
    io,
    net::UdpSocket,
    time::{Duration, SystemTime, UNIX_EPOCH},
    collections::VecDeque,
};
use rand::Rng;

// Shared data structure as specified in integration plan
#[repr(C)]
#[derive(Debug, Clone)]
struct SemanticAtom {
    timestamp: u64,
    atom_id: u32,
    payload: [u8; 24],
    trust_pqc: bool,
    energy_cost: u32,
    source_node: u16,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transmission frequency in milliseconds (default: 100)
    #[arg(short, long, default_value = "100")]
    frequency: u64,

    /// Operation mode: normal, anomaly, or attack
    #[arg(short, long, default_value = "normal")]
    mode: String,

    /// UDP port for transmission (default: 5555)
    #[arg(short, long, default_value = "5555")]
    port: u16,
}

#[derive(Debug, Clone)]
enum Mode {
    Normal,
    Anomaly,
    Attack,
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "anomaly" => Mode::Anomaly,
            "attack" => Mode::Attack,
            _ => Mode::Normal,
        }
    }
}

// Application state
struct App {
    current_atom_id: u32,
    last_payload: String,
    heartbeat_phase: f64,
    mode: Mode,
    atoms_sent: u64,
    transmission_rate: u64,
}

impl App {
    fn new(mode: Mode, frequency: u64) -> Self {
        Self {
            current_atom_id: 0,
            last_payload: "Initializing...".to_string(),
            heartbeat_phase: 0.0,
            mode,
            atoms_sent: 0,
            transmission_rate: frequency,
        }
    }

    fn update_heartbeat(&mut self) {
        self.heartbeat_phase += 0.2;
        if self.heartbeat_phase > 2.0 * std::f64::consts::PI {
            self.heartbeat_phase -= 2.0 * std::f64::consts::PI;
        }
    }

    fn increment_atom_id(&mut self) {
        self.current_atom_id = self.current_atom_id.wrapping_add(1);
        self.atoms_sent += 1;
    }

    fn format_payload(&self, payload: &[u8]) -> String {
        payload.iter()
            .take(8) // Show first 8 bytes for readability
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn create_semantic_atom(atom_id: u32, mode: &Mode, source_node: u16) -> SemanticAtom {
    let mut rng = rand::thread_rng();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Generate random payload
    let mut payload = [0u8; 24];
    rng.fill(&mut payload);

    let (trust_pqc, energy_cost) = match mode {
        Mode::Normal => (true, 12 + rng.gen_range(0..4)), // 12-15 μJ
        Mode::Anomaly => {
            // Randomly spike energy cost
            if rng.gen_bool(0.3) {
                (true, 50 + rng.gen_range(0..100)) // 50-149 μJ (anomaly)
            } else {
                (true, 12 + rng.gen_range(0..4))
            }
        }
        Mode::Attack => {
            // Set trust_pqc to false for attack mode
            (false, 12 + rng.gen_range(0..4))
        }
    };

    SemanticAtom {
        timestamp,
        atom_id,
        payload,
        trust_pqc,
        energy_cost,
        source_node,
    }
}

fn send_atom(socket: &UdpSocket, atom: &SemanticAtom, port: u16) -> io::Result<()> {
    let atom_bytes = unsafe {
        std::slice::from_raw_parts(
            atom as *const SemanticAtom as *const u8,
            std::mem::size_of::<SemanticAtom>(),
        )
    };
    
    let addr = format!("127.0.0.1:{}", port);
    socket.send_to(atom_bytes, &addr)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mode = Mode::from(args.mode.as_str());
    let frequency = args.frequency;
    let port = args.port;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application state
    let mut app = App::new(mode.clone(), frequency);

    // Setup UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let mut last_transmission = std::time::Instant::now();
    let transmission_interval = Duration::from_millis(frequency);

    loop {
        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        // Update heartbeat animation
        app.update_heartbeat();

        // Send atom at specified frequency
        if last_transmission.elapsed() >= transmission_interval {
            let atom = create_semantic_atom(app.current_atom_id, &mode, 1);
            
            if let Err(e) = send_atom(&socket, &atom, port) {
                // Handle send error (e.g., no receiver)
                app.last_payload = format!("TX Error: {}", e);
            } else {
                app.last_payload = app.format_payload(&atom.payload);
                app.increment_atom_id();
            }
            
            last_transmission = std::time::Instant::now();
        }

        // Render UI
        terminal.draw(|f| {
            let size = f.size();
            render_ui(f, &app, size);
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

fn render_ui(f: &mut Frame, app: &App, size: Rect) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Header
    let title = Paragraph::new("SAMS GHOST-NODE // TRANSMISSION CONSOLE v1.0")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
    f.render_widget(title, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Atom ID & Status
            Constraint::Length(8),  // Payload display
            Constraint::Length(5),  // Heartbeat
            Constraint::Min(0),    // Stats
        ])
        .split(chunks[1]);

    // Current Atom ID
    let atom_id_text = format!("CURRENT ATOM ID: {}", app.current_atom_id);
    let atom_id_para = Paragraph::new(atom_id_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default()
            .title(" ATOM TRANSMISSION ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double));
    f.render_widget(atom_id_para, main_chunks[0]);

    // Last Payload
    let payload_para = Paragraph::new(app.last_payload.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default()
            .title(" LAST PAYLOAD (HEX) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double));
    f.render_widget(payload_para, main_chunks[1]);

    // Heartbeat animation
    let heartbeat_intensity = ((app.heartbeat_phase.sin() + 1.0) / 2.0) as u16;
    let heartbeat_text = if heartbeat_intensity > 128 {
        "● TRANSMITTING"
    } else {
        "○ STANDBY"
    };
    let heartbeat_color = if heartbeat_intensity > 128 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let heartbeat_para = Paragraph::new(heartbeat_text)
        .style(Style::default().fg(heartbeat_color).add_modifier(Modifier::BOLD))
        .block(Block::default()
            .title(" HEARTBEAT ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double));
    f.render_widget(heartbeat_para, main_chunks[2]);

    // Statistics
    let stats_text = format!(
        "MODE: {:?} | RATE: {}ms | ATOMS SENT: {} | TRUST: {}",
        app.mode,
        app.transmission_rate,
        app.atoms_sent,
        match app.mode {
            Mode::Attack => "DISABLED",
            _ => "ENABLED"
        }
    );
    let stats_para = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default()
            .title(" TRANSMISSION STATISTICS ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double));
    f.render_widget(stats_para, main_chunks[3]);

    // Status bar
    let status_text = " [q] Quit | Ghost-Node Active ";
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
    f.render_widget(status, chunks[2]);
}
