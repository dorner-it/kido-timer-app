use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::protocol::{parse_frame, FRAME_SIZE, HEADER};

pub enum SerialEvent {
    Frame(crate::protocol::TimerFrame),
    Error(String),
    Disconnected,
}

/// Spawn a thread that reads from the serial port, syncs frames, and sends parsed events.
pub fn spawn_serial_reader(
    port_name: &str,
    baud_rate: u32,
    tx: Sender<SerialEvent>,
) -> JoinHandle<()> {
    let port_name = port_name.to_string();

    thread::spawn(move || {
        let port = serialport::new(&port_name, baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_millis(100))
            .open();

        let mut port = match port {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(SerialEvent::Error(format!(
                    "Failed to open {}: {}",
                    port_name, e
                )));
                let _ = tx.send(SerialEvent::Disconnected);
                return;
            }
        };

        let mut buffer: Vec<u8> = Vec::with_capacity(256);
        let mut read_buf = [0u8; 128];

        loop {
            // Check if receiver is still alive
            match port.read(&mut read_buf) {
                Ok(n) if n > 0 => {
                    buffer.extend_from_slice(&read_buf[..n]);
                }
                Ok(_) => {
                    // No data, continue
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Timeout is normal, continue
                }
                Err(e) => {
                    let _ = tx.send(SerialEvent::Error(format!("Read error: {}", e)));
                    let _ = tx.send(SerialEvent::Disconnected);
                    return;
                }
            }

            // Process all complete frames in the buffer
            loop {
                // Find the header "RW:" in the buffer
                let header_pos = buffer
                    .windows(HEADER.len())
                    .position(|w| w == HEADER);

                let header_pos = match header_pos {
                    Some(pos) => pos,
                    None => {
                        // No header found -- keep last 2 bytes in case header spans reads
                        if buffer.len() > 2 {
                            let keep_from = buffer.len() - 2;
                            buffer.drain(..keep_from);
                        }
                        break;
                    }
                };

                // Discard any bytes before the header (resync)
                if header_pos > 0 {
                    buffer.drain(..header_pos);
                }

                // Check if we have a full frame
                if buffer.len() < FRAME_SIZE {
                    break; // Wait for more data
                }

                // Extract the frame
                let mut frame_bytes = [0u8; FRAME_SIZE];
                frame_bytes.copy_from_slice(&buffer[..FRAME_SIZE]);

                match parse_frame(&frame_bytes) {
                    Ok(frame) => {
                        if tx.send(SerialEvent::Frame(frame)).is_err() {
                            return; // Receiver dropped, exit thread
                        }
                        buffer.drain(..FRAME_SIZE);
                    }
                    Err(e) => {
                        // Bad frame -- skip past the header and try to resync
                        if tx
                            .send(SerialEvent::Error(format!("Parse error: {}", e)))
                            .is_err()
                        {
                            return;
                        }
                        buffer.drain(..HEADER.len());
                    }
                }
            }

            // Prevent unbounded buffer growth
            if buffer.len() > 4096 {
                let keep_from = buffer.len() - 256;
                buffer.drain(..keep_from);
            }
        }
    })
}
