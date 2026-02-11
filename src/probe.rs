/// Probe candidates for discovering TRV Kocab timer time-adjustment commands.
///
/// Round 2: The first batch (no lane) had no effect. Since the protocol
/// includes a lane field and the device operates on lanes 1-4, the command
/// likely requires a channel/lane identifier. We also try spaced variants,
/// zero-padded values, CRLF terminators, sports-timing terms (PEN/COR),
/// and millisecond values. All candidates use lane/channel 1 for consistency.

pub struct ProbeCandidate {
    pub label: &'static str,
    pub bytes: &'static [u8],
}

/// Confirmed reset command: RST followed by carriage return.
pub const RESET_CMD: &[u8] = b"RST\r";

pub static CANDIDATES: &[ProbeCandidate] = &[
    // -- Lane-prefixed: CMD<lane> <secs>\r --
    ProbeCandidate {
        label: "ADD1 5\\r",
        bytes: b"ADD1 5\r",
    },
    ProbeCandidate {
        label: "ADD1 10\\r",
        bytes: b"ADD1 10\r",
    },
    ProbeCandidate {
        label: "ADD1 15\\r",
        bytes: b"ADD1 15\r",
    },
    // -- Spaced: CMD <lane> <secs>\r --
    ProbeCandidate {
        label: "ADD 1 5\\r",
        bytes: b"ADD 1 5\r",
    },
    ProbeCandidate {
        label: "ADJ 1 5\\r",
        bytes: b"ADJ 1 5\r",
    },
    ProbeCandidate {
        label: "COR 1 5\\r",
        bytes: b"COR 1 5\r",
    },
    ProbeCandidate {
        label: "PEN 1 5\\r",
        bytes: b"PEN 1 5\r",
    },
    // -- Lane after value: CMD <secs> <lane>\r --
    ProbeCandidate {
        label: "ADD 5 1\\r",
        bytes: b"ADD 5 1\r",
    },
    ProbeCandidate {
        label: "COR 5 1\\r",
        bytes: b"COR 5 1\r",
    },
    ProbeCandidate {
        label: "PEN 5 1\\r",
        bytes: b"PEN 5 1\r",
    },
    // -- Zero-padded seconds --
    ProbeCandidate {
        label: "ADD 1 05\\r",
        bytes: b"ADD 1 05\r",
    },
    ProbeCandidate {
        label: "COR 1 05\\r",
        bytes: b"COR 1 05\r",
    },
    // -- Compact: CMD<lane><secs>\r (no spaces) --
    ProbeCandidate {
        label: "ADD15\\r (L1 5s)",
        bytes: b"ADD15\r",
    },
    ProbeCandidate {
        label: "COR15\\r (L1 5s)",
        bytes: b"COR15\r",
    },
    ProbeCandidate {
        label: "PEN15\\r (L1 5s)",
        bytes: b"PEN15\r",
    },
    // -- Millisecond values --
    ProbeCandidate {
        label: "ADD 1 5000\\r",
        bytes: b"ADD 1 5000\r",
    },
    ProbeCandidate {
        label: "COR 1 5000\\r",
        bytes: b"COR 1 5000\r",
    },
    // -- CRLF terminators --
    ProbeCandidate {
        label: "ADD 1 5\\r\\n",
        bytes: b"ADD 1 5\r\n",
    },
    ProbeCandidate {
        label: "COR 1 5\\r\\n",
        bytes: b"COR 1 5\r\n",
    },
    // -- Bare commands (might toggle/prompt a mode) --
    ProbeCandidate {
        label: "PEN\\r",
        bytes: b"PEN\r",
    },
    ProbeCandidate {
        label: "COR\\r",
        bytes: b"COR\r",
    },
    ProbeCandidate {
        label: "PLS\\r",
        bytes: b"PLS\r",
    },
    ProbeCandidate {
        label: "MOD\\r",
        bytes: b"MOD\r",
    },
    // -- Protocol-style with lane byte + time LE u16 --
    // RW: + version(01) + lane(01) + 5000ms LE (88 13)
    ProbeCandidate {
        label: "RW:01 L1 5000ms",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x01, 0x88, 0x13],
    },
    // RW: + version(01) + lane(01) + 5000ms LE + CR
    ProbeCandidate {
        label: "RW:01 L1 5s +CR",
        bytes: &[0x52, 0x57, 0x3A, 0x01, 0x01, 0x88, 0x13, 0x0D],
    },
];
