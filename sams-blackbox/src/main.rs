use std::collections::VecDeque;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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

const ATOM_SIZE: usize = 32; // 32-byte SemanticAtom
const TIMESTAMP_SIZE: usize = 8; // 8-byte timestamp prefix
const RECORD_SIZE: usize = ATOM_SIZE + TIMESTAMP_SIZE; // 40 bytes total

#[derive(Debug, Clone)]
pub struct BinaryRecord {
    pub timestamp: u64,
    pub atom_data: Vec<u8>,
    pub arrival_time: Instant,
    pub write_latency: Duration,
}

#[derive(Debug, Clone)]
pub struct AuditMetrics {
    pub total_records: u64,
    pub current_file_size: u64,
    pub avg_write_latency: f64,
    pub last_update: Instant,
}

impl Default for AuditMetrics {
    fn default() -> Self {
        Self {
            total_records: 0,
            current_file_size: 0,
            avg_write_latency: 0.0,
            last_update: Instant::now(),
        }
    }
}

pub struct BlackBoxAuditor {
    metrics: Arc<RwLock<AuditMetrics>>,
    recent_records: Arc<RwLock<VecDeque<BinaryRecord>>>,
    ui_running: Arc<RwLock<bool>>,
    audit_file: Arc<RwLock<BufWriter<std::fs::File>>>,
}

impl BlackBoxAuditor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create/open audit file
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("audit.samslog")?;
        
        let writer = BufWriter::new(file);
        
        Ok(Self {
            metrics: Arc::new(RwLock::new(AuditMetrics::default())),
            recent_records: Arc::new(RwLock::new(VecDeque::with_capacity(5))),
            ui_running: Arc::new(RwLock::new(true)),
            audit_file: Arc::new(RwLock::new(writer)),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, _rx) = mpsc::channel::<BinaryRecord>(1000);
        
        // Start UDP listener
        let metrics_clone = Arc::clone(&self.metrics);
        let recent_clone = Arc::clone(&self.recent_records);
        let audit_clone = Arc::clone(&self.audit_file);
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::listen_udp(tx_clone, metrics_clone, recent_clone, audit_clone).await {
                eprintln!("UDP listener error: {}", e);
            }
        });

        // Start UI
        let ui_running = Arc::clone(&self.ui_running);
        let metrics_ui = Arc::clone(&self.metrics);
        let recent_ui = Arc::clone(&self.recent_records);
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_ui(metrics_ui, recent_ui, ui_running).await {
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
        tx: mpsc::Sender<BinaryRecord>,
        metrics: Arc<RwLock<AuditMetrics>>,
        recent_records: Arc<RwLock<VecDeque<BinaryRecord>>>,
        audit_file: Arc<RwLock<BufWriter<std::fs::File>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:5556").await?;
        println!("SAMS Black-Box Auditor listening on UDP port 5556");
        
        let mut buf = [0u8; 1024];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, _addr)) => {
                    let data = &buf[..len];
                    
                    // Process binary data (expect 32-byte atoms)
                    if len == ATOM_SIZE {
                        let _start_time = Instant::now();
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        
                        // Create binary record
                        let record = BinaryRecord {
                            timestamp,
                            atom_data: data.to_vec(),
                            arrival_time: Instant::now(),
                            write_latency: Duration::default(),
                        };
                        
                        // Write to audit file (timestamp + atom data)
                        let write_start = Instant::now();
                        {
                            let mut writer = audit_file.write().await;
                            
                            // Write timestamp (8 bytes)
                            writer.write_all(&timestamp.to_le_bytes())?;
                            
                            // Write atom data (32 bytes)
                            writer.write_all(data)?;
                            writer.flush()?;
                        }
                        let write_duration = write_start.elapsed();
                        
                        // Update metrics
                        {
                            let mut m = metrics.write().await;
                            m.total_records += 1;
                            m.current_file_size += RECORD_SIZE as u64;
                            
                            // Update average write latency
                            let total_time = m.avg_write_latency * (m.total_records - 1) as f64;
                            m.avg_write_latency = (total_time + write_duration.as_micros() as f64) / m.total_records as f64;
                            m.last_update = Instant::now();
                        }
                        
                        // Update recent records (keep last 5)
                        {
                            let mut recent = recent_records.write().await;
                            let mut final_record = record.clone();
                            final_record.write_latency = write_duration;
                            recent.push_back(final_record);
                            if recent.len() > 5 {
                                recent.pop_front();
                            }
                        }
                        
                        // Send to UI (optional, for display)
                        if tx.send(record).await.is_err() {
                            break;
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

    async fn run_ui(
        metrics: Arc<RwLock<AuditMetrics>>,
        recent_records: Arc<RwLock<VecDeque<BinaryRecord>>>,
        ui_running: Arc<RwLock<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
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
                Self::render_ui(f, &metrics, &recent_records);
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
        metrics: &Arc<RwLock<AuditMetrics>>,
        recent_records: &Arc<RwLock<VecDeque<BinaryRecord>>>,
    ) {
        let size = f.area();
        let forensic_red = Color::Red;
        let forensic_gray = Color::Rgb(40, 40, 40);
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(8),  // Stats
                Constraint::Min(5),     // Live Feed
            ])
            .split(size);

        // Header
        let header = Paragraph::new("SAMS BLACK-BOX AUDITOR v1.0")
            .style(Style::default().fg(forensic_red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(forensic_red)));
        f.render_widget(header, chunks[0]);

        // Stats
        let stats_widget = Self::create_stats_widget(&metrics, forensic_red, forensic_gray);
        f.render_widget(stats_widget, chunks[1]);

        // Live Feed
        let feed_widget = Self::create_live_feed_widget(&recent_records, forensic_red, forensic_gray);
        f.render_widget(feed_widget, chunks[2]);
    }

    fn create_stats_widget(metrics: &Arc<RwLock<AuditMetrics>>, red: Color, _gray: Color) -> Paragraph<'static> {
        let m = metrics.blocking_read();
        let file_size_mb = m.current_file_size as f64 / (1024.0 * 1024.0);
        
        let stats_text = format!(
            "Total Records Saved: {} | Current File Size: {:.2} MB | Disk Write Latency: {:.2}μs",
            m.total_records, file_size_mb, m.avg_write_latency
        );

        Paragraph::new(stats_text)
            .style(Style::default().fg(red))
            .block(Block::default().borders(Borders::ALL).title("Audit Statistics").border_style(Style::default().fg(red)))
    }

    fn create_live_feed_widget(recent_records: &Arc<RwLock<VecDeque<BinaryRecord>>>, red: Color, gray: Color) -> List<'static> {
        let recent = recent_records.blocking_read();
        let items: Vec<ListItem> = recent.iter().rev().take(5).enumerate().map(|(_i, record)| {
            // Convert atom data to hex string for forensic display
            let hex_string: String = record.atom_data
                .iter()
                .enumerate()
                .map(|(j, &byte)| {
                    if j % 8 == 0 && j > 0 {
                        format!(" 0x{:02X}", byte)
                    } else {
                        format!("0x{:02X}", byte)
                    }
                })
                .collect();
            
            let timestamp_str = format_timestamp(record.timestamp);
            let text = format!("[{}] {} ({}μs)", timestamp_str, hex_string, record.write_latency.as_micros());
            ListItem::new(text)
        }).collect();

        List::new(items)
            .style(Style::default().fg(red))
            .block(Block::default().borders(Borders::ALL).title("Live Binary Feed (Last 5 Records)").border_style(Style::default().fg(gray)))
    }
}

fn format_timestamp(nanos: u64) -> String {
    let seconds = nanos / 1_000_000_000;
    let remainder = nanos % 1_000_000_000;
    format!("{}.{:09}s", seconds, remainder)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting SAMS Black-Box Auditor v1.0...");
    println!("Press 'q' to quit");
    println!("Logging to: audit.samslog");
    
    let auditor = BlackBoxAuditor::new()?;
    auditor.run().await?;
    
    Ok(())
}
