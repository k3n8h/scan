import { describe, it, expect } from 'vitest';
import { scanEstimate, kangarooEstimate, formatDuration } from './planner';

describe('planner', () => {
  it('scan time doubles per puzzle, kangaroo grows by sqrt(2)', () => {
    expect(scanEstimate(41).seconds / scanEstimate(40).seconds).toBeCloseTo(2, 9);
    expect(kangarooEstimate(41).seconds / kangarooEstimate(40).seconds).toBeCloseTo(Math.SQRT2, 9);
  });
  it('more throughput means proportionally less time', () => {
    expect(scanEstimate(60, 10).seconds).toBeCloseTo(scanEstimate(60, 1).seconds / 10, 6);
  });
  it('matches the measured puzzle-55 run within an order of magnitude', () => {
    const s = kangarooEstimate(55).seconds; // measured ~8.6 s
    expect(s).toBeGreaterThan(1);
    expect(s).toBeLessThan(86);
  });
  it('formats durations', () => {
    expect(formatDuration(0.5)).toBe('500 ms');
    expect(formatDuration(3600)).toBe('60.0 min');
    expect(formatDuration(86400 * 400)).toMatch(/years/);
  });
});
