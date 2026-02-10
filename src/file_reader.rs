use std::io::{self, BufRead};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::protocol::{parse_frame, FRAME_SIZE, HEADER};
use crate::serial_reader::SerialEvent;

/// Parse a hex dump file (xxd-style) into raw bytes.
///
/// Expected line format:
/// ```text
/// 00000000  52 57 3A 01 00 01 00 00  00 00 00 00 00 00 00 00  |RW:.............|
/// ```
///
/// Lines starting with `---` or lacking hex data are skipped.
pub fn parse_hexdump(path: &str) -> io::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut bytes = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip empty lines and separator lines
        if trimmed.is_empty() || trimmed.starts_with("---") {
            continue;
        }

        // Data lines start with a hex offset followed by two spaces
        // Format: "00000000  52 57 3A ... |...|"
        let Some(hex_start) = trimmed.find("  ") else {
            continue;
        };

        let after_offset = &trimmed[hex_start + 2..];

        // Strip the ASCII representation at the end: "  |...|"
        let hex_part = match after_offset.rfind("  |") {
            Some(pos) => &after_offset[..pos],
            None => after_offset,
        };

        // Parse individual hex bytes
        for token in hex_part.split_whitespace() {
            if let Ok(byte) = u8::from_str_radix(token, 16) {
                bytes.push(byte);
            }
        }
    }

    Ok(bytes)
}

/// Spawn a thread that replays a hex dump file through the SerialEvent channel.
///
/// `speed` controls replay rate:
/// - 1.0 = real-time (~41ms per frame, matching 9600 baud)
/// - 0.0 = max speed (no delay)
/// - 2.0 = 2x speed, etc.
pub fn spawn_file_reader(
    path: &str,
    speed: f64,
    tx: Sender<SerialEvent>,
) -> JoinHandle<()> {
    let path = path.to_string();

    thread::spawn(move || {
        let raw_bytes = match parse_hexdump(&path) {
            Ok(b) => b,
            Err(e) => {
                let _ = tx.send(SerialEvent::Error(format!(
                    "Failed to read {}: {}",
                    path, e
                )));
                let _ = tx.send(SerialEvent::Disconnected);
                return;
            }
        };

        // Process bytes using the same sync logic as serial_reader
        let buffer = raw_bytes;
        let mut offset = 0;

        while offset < buffer.len() {
            // Find "RW:" header
            let remaining = &buffer[offset..];
            let header_pos = remaining
                .windows(HEADER.len())
                .position(|w| w == HEADER);

            let header_pos = match header_pos {
                Some(pos) => pos,
                None => break, // No more headers
            };

            offset += header_pos;

            // Check if we have a full frame
            if offset + FRAME_SIZE > buffer.len() {
                break;
            }

            // Extract and parse the frame
            let mut frame_bytes = [0u8; FRAME_SIZE];
            frame_bytes.copy_from_slice(&buffer[offset..offset + FRAME_SIZE]);

            match parse_frame(&frame_bytes) {
                Ok(frame) => {
                    if tx.send(SerialEvent::Frame(frame)).is_err() {
                        return; // Receiver dropped
                    }
                    offset += FRAME_SIZE;

                    // Pace delivery
                    if speed > 0.0 {
                        let delay_ms = (41.0 / speed) as u64;
                        thread::sleep(Duration::from_millis(delay_ms));
                    }
                }
                Err(e) => {
                    if tx
                        .send(SerialEvent::Error(format!("Parse error: {}", e)))
                        .is_err()
                    {
                        return;
                    }
                    offset += HEADER.len(); // Skip past header to resync
                }
            }
        }

        let _ = tx.send(SerialEvent::Disconnected);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hexdump_basic() {
        let tmp = std::env::temp_dir().join("test_hexdump.txt");
        std::fs::write(
            &tmp,
            "--- Connected to COM7 at 9600 baud (8N1, hex) ---\n\
             00000000  52 57 3A 01 00 01 00 00  00 00 00 00 00 00 00 00  |RW:.............|",
        )
        .unwrap();
        let bytes = parse_hexdump(tmp.to_str().unwrap()).unwrap();
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[..3], &[0x52, 0x57, 0x3A]);
        std::fs::remove_file(tmp).ok();
    }

    #[test]
    fn test_parse_hexdump_skips_separators() {
        let tmp = std::env::temp_dir().join("test_hexdump_sep.txt");
        std::fs::write(
            &tmp,
            "--- some separator ---\n\
             --- another ---\n\
             00000000  AA BB CC  |...|",
        )
        .unwrap();
        let bytes = parse_hexdump(tmp.to_str().unwrap()).unwrap();
        assert_eq!(bytes, vec![0xAA, 0xBB, 0xCC]);
        std::fs::remove_file(tmp).ok();
    }

    #[test]
    fn test_spawn_file_reader_sends_frames() {
        use std::sync::mpsc;

        // Build a minimal hex dump with one valid 41-byte frame
        let mut hex_line = String::from("00000000 ");
        let mut frame = vec![0u8; 41];
        frame[0] = 0x52; // R
        frame[1] = 0x57; // W
        frame[2] = 0x3A; // :
        frame[3] = 0x01; // version
        frame[4] = 0x00; // standby
        frame[5] = 0x01; // lane 1
        frame[39] = 0x0D; // terminator
        for (i, b) in frame.iter().enumerate() {
            hex_line.push_str(&format!(" {:02X}", b));
            if i == 15 {
                hex_line.push(' ');
            }
        }
        hex_line.push_str("  |");
        for b in &frame {
            if b.is_ascii_graphic() || *b == b' ' {
                hex_line.push(*b as char);
            } else {
                hex_line.push('.');
            }
        }
        hex_line.push('|');

        let tmp = std::env::temp_dir().join("test_hexdump_frame.txt");
        // Write across multiple lines (16 bytes each) to match real format
        let mut content = String::new();
        let chunks: Vec<&[u8]> = frame.chunks(16).collect();
        let mut file_offset = 0usize;
        for chunk in &chunks {
            content.push_str(&format!("{:08X} ", file_offset));
            for b in *chunk {
                content.push_str(&format!(" {:02X}", b));
            }
            content.push_str("  |");
            for b in *chunk {
                if b.is_ascii_graphic() || *b == b' ' {
                    content.push(*b as char);
                } else {
                    content.push('.');
                }
            }
            content.push_str("|\n");
            file_offset += chunk.len();
        }

        std::fs::write(&tmp, &content).unwrap();

        let (tx, rx) = mpsc::channel();
        let handle = spawn_file_reader(tmp.to_str().unwrap(), 0.0, tx);
        handle.join().unwrap();

        let mut frames = 0;
        let mut disconnected = false;
        while let Ok(evt) = rx.try_recv() {
            match evt {
                SerialEvent::Frame(_) => frames += 1,
                SerialEvent::Disconnected => disconnected = true,
                SerialEvent::Error(e) => panic!("unexpected error: {}", e),
            }
        }
        assert_eq!(frames, 1);
        assert!(disconnected);

        std::fs::remove_file(tmp).ok();
    }
}
