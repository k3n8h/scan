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

Method: one point addition per key (P += G) with batched affine conversion, rayon-parallel over 2^20-key chunks.
Measured ~0.66 Mkeys/s on 4 cores. **Feasibility:** puzzle *n* has 2^(n-1) keys, so on this engine puzzle 71 (2^70) would take
~5×10^7 years. Only GPU-class throughput and/or public-key (Kangaroo) methods make high puzzles plausible, and even those
are measured in huge GPU-years; low unsolved puzzles are not a realistic target for CPU. Not implemented: BSGS/Kangaroo
(need the puzzle public key, not in the dataset), GPU kernels, vanity search.
