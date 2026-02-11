/// Probe candidates for discovering TRV Kocab timer time-adjustment commands.
///
/// The confirmed reset command is `RST\r` (0x52 0x53 0x54 0x0D). The original
/// Windows application can also add 5/10/15 seconds to channel times. Since
/// the reset command is a simple CR-terminated ASCII string, the time-adjustment
/// commands likely follow the same pattern. This module provides plausible
/// candidates to try one at a time while observing channel time changes.

pub struct ProbeCandidate {
    pub label: &'static str,
    pub bytes: &'static [u8],
}

/// Confirmed reset command: RST followed by carriage return.
pub const RESET_CMD: &[u8] = b"RST\r";

pub static CANDIDATES: &[ProbeCandidate] = &[
    // -- Direct value commands (CR-terminated) --
    ProbeCandidate {
        label: "+5\\r",
        bytes: b"+5\r",
    },
    ProbeCandidate {
        label: "+10\\r",
        bytes: b"+10\r",
    },
    ProbeCandidate {
        label: "+15\\r",
        bytes: b"+15\r",
    },
    // -- ADD prefix --
    ProbeCandidate {
        label: "ADD5\\r",
        bytes: b"ADD5\r",
    },
    ProbeCandidate {
        label: "ADD10\\r",
        bytes: b"ADD10\r",
    },
    ProbeCandidate {
        label: "ADD15\\r",
        bytes: b"ADD15\r",
    },
    // -- T prefix (Time) --
    ProbeCandidate {
        label: "T+5\\r",
        bytes: b"T+5\r",
    },
    ProbeCandidate {
        label: "T+10\\r",
        bytes: b"T+10\r",
    },
    ProbeCandidate {
        label: "T+15\\r",
        bytes: b"T+15\r",
    },
    // -- ADJ prefix (Adjust) --
    ProbeCandidate {
        label: "ADJ5\\r",
        bytes: b"ADJ5\r",
    },
    ProbeCandidate {
        label: "ADJ10\\r",
        bytes: b"ADJ10\r",
    },
    ProbeCandidate {
        label: "ADJ15\\r",
        bytes: b"ADJ15\r",
    },
    // -- Protocol-style (RW: header + time bytes, LE u16 ms) --
    // 5000ms = 0x1388 -> bytes 0x88 0x13
    ProbeCandidate {
        label: "RW:01 5000ms LE",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x88, 0x13],
    },
    // 10000ms = 0x2710 -> bytes 0x10 0x27
    ProbeCandidate {
        label: "RW:01 10000ms LE",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x10, 0x27],
    },
    // 15000ms = 0x3A98 -> bytes 0x98 0x3A
    ProbeCandidate {
        label: "RW:01 15000ms LE",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x98, 0x3A],
    },
    // -- Single-value exploration --
    ProbeCandidate {
        label: "SET\\r",
        bytes: b"SET\r",
    },
    ProbeCandidate {
        label: "TIME\\r",
        bytes: b"TIME\r",
    },
    ProbeCandidate {
        label: "INC\\r",
        bytes: b"INC\r",
    },
    ProbeCandidate {
        label: "ADJ\\r",
        bytes: b"ADJ\r",
    },
];
