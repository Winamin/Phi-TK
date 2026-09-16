#!/usr/bin/env node
/**
 * Regenerates the MD3 spring easings in `src/assets/main.css`.
 *
 * ── Why this exists ─────────────────────────────────────────────────────────
 * Material 3's expressive motion is spring-based, and the spec publishes each
 * spring as a (damping ratio ζ, stiffness k) pair. Turning that pair into a CSS
 * `linear()` easing is mechanical — but it is also the only part of the motion
 * system that has to be *re-derived* whenever the upstream numbers change, so it
 * lives here instead of being hand-copied into the stylesheet.
 *
 * The values currently committed were derived from the physics below rather than
 * read off m3.material.io, because this environment has no outbound network
 * access. If you compare them against the spec and they differ, edit the SPRINGS
 * table and re-run:
 *
 *     node scripts/gen-springs.mjs
 *
 * ── The physics ────────────────────────────────────────────────────────────
 * A unit displacement released into a damped spring, where ω₀ = √k (unit mass)
 * and ω_d = ω₀√(1-ζ²):
 *
 *     x(t) = e^(-ζω₀t) · (cos ω_d t + (ζω₀ / ω_d) · sin ω_d t)
 *     p(t) = 1 - x(t)
 *
 * Peak overshoot is `e^(-πζ/√(1-ζ²))`, which is why ζ = 1.0 is flat at exactly 1
 * (critically damped — no overshoot, correct for colour and opacity) while ζ < 1
 * passes it. Material 3 relies on that split: spatial motion may overshoot,
 * effects motion must not.
 *
 * The curve is sampled over its settle time and emitted as `linear()`.
 */

// ── Spring definitions ──────────────────────────────────────────────────────
// `damping` is the ratio ζ; `stiffness` is k. `samples` controls curve fidelity
// versus stylesheet size — 40 is visually smooth at these durations.

const SPRINGS = [
  // Spatial: position, size, shape. May overshoot.
  { token: 'spatial-fast', damping: 0.9, stiffness: 1400, samples: 40 },
  { token: 'spatial-default', damping: 0.9, stiffness: 700, samples: 48 },
  { token: 'spatial-slow', damping: 0.9, stiffness: 300, samples: 56 },

  // Effects: colour, opacity, elevation. Critically damped — never overshoots.
  { token: 'effects-fast', damping: 1.0, stiffness: 3800, samples: 32 },
  { token: 'effects-default', damping: 1.0, stiffness: 1600, samples: 40 },
  { token: 'effects-slow', damping: 1.0, stiffness: 800, samples: 48 },

  // Expressive accent used for press feedback. ζ = 0.7 gives ~4.6% overshoot,
  // which is the smallest amount that reads as a deliberate bounce.
  { token: 'bouncy', damping: 0.7, stiffness: 600, samples: 56 },
];

/** Tolerance for "the spring has settled", as a fraction of the travel. */
const SETTLE_TOLERANCE = 0.001;
/** Give up looking for a settle point after this long (a spring always settles). */
const MAX_SECONDS = 3;

function makeSpring(stiffness, damping) {
  const w0 = Math.sqrt(stiffness);
  const wd = w0 * Math.sqrt(Math.max(1e-9, 1 - damping * damping));
  return (t) => {
    const x =
      damping >= 1
        ? Math.exp(-w0 * t) * (1 + w0 * t)
        : Math.exp(-damping * w0 * t) * (Math.cos(wd * t) + ((damping * w0) / wd) * Math.sin(wd * t));
    return 1 - x;
  };
}

/**
 * Settle time is the LAST moment the curve sits outside the tolerance band, not
 * the first crossing. An underdamped spring passes through 1.0 on its way up, so
 * stopping at the first crossing would truncate the entire overshoot and emit a
 * curve that never bounces.
 */
function settleTime(p, tolerance) {
  let last = 0;
  for (let t = 0; t <= MAX_SECONDS; t += 0.0005) {
    if (Math.abs(p(t) - 1) >= tolerance) last = t;
  }
  return last;
}

function peak(p, T) {
  let max = 0;
  for (let t = 0; t <= T; t += 0.0005) max = Math.max(max, p(t));
  return max;
}

/** Trims trailing zeros so the emitted CSS stays compact. */
function fmt(v, isFirst, isLast) {
  if (isFirst) return '0';
  if (isLast) return '1';
  return v.toFixed(4).replace(/0+$/, '').replace(/\.$/, '');
}

function toLinear(p, T, samples) {
  const parts = [];
  for (let i = 0; i <= samples; i++) {
    parts.push(fmt(p((i / samples) * T), i === 0, i === samples));
  }
  return `linear(${parts.join(', ')})`;
}

const rows = [];
for (const { token, damping, stiffness, samples } of SPRINGS) {
  const p = makeSpring(stiffness, damping);
  const T = settleTime(p, SETTLE_TOLERANCE);
  const overshoot = (peak(p, T) - 1) * 100;
  rows.push({
    token,
    css: toLinear(p, T, samples),
    ms: Math.round(T * 1000),
    overshoot,
    damping,
    stiffness,
  });
}

const report = rows
  .map(
    (r) =>
      `  ${r.token.padEnd(16)} ζ=${r.damping}  k=${String(r.stiffness).padEnd(5)}` +
      `  settle=${String(r.ms).padStart(4)}ms  overshoot=${r.overshoot >= 0 ? '+' : ''}${r.overshoot.toFixed(2)}%`,
  )
  .join('\n');

console.log('Derived springs:\n' + report + '\n');
console.log('Durations (--app-motion-duration-*):');
for (const r of rows) console.log(`  --app-motion-duration-${r.token}: ${r.ms}ms;`);
console.log('\nEasings (@supports block):');
for (const r of rows) console.log(`  --app-motion-spring-${r.token}: ${r.css};`);
console.log(
  '\nA negative overshoot above means the curve approaches 1 from below and never ' +
    'exceeds it — correct for the effects springs, which must stay critically damped.',
);

// Sanity assertions: the two properties the whole system depends on.
const spatialOk = rows.filter((r) => r.token.startsWith('spatial') || r.token === 'bouncy').every((r) => r.overshoot > 0);
const effectsOk = rows.filter((r) => r.token.startsWith('effects')).every((r) => r.overshoot <= 0);
if (!spatialOk || !effectsOk) {
  console.error('\nFAIL: spatial/bouncy must overshoot and effects must not.');
  process.exit(1);
}
console.log('\nOK: spatial & bouncy overshoot; effects are critically damped.');
