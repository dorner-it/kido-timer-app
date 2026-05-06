use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    SingleLane,
    TwoLaneParallel,
    Relay,
    Individual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Live,
    Offline,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PenaltyStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HmacKeyResponse {
    pub hmac_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionListItem {
    pub id: Uuid,
    pub name: String,
    pub date: String,
    pub mode: TimerMode,
    pub sync_mode: SyncMode,
    pub is_active: bool,
    pub current_run_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionMeta {
    pub id: Uuid,
    pub owner_sub: String,
    pub name: String,
    pub date: String,
    pub location: Option<String>,
    pub mode: TimerMode,
    pub sync_mode: SyncMode,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub current_run_id: Option<Uuid>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub name: String,
    pub start_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    pub id: Uuid,
    pub team_id: Uuid,
    pub competition_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: String,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub team_id: Uuid,
    #[serde(default)]
    pub runner_id: Option<Uuid>,
    pub run_number: u32,
    #[serde(default)]
    pub lane: Option<u8>,
    pub status: RunStatus,
    #[serde(default)]
    pub original_time_ms: Option<u32>,
    #[serde(default)]
    pub correction_ms: i32,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub ended_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub run_id: Uuid,
    pub team_id: Uuid,
    #[serde(default)]
    pub runner_id: Option<Uuid>,
    pub decider_slot: u8,
    pub delta_ms: i32,
    #[serde(default)]
    pub reason_code: Option<String>,
    pub status: PenaltyStatus,
    pub created_at: String,
    #[serde(default)]
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub reviewed_by: Option<String>,
}

/// Inner payload of a `.kido` envelope, also the response shape of
/// `GET /api/export/competitions/:id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionPayload {
    pub schema_version: u32,
    pub exported_at: String,
    pub owner_sub: String,
    pub competition: CompetitionMeta,
    #[serde(default)]
    pub teams: Vec<Team>,
    #[serde(default)]
    pub runners: Vec<Runner>,
    #[serde(default)]
    pub runs: Vec<Run>,
    #[serde(default)]
    pub approved_penalties: Vec<Penalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KidoEnvelope {
    pub v: u32,
    pub alg: String,
    pub owner_sub: String,
    pub issued_at: String,
    /// standard base64 (with padding) of the payload UTF-8 JSON bytes
    pub payload: String,
    /// base64url (no padding) of HMAC-SHA256 over `payload` bytes
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResultRequest {
    pub run_number: u32,
    pub status: RunStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_time_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
}

/// Frontend-facing identity payload (mirrors what the user pasted but adds
/// the resolved profile). Plaintext API key is intentionally omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudIdentity {
    pub base_url: String,
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub key_id: String,
}

/// Persisted credential blob written to the OS keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedAccount {
    pub base_url: String,
    pub api_key: String,
    /// base64url no-pad — wire form, decoded only for HMAC ops.
    pub hmac_key_b64: String,
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub key_id: String,
}

impl PairedAccount {
    pub fn identity(&self) -> CloudIdentity {
        CloudIdentity {
            base_url: self.base_url.clone(),
            sub: self.sub.clone(),
            email: self.email.clone(),
            display_name: self.display_name.clone(),
            key_id: self.key_id.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CloudEvent {
    Paired {
        identity: CloudIdentity,
    },
    Cleared,
    SnapshotChanged {
        snapshot: CompetitionPayload,
    },
    SnapshotError {
        message: String,
    },
    ResultPosted {
        run_id: Uuid,
        status: RunStatus,
    },
    ResultPostFailed {
        run_id: Uuid,
        message: String,
    },
    FailedPostsChanged {
        failures: Vec<FailedPost>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedPost {
    pub run_id: Uuid,
    pub competition_id: Uuid,
    pub run_number: u32,
    pub status: RunStatus,
    pub original_time_ms: Option<u32>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub error: String,
    pub failed_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenKidoResult {
    pub adopted: bool,
    pub payload: CompetitionPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict: Option<KidoConflict>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KidoConflict {
    pub current_competition_id: Uuid,
    pub current_competition_name: String,
    pub new_competition_id: Uuid,
    pub new_competition_name: String,
}
