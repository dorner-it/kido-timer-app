import { useEffect, useState } from "react";
import { deviceModeDE, stateFlagDE, t } from "../lib/i18n";
import type { ConnectionStatus, FrameDto } from "../lib/types";

interface Props {
  status: ConnectionStatus;
  source: string | null;
  frame: FrameDto | null;
  fps: number;
  frameCount: number;
  onMenu: () => void;
}

const STATUS_DOT: Record<ConnectionStatus, { color: string; label: string }> = {
  idle: { color: "bg-ink-300", label: t.conn.statusIdle },
  connecting: { color: "bg-status-armed animate-pulse", label: t.conn.statusConnecting },
  connected: { color: "bg-status-confirmed", label: t.conn.statusConnected },
  error: { color: "bg-status-unknown animate-pulse", label: t.conn.statusError },
};

function useNowClock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, []);
  return now;
}

export function TopBar({ status, source, frame, fps, frameCount, onMenu }: Props) {
  const dot = STATUS_DOT[status];
  const now = useNowClock();
  const clock = now.toLocaleTimeString("de-DE", { hour12: false });

  return (
    <header className="flex items-center justify-between gap-6 px-7 py-4 border-b border-ink-50/[0.06]">
      <div className="flex items-center gap-4">
        <BrandMark />
        <div className="hidden md:flex flex-col leading-tight">
          <span className="font-display text-[15px] font-medium tracking-wide text-ink-50">
            {t.app.brand}
          </span>
          <span className="label">{t.app.subtitle}</span>
        </div>
      </div>

      <div className="flex flex-1 items-center justify-center">
        <div className="flex items-center gap-3 rounded-full border border-ink-50/[0.06] bg-ink-800/60 px-4 py-1.5 backdrop-blur">
          <span className={`h-2 w-2 rounded-full ${dot.color}`} />
          <span className="font-mono text-[11px] uppercase tracking-[0.22em] text-ink-100">
            {dot.label}
          </span>
          <span className="text-ink-300">·</span>
          <span className="font-mono text-[11px] tracking-wider text-ink-200/90 truncate max-w-[44ch]">
            {source ?? t.conn.noSource}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-5 text-right">
        <Stat label={t.topbar.mode} value={frame ? deviceModeDE(frame.deviceMode) : "—"} />
        <Stat label={t.topbar.lane} value={frame ? `B${frame.lane}` : "—"} />
        <Stat label={t.topbar.state} value={frame ? stateFlagDE(frame.stateFlag) : "—"} />
        <Stat label={t.topbar.fps} value={fps.toFixed(0)} />
        <Stat label={t.topbar.frames} value={frameCount.toLocaleString("de-DE")} />
        <div className="flex flex-col items-end leading-tight">
          <span className="font-mono tnum text-[18px] tracking-wider text-ink-50">{clock}</span>
          <span className="label">{t.topbar.localTime}</span>
        </div>
        <button
          type="button"
          onClick={onMenu}
          aria-label={t.topbar.menu}
          className="grid h-10 w-10 place-items-center rounded-xl border border-ink-50/10 bg-ink-50/[0.04] text-ink-50 transition hover:bg-ink-50/10"
        >
          {/* Hamburger icon */}
          <svg width="16" height="14" viewBox="0 0 16 14" fill="none">
            <path
              d="M1 1h14M1 7h14M1 13h14"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>
    </header>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="hidden lg:flex flex-col items-end leading-tight">
      <span className="font-mono text-[13px] uppercase tracking-wider text-ink-50">{value}</span>
      <span className="label">{label}</span>
    </div>
  );
}

function BrandMark() {
  return (
    <div className="relative grid h-10 w-10 place-items-center rounded-xl bg-gradient-to-br from-signal to-status-captured shadow-[0_8px_24px_-8px_rgba(255,77,46,0.7)]">
      <span className="font-stencil text-[18px] font-bold leading-none text-ink-900">KiDo</span>
      <span className="absolute -bottom-0.5 -right-0.5 h-2 w-2 rounded-full bg-status-confirmed ring-2 ring-ink-900" />
    </div>
  );
}
