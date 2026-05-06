import { useMemo } from "react";
import { t } from "../lib/i18n";
import type { CompetitionPayload, Run } from "../lib/cloudTypes";
import { pickRunForLane } from "../lib/useCloud";

interface Props {
  snapshot: CompetitionPayload;
  laneOverrides: Record<number, string>;
  onSetOverride: (lane: number, runId: string | null) => void;
}

const LANE_COUNT = 4;

/**
 * Lane-assignment panel for the desktop operator. Auto-binding only fits
 * SingleLane / TwoLaneParallel; for Relay and Individual modes the operator
 * picks which run is on each physical lane.
 */
export function LaneAssignmentPanel({
  snapshot,
  laneOverrides,
  onSetOverride,
}: Props) {
  const eligibleRuns = useMemo(
    () =>
      snapshot.runs
        .filter((r) => r.status === "active" || r.status === "pending")
        .sort((a, b) => a.run_number - b.run_number),
    [snapshot.runs],
  );

  // Hide for the simple modes — auto-binding by lane works there.
  const mode = snapshot.competition.mode;
  if (mode === "single_lane" || mode === "two_lane_parallel") {
    return null;
  }
  if (eligibleRuns.length === 0) return null;

  return (
    <section className="mx-7 mt-3 rounded-2xl border border-ink-50/[0.05] bg-ink-700/40 px-5 py-4">
      <header className="mb-3">
        <span className="label">{t.laneAssign.title}</span>
        <p className="mt-0.5 font-mono text-[11px] text-ink-200/70">
          {t.laneAssign.subtitle}
        </p>
      </header>

      <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
        {Array.from({ length: LANE_COUNT }, (_, i) => i + 1).map((lane) => {
          const overrideId = laneOverrides[lane] ?? null;
          const auto = pickRunForLane(snapshot, lane, {});
          const selected = overrideId
            ? eligibleRuns.find((r) => r.id === overrideId) ?? null
            : auto;
          return (
            <div
              key={lane}
              className="flex items-center gap-3 rounded-xl border border-ink-50/[0.05] bg-ink-50/[0.02] px-3 py-2"
            >
              <span className="w-12 shrink-0 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-200/70">
                {t.laneAssign.laneShort} {lane}
              </span>
              <select
                value={overrideId ?? ""}
                onChange={(e) =>
                  onSetOverride(lane, e.target.value || null)
                }
                className="input flex-1 text-[12px]"
              >
                <option value="">{t.laneAssign.auto}</option>
                {eligibleRuns.map((r) => (
                  <option key={r.id} value={r.id}>
                    {labelForRun(snapshot, r)}
                  </option>
                ))}
              </select>
              <RunHint snapshot={snapshot} run={selected} isAuto={!overrideId} />
            </div>
          );
        })}
      </div>
    </section>
  );
}

function RunHint({
  snapshot,
  run,
  isAuto,
}: {
  snapshot: CompetitionPayload;
  run: Run | null;
  isAuto: boolean;
}) {
  if (!run) {
    return (
      <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-ink-200/50">
        {t.laneAssign.none}
      </span>
    );
  }
  const team = snapshot.teams.find((tm) => tm.id === run.team_id);
  const runner = run.runner_id
    ? snapshot.runners.find((r) => r.id === run.runner_id)
    : null;
  return (
    <span className="font-mono text-[10px] tracking-wider text-ink-200/70 truncate max-w-[40%]">
      {isAuto ? "auto · " : ""}
      {team?.name ?? "?"}
      {runner ? ` · ${runner.first_name} ${runner.last_name}` : ""}
    </span>
  );
}

function labelForRun(snapshot: CompetitionPayload, r: Run): string {
  const team = snapshot.teams.find((tm) => tm.id === r.team_id);
  const runner = r.runner_id
    ? snapshot.runners.find((x) => x.id === r.runner_id)
    : null;
  const parts = [
    `${t.laneAssign.runLabel} ${r.run_number}`,
    team ? team.name : "?",
  ];
  if (runner) parts.push(`${runner.first_name} ${runner.last_name}`);
  if (r.lane != null) parts.push(`${t.laneAssign.laneShort} ${r.lane}`);
  parts.push(r.status === "active" ? t.laneAssign.statusActive : t.laneAssign.statusPending);
  return parts.join(" · ");
}
