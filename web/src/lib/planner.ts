/** Measured on a 4-core Xeon @2.1GHz (see README): kangaroo ≈ 1.6·√W ops at ~23 Mops/s; scan ≈ 15 Mkeys/s. */
export const KANGAROO_OPS_FACTOR = 1.6;
export const BASELINE_KANGAROO_MOPS = 23;
export const BASELINE_SCAN_MKEYS = 15;

export interface Estimate {
  seconds: number;
  ops: number;
}

/** Puzzle n searches 2^(n-1) keys. `scale` multiplies throughput relative to the 4-core baseline. */
export function scanEstimate(puzzle: number, scale = 1): Estimate {
  const ops = 2 ** (puzzle - 1);
  return { ops, seconds: ops / (BASELINE_SCAN_MKEYS * 1e6 * scale) };
}

/** Only valid when the puzzle's public key is known. */
export function kangarooEstimate(puzzle: number, scale = 1): Estimate {
  const ops = KANGAROO_OPS_FACTOR * 2 ** ((puzzle - 1) / 2);
  return { ops, seconds: ops / (BASELINE_KANGAROO_MOPS * 1e6 * scale) };
}

export function formatDuration(s: number): string {
  if (!isFinite(s)) return '∞';
  if (s < 1) return `${(s * 1000).toFixed(0)} ms`;
  if (s < 120) return `${s.toFixed(1)} s`;
  if (s < 7200) return `${(s / 60).toFixed(1)} min`;
  if (s < 172800) return `${(s / 3600).toFixed(1)} h`;
  const days = s / 86400;
  if (days < 365) return `${days.toFixed(0)} days`;
  const y = days / 365;
  return y < 1e4 ? `${y.toFixed(y < 10 ? 1 : 0)} years` : `${y.toExponential(1)} years`;
}
