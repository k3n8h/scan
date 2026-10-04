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

### Methods, measured

**1. Sequential scan (`scan solve`)** works from the address alone. `core/src/fast.rs`: centered batches (C ± i·G) share
one field inversion (original k256 "batch" normalize inverted per key: 4,763 ns/key), then an 8-lane AVX2 RIPEMD-160
(`rmd8.rs`, verified against the reference on 1,600 random inputs). 0.66 → 9.2 → **~15 Mkeys/s** on 4 cores.
Puzzle *n* needs up to 2^(n-1) keys, so puzzle 71 is ~2.5×10^6 core-years at this rate.

**2. Pollard's kangaroo (`scan kangaroo <puzzle> --pubkey <hex>`)** works when the puzzle's **public key** is known
(an address is only a hash, so this needs the key to have been revealed on-chain). ~√W operations instead of W.
`cli/src/kang.rs`: 2,048 kangaroos in lock-step sharing one inversion, distinguished points, and three measured improvements:

| change | effect (ops/√W, mean) |
|---|---|
| naive jump size (0.25·√W) | ~2,900 |
| jump size tuned for the herd (256·√W) | ~3.2 |
| + equivalence-class walk (P and −P folded), with 8-step cycle detection | ~1.6 (1.5–1.8× fewer ops, ~1.4× less wall time) |
| + jump scale fitted to range size: 8192·2^((bits−40)/4) | ~1.0–1.6 at 40–52 bits |

The first negation attempt was 20–100× *worse* (fruitless cycles, and the folded walk needs much larger jumps because
it is a reflecting random walk); detection + re-tuning fixed it. The best scale drifts with range size (sweeps at 40, 44,
48, 50, 52 bits, 6–15 runs each), hence the fitted formula; it was tuned for 4 threads / 2,048 kangaroos, other core counts
are untested. Real runs through the CLI: puzzle 50 in 1.7–3.4 s, puzzle 55 in ~8.6 s (was 25 s plain). The key must hash to
the puzzle's address, otherwise the command refuses.

Estimated time at ~1.6·√W ops, ~23 Mops/s, this 4-core box (extrapolated from the measurements, not run):

| puzzle | 66 | 71 | 75 | 80 | 90 | 100 | 110 | 120 | 135 | 160 |
|---|---|---|---|---|---|---|---|---|---|---|
| time | 0.1 h | 0.7 h | 2.7 h | 15.0 h | 20 d | 1.8 y | 56 y | 1.8e+03 y | 3.3e+05 y | 1.9e+09 y |

Caveat: those numbers apply **only to puzzles whose public key is known**. I have not verified which unsolved puzzles
have an exposed key (the dataset has addresses only), so this tool cannot be pointed at one until you supply a key.
Without a public key only method 1 applies. Kangaroo is limited to puzzles ≤ 126 (u128 distances).

**3. Statistical look at the 82 solved keys** (position in range, bit balance, residues mod 2–16, serial correlation,
k_n vs k_(n-1)): consistent with uniform random keys. One mod-8 skew appeared in the n≈50–70 half but not in n=20–49, and
the worst of four modulus tests is that extreme ~13% of the time under pure chance, so it is not evidence of structure.
No bias to exploit was found; this does not prove none exists.

**Not tried:** GPU kernels, BSGS (memory-bound; kangaroo already beats it here), multi-machine DP sharing.
Reversing hash160 or finding structure in the puzzle keys: no known approach, nothing tested.
