export type TimerMode = "single_lane" | "two_lane_parallel" | "relay" | "individual";
export type SyncMode = "live" | "offline";
export type RunStatus = "pending" | "active" | "completed" | "cancelled";
export type PenaltyStatus = "pending" | "approved" | "rejected";

export interface CloudIdentity {
  baseUrl: string;
  sub: string;
  email: string;
  displayName: string;
  keyId: string;
}

export interface CompetitionListItem {
  id: string;
  name: string;
  date: string; // YYYY-MM-DD
  mode: TimerMode;
  sync_mode: SyncMode;
  is_active: boolean;
  current_run_id: string | null;
}

export interface CompetitionMeta {
  id: string;
  owner_sub: string;
  name: string;
  date: string;
  location: string | null;
  mode: TimerMode;
  sync_mode: SyncMode;
  is_active?: boolean | null;
  current_run_id?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface Team {
  id: string;
  competition_id: string;
  name: string;
  start_number: number;
}

export interface Runner {
  id: string;
  team_id: string;
  competition_id: string;
  first_name: string;
  last_name: string;
  birth_date: string;
  position: number;
}

export interface Run {
  id: string;
  competition_id: string;
  team_id: string;
  runner_id: string | null;
  run_number: number;
  lane: number | null;
  status: RunStatus;
  original_time_ms: number | null;
  correction_ms: number;
  started_at: string | null;
  ended_at: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface Penalty {
  id: string;
  competition_id: string;
  run_id: string;
  team_id: string;
  runner_id: string | null;
  decider_slot: number;
  delta_ms: number;
  reason_code: string | null;
  status: PenaltyStatus;
  created_at: string;
  reviewed_at: string | null;
  reviewed_by: string | null;
}

export interface CompetitionPayload {
  schema_version: number;
  exported_at: string;
  owner_sub: string;
  competition: CompetitionMeta;
  teams: Team[];
  runners: Runner[];
  runs: Run[];
  approved_penalties: Penalty[];
}

export interface FailedPost {
  runId: string;
  competitionId: string;
  runNumber: number;
  status: RunStatus;
  originalTimeMs: number | null;
  startedAt: string | null;
  endedAt: string | null;
  error: string;
  failedAt: string;
}

export interface KidoConflict {
  currentCompetitionId: string;
  currentCompetitionName: string;
  newCompetitionId: string;
  newCompetitionName: string;
}

export interface OpenKidoResult {
  adopted: boolean;
  payload: CompetitionPayload;
  conflict?: KidoConflict | null;
}

export type CloudEvent =
  | { type: "paired"; identity: CloudIdentity }
  | { type: "cleared" }
  | { type: "snapshotChanged"; snapshot: CompetitionPayload }
  | { type: "snapshotError"; message: string }
  | { type: "resultPosted"; runId: string; status: RunStatus }
  | { type: "resultPostFailed"; runId: string; message: string }
  | { type: "failedPostsChanged"; failures: FailedPost[] };
