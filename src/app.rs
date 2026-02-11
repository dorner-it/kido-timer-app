use std::io::Write;
use std::time::Instant;

use crate::probe::CANDIDATES;
use crate::protocol::{
    ChannelStatus, DeviceMode, StateFlag, TimeChannel, TimerFrame, NUM_CHANNELS,
};
use crate::serial_reader::SerialEvent;

/// A log entry recording one probe send attempt.
#[allow(dead_code)]
pub struct ProbeLogEntry {
    pub label: String,
    pub hex: String,
    pub success: bool,
    pub device_mode_before: DeviceMode,
    pub device_mode_after: Option<DeviceMode>,
    pub frame_count_before: u64,
}

pub struct App {
    pub latest_frame: Option<TimerFrame>,
    pub device_mode: DeviceMode,
    pub lane: u8,
    pub state_flag: StateFlag,
    pub channels: [TimeChannel; NUM_CHANNELS],
    pub frame_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub connected: bool,
    pub running: bool,
    pub frames_per_second: f64,

    // FPS tracking internals
    fps_timestamps: Vec<Instant>,

    // Probe mode
    pub probe_active: bool,
    pub probe_index: usize,
    pub probe_log: Vec<ProbeLogEntry>,
    pub probe_write_port: Option<Box<dyn serialport::SerialPort>>,
    pub probe_auto_running: bool,
}

impl App {
    pub fn new(write_port: Option<Box<dyn serialport::SerialPort>>) -> Self {
        Self {
            latest_frame: None,
            device_mode: DeviceMode::Standby,
            lane: 0,
            state_flag: StateFlag::Measuring,
            channels: [TimeChannel {
                time_ms: 0,
                status: ChannelStatus::Inactive,
            }; NUM_CHANNELS],
            frame_count: 0,
            error_count: 0,
            last_error: None,
            connected: false,
            running: true,
            frames_per_second: 0.0,
            fps_timestamps: Vec::new(),

            probe_active: false,
            probe_index: 0,
            probe_log: Vec::new(),
            probe_write_port: write_port,
            probe_auto_running: false,
        }
    }

    /// Returns true if probe mode is available (serial write handle exists).
    pub fn probe_available(&self) -> bool {
        self.probe_write_port.is_some()
    }

    pub fn apply_event(&mut self, event: SerialEvent) {
        match event {
            SerialEvent::Frame(frame) => {
                self.connected = true;
                self.device_mode = frame.device_mode;
                self.lane = frame.lane;
                self.state_flag = frame.state_flag;
                self.channels = frame.channels;
                self.latest_frame = Some(frame);
                self.frame_count += 1;
                self.update_fps();

                // Stamp device_mode_after on the most recent probe log entry
                // (first frame received after a send)
                if let Some(entry) = self.probe_log.last_mut() {
                    if entry.device_mode_after.is_none() {
                        entry.device_mode_after = Some(self.device_mode);
                    }
                }
            }
            SerialEvent::Error(msg) => {
                self.error_count += 1;
                self.last_error = Some(msg);
            }
            SerialEvent::Disconnected => {
                self.connected = false;
            }
        }
    }

    /// Send the currently selected probe candidate to the device.
    pub fn probe_send_current(&mut self) {
        let candidate = &CANDIDATES[self.probe_index];
        let hex = candidate
            .bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let success = if let Some(ref mut port) = self.probe_write_port {
            port.write_all(candidate.bytes).is_ok()
        } else {
            false
        };

        self.probe_log.push(ProbeLogEntry {
            label: candidate.label.to_string(),
            hex,
            success,
            device_mode_before: self.device_mode,
            device_mode_after: None,
            frame_count_before: self.frame_count,
        });
    }

    /// Advance auto-send: send one candidate and move to the next.
    /// Returns false when all candidates have been sent.
    pub fn probe_auto_tick(&mut self) -> bool {
        if !self.probe_auto_running {
            return false;
        }
        self.probe_send_current();
        if self.probe_index + 1 < CANDIDATES.len() {
            self.probe_index += 1;
            true
        } else {
            self.probe_auto_running = false;
            false
        }
    }

    fn update_fps(&mut self) {
        let now = Instant::now();
        self.fps_timestamps.push(now);

        // Keep only timestamps from the last second
        let one_sec_ago = now - std::time::Duration::from_secs(1);
        self.fps_timestamps.retain(|&t| t >= one_sec_ago);

        self.frames_per_second = self.fps_timestamps.len() as f64;
    }
}
