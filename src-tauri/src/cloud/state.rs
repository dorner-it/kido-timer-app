use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::time::sleep;
use uuid::Uuid;

use super::client::CloudClient;
use super::keychain;
use super::kido;
use super::types::{
    CloudEvent, CloudIdentity, CompetitionListItem, CompetitionPayload, FailedPost, KidoConflict,
    KidoEnvelope, OpenKidoResult, PairedAccount, RunResultRequest, RunStatus,
};

pub const CLOUD_EVENT: &str = "kido://cloud-event";
const POLL_FAST_MS: u64 = 1500;
const POLL_SLOW_MS: u64 = 5000;

/// Per-(competition, run) memo of the last payload we successfully POSTed,
/// so we don't spam identical writes on every poll.
#[derive(Clone, Default)]
struct PostMemo {
    last_body: Option<RunResultRequest>,
}

#[derive(Clone)]
struct FailureRecord {
    competition_id: Uuid,
    body: RunResultRequest,
    error: String,
    failed_at: String,
}

#[derive(Default)]
struct CloudInner {
    account: Option<PairedAccount>,
    snapshot: Option<CompetitionPayload>,
    selected_competition_id: Option<Uuid>,
    post_memos: HashMap<Uuid, PostMemo>,
    failed_posts: HashMap<Uuid, FailureRecord>,
}

#[derive(Default)]
pub struct CloudState {
    inner: Arc<Mutex<CloudInner>>,
    poll_task: Mutex<Option<JoinHandle<()>>>,
}

impl CloudState {
    /// On startup, try to load the previously-paired account from the keychain.
    /// Errors are non-fatal — they just mean the user has to re-pair.
    pub fn load_from_keychain(&self) -> Result<Option<CloudIdentity>, String> {
        let account = keychain::load()?;
        let identity = account.as_ref().map(|a| a.identity());
        self.inner.lock().account = account;
        Ok(identity)
    }

    pub fn current_identity(&self) -> Option<CloudIdentity> {
        self.inner.lock().account.as_ref().map(|a| a.identity())
    }

    pub fn current_snapshot(&self) -> Option<CompetitionPayload> {
        self.inner.lock().snapshot.clone()
    }

    fn client(&self) -> Result<CloudClient, String> {
        let guard = self.inner.lock();
        let account = guard
            .account
            .as_ref()
            .ok_or_else(|| "not paired".to_string())?;
        CloudClient::new(account.base_url.clone(), account.api_key.clone())
    }

    /// Pair: validate the API key against the server, fetch the HMAC key,
    /// persist to the keychain, and return the resolved identity.
    pub async fn pair(
        &self,
        app: AppHandle,
        base_url: String,
        api_key: String,
    ) -> Result<CloudIdentity, String> {
        let base_url = base_url.trim_end_matches('/').to_string();
        let client = CloudClient::new(base_url.clone(), api_key.clone())?;
        let me = client.me().await?;
        let hmac = client.hmac_key().await?;

        let account = PairedAccount {
            base_url,
            api_key,
            hmac_key_b64: hmac.hmac_key,
            sub: me.sub,
            email: me.email,
            display_name: me.display_name,
            key_id: me.key_id,
        };
        keychain::save(&account)?;

        let identity = account.identity();
        {
            let mut guard = self.inner.lock();
            guard.account = Some(account);
            guard.snapshot = None;
            guard.selected_competition_id = None;
            guard.post_memos.clear();
            guard.failed_posts.clear();
        }
        emit(&app, CloudEvent::Paired { identity: identity.clone() });
        Ok(identity)
    }

    /// Forget the paired account.
    pub fn clear(&self, app: AppHandle) -> Result<(), String> {
        self.stop_polling();
        keychain::clear()?;
        {
            let mut guard = self.inner.lock();
            guard.account = None;
            guard.snapshot = None;
            guard.selected_competition_id = None;
            guard.post_memos.clear();
            guard.failed_posts.clear();
        }
        emit(&app, CloudEvent::Cleared);
        Ok(())
    }

    pub async fn list_competitions(&self) -> Result<Vec<CompetitionListItem>, String> {
        self.client()?.list_competitions().await
    }

    pub async fn fetch_snapshot(
        &self,
        app: AppHandle,
        id: Uuid,
    ) -> Result<CompetitionPayload, String> {
        let snapshot = self.client()?.get_competition(id).await?;
        if snapshot.schema_version != 1 {
            return Err(format!(
                "unsupported schema_version {}",
                snapshot.schema_version
            ));
        }
        {
            let mut guard = self.inner.lock();
            guard.snapshot = Some(snapshot.clone());
            guard.selected_competition_id = Some(id);
        }
        emit(&app, CloudEvent::SnapshotChanged { snapshot: snapshot.clone() });
        Ok(snapshot)
    }

    /// Select a competition and (re)start the background poller.
    pub async fn select_competition(
        &self,
        app: AppHandle,
        id: Uuid,
    ) -> Result<CompetitionPayload, String> {
        let snapshot = self.fetch_snapshot(app.clone(), id).await?;
        self.start_polling(app, id);
        Ok(snapshot)
    }

    pub fn deselect(&self, app: AppHandle) {
        self.stop_polling();
        {
            let mut guard = self.inner.lock();
            guard.snapshot = None;
            guard.selected_competition_id = None;
            guard.post_memos.clear();
            guard.failed_posts.clear();
        }
        emit(&app, CloudEvent::FailedPostsChanged { failures: vec![] });
    }

    fn start_polling(&self, app: AppHandle, id: Uuid) {
        self.stop_polling();
        let inner = self.inner.clone();
        let handle = tauri::async_runtime::spawn(async move {
            loop {
                let client = match make_client(&inner) {
                    Some(c) => c,
                    None => break,
                };
                let cadence = match client.get_competition(id).await {
                    Ok(snapshot) => {
                        let changed = {
                            let mut guard = inner.lock();
                            let differs = guard
                                .snapshot
                                .as_ref()
                                .map(|s| !payload_equiv(s, &snapshot))
                                .unwrap_or(true);
                            if differs {
                                guard.snapshot = Some(snapshot.clone());
                            }
                            differs
                        };
                        if changed {
                            emit(&app, CloudEvent::SnapshotChanged { snapshot: snapshot.clone() });
                        }
                        if has_active_run(&snapshot) {
                            POLL_FAST_MS
                        } else {
                            POLL_SLOW_MS
                        }
                    }
                    Err(e) => {
                        emit(&app, CloudEvent::SnapshotError { message: e });
                        POLL_SLOW_MS
                    }
                };
                sleep(Duration::from_millis(cadence)).await;
            }
        });
        *self.poll_task.lock() = Some(handle);
    }

    fn stop_polling(&self) {
        if let Some(handle) = self.poll_task.lock().take() {
            handle.abort();
        }
    }

    /// Idempotent POST: dedup'd against the last successful body for this
    /// run_id. Returns `true` if a request was sent, `false` if dedup'd.
    /// On HTTP success, clears any prior failure for this run.
    pub async fn maybe_post_run_status(
        &self,
        app: AppHandle,
        competition_id: Uuid,
        run_id: Uuid,
        body: RunResultRequest,
    ) -> Result<bool, String> {
        // Dedup against the last successful body.
        {
            let guard = self.inner.lock();
            if let Some(memo) = guard.post_memos.get(&run_id) {
                if let Some(last) = &memo.last_body {
                    if bodies_equal(last, &body) {
                        return Ok(false);
                    }
                }
            }
        }
        self.do_post(app, competition_id, run_id, body)
            .await
            .map(|_| true)
    }

    /// Force POST without dedup — used for manual retry.
    pub async fn retry_post(
        &self,
        app: AppHandle,
        run_id: Uuid,
    ) -> Result<(), String> {
        let (competition_id, body) = {
            let guard = self.inner.lock();
            let failure = guard
                .failed_posts
                .get(&run_id)
                .ok_or_else(|| format!("no pending failure for run {run_id}"))?;
            (failure.competition_id, failure.body.clone())
        };
        self.do_post(app, competition_id, run_id, body).await
    }

    async fn do_post(
        &self,
        app: AppHandle,
        competition_id: Uuid,
        run_id: Uuid,
        body: RunResultRequest,
    ) -> Result<(), String> {
        let client = self.client()?;
        match client.post_run_result(competition_id, run_id, &body).await {
            Ok(updated) => {
                let failures_after = {
                    let mut guard = self.inner.lock();
                    let memo = guard.post_memos.entry(run_id).or_default();
                    memo.last_body = Some(body.clone());
                    if let Some(snap) = guard.snapshot.as_mut() {
                        if let Some(slot) = snap.runs.iter_mut().find(|r| r.id == run_id) {
                            *slot = updated.clone();
                        }
                    }
                    let removed = guard.failed_posts.remove(&run_id).is_some();
                    if removed {
                        Some(snapshot_failures(&guard))
                    } else {
                        None
                    }
                };
                emit(
                    &app,
                    CloudEvent::ResultPosted {
                        run_id,
                        status: updated.status,
                    },
                );
                if let Some(failures) = failures_after {
                    emit(&app, CloudEvent::FailedPostsChanged { failures });
                }
                Ok(())
            }
            Err(e) => {
                let failures_after = {
                    let mut guard = self.inner.lock();
                    guard.failed_posts.insert(
                        run_id,
                        FailureRecord {
                            competition_id,
                            body,
                            error: e.clone(),
                            failed_at: now_rfc3339(),
                        },
                    );
                    snapshot_failures(&guard)
                };
                emit(
                    &app,
                    CloudEvent::ResultPostFailed {
                        run_id,
                        message: e.clone(),
                    },
                );
                emit(&app, CloudEvent::FailedPostsChanged { failures: failures_after });
                Err(e)
            }
        }
    }

    pub fn clear_failed_post(&self, app: AppHandle, run_id: Uuid) {
        let failures_after = {
            let mut guard = self.inner.lock();
            if guard.failed_posts.remove(&run_id).is_none() {
                return;
            }
            snapshot_failures(&guard)
        };
        emit(&app, CloudEvent::FailedPostsChanged { failures: failures_after });
    }

    pub fn list_failed_posts(&self) -> Vec<FailedPost> {
        snapshot_failures(&self.inner.lock())
    }

    /// Open a `.kido` envelope. If `force` is false and a different
    /// competition is currently loaded, returns `OpenKidoResult { adopted:
    /// false, conflict: Some(...) }` and leaves the snapshot untouched.
    pub fn open_kido_file(
        &self,
        app: AppHandle,
        path: String,
        force: bool,
    ) -> Result<OpenKidoResult, String> {
        let bytes = std::fs::read(&path).map_err(|e| format!("read failed: {e}"))?;
        let envelope: KidoEnvelope =
            serde_json::from_slice(&bytes).map_err(|e| format!("envelope parse failed: {e}"))?;

        let (hmac_key_b64, expected_sub) = {
            let guard = self.inner.lock();
            let account = guard
                .account
                .as_ref()
                .ok_or_else(|| "not paired — pair the desktop with a web account first".to_string())?;
            (account.hmac_key_b64.clone(), account.sub.clone())
        };

        let payload = kido::verify_envelope(&envelope, &hmac_key_b64, &expected_sub)?;

        if !force {
            let conflict = {
                let guard = self.inner.lock();
                guard.snapshot.as_ref().and_then(|current| {
                    if current.competition.id == payload.competition.id {
                        None
                    } else {
                        Some(KidoConflict {
                            current_competition_id: current.competition.id,
                            current_competition_name: current.competition.name.clone(),
                            new_competition_id: payload.competition.id,
                            new_competition_name: payload.competition.name.clone(),
                        })
                    }
                })
            };
            if let Some(conflict) = conflict {
                return Ok(OpenKidoResult {
                    adopted: false,
                    payload,
                    conflict: Some(conflict),
                });
            }
        }

        let comp_id = payload.competition.id;
        {
            let mut guard = self.inner.lock();
            guard.snapshot = Some(payload.clone());
            guard.selected_competition_id = Some(comp_id);
            guard.post_memos.clear();
            guard.failed_posts.clear();
        }
        emit(
            &app,
            CloudEvent::SnapshotChanged {
                snapshot: payload.clone(),
            },
        );
        emit(&app, CloudEvent::FailedPostsChanged { failures: vec![] });
        Ok(OpenKidoResult {
            adopted: true,
            payload,
            conflict: None,
        })
    }

    /// Sign and write the current snapshot to a `.kido` file at `path`.
    pub fn export_kido_file(&self, path: String) -> Result<(), String> {
        let (snapshot, hmac_key_b64, sub) = {
            let guard = self.inner.lock();
            let snapshot = guard
                .snapshot
                .as_ref()
                .ok_or_else(|| "no competition loaded".to_string())?
                .clone();
            let account = guard
                .account
                .as_ref()
                .ok_or_else(|| "not paired".to_string())?;
            (snapshot, account.hmac_key_b64.clone(), account.sub.clone())
        };
        let restamped = kido::restamp_for_export(&snapshot, &sub)?;
        let envelope = kido::build_envelope(&restamped, &hmac_key_b64)?;
        let json = serde_json::to_vec_pretty(&envelope)
            .map_err(|e| format!("envelope serialize failed: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("write failed: {e}"))
    }
}

fn make_client(inner: &Arc<Mutex<CloudInner>>) -> Option<CloudClient> {
    let guard = inner.lock();
    let account = guard.account.as_ref()?;
    CloudClient::new(account.base_url.clone(), account.api_key.clone()).ok()
}

fn emit(app: &AppHandle, event: CloudEvent) {
    let _ = app.emit(CLOUD_EVENT, event);
}

fn snapshot_failures(inner: &CloudInner) -> Vec<FailedPost> {
    inner
        .failed_posts
        .iter()
        .map(|(run_id, rec)| FailedPost {
            run_id: *run_id,
            competition_id: rec.competition_id,
            run_number: rec.body.run_number,
            status: rec.body.status,
            original_time_ms: rec.body.original_time_ms,
            started_at: rec.body.started_at.clone(),
            ended_at: rec.body.ended_at.clone(),
            error: rec.error.clone(),
            failed_at: rec.failed_at.clone(),
        })
        .collect()
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn has_active_run(snapshot: &CompetitionPayload) -> bool {
    snapshot
        .runs
        .iter()
        .any(|r| matches!(r.status, RunStatus::Active))
}

fn payload_equiv(a: &CompetitionPayload, b: &CompetitionPayload) -> bool {
    a.competition.is_active == b.competition.is_active
        && a.competition.current_run_id == b.competition.current_run_id
        && a.competition.sync_mode == b.competition.sync_mode
        && a.runs.len() == b.runs.len()
        && a.runs.iter().zip(b.runs.iter()).all(|(x, y)| {
            x.id == y.id
                && x.status == y.status
                && x.original_time_ms == y.original_time_ms
                && x.correction_ms == y.correction_ms
                && x.lane == y.lane
        })
        && a.approved_penalties.len() == b.approved_penalties.len()
}

fn bodies_equal(a: &RunResultRequest, b: &RunResultRequest) -> bool {
    a.run_number == b.run_number
        && a.status == b.status
        && a.original_time_ms == b.original_time_ms
        && a.started_at == b.started_at
        && a.ended_at == b.ended_at
}
