# scan — Bitcoin Puzzle Workbench

One app replacing the original 55-component AI Studio project.

| Part | Language | Why |
|---|---|---|
| `web/` UI + key math (`@noble/curves`, audited) | TypeScript / React | runs locally, no server, no API keys |
| `solvers/mnemonic_z3_solver.py` | Python | Z3 constraint solving has no good JS equivalent |
| `data/puzzles.json` | JSON | single source of truth (160 puzzles) |

```
cd web && npm install && npm run dev     # develop
npm test                                  # 7 tests, incl. verifying every solved puzzle key -> address
npm run build                             # typecheck + production bundle (~96 kB gzip)
```

Features: puzzle catalogue with in-browser verification of published solutions, key/WIF/address tools
(P2PKH, P2WPKH), CSPRNG key generation, brainwallet weakness demo.

## Audit of the original zip
- **Incomplete source**: 21 modules (`hooks/usePuzzleScan`, `utils/cryptoUtils`, …) imported by components exist only as minified `dist` output; the original could not be rebuilt.
- **Two duplicate puzzle datasets** (`constants/puzzles.ts`, `src/utils/puzzlesData.ts`) → merged into `data/puzzles.json`; all 82 published solutions verified.
- `dist/` and a 1.4 MB build were committed; `find_1PWo3_fast.cjs` was empty; `2/2.py` brute-forces 12-word permutations (infeasible: > 10^38 combinations) — dropped.
- `Math.random` used in 28 places, some near key generation → replaced by CSPRNG.
- Unused Gemini SDK/API key plumbing and a server dependency removed.
- **Not rebuilt**: high-throughput key-search engines (range scanner, BSGS/Kangaroo, vanity). Deliberately left out.

## Native range scanner (`cli/`, Rust)

Scans slices of a puzzle's key range for its published address. Targets come only from `data/puzzles.json` (`scan solve <puzzle#>`).

```
cargo build --release
./target/release/scan bench
./target/release/scan solve 32 --offset 0x3834df6e --keys 4000000   # known-answer check
./target/release/scan solve 71 --random --keys 1000000000          # a slice of an unsolved range
```

Method (`core/src/fast.rs`): the range is walked in centered batches of 1,025 keys (C ± i·G for i ≤ 512, using a
precomputed table), so a whole batch shares **one** field inversion (Montgomery's trick) on a hand-written 4×64-bit secp256k1
field. Profiling showed the original k256 `batch_normalize` actually inverted per key (4,763 ns/key, >90% of runtime);
this took the scanner from 0.66 to ~9.2 Mkeys/s on 4 cores (13×). The reference implementation (`range.rs`) is kept for
cross-checks and tiny start keys; tests verify the fast path finds the key at every batch boundary.
Remaining cost is hash160 (~300 ns/key, SHA-NI + software RIPEMD-160).

**Feasibility, honestly:** puzzle *n* has 2^(n-1) keys. At ~9 Mkeys/s, puzzle 71 (2^70) takes ~4 million years on this
machine, and each extra puzzle number doubles that. I found no mathematical shortcut for the unsolved puzzles (keys behave as
uniformly random inside their range) and did no web research, so I'm not claiming one. Even a GPU fleet only makes the
lowest unsolved puzzles conceivable. Not implemented: BSGS/Kangaroo (need puzzle public keys, absent from the dataset),
GPU kernels, vanity search.
