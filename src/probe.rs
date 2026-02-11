/// Candidate byte sequences for probing the TRV Kocab timer reset command.
///
/// The original Windows application has a "reset counter" feature, but the
/// exact bytes it sends are unknown. This module provides a list of plausible
/// candidates to try one at a time while observing device behavior.

use crate::protocol::{FRAME_SIZE, HEADER, TERMINATOR, TRAILING_NULL};

pub struct ProbeCandidate {
    pub label: &'static str,
    pub bytes: &'static [u8],
}

/// Build the 41-byte standby frame that the device sends when idle.
/// We try sending it back as a possible reset trigger.
const fn standby_frame() -> [u8; FRAME_SIZE] {
    let mut f = [0u8; FRAME_SIZE];
    f[0] = HEADER[0]; // 'R'
    f[1] = HEADER[1]; // 'W'
    f[2] = HEADER[2]; // ':'
    f[3] = 0x01; // version
    // bytes 4..38 stay 0x00 (standby, lane 0, measuring, all inactive)
    f[39] = TERMINATOR; // 0x0D
    f[40] = TRAILING_NULL; // 0x00
    f
}

static STANDBY_FRAME: [u8; FRAME_SIZE] = standby_frame();

pub static CANDIDATES: &[ProbeCandidate] = &[
    // -- Single bytes --
    ProbeCandidate {
        label: "0x00 (NUL)",
        bytes: &[0x00],
    },
    ProbeCandidate {
        label: "0x01 (SOH)",
        bytes: &[0x01],
    },
    ProbeCandidate {
        label: "0x0D (CR)",
        bytes: &[0x0D],
    },
    ProbeCandidate {
        label: "0x0A (LF)",
        bytes: &[0x0A],
    },
    ProbeCandidate {
        label: "0xFF",
        bytes: &[0xFF],
    },
    // -- ASCII commands (CR-terminated) --
    ProbeCandidate {
        label: "R\\r",
        bytes: b"R\r",
    },
    ProbeCandidate {
        label: "RESET\\r",
        bytes: b"RESET\r",
    },
    ProbeCandidate {
        label: "RST\\r",
        bytes: b"RST\r",
    },
    ProbeCandidate {
        label: "C\\r",
        bytes: b"C\r",
    },
    ProbeCandidate {
        label: "CLR\\r",
        bytes: b"CLR\r",
    },
    ProbeCandidate {
        label: "CLEAR\\r",
        bytes: b"CLEAR\r",
    },
    // -- Protocol-mirrored (RW: header) --
    ProbeCandidate {
        label: "RW: + 0x00",
        bytes: &[0x52, 0x57, 0x3A, 0x00],
    },
    ProbeCandidate {
        label: "RW: + 01 00",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x00],
    },
    ProbeCandidate {
        label: "RW: + 01 FF",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0xFF],
    },
    ProbeCandidate {
        label: "Standby frame (41B)",
        bytes: &STANDBY_FRAME,
    },
    // -- Common embedded control bytes --
    ProbeCandidate {
        label: "STX+ETX",
        bytes: &[0x02, 0x03],
    },
    ProbeCandidate {
        label: "ESC (0x1B)",
        bytes: &[0x1B],
    },
    ProbeCandidate {
        label: "CR+LF",
        bytes: &[0x0D, 0x0A],
    },
];
