import { useCallback, useEffect, useRef, useState } from "react";
import { TopBar } from "./components/TopBar";
import { LaneGrid } from "./components/LaneGrid";
import { Drawer } from "./components/Drawer";
import { ErrorBanner } from "./components/ErrorBanner";
import { SetupModal } from "./components/SetupModal";
import { ProtocolView } from "./components/ProtocolView";
import { UpdateBanner } from "./components/UpdateBanner";
import { CloudPairingModal } from "./components/CloudPairingModal";
import { CompetitionPicker } from "./components/CompetitionPicker";
import { CompetitionBanner } from "./components/CompetitionBanner";
import { LaneAssignmentPanel } from "./components/LaneAssignmentPanel";
import { KidoConflictDialog } from "./components/KidoConflictDialog";
import { useConnection } from "./lib/useConnection";
import { useUpdater } from "./lib/useUpdater";
import { useCloud } from "./lib/useCloud";
import { loadSettings, patchSettings, type AppSettings, type Theme } from "./lib/persistence";
import { t } from "./lib/i18n";
import type { ChannelStatus, FrameDto } from "./lib/types";
import type { KidoConflict } from "./lib/cloudTypes";

type View = "lanes" | "protocol";

const NUM_LANES = 4;

export default function App() {
  const [selectedLane, setSelectedLane] = useState<number | null>(null);
  const [view, setView] = useState<View>("lanes");
  const [drawerOpen, setDrawerOpen] = useState(false);

  const [settings, setSettings] = useState<AppSettings>(() => loadSettings());
  const [setupOpen, setSetupOpen] = useState<boolean>(() => !loadSettings().hasCompletedSetup);
  const [autoTried, setAutoTried] = useState(false);
  const [pairingOpen, setPairingOpen] = useState(false);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [pendingKidoConflict, setPendingKidoConflict] = useState<{
    path: string;
    conflict: KidoConflict;
  } | null>(null);
  const [kidoConflictBusy, setKidoConflictBusy] = useState(false);

  const conn = useConnection();
  const corrections = conn.state.corrections;
  const updater = useUpdater();
  const cloud = useCloud();

  // Apply the chosen theme to <html> so CSS variables flip.
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", settings.theme);
  }, [settings.theme]);

  const handleThemeChange = useCallback((theme: Theme) => {
    setSettings((s) => {
      const next = { ...s, theme };
      patchSettings(next);
      return next;
    });
  }, []);

  // Auto-connect on first mount when settings allow.
  useEffect(() => {
    if (autoTried) return;
    setAutoTried(true);
    if (!settings.hasCompletedSetup) return;
    if (!settings.autoConnect) return;
    if (settings.port) {
      conn.connect(settings.port, settings.baud);
    } else if (settings.demoPath) {
      conn.startDemoMode(settings.demoPath, settings.demoSpeed);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const adjustChannel = useCallback(
    (idx: number, delta: number) => {
      conn.adjustCorrection(idx, delta);
      setSelectedLane(idx);
    },
    [conn],
  );

  const clearChannel = useCallback(
    (idx: number) => {
      conn.clearCorrection(idx);
    },
    [conn],
  );

  // ──────────────────────────────────────────────────────────────────────
  // Live ingest: watch frame channel transitions and POST status changes
  // (active on start, completed on confirm, cancelled when an in-progress
  // run goes back to idle without confirming).
  // ──────────────────────────────────────────────────────────────────────
  const lastChannelStatusRef = useRef<(ChannelStatus | null)[]>(
    Array(NUM_LANES).fill(null),
  );
  const startedAtRef = useRef<(string | null)[]>(Array(NUM_LANES).fill(null));
  const lastConfirmedKeyRef = useRef<Set<string>>(new Set());

  const cloudSnapshot = cloud.state.snapshot;
  const cloudPostLaneStatus = cloud.postLaneStatus;
  const frame = conn.state.frame;

  useEffect(() => {
    if (!frame) return;
    if (!cloudSnapshot) return;
    if (cloudSnapshot.discipline.sync_mode !== "live") return;

    frame.channels.forEach((ch, idx) => {
      const lane = idx + 1;
      const prev = lastChannelStatusRef.current[idx];
      const next = ch.status;
      lastChannelStatusRef.current[idx] = next;
      if (prev === next) return;

      const nowIso = new Date().toISOString();

      // Inactive → Running: a new run started on this lane.
      if (next === "running" && prev !== "running" && prev !== "captured") {
        startedAtRef.current[idx] = nowIso;
        void cloudPostLaneStatus(lane, {
          status: "active",
          startedAt: nowIso,
        });
        return;
      }

      // Running/Captured → Inactive without confirming: cancelled.
      if (
        next === "inactive" &&
        (prev === "running" || prev === "captured")
      ) {
        const startedAt = startedAtRef.current[idx];
        startedAtRef.current[idx] = null;
        void cloudPostLaneStatus(lane, {
          status: "cancelled",
          startedAt,
          endedAt: nowIso,
        });
      }
    });
  }, [frame, cloudSnapshot, cloudPostLaneStatus]);

  // Completed: dispatched off the dedup'd history list, since the protocol
  // emits Confirmed for many frames in a row but we only want to send once.
  useEffect(() => {
    if (!cloudSnapshot) return;
    if (cloudSnapshot.discipline.sync_mode !== "live") return;
    const head = conn.state.history[0];
    if (!head) return;
    if (lastConfirmedKeyRef.current.has(head.id)) return;
    lastConfirmedKeyRef.current.add(head.id);
    const lane = head.channel;
    const startedAt = startedAtRef.current[lane - 1];
    const endedAt = new Date(head.timestamp).toISOString();
    startedAtRef.current[lane - 1] = null;
    void cloudPostLaneStatus(lane, {
      status: "completed",
      originalTimeMs: head.originalTimeMs,
      startedAt,
      endedAt,
    });
  }, [conn.state.history, cloudSnapshot, cloudPostLaneStatus]);

  // Reset transition state when the snapshot changes (different competition
  // or none) so stale lane states don't leak across selections.
  useEffect(() => {
    lastChannelStatusRef.current = Array(NUM_LANES).fill(null);
    startedAtRef.current = Array(NUM_LANES).fill(null);
    lastConfirmedKeyRef.current = new Set();
  }, [cloudSnapshot?.discipline.id]);

  // Global keyboard shortcuts (only on the lanes view)
  useEffect(() => {
    if (view !== "lanes") return;
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const tag = target?.tagName;
      if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
      if (
        setupOpen ||
        drawerOpen ||
        pairingOpen ||
        pickerOpen ||
        pendingKidoConflict
      )
        return;

      switch (e.key) {
        case "1":
        case "2":
        case "3":
        case "4":
          setSelectedLane(Number(e.key) - 1);
          break;
        case "+":
        case "=":
          if (selectedLane !== null) adjustChannel(selectedLane, 5000);
          break;
        case "-":
        case "_":
          if (selectedLane !== null) adjustChannel(selectedLane, -5000);
          break;
        case "c":
        case "C":
          if (selectedLane !== null) clearChannel(selectedLane);
          break;
        case "Escape":
          setSelectedLane(null);
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    selectedLane,
    adjustChannel,
    clearChannel,
    view,
    setupOpen,
    drawerOpen,
    pairingOpen,
    pickerOpen,
    pendingKidoConflict,
  ]);

  const handleSetupConfirm = useCallback(
    (next: AppSettings, mode: "serial" | "demo" | "cloud") => {
      const persisted = patchSettings(next);
      setSettings(persisted);
      setSetupOpen(false);
      setDrawerOpen(false);
      if (mode === "serial" && persisted.port) {
        conn.connect(persisted.port, persisted.baud);
      } else if (mode === "demo" && persisted.demoPath) {
        conn.startDemoMode(persisted.demoPath, persisted.demoSpeed);
      }
    },
    [conn],
  );

  const handleSetupPair = useCallback(
    async (baseUrl: string, apiKey: string) => {
      await cloud.pair(baseUrl, apiKey);
      const persisted = patchSettings({ cloudBaseUrl: baseUrl });
      setSettings(persisted);
    },
    [cloud],
  );

  const handleEditConnection = useCallback(() => {
    setDrawerOpen(false);
    setSetupOpen(true);
  }, []);

  const handleDisconnect = useCallback(async () => {
    await conn.disconnect();
    setDrawerOpen(false);
  }, [conn]);

  const handleReset = useCallback(async () => {
    await conn.reset();
  }, [conn]);

  const handleCloudPair = useCallback(() => {
    setDrawerOpen(false);
    setPairingOpen(true);
  }, []);

  const handleCloudPairSubmit = useCallback(
    async (baseUrl: string, apiKey: string) => {
      await cloud.pair(baseUrl, apiKey);
      const persisted = patchSettings({ cloudBaseUrl: baseUrl });
      setSettings(persisted);
      setPairingOpen(false);
    },
    [cloud],
  );

  const handlePickCompetition = useCallback(() => {
    setDrawerOpen(false);
    setPickerOpen(true);
  }, []);

  const handleOpenKido = useCallback(async () => {
    const result = await cloud.openKido();
    if (result && !result.result.adopted && result.result.conflict) {
      setPendingKidoConflict({
        path: result.path,
        conflict: result.result.conflict,
      });
    }
  }, [cloud]);

  const confirmKidoOverwrite = useCallback(async () => {
    if (!pendingKidoConflict) return;
    setKidoConflictBusy(true);
    try {
      await cloud.openKidoForce(pendingKidoConflict.path);
      setPendingKidoConflict(null);
    } finally {
      setKidoConflictBusy(false);
    }
  }, [cloud, pendingKidoConflict]);

  return (
    <div className="flex h-screen w-screen flex-col">
      <TopBar
        status={conn.state.status}
        source={conn.state.source}
        frame={conn.state.frame}
        fps={conn.state.fps}
        frameCount={conn.state.frameCount}
        onMenu={() => setDrawerOpen(true)}
      />

      <main className="min-h-0 flex-1 flex flex-col">
        {view === "lanes" && (
          <>
            <CompetitionBanner
              snapshot={cloud.state.snapshot}
              onClose={() => void cloud.deselect()}
            />
            {cloud.state.snapshot && (
              <LaneAssignmentPanel
                snapshot={cloud.state.snapshot}
                laneOverrides={cloud.state.laneOverrides}
                onSetOverride={cloud.setLaneOverride}
              />
            )}
          </>
        )}
        {view === "lanes" ? (
          <LanesView
            frame={conn.state.frame}
            corrections={corrections}
            selectedLane={selectedLane}
            onSelectLane={setSelectedLane}
            onAdjust={adjustChannel}
            onClear={clearChannel}
          />
        ) : (
          <ProtocolView
            entries={conn.state.history}
            onBack={() => setView("lanes")}
            onClear={conn.clearHistory}
          />
        )}
      </main>

      <Drawer
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        status={conn.state.status}
        source={conn.state.source}
        theme={settings.theme}
        onThemeChange={handleThemeChange}
        onEditConnection={handleEditConnection}
        onDisconnect={handleDisconnect}
        onReset={handleReset}
        onOpenProtocol={() => {
          setView("protocol");
          setDrawerOpen(false);
        }}
        cloudIdentity={cloud.state.identity}
        cloudSnapshot={cloud.state.snapshot}
        cloudLoading={cloud.state.loading}
        cloudResultMessage={cloud.state.lastResultMessage}
        cloudFailedPosts={cloud.state.failedPosts}
        onCloudPair={handleCloudPair}
        onCloudUnpair={cloud.clear}
        onCloudPickCompetition={handlePickCompetition}
        onCloudDeselect={cloud.deselect}
        onCloudOpenKido={async () => {
          setDrawerOpen(false);
          await handleOpenKido();
        }}
        onCloudExportKido={cloud.exportKido}
        onCloudDismissResultMessage={cloud.dismissResultMessage}
        onCloudRetryFailed={cloud.retryFailedPost}
        onCloudDismissFailed={cloud.dismissFailedPost}
      />

      <SetupModal
        open={setupOpen}
        initial={settings}
        cloudIdentity={cloud.state.identity}
        required={!settings.hasCompletedSetup}
        onCancel={() => setSetupOpen(false)}
        onConfirm={handleSetupConfirm}
        onPair={handleSetupPair}
      />

      <CloudPairingModal
        open={pairingOpen}
        initialBaseUrl={settings.cloudBaseUrl}
        onCancel={() => setPairingOpen(false)}
        onSubmit={handleCloudPairSubmit}
      />

      <CompetitionPicker
        open={pickerOpen}
        disciplines={cloud.state.disciplines}
        loading={cloud.state.loading}
        error={cloud.state.error}
        selectedId={cloud.state.snapshot?.discipline.id ?? null}
        onRefresh={cloud.refreshDisciplines}
        onPick={cloud.selectDiscipline}
        onClose={() => setPickerOpen(false)}
      />

      <KidoConflictDialog
        conflict={pendingKidoConflict?.conflict ?? null}
        busy={kidoConflictBusy}
        onCancel={() => setPendingKidoConflict(null)}
        onConfirm={confirmKidoOverwrite}
      />

      <ErrorBanner message={conn.state.error} onDismiss={conn.clearError} />

      <UpdateBanner
        state={updater.state}
        onInstall={updater.install}
        onDismiss={updater.dismiss}
      />
    </div>
  );
}

interface LanesViewProps {
  frame: FrameDto | null;
  corrections: number[];
  selectedLane: number | null;
  onSelectLane: (lane: number | null) => void;
  onAdjust: (channel: number, delta: number) => void;
  onClear: (channel: number) => void;
}

function LanesView({
  frame,
  corrections,
  selectedLane,
  onSelectLane,
  onAdjust,
  onClear,
}: LanesViewProps) {
  return (
    <div className="flex h-full flex-col gap-4 px-7 py-6">
      <div className="flex items-end justify-between">
        <div>
          <span className="label">{t.lanes.sectionKicker}</span>
          <h2 className="font-display text-[22px] font-medium tracking-tight text-ink-50">
            {t.lanes.sectionTitle}
          </h2>
        </div>
        <p className="hidden font-mono text-[10px] uppercase tracking-[0.18em] text-ink-200/60 md:block">
          {t.lanes.focusHint}
        </p>
      </div>
      <div className="min-h-0 flex-1">
        <LaneGrid
          frame={frame}
          corrections={corrections}
          selectedLane={selectedLane}
          onSelectLane={onSelectLane}
          onAdjust={onAdjust}
          onClear={onClear}
        />
      </div>
    </div>
  );
}
