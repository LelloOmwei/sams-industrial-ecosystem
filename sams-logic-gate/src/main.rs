// Conditional compilation for pluggable SLC architecture
#[cfg(feature = "closed-source")]
mod slc_core;

#[cfg(feature = "open-source")]
mod mock_logic;

mod common_types;
mod logic_controller;

use common_types::{SemanticAtom, ProcessedSemanticAtom, LogicController, SystemHealth};
use logic_controller::create_logic_controller;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub atoms_processed: u64,
    pub atoms_rejected: u64,
    pub rules_triggered: u64,
    pub high_load_count: u64,
    pub security_alerts: u64,
    pub interventions: u64,
    pub avg_latency: f64,
    pub logic_execution_times: Vec<f64>,
    pub system_health: SystemHealth,
    pub last_update: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            atoms_processed: 0,
            atoms_rejected: 0,
            rules_triggered: 0,
            high_load_count: 0,
            security_alerts: 0,
            interventions: 0,
            avg_latency: 0.0,
            logic_execution_times: Vec::new(),
            system_health: SystemHealth::Optimal,
            last_update: Instant::now(),
        }
    }
}

pub struct SemanticLogicGate {
    _slc: Box<dyn LogicController + Send + Sync>,
    metrics: Arc<RwLock<Metrics>>,
    recent_atoms: Arc<RwLock<Vec<ProcessedSemanticAtom>>>,
    ui_running: Arc<RwLock<bool>>,
}

impl SemanticLogicGate {
    pub fn new() -> Self {
        Self {
            _slc: create_logic_controller(),
            metrics: Arc::new(RwLock::new(Metrics::default())),
            recent_atoms: Arc::new(RwLock::new(Vec::new())),
            ui_running: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel::<ProcessedSemanticAtom>(1000);
        
        // Start UDP listener
        let slc_instance = create_logic_controller();
        let slc = Arc::new(slc_instance);
        let metrics_clone = Arc::clone(&self.metrics);
        let recent_atoms_clone = Arc::clone(&self.recent_atoms);
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::listen_udp(tx_clone, slc, metrics_clone, recent_atoms_clone).await {
                eprintln!("UDP listener error: {}", e);
            }
        });

        // Start UDP forwarder
        tokio::spawn(async move {
            if let Err(e) = Self::forward_udp(rx).await {
                eprintln!("UDP forwarder error: {}", e);
            }
        });

        // Start UI
        let ui_running = Arc::clone(&self.ui_running);
        let metrics_ui = Arc::clone(&self.metrics);
        let recent_atoms_ui = Arc::clone(&self.recent_atoms);
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_ui(metrics_ui, recent_atoms_ui, ui_running).await {
                eprintln!("UI error: {}", e);
            }
        });

        // Keep the main task alive
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let running = *self.ui_running.read().await;
            if !running {
                break;
            }
        }

        Ok(())
    }

    async fn listen_udp(
        tx: mpsc::Sender<ProcessedSemanticAtom>,
        slc: Arc<Box<dyn LogicController + Send + Sync>>,
        metrics: Arc<RwLock<Metrics>>,
        recent_atoms: Arc<RwLock<Vec<ProcessedSemanticAtom>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:5555").await?;
        println!("SAMS Semantic Logic Controller listening on UDP port 5555");
        
        let mut buf = [0u8; 4096];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, _addr)) => {
                    let data = &buf[..len];
                    
                    // Parse incoming atom
                    if let Ok(atom_str) = std::str::from_utf8(data) {
                        if let Ok(mut atom) = serde_json::from_str::<SemanticAtom>(atom_str) {
                            // Add payload if not present (for testing)
                            if atom.payload.is_none() {
                                atom.payload = Some(vec![0; 8]);
                            }
                            
                            // Process through SLC
                            if let Some(processed) = slc.process_atom(atom) {
                                let mut rules_triggered = processed.tags_added.len() as u64;
                                if processed.security_alert.is_some() {
                                    rules_triggered += 1;
                                }
                                
                                // Update metrics
                                {
                                    let mut m = metrics.write().await;
                                    m.atoms_processed += 1;
                                    m.rules_triggered += rules_triggered;
                                    if processed.original.energy_cost > 100.0 {
                                        m.high_load_count += 1;
                                    }
                                    if processed.security_alert.is_some() {
                                        m.security_alerts += 1;
                                    }
                                    if processed.intervention_applied {
                                        m.interventions += 1;
                                    }
                                    
                                    // Update system health
                                    m.system_health = processed.system_health.clone();
                                    
                                    // Track logic execution time
                                    let exec_time = processed.processing_time.as_nanos() as f64 / 1000.0; // in microseconds
                                    m.logic_execution_times.push(exec_time);
                                    if m.logic_execution_times.len() > 100 {
                                        m.logic_execution_times.remove(0);
                                    }
                                    
                                    // Update average latency
                                    let total_time = m.avg_latency * (m.atoms_processed - 1) as f64;
                                    m.avg_latency = (total_time + exec_time) / m.atoms_processed as f64;
                                    m.last_update = Instant::now();
                                }
                                
                                // Keep recent atoms (last 10)
                                {
                                    let mut recent = recent_atoms.write().await;
                                    recent.push(processed.clone());
                                    if recent.len() > 10 {
                                        recent.remove(0);
                                    }
                                }
                                
                                // Send to forwarder
                                if tx.send(processed).await.is_err() {
                                    break;
                                }
                            } else {
                                // Atom was rejected (validation failure)
                                let mut m = metrics.write().await;
                                m.atoms_rejected += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                }
            }
        }
        
        Ok(())
    }

    async fn forward_udp(mut rx: mpsc::Receiver<ProcessedSemanticAtom>) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let target_addr: SocketAddr = "127.0.0.1:5556".parse()?;
        
        println!("SAMS SLC forwarding to UDP port 5556");
        
        while let Some(processed) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&processed.original) {
                let data = json.as_bytes();
                if let Err(e) = socket.send_to(data, target_addr).await {
                    eprintln!("Error forwarding atom: {}", e);
                }
            }
        }
        
        Ok(())
    }

    async fn run_ui(
        metrics: Arc<RwLock<Metrics>>,
        recent_atoms: Arc<RwLock<Vec<ProcessedSemanticAtom>>>,
        ui_running: Arc<RwLock<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let _last_update = Instant::now();
        
        loop {
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }

            // Update UI
            terminal.draw(|f| {
                Self::render_ui(f, &metrics, &recent_atoms);
            })?;

            // Check if we should stop
            let running = *ui_running.read().await;
            if !running {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render_ui(
        f: &mut Frame,
        metrics: &Arc<RwLock<Metrics>>,
        recent_atoms: &Arc<RwLock<Vec<ProcessedSemanticAtom>>>,
    ) {
        let size = f.area();
        let industrial_cyan = Color::Cyan;
        let _slate_color = Color::Rgb(70, 70, 80);
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(15), // Logic Flow Diagram
                Constraint::Length(10), // Metrics
                Constraint::Min(5),     // Recent Atoms
            ])
            .split(size);

        // Header
        let header = Paragraph::new("SAMS SEMANTIC LOGIC CONTROLLER (SLC) v0.1.0")
            .style(Style::default().fg(industrial_cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(industrial_cyan)));
        f.render_widget(header, chunks[0]);

        // Logic Flow Diagram
        let flow_diagram = Self::create_flow_diagram(industrial_cyan);
        f.render_widget(flow_diagram, chunks[1]);

        // Metrics
        let metrics_widget = Self::create_metrics_widget(&metrics, industrial_cyan);
        f.render_widget(metrics_widget, chunks[2]);

        // Recent Atoms
        let recent_widget = Self::create_recent_atoms_widget(&recent_atoms, industrial_cyan);
        f.render_widget(recent_widget, chunks[3]);
    }

    fn create_flow_diagram(cyan: Color) -> Paragraph<'static> {
        let flow_art = r#"
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ GHOST-NODE  │───▶│ SLC CORE    │───▶│ DASHBOARD   │
│   (5555)    │    │  (PROCESS)  │    │   (5556)    │
└─────────────┘    └─────────────┘    └─────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
   SAMS Atoms    Semantic Logic   Processed Data
                    ┌─────────────┐
                    │ VALIDATION  │
                    │ STATEFUL    │
                    │ INTERVENTION│
                    └─────────────┘
"#;

        Paragraph::new(flow_art)
            .style(Style::default().fg(cyan))
            .block(Block::default().borders(Borders::ALL).title("SLC Logic Flow").border_style(Style::default().fg(cyan)))
    }

    fn create_metrics_widget(metrics: &Arc<RwLock<Metrics>>, cyan: Color) -> Paragraph<'_> {
        let m = metrics.blocking_read();
        let health_color = match m.system_health {
            SystemHealth::Optimal => Color::Green,
            SystemHealth::Warning => Color::Yellow,
            SystemHealth::Critical => Color::Red,
            SystemHealth::Intervention => Color::Magenta,
        };
        
        let avg_logic_time = if m.logic_execution_times.is_empty() {
            0.0
        } else {
            m.logic_execution_times.iter().sum::<f64>() / m.logic_execution_times.len() as f64
        };
        
        let metrics_text = format!(
            "Processed: {} | Rejected: {} | Rules: {} | High Load: {} | Security: {} | Interventions: {} | Health: {:?} | Avg Logic: {:.2}μs",
            m.atoms_processed, m.atoms_rejected, m.rules_triggered, m.high_load_count, 
            m.security_alerts, m.interventions, m.system_health, avg_logic_time
        );

        Paragraph::new(metrics_text)
            .style(Style::default().fg(cyan))
            .block(Block::default().borders(Borders::ALL).title("SLC System Metrics").border_style(Style::default().fg(health_color)))
    }

    fn create_recent_atoms_widget(recent_atoms: &Arc<RwLock<Vec<ProcessedSemanticAtom>>>, cyan: Color) -> List<'_> {
        let recent = recent_atoms.blocking_read();
        let items: Vec<ListItem> = recent.iter().take(5).map(|atom| {
            let status = if atom.intervention_applied {
                "🔧 INTERVENE"
            } else if atom.security_alert.is_some() {
                "🚨 ALERT"
            } else if atom.original.energy_cost > 100.0 {
                "⚡ HIGH"
            } else {
                "✅ OK"
            };
            
            let text = format!("{} {} - {:.2}μJ - {} tags - {:.2}μs", 
                atom.original.id, 
                status, 
                atom.original.energy_cost,
                atom.tags_added.len(),
                atom.processing_time.as_nanos() as f64 / 1000.0
            );
            ListItem::new(text)
        }).collect();

        List::new(items)
            .style(Style::default().fg(cyan))
            .block(Block::default().borders(Borders::ALL).title("Recent Semantic Atoms").border_style(Style::default().fg(cyan)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting SAMS Semantic Logic Controller v0.1.0...");
    println!("Press 'q' to quit");
    
    let slc = SemanticLogicGate::new();
    slc.run().await?;
    
    Ok(())
}
