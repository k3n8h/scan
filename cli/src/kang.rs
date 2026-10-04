//! Pollard's kangaroo for a known public key in a bounded range [start, start + 2^(bits-1)).
//! Many kangaroos per thread share one field inversion per step (Montgomery's trick).
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{AffinePoint, ProjectivePoint, Scalar};
use rand::Rng;
use rayon::prelude::*;
use scan_core::fe::{self, Fe};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

const JUMPS: usize = 64;
const HERD: usize = 256; // per kind, per thread

fn xy(p: &AffinePoint) -> (Fe, Fe) {
    let e = p.to_encoded_point(false);
    let b = e.as_bytes();
    (fe::from_be(&b[1..33]), fe::from_be(&b[33..65]))
}

fn mul_g(k: u128) -> AffinePoint {
    (ProjectivePoint::GENERATOR * Scalar::from(k)).to_affine()
}

pub struct Outcome {
    pub key: Option<u128>,
    pub ops: u64,
}

/// Find k in [start, start + 2^(bits-1)) with k·G == q. `bits` ≤ 126.
pub fn solve(q: &AffinePoint, start: u128, bits: u32, jump_scale: f64, max_ops: u64, neg: bool) -> Outcome {
    assert!((2..=126).contains(&bits));
    let w: u128 = 1u128 << (bits - 1);
    let mid = start + w / 2;
    let sqrt_w = (w as f64).sqrt();
    // Jump distances: uniform in [1, 2*mean]; mean ≈ jump_scale * sqrt(W).
    let mean = (sqrt_w * jump_scale).max(1.0) as u128;
    let mut rng = rand::thread_rng();
    let dist: Vec<i128> = (0..JUMPS).map(|_| rng.gen_range(1..=2 * mean) as i128).collect();
    let jp: Vec<(Fe, Fe)> = dist.iter().map(|&d| xy(&mul_g(d as u128))).collect();
    // DP when the low `dbits` bits of x are zero; keeps the table to ~tens of thousands of entries.
    let dbits = ((sqrt_w.log2() as i32) - 12).clamp(0, 40) as u32;
    let dmask: u64 = if dbits == 0 { 0 } else { (1u64 << dbits) - 1 };
    let spread = (sqrt_w as i128).max(1);

    let table: Mutex<HashMap<u128, (i128, bool)>> = Mutex::new(HashMap::new());
    let found = AtomicBool::new(false);
    let result = Mutex::new(None::<u128>);
    let ops = AtomicU64::new(0);
    let qx = *q;
    let threads = rayon::current_num_threads();

    (0..threads).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let n = 2 * HERD;
        let (mut px, mut py) = (vec![fe::ZERO; n], vec![fe::ZERO; n]);
        let mut off = vec![0i128; n]; // signed key offset of the kangaroo (relative to `mid`)
        let tame = |i: usize| i < HERD;
        // Work relative to the range midpoint: wild base is Q - mid·G, tame base is the identity.
        let q_shift = ProjectivePoint::from(qx) - ProjectivePoint::from(mul_g(mid));
        for i in 0..n {
            let mut r = rng.gen_range(-spread..=spread);
            if r == 0 {
                r = 1;
            }
            let rg = if r >= 0 { ProjectivePoint::from(mul_g(r as u128)) } else { -ProjectivePoint::from(mul_g((-r) as u128)) };
            let p = if tame(i) { rg } else { q_shift + rg };
            let (x, y) = xy(&p.to_affine());
            px[i] = x;
            py[i] = y;
            off[i] = r;
        }
        let mut hist = vec![[0u64; 8]; n];
        let mut hpos = vec![0usize; n];
        let mut esc = vec![false; n];
        let mut dx = vec![fe::ZERO; n];
        let mut scratch: Vec<Fe> = Vec::with_capacity(n);
        let mut idx = vec![0usize; n];
        let mut local = 0u64;
        while !found.load(Ordering::Relaxed) {
            if neg {
                // Fold P and -P into one class: keep the representative with even y.
                for i in 0..n {
                    if fe::normalize(&py[i])[0] & 1 == 1 {
                        py[i] = fe::sub(&fe::ZERO, &py[i]);
                        off[i] = -off[i];
                    }
                }
            }
            for i in 0..n {
                // Escape the short cycles negation can create by taking an alternate jump.
                let bump = if esc[i] { esc[i] = false; 1 } else { 0 };
                idx[i] = ((px[i][0] as usize) + bump) & (JUMPS - 1);
                let d = fe::sub(&jp[idx[i]].0, &px[i]);
                dx[i] = if fe::normalize(&d) == fe::ZERO { fe::ONE } else { d };
            }
            fe::batch_inv(&mut dx, &mut scratch);
            for i in 0..n {
                let (jx, jy) = &jp[idx[i]];
                let lam = fe::mul(&fe::sub(jy, &py[i]), &dx[i]);
                let x3 = fe::sub(&fe::sub(&fe::sqr(&lam), &px[i]), jx);
                let y3 = fe::sub(&fe::mul(&lam, &fe::sub(&px[i], &x3)), &py[i]);
                px[i] = x3;
                py[i] = y3;
                off[i] += dist[idx[i]];
                if neg {
                    // Fruitless-cycle detection: revisiting a recent x means a short loop.
                    let xl = px[i][0];
                    if hist[i].contains(&xl) {
                        esc[i] = true;
                    }
                    hist[i][hpos[i]] = xl;
                    hpos[i] = (hpos[i] + 1) & 7;
                }
                if px[i][0] & dmask == 0 {
                    // Canonicalize before recording so trails from both signs meet.
                    let (mut oy, mut oo) = (py[i], off[i]);
                    if neg && fe::normalize(&oy)[0] & 1 == 1 {
                        oy = fe::sub(&fe::ZERO, &oy);
                        oo = -oo;
                    }
                    let _ = oy;
                    let xn = fe::normalize(&px[i]);
                    let key = (xn[0] as u128) | ((xn[1] as u128) << 64);
                    let mut t = table.lock().unwrap();
                    match t.get(&key).copied() {
                        Some((o2, was_tame)) if was_tame != tame(i) => {
                            let (ot, ow) = if tame(i) { (oo, o2) } else { (o2, oo) };
                            // tame: ot·G ; wild: (k' + ow)·G with k' = k - mid
                            for kp in [ot - ow, ow - ot] {
                                let kk = mid as i128 + kp;
                                if kk > 0 && mul_g(kk as u128) == qx {
                                    *result.lock().unwrap() = Some(kk as u128);
                                    found.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                        _ => {
                            t.insert(key, (oo, tame(i)));
                        }
                    }
                }
            }
            local += n as u64;
            if local >= 4096 {
                if ops.fetch_add(local, Ordering::Relaxed) + local >= max_ops {
                    found.store(true, Ordering::Relaxed);
                }
                local = 0;
            }
        }
        ops.fetch_add(local, Ordering::Relaxed);
    });
    let key = *result.lock().unwrap();
    Outcome { key, ops: ops.load(Ordering::Relaxed) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_random_keys_in_several_ranges() {
        for (bits, neg) in [(24u32, false), (30, false), (34, false), (24, true), (30, true), (34, true)] {
            for _ in 0..3 {
                let start = 1u128 << (bits - 1);
                let k = start + rand::thread_rng().gen_range(0..start);
                let q = mul_g(k);
                let o = solve(&q, start, bits, 8192.0, 1 << 34, neg);
                assert_eq!(o.key, Some(k), "bits {bits}");
            }
        }
    }

    #[test]
    fn gives_up_at_the_op_cap_without_a_wrong_answer() {
        let bits = 40u32;
        let start = 1u128 << (bits - 1);
        let q = mul_g(start + 12345);
        let o = solve(&q, start, bits, 8192.0, 4096, true);
        assert!(o.key.is_none() || o.key == Some(start + 12345));
    }
}
