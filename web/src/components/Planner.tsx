import { useState } from 'react';
import { formatDuration, kangarooEstimate, scanEstimate } from '../lib/planner';
import { PUZZLES } from '../lib/puzzles';

export function Planner() {
  const unsolved = PUZZLES.filter((p) => !p.solved);
  const [puzzle, setPuzzle] = useState(unsolved[0]?.id ?? 71);
  const [scale, setScale] = useState(1);
  const scan = scanEstimate(puzzle, scale);
  const kang = kangarooEstimate(puzzle, scale);
  return (
    <section className="card">
      <h2>Feasibility planner</h2>
      <p className="dim">
        Estimates from measurements on a 4-core CPU (see README). Scanning works from an address alone; kangaroo needs the
        puzzle's <em>public key</em>, which only some puzzles have revealed. Nothing here searches for keys.
      </p>
      <div className="row">
        <label>Puzzle <select value={puzzle} onChange={(e) => setPuzzle(Number(e.target.value))}>
          {PUZZLES.map((p) => <option key={p.id} value={p.id}>{p.id}{p.solved ? ' (solved)' : ''}</option>)}
        </select></label>
        <label>Throughput ×<input type="number" min={0.01} step={0.5} value={scale} onChange={(e) => setScale(Math.max(0.01, Number(e.target.value) || 1))} style={{ width: 100 }} /></label>
      </div>
      <dl>
        <dt>Address-only scan</dt><dd>{formatDuration(scan.seconds)} <span className="dim">({scan.ops.toExponential(2)} keys)</span></dd>
        <dt>Kangaroo (public key known)</dt><dd>{formatDuration(kang.seconds)} <span className="dim">({kang.ops.toExponential(2)} ops)</span></dd>
      </dl>
    </section>
  );
}
