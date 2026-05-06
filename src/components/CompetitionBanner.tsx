import { t } from "../lib/i18n";
import type { CompetitionPayload, Run } from "../lib/cloudTypes";

interface Props {
  snapshot: CompetitionPayload | null;
  onClose: () => void;
}

export function CompetitionBanner({ snapshot, onClose }: Props) {
  if (!snapshot) return null;
  const comp = snapshot.competition;
  const currentRun = comp.current_run_id
    ? snapshot.runs.find((r) => r.id === comp.current_run_id) ?? null
    : snapshot.runs.find((r) => r.status === "active") ?? null;
  const team = currentRun
    ? snapshot.teams.find((tm) => tm.id === currentRun.team_id) ?? null
    : null;
  const runner =
    currentRun && currentRun.runner_id
      ? snapshot.runners.find((r) => r.id === currentRun.runner_id) ?? null
      : null;

  return (
    <div className="mx-7 mt-4 rounded-2xl border border-ink-50/[0.05] bg-ink-700/40 px-5 py-4">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <span className="label">{t.banner.activeCompetition}</span>
          <h3 className="mt-1 font-display text-[18px] font-medium tracking-tight text-ink-50 truncate">
            {comp.name}
          </h3>
          <p className="mt-0.5 font-mono text-[11px] text-ink-200/70">
            {formatDate(comp.date)}
            {comp.location ? ` · ${comp.location}` : ""}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {comp.sync_mode === "live" ? (
            <span className="rounded px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.2em] bg-signal/15 text-signal-glow ring-1 ring-signal/30">
              {t.cloud.liveBadge}
            </span>
          ) : (
            <span className="rounded px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.2em] bg-status-unknown/15 text-status-unknown ring-1 ring-status-unknown/30">
              {t.cloud.offlineBadge}
            </span>
          )}
          <button
            type="button"
            onClick={onClose}
            aria-label={t.banner.closeSnapshot}
            className="grid h-7 w-7 place-items-center rounded-md border border-ink-50/10 bg-ink-50/[0.03] font-mono text-[11px] text-ink-200 hover:bg-ink-50/10"
          >
            ✕
          </button>
        </div>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-3 md:grid-cols-3">
        <Tile label={t.banner.currentRun}>
          {currentRun ? <RunSummary run={currentRun} /> : <Empty />}
        </Tile>
        <Tile label="Team">
          {team ? (
            <p className="font-display text-[14px] text-ink-50 truncate">
              <span className="font-mono text-[11px] tracking-wider text-ink-200/70 mr-2">
                #{team.start_number}
              </span>
              {team.name}
            </p>
          ) : (
            <Empty />
          )}
        </Tile>
        <Tile label={t.banner.runner}>
          {runner ? (
            <p className="font-display text-[14px] text-ink-50 truncate">
              {runner.first_name} {runner.last_name}
            </p>
          ) : (
            <Empty />
          )}
        </Tile>
      </div>

      <p className="mt-3 font-mono text-[10px] uppercase tracking-[0.2em] text-ink-200/60">
        {comp.sync_mode === "live" ? t.banner.syncLive : t.banner.syncOffline}
      </p>
    </div>
  );
}

function Tile({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-xl border border-ink-50/[0.05] bg-ink-50/[0.02] px-3 py-2">
      <span className="label">{label}</span>
      <div className="mt-1">{children}</div>
    </div>
  );
}

function Empty() {
  return (
    <p className="font-mono text-[12px] text-ink-200/60">{t.banner.none}</p>
  );
}

function RunSummary({ run }: { run: Run }) {
  return (
    <p className="font-display text-[14px] text-ink-50">
      {t.banner.runOf} {run.run_number}
      {run.lane != null && (
        <span className="ml-2 font-mono text-[11px] tracking-wider text-ink-200/70">
          · {t.banner.lane} {run.lane}
        </span>
      )}
      <span className="ml-2 font-mono text-[10px] uppercase tracking-[0.2em] text-ink-200/60">
        {statusLabel(run.status)}
      </span>
    </p>
  );
}

function statusLabel(s: string): string {
  switch (s) {
    case "pending":
      return "Wartend";
    case "active":
      return "Aktiv";
    case "completed":
      return "Beendet";
    case "cancelled":
      return "Abgebrochen";
    default:
      return s;
  }
}

function formatDate(iso: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  if (!m) return iso;
  return `${m[3]}.${m[2]}.${m[1]}`;
}
