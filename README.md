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
