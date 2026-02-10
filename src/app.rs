use std::time::Instant;

use crate::protocol::{
    ChannelStatus, DeviceMode, StateFlag, TimeChannel, TimerFrame, NUM_CHANNELS,
};
use crate::serial_reader::SerialEvent;

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
}

impl App {
    pub fn new() -> Self {
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
        }
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

    fn update_fps(&mut self) {
        let now = Instant::now();
        self.fps_timestamps.push(now);

        // Keep only timestamps from the last second
        let one_sec_ago = now - std::time::Duration::from_secs(1);
        self.fps_timestamps.retain(|&t| t >= one_sec_ago);

        self.frames_per_second = self.fps_timestamps.len() as f64;
    }
}
