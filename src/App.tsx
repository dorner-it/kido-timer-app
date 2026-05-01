import { useCallback, useEffect, useState } from "react";
import { TopBar } from "./components/TopBar";
import { LaneGrid } from "./components/LaneGrid";
import { Drawer } from "./components/Drawer";
import { ErrorBanner } from "./components/ErrorBanner";
import { SetupModal } from "./components/SetupModal";
import { ProtocolView } from "./components/ProtocolView";
import { UpdateBanner } from "./components/UpdateBanner";
import { useConnection } from "./lib/useConnection";
import { useUpdater } from "./lib/useUpdater";
import { loadSettings, patchSettings, type AppSettings, type Theme } from "./lib/persistence";
import { t } from "./lib/i18n";

type View = "lanes" | "protocol";

export default function App() {
  const [selectedLane, setSelectedLane] = useState<number | null>(null);
  const [view, setView] = useState<View>("lanes");
  const [drawerOpen, setDrawerOpen] = useState(false);

  const [settings, setSettings] = useState<AppSettings>(() => loadSettings());
  const [setupOpen, setSetupOpen] = useState<boolean>(() => !loadSettings().hasCompletedSetup);
  const [autoTried, setAutoTried] = useState(false);

  const conn = useConnection();
  const corrections = conn.state.corrections;
  const updater = useUpdater();

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

  // Global keyboard shortcuts (only on the lanes view)
  useEffect(() => {
    if (view !== "lanes") return;
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const tag = target?.tagName;
      if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
      if (setupOpen || drawerOpen) return;

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
  }, [selectedLane, adjustChannel, clearChannel, view, setupOpen, drawerOpen]);

  const handleSetupConfirm = useCallback(
    (next: AppSettings, mode: "serial" | "demo") => {
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

      <main className="min-h-0 flex-1">
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
      />

      <SetupModal
        open={setupOpen}
        initial={settings}
        required={!settings.hasCompletedSetup}
        onCancel={() => setSetupOpen(false)}
        onConfirm={handleSetupConfirm}
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
  frame: import("./lib/types").FrameDto | null;
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
