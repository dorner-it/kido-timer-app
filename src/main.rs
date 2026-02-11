mod app;
mod file_reader;
mod probe;
mod protocol;
mod serial_reader;
mod ui;

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use file_reader::spawn_file_reader;
use probe::CANDIDATES;
use serial_reader::spawn_serial_reader;

#[derive(Parser)]
#[command(name = "serialconverter")]
#[command(about = "TRV Kocab Sports Timer serial protocol parser and TUI dashboard")]
struct Cli {
    /// Serial port (e.g., /dev/ttyUSB0 or COM7)
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate
    #[arg(short, long, default_value_t = 9600)]
    baud: u32,

    /// Replay a hex dump file instead of reading from a serial port
    #[arg(short, long)]
    file: Option<String>,

    /// Replay speed multiplier (1.0 = real-time, 0 = max speed)
    #[arg(short, long, default_value_t = 1.0)]
    speed: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Validate: exactly one of --port or --file must be provided
    match (&cli.port, &cli.file) {
        (None, None) => {
            eprintln!("Error: provide either --port or --file");
            std::process::exit(1);
        }
        (Some(_), Some(_)) => {
            eprintln!("Error: --port and --file are mutually exclusive");
            std::process::exit(1);
        }
        _ => {}
    }

    // Set up channel for serial events
    let (tx, rx) = mpsc::channel();

    // Spawn reader thread and get optional write handle
    let (_reader_handle, write_port) = if let Some(ref file_path) = cli.file {
        (spawn_file_reader(file_path, cli.speed, tx), None)
    } else {
        spawn_serial_reader(cli.port.as_deref().unwrap(), cli.baud, tx)
    };

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(write_port);

    // Main event loop (~30fps)
    let tick_rate = Duration::from_millis(33);

    while app.running {
        // Drain all pending serial events
        while let Ok(event) = rx.try_recv() {
            app.apply_event(event);
        }

        // Auto-send probe tick (one candidate per tick when active)
        if app.probe_auto_running {
            app.probe_auto_tick();
        }

        // Render UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll for keyboard input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            app.running = false;
                        }
                        // Probe mode toggle
                        KeyCode::Char('p') | KeyCode::Char('P') if app.probe_available() => {
                            app.probe_active = !app.probe_active;
                            app.probe_auto_running = false;
                        }
                        // Probe candidate navigation
                        KeyCode::Left | KeyCode::Up if app.probe_active => {
                            if app.probe_index > 0 {
                                app.probe_index -= 1;
                            } else {
                                app.probe_index = CANDIDATES.len() - 1;
                            }
                            app.probe_auto_running = false;
                        }
                        KeyCode::Right | KeyCode::Down if app.probe_active => {
                            if app.probe_index + 1 < CANDIDATES.len() {
                                app.probe_index += 1;
                            } else {
                                app.probe_index = 0;
                            }
                            app.probe_auto_running = false;
                        }
                        // Send current probe candidate
                        KeyCode::Enter if app.probe_active => {
                            app.probe_send_current();
                        }
                        // Auto-send all candidates
                        KeyCode::Char('a') | KeyCode::Char('A') if app.probe_active => {
                            app.probe_index = 0;
                            app.probe_auto_running = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Received {} frames, {} errors.", app.frame_count, app.error_count);

    Ok(())
}
