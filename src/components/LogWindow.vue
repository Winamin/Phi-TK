<i18n>
en:
  log:
    title: Output Log
    search: Search logs…
    all: All
    info: Info
    warning: Warnings
    error: Errors
    copy: Copy
    copied: Copied
    copy-failed: Copy failed
    copy-image: Copy screenshot
    image-copied: Screenshot copied
    image-copy-failed: Could not copy screenshot
    copy-log: Copy log
    save: Save
    saved: Saved
    save-failed: Save failed
    save-image: Save image
    image-saved: Image saved
    image-save-failed: Could not save image
    save-log: Save log
    close: Close
    simple: Simplified
    detail: Detailed
    lines: '{shown} / {total} lines'
    selection-hint: '{count} lines selected · right-click to clear'
    empty: No output
    no-match: No matching lines
    clear-search: Clear search
    view-detail: View in detailed log
    stage:
      progress: Render
      summary: Summary
      env: Setup
      plain: Log
    dash:
      hardware: Hardware acceleration type
      resolution: Resolution
      source: 'Source {value}'
      aspect: 'Aspect ratio {value}'
      config: Render configuration
      codec: Codec
      quality: Quality
      constant-qp: Constant QP {qp}
      quality-hint: 'Vulkan Video is constant-QP: CRF is passed through as QP, and CBR is converted from the target bitrate.'
      preset: Preset
      builtin: Built-in
      frames: Total frames
      ffmpeg: FFmpeg
      progress: Render progress
      per-thousand: Per 1,000 frames
      average: Average
      fastest: Fastest
      slowest: Slowest
      stages: Time breakdown
      stages-sub: Where the time went
      end-to-end: End-to-end
      render-time: Render
      unknown: Unknown
      no-progress: No render progress data
      no-stages: No timing data
      not-recorded: Not recorded
      issues: '{errors} errors · {warnings} warnings'
      chart-hint: '{percent}% · {time} per 1k frames'
      stage:
        load: Chart & audio load
        mixing: Audio mixing
        audio: Audio encode
        probe: Encoder probe
        readback: Readback setup
        scene: Scene load
        ffmpeg: FFmpeg startup
        finalize: Finalize
        update: Update
        render: Scene render
        yuv: YUV convert
        nv12: NV12 copy

zh-CN:
  log:
    title: 输出日志
    search: 搜索日志…
    all: 全部
    info: 信息
    warning: 警告
    error: 错误
    copy: 复制
    copied: 已复制
    copy-failed: 复制失败
    copy-image: 复制截图
    image-copied: 截图已复制
    image-copy-failed: 截图复制失败
    copy-log: 复制日志
    save: 保存
    saved: 已保存
    save-failed: 保存失败
    save-image: 保存图片
    image-saved: 图片已保存
    image-save-failed: 图片保存失败
    save-log: 保存日志
    close: 关闭
    simple: 简化
    detail: 详细
    lines: '{shown} / {total} 行'
    selection-hint: '已选择 {count} 行 · 右键取消选择'
    empty: 暂无输出
    no-match: 没有匹配的日志行
    clear-search: 清除搜索
    view-detail: 在详细日志中查看
    stage:
      progress: 渲染
      summary: 汇总
      env: 环境
      plain: 日志
    dash:
      hardware: 硬件加速类型
      resolution: 分辨率
      source: '源 {value}'
      aspect: '比例 {value}'
      config: 渲染配置
      codec: 编码格式
      quality: 画质
      constant-qp: 恒定 QP {qp}
      quality-hint: 'Vulkan Video 使用恒定 QP：CRF 数值直接作为 QP，CBR 由目标码率换算。'
      preset: 速度预设
      builtin: 内置
      frames: 总帧数
      ffmpeg: FFmpeg
      progress: 渲染进度
      per-thousand: 每千帧
      average: 平均
      fastest: 最快
      slowest: 最慢
      stages: 耗时分布
      stages-sub: 各阶段花费的时间
      end-to-end: 端到端
      render-time: 渲染
      unknown: 未知
      no-progress: 没有渲染进度数据
      no-stages: 没有耗时数据
      not-recorded: 未记录
      issues: '{errors} 个错误 · {warnings} 个警告'
      chart-hint: '{percent}% · 每千帧 {time}'
      stage:
        load: 谱面与音频加载
        mixing: 音频混音
        audio: 音频编码
        probe: 编码器探测
        readback: 回读设置
        scene: 场景加载
        ffmpeg: FFmpeg 启动
        finalize: 收尾
        update: 更新
        render: 场景渲染
        yuv: YUV 转换
        nv12: NV12 拷贝

</i18n>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { save } from '@tauri-apps/plugin-dialog';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { toBlob } from 'html-to-image';
import { toast } from '@/common';
import type { Task } from '@/model';

const props = defineProps<{
  open: boolean;
  task: Task | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t, te } = useI18n();

type LogLevel = 'info' | 'warn' | 'error';
type LogStage = 'progress' | 'summary' | 'env' | 'plain';
type LevelFilter = 'all' | LogLevel;
type ViewMode = 'simple' | 'detail';

interface LogLine {
  no: number;
  stream: 'stdout' | 'stderr';
  level: LogLevel;
  stage: LogStage;
  text: string;
}

interface ProgressSample {
  percent: number;
  frames: number;
  total: number;
  /** Seconds spent on this progress interval. */
  interval: number;
  /** Normalised interval cost: seconds per 1,000 frames. */
  perThousand: number;
}

interface StageTime {
  /** Known stage id, or null for an unrecognised log label. */
  key: string | null;
  label: string;
  seconds: number;
  group: 'setup' | 'render';
}

type HardwareKind = 'hardware' | 'software' | 'disabled' | null;

interface LogInsights {
  encoder: string | null;
  hardwareKind: HardwareKind;
  resolution: string | null;
  sourceResolution: string | null;
  aspect: string | null;
  codec: string | null;
  quality: string | null;
  preset: string | null;
  presetBuiltin: boolean;
  ffmpegPath: string | null;
  totalFrames: number | null;
  averageFps: number | null;
  endToEnd: number | null;
  renderTime: number | null;
  loopTime: number | null;
  progress: ProgressSample[];
  stages: StageTime[];
}

const ANSI = /\x1b\[[0-9;]*m/g;

function classifyLevel(text: string): LogLevel {
  // The old ffmpeg message is actionable but not fatal, so it stays a warning even
  // though it contains "Failed".
  if (/^Failed to find global ffmpeg/.test(text)) return 'warn';
  if (/\b(error|failed|failure|exception|panic|fatal|abnormally)\b/i.test(text) || /错误|失败|异常/.test(text)) {
    return 'error';
  }
  if (/\bwarn(?:ing)?\b/i.test(text) || /警告/.test(text)) return 'warn';
  return 'info';
}

function classifyStage(text: string): LogStage {
  if (text.startsWith('Rendering:')) return 'progress';
  if (/^(Render Time|Render loop Time|prpr stage times|Writer thread|Render thread|End-to-End)/.test(text)) {
    return 'summary';
  }
  if (/^(Failed to find global ffmpeg|Failed to initialize|No Vulkan)/.test(text)) return 'env';
  return 'plain';
}

function parseLog(raw: string, failed: boolean): LogLine[] {
  const lines: LogLine[] = [];
  let stream: 'stdout' | 'stderr' = failed ? 'stderr' : 'stdout';
  let no = 0;
  for (const original of raw.replace(ANSI, '').split(/\r?\n/)) {
    const text = original.replace(/\s+$/, '');
    if (text === '[STDOUT]') {
      stream = 'stdout';
      continue;
    }
    if (text === '[STDERR]') {
      stream = 'stderr';
      continue;
    }
    if (!text) continue;
    no += 1;
    lines.push({
      no,
      stream,
      level: classifyLevel(text),
      stage: classifyStage(text),
      text,
    });
  }
  // A failed task's message is not necessarily worded with "error"; make sure the
  // first line still reads as the failure instead of an ordinary info line.
  if (failed && lines.length && !lines.some((line) => line.level === 'error')) {
    lines[0].level = 'error';
  }
  return lines;
}

const level = ref<LevelFilter>('all');
const mode = ref<ViewMode>('simple');
const query = ref('');
const exporting = ref(false);
const root = ref<HTMLElement>();
const body = ref<HTMLElement>();

interface LogSelection {
  /** First clicked row, kept while the range grows or shrinks. */
  anchor: number;
  /** Most recently clicked row. */
  end: number;
}
const logSelection = ref<LogSelection | null>(null);

const rawText = computed(() => {
  const task = props.task;
  if (!task) return '';
  if (task.status.type === 'done') return task.status.output;
  if (task.status.type === 'failed') return task.status.error;
  return '';
});

const failed = computed(() => props.task?.status.type === 'failed');
const allLines = computed(() => parseLog(rawText.value, failed.value));

const counts = computed(() => {
  const result: Record<LogLevel, number> = { info: 0, warn: 0, error: 0 };
  for (const line of allLines.value) result[line.level] += 1;
  return result;
});

const filteredLines = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  return allLines.value.filter(
    (line) =>
      (level.value === 'all' || line.level === level.value) &&
      (!keyword || line.text.toLowerCase().includes(keyword)),
  );
});

const selectedLines = computed(() => {
  const selection = logSelection.value;
  if (!selection) return null;
  const start = Math.min(selection.anchor, selection.end);
  const end = Math.max(selection.anchor, selection.end);
  return filteredLines.value.slice(start, end + 1);
});
const selectedLineCount = computed(() => selectedLines.value?.length ?? 0);

const fullLogText = computed(() => allLines.value.map((line) => line.text).join('\n'));

const lineCountLabel = computed(() =>
  t('log.lines', {
    shown: logSelection.value ? selectedLineCount.value : filteredLines.value.length,
    total: allLines.value.length,
  }),
);

const levelStats = computed(() => [
  { value: 'all' as const, label: t('log.all'), count: allLines.value.length },
  { value: 'info' as const, label: t('log.info'), count: counts.value.info },
  { value: 'warn' as const, label: t('log.warning'), count: counts.value.warn },
  { value: 'error' as const, label: t('log.error'), count: counts.value.error },
]);

const issueLine = computed(
  () => allLines.value.find((line) => line.level === 'error') ??
    allLines.value.find((line) => line.level === 'warn') ??
    null,
);

/**
 * Pull the human-facing essentials out of the render process' plain-text log.
 * These messages are emitted by the Rust side and are not localised, so matching
 * on stable English prefixes is intentional — the log viewer translates what it
 * extracts instead of trying to translate the raw log.
 */
const STAGE_KEYS: Record<string, string> = {
  'chart + audio load': 'load',
  'mixing': 'mixing',
  'audio encode + resample': 'audio',
  'encoder probe': 'probe',
  'offscreen/readback setup': 'readback',
  'player + scene load': 'scene',
  'ffmpeg spawn + threads': 'ffmpeg',
  'drain + finalize': 'finalize',
  'update': 'update',
  'scene render': 'render',
  'YUV convert': 'yuv',
  'NV12 copy/submit': 'nv12',
};

function firstMatch(lines: LogLine[], pattern: RegExp): RegExpMatchArray | null {
  for (const line of lines) {
    const match = line.text.match(pattern);
    if (match) return match;
  }
  return null;
}

function readSeconds(lines: LogLine[], pattern: RegExp): number | null {
  const match = firstMatch(lines, pattern);
  return match ? Number(match[1]) : null;
}

function normaliseCodec(codec: string | null): string | null {
  if (!codec) return null;
  const lower = codec.toLowerCase();
  if (lower === 'h264' || lower === 'h.264') return 'H.264';
  if (lower === 'hevc' || lower === 'h265' || lower === 'h.265') return 'HEVC';
  if (lower === 'av1') return 'AV1';
  return codec.toUpperCase();
}

function aspectOf(resolution: string | null): string | null {
  if (!resolution) return null;
  const [width, height] = resolution.split('×').map(Number);
  if (!width || !height) return null;
  const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
  const divisor = gcd(width, height);
  return `${width / divisor}:${height / divisor}`;
}

function buildInsights(lines: LogLine[]): LogInsights {
  const vulkanEnabled = lines.some((line) => /Vulkan Video H\.264:\s*enabled/i.test(line.text));
  const hardwareOff = lines.some((line) => /hardware acceleration is off/i.test(line.text));
  const cpuPreference = lines.some((line) => /cpu preference/i.test(line.text));
  const selected = firstMatch(lines, /Selected encoder:\s*(.+)$/)?.[1]?.trim() ?? null;

  let encoder = selected;
  if (vulkanEnabled) encoder = 'Vulkan Video';
  if (!encoder) {
    const enabled = firstMatch(lines, /^\s*(h264|hevc|av1)_(?:nvenc|qsv|amf|vulkan):\s*true\s*$/i);
    if (enabled) encoder = enabled[0].replace(/:\s*true\s*$/i, '').trim();
  }

  let hardwareKind: HardwareKind = null;
  if (hardwareOff || cpuPreference) {
    hardwareKind = 'disabled';
  } else if (encoder) {
    hardwareKind = vulkanEnabled || /(nvenc|qsv|amf|vulkan|d3d12|dx12|cuvid)/i.test(encoder)
      ? 'hardware'
      : 'software';
  }

  const scaled = firstMatch(lines, /(\d{2,5})x(\d{2,5})\s*->\s*(\d{2,5})x(\d{2,5})/);
  let sourceResolution: string | null = null;
  let resolution: string | null = null;
  if (scaled) {
    sourceResolution = `${Number(scaled[1])}×${Number(scaled[2])}`;
    resolution = `${Number(scaled[3])}×${Number(scaled[4])}`;
  } else {
    const size = firstMatch(lines, /Vulkan Video H\.264:.*?\([^)]*?(\d{2,5})x(\d{2,5})/i) ??
      firstMatch(lines, /initialized\s+(?:h\.?264|hevc|av1)\s+encoder\s+(\d{2,5})x(\d{2,5})/i);
    if (size) resolution = `${Number(size[1])}×${Number(size[2])}`;
  }

  const rawCodec = firstMatch(lines, /Video codec:\s*([a-z0-9.]+)/i)?.[1] ?? null;
  let codec = normaliseCodec(rawCodec);
  if (!codec && vulkanEnabled) codec = 'H.264';
  if (!codec && encoder) {
    const inferred = encoder.match(/^(h264|hevc|av1)_/i)?.[1];
    if (inferred) codec = normaliseCodec(inferred);
  }

  const qualityMatch = firstMatch(lines, /Vulkan encoder quality:\s*(.+?)\s*->\s*qp=(\d+)/i);
  const cbrMatch = firstMatch(lines, /CBR target '([^']+)' mapped to qp=(\d+)/i);
  const qp = firstMatch(lines, /\bqp=(\d+)/i)?.[1] ?? null;
  // The Rust Vulkan encoder is constant-QP only: CRF is passed through as the QP
  // (and CBR is mapped to a QP). Say so instead of implying a quality conversion.
  const quality = qualityMatch
    ? `${qualityMatch[1].trim()} · ${t('log.dash.constant-qp', { qp: qualityMatch[2] })}`
    : cbrMatch
      ? `CBR ${cbrMatch[1]} · ${t('log.dash.constant-qp', { qp: cbrMatch[2] })}`
      : qp
        ? `QP ${qp}`
        : null;

  const presetRaw = firstMatch(lines, /Speed preset:\s*(.+)$/i)?.[1]?.trim() ?? null;
  const preset = presetRaw && !presetRaw.startsWith('(')
    ? presetRaw.replace(/^-\S+\s+/, '')
    : null;
  const presetBuiltin = !preset && (vulkanEnabled || Boolean(presetRaw?.includes('built-in')));

  const avgFpsMatch =
    firstMatch(lines, /Video Pipeline Avg FPS:\s*([\d.]+)/i) ??
    firstMatch(lines, /prpr avg fps:\s*([\d.]+)/i) ??
    firstMatch(lines, /End-to-End Avg FPS:\s*([\d.]+)/i);
  const averageFps = avgFpsMatch ? Number(avgFpsMatch[1]) : null;

  const progress: ProgressSample[] = [];
  let previousFrames = 0;
  for (const line of lines) {
    const match = line.text.match(
      /^Rendering:.*?(\d+)%.*?Time:\s*([\d.]+)s.*?Frames:\s*(\d+)\/(\d+)/,
    );
    if (!match) continue;
    const frames = Number(match[3]);
    const total = Number(match[4]);
    if (total <= 0 || frames <= previousFrames) continue;
    const interval = Number(match[2]);
    const delta = frames - previousFrames;
    progress.push({
      percent: Number(match[1]),
      frames,
      total,
      interval,
      perThousand: (interval / delta) * 1000,
    });
    previousFrames = frames;
  }

  let totalFrames = progress.length ? progress[progress.length - 1].total : null;
  if (totalFrames == null) {
    const framesMatch = firstMatch(lines, /Video Pipeline Avg FPS:.*?\((\d+) frames/);
    if (framesMatch) totalFrames = Number(framesMatch[1]);
  }

  const stages: StageTime[] = [];
  for (const line of lines) {
    const setup = line.text.match(/^setup ([^:]+):\s*([\d.]+)s/);
    if (setup) {
      const label = setup[1].trim();
      // "render loop" is the whole rendering phase, not one of its stages; showing
      // it beside the per-stage breakdown would dwarf every other bar.
      if (label === 'render loop') continue;
      stages.push({
        key: STAGE_KEYS[label] ?? null,
        label,
        seconds: Number(setup[2]),
        group: 'setup',
      });
      continue;
    }

    const prpr = line.text.match(/^prpr stage times:\s*(.+)$/);
    if (!prpr) continue;
    for (const part of prpr[1].split('|')) {
      const stage = part.trim().match(/^(.+?)\s+([\d.]+)s\s+\(/);
      if (!stage) continue;
      const label = stage[1].trim();
      stages.push({
        key: STAGE_KEYS[label] ?? null,
        label,
        seconds: Number(stage[2]),
        group: 'render',
      });
    }
  }

  return {
    encoder,
    hardwareKind,
    resolution,
    sourceResolution,
    aspect: aspectOf(resolution),
    codec,
    quality,
    preset,
    presetBuiltin,
    ffmpegPath: firstMatch(lines, /^Using ffmpeg:\s*(.+)$/)?.[1]?.trim() ?? null,
    totalFrames,
    averageFps,
    endToEnd: readSeconds(lines, /^End-to-End Time:\s*([\d.]+)s/),
    renderTime: readSeconds(lines, /^Render Time:\s*([\d.]+)s/),
    loopTime: readSeconds(lines, /^Render loop Time:\s*([\d.]+)s/),
    progress,
    stages,
  };
}

const insights = computed(() => buildInsights(allLines.value));

const coverUrl = computed(() => {
  const cover = props.task?.cover;
  if (!cover) return null;
  try {
    return convertFileSrc(cover);
  } catch {
    return null;
  }
});

const coverStyle = computed(() =>
  coverUrl.value ? { '--log-cover': `url("${coverUrl.value}")` } : undefined,
);

const content = ref<HTMLElement>();
let coverObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;

type CoverTone = 'dark' | 'light';

interface CoverSample {
  url: string;
  width: number;
  height: number;
  data: Uint8ClampedArray;
  /** Dominant tone of the artwork, used as the fallback for text colours. */
  tone: CoverTone;
}

let coverSample: CoverSample | null = null;
let coverSampleToken = 0;

function clamp01(value: number) {
  return Math.min(1, Math.max(0, value));
}

function channelLuminance(value: number) {
  const channel = value / 255;
  return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function relativeLuminance(rgb: [number, number, number]): number {
  return (
    0.2126 * channelLuminance(rgb[0]) +
    0.7152 * channelLuminance(rgb[1]) +
    0.0722 * channelLuminance(rgb[2])
  );
}

function contrastRatio(a: number, b: number): number {
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

/** Average WCAG luminance of the opaque pixels in the cover sample. */
function averageCoverLuminance(data: Uint8ClampedArray): number {
  let sum = 0;
  let count = 0;
  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] < 200) continue;
    sum += relativeLuminance([data[i], data[i + 1], data[i + 2]]);
    count += 1;
  }
  return count ? sum / count : 0.5;
}

/** Alpha of the theme wash at a horizontal position (matches the CSS stops). */
function coverWashAlpha(position: number): number {
  const p = clamp01(position);
  if (p <= 0.4) return 0;
  if (p <= 0.58) return 0.35 * ((p - 0.4) / 0.18);
  if (p <= 0.78) return 0.35 + 0.37 * ((p - 0.58) / 0.2);
  return 0.72 + 0.28 * ((p - 0.78) / 0.22);
}

/**
 * Pick the text tone with the stronger contrast against a sampled background.
 * A small difference falls back to the artwork's dominant tone so adjacent
 * blocks do not flicker between light and dark for an ambiguous mid-tone.
 */
function coverToneForLuminance(luminance: number, fallback: CoverTone): CoverTone {
  const lightText = contrastRatio(1, luminance);
  const darkText = contrastRatio(0.012, luminance);
  if (Math.abs(lightText - darkText) < 1) return fallback;
  return lightText > darkText ? 'dark' : 'light';
}

/** Resolve a theme surface token to an RGB triplet through a hidden probe. */
function resolveThemeSurface(host: HTMLElement): [number, number, number] | null {
  const probe = document.createElement('span');
  probe.style.cssText =
    'position:absolute;left:-9999px;top:0;color:rgb(var(--mdui-color-surface-container-high))';
  host.appendChild(probe);
  const color = getComputedStyle(probe).color;
  probe.remove();
  return parseCssColor(color);
}

function parseCssColor(value: string): [number, number, number] | null {
  const rgb = value.match(/^rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)/i);
  if (rgb) return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])];
  const srgb = value.match(/^color\(srgb\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)/i);
  if (srgb) {
    return [Number(srgb[1]) * 255, Number(srgb[2]) * 255, Number(srgb[3]) * 255];
  }
  return null;
}

/** Resolve the chart's accent through a probe, since it is built with color-mix(). */
function resolveBarColor(card: HTMLElement): [number, number, number] | null {
  const probe = document.createElement('span');
  probe.style.cssText = 'position:absolute;left:-9999px;top:0;color:var(--log-accent)';
  card.appendChild(probe);
  const color = getComputedStyle(probe).color;
  probe.remove();
  return parseCssColor(color);
}

/**
 * The average marker has to read on top of the dark bars, so pick the colour
 * from the cover's visible palette that is furthest from the bar accent in
 * contrast terms. The skin and highlights of an illustration win naturally.
 */
function pickCoverInk(
  sample: CoverSample,
  x0: number,
  x1: number,
  y0: number,
  y1: number,
  card: HTMLElement,
): { color: string; halo: string } | null {
  const bar = resolveBarColor(card);
  if (!bar) return null;
  const barLuminance = relativeLuminance(bar);

  const left = Math.max(0, Math.min(sample.width - 1, Math.floor(x0 * sample.width)));
  const right = Math.max(left + 1, Math.min(sample.width, Math.ceil(x1 * sample.width)));
  const top = Math.max(0, Math.min(sample.height - 1, Math.floor(y0 * sample.height)));
  const bottom = Math.max(top + 1, Math.min(sample.height, Math.ceil(y1 * sample.height)));

  // Coarse 5-bit RGB buckets keep the palette representative without turning
  // every anti-aliased pixel into a candidate.
  const buckets = new Map<number, { r: number; g: number; b: number; count: number }>();
  let total = 0;
  for (let y = top; y < bottom; y += 1) {
    const row = y * sample.width;
    for (let x = left; x < right; x += 1) {
      const i = (row + x) * 4;
      if (sample.data[i + 3] < 200) continue;
      const r = sample.data[i];
      const g = sample.data[i + 1];
      const b = sample.data[i + 2];
      const key = ((r >> 3) << 10) | ((g >> 3) << 5) | (b >> 3);
      const bucket = buckets.get(key);
      if (bucket) {
        bucket.r += r;
        bucket.g += g;
        bucket.b += b;
        bucket.count += 1;
      } else {
        buckets.set(key, { r, g, b, count: 1 });
      }
      total += 1;
    }
  }
  if (!total) return null;

  const minCount = Math.max(8, total * 0.003);
  let best: { r: number; g: number; b: number; contrast: number; count: number } | null = null;
  for (const bucket of buckets.values()) {
    if (bucket.count < minCount) continue;
    const r = bucket.r / bucket.count;
    const g = bucket.g / bucket.count;
    const b = bucket.b / bucket.count;
    const contrast = contrastRatio(barLuminance, relativeLuminance([r, g, b]));
    if (!best || contrast > best.contrast || (contrast === best.contrast && bucket.count > best.count)) {
      best = { r, g, b, contrast, count: bucket.count };
    }
  }
  if (!best) return null;

  const luminance = relativeLuminance([best.r, best.g, best.b]);
  return {
    color: `rgb(${Math.round(best.r)} ${Math.round(best.g)} ${Math.round(best.b)})`,
    halo: luminance > 0.4 ? 'rgba(20, 18, 24, 0.72)' : 'rgba(255, 255, 255, 0.9)',
  };
}

let textMeasureContext: CanvasRenderingContext2D | null = null;

/** Approximate the painted width of an element's text, not its full block box. */
function measureTextWidth(el: HTMLElement): number {
  const text = (el.textContent ?? '').trim();
  if (!text) return 0;
  const style = getComputedStyle(el);
  if (!textMeasureContext) {
    textMeasureContext = document.createElement('canvas').getContext('2d');
  }
  if (!textMeasureContext) return 0;
  textMeasureContext.font =
    `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  return textMeasureContext.measureText(text).width;
}

/**
 * The text blocks that sit directly on the artwork. Tiles and the average-marker
 * label are deliberately excluded: they have their own surfaces / sampled ink.
 */
const COVER_TEXT_SELECTORS = [
  '.dash-card-title',
  '.dash-card-sub',
  '.dash-metric',
  '.dash-meta',
  '.dash-config-grid dt',
  '.dash-config-grid dd:not(.dash-path)',
  '.dash-progress-meta > span',
  '.dash-chart',
  '.dash-chart-axis > span',
  '.stage-legend-item',
  '.dash-empty',
].join(',');

/**
 * Sample the artwork plus the theme wash behind one text block. Using the
 * painted text width (rather than the block's full card-wide box) is important:
 * a short title on the left must not inherit the light wash from the far right.
 */
function localSampleRect(el: HTMLElement, rect: DOMRect): DOMRect {
  const pad = 4;
  let rangeRect: DOMRect | null = null;
  if ((el.textContent ?? '').trim()) {
    try {
      const range = document.createRange();
      range.selectNodeContents(el);
      const raw = range.getBoundingClientRect();
      if (raw.width > 1 && raw.height > 1) {
        const left = Math.max(rect.left, raw.left);
        const top = Math.max(rect.top, raw.top);
        const right = Math.min(rect.right, raw.right);
        const bottom = Math.min(rect.bottom, raw.bottom);
        if (right > left && bottom > top) {
          rangeRect = new DOMRect(left, top, right - left, bottom - top);
        }
      }
    } catch {
      // A range can fail for disconnected nodes; fall back to measuring.
    }
  }
  if (rangeRect && rangeRect.width < rect.width * 0.92) {
    return new DOMRect(
      rangeRect.left - pad,
      rangeRect.top - pad,
      rangeRect.width + pad * 2,
      rangeRect.height + pad * 2,
    );
  }

  const measured = Math.min(rect.width, Math.max(24, measureTextWidth(el) + pad * 2));
  const width = el.classList.contains('dash-chart') ? rect.width : measured;
  const left =
    el.classList.contains('dash-chart') || el.classList.contains('dash-empty')
      ? rect.left + (rect.width - width) / 2
      : rect.left;
  return new DOMRect(left, rect.top, width, rect.height);
}

/**
 * Text on artwork follows the local background, not the theme: dark artwork
 * gets light text, a washed-out/light area keeps dark text. The theme still
 * controls the tiles, icon chips and window chrome.
 */
function applyCoverTones() {
  const el = content.value;
  if (!el) return;
  // Clear every old marker, not only the current selector matches, so nodes
  // that have just been excluded (such as the frosted FFmpeg path) drop stale
  // tones after a live edit or mode change.
  el.querySelectorAll<HTMLElement>('[data-log-tone]').forEach((node) => {
    node.removeAttribute('data-log-tone');
  });
  const nodes = [...el.querySelectorAll<HTMLElement>(COVER_TEXT_SELECTORS)];

  const sample = coverSample;
  if (!sample || sample.url !== coverUrl.value) {
    el.removeAttribute('data-cover-tone');
    return;
  }
  const theme = resolveThemeSurface(el);
  const width = el.offsetWidth;
  const height = el.offsetHeight;
  if (!theme || !width || !height) {
    el.removeAttribute('data-cover-tone');
    return;
  }

  el.dataset.coverTone = sample.tone;
  const contentRect = el.getBoundingClientRect();
  const scale = Math.max(width / sample.width, height / sample.height);

  for (const node of nodes) {
    const rect = node.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) continue;
    const box = localSampleRect(node, rect);
    if (box.width < 1 || box.height < 1) continue;
    const cols = Math.max(2, Math.min(10, Math.round(box.width / 12)));
    const rows = Math.max(2, Math.min(6, Math.round(box.height / 10)));
    let r = 0;
    let g = 0;
    let b = 0;
    let count = 0;
    for (let row = 0; row < rows; row += 1) {
      for (let col = 0; col < cols; col += 1) {
        const x = box.left - contentRect.left + ((col + 0.5) / cols) * box.width;
        const y = box.top - contentRect.top + ((row + 0.5) / rows) * box.height;
        if (x < 0 || y < 0 || x >= width || y >= height) continue;
        const sampleX = Math.min(sample.width - 1, Math.max(0, Math.floor(x / scale)));
        const sampleY = Math.min(sample.height - 1, Math.max(0, Math.floor(y / scale)));
        const i = (sampleY * sample.width + sampleX) * 4;
        const imageAlpha = sample.data[i + 3] / 255;
        const layerAlpha = imageAlpha * (1 - coverWashAlpha(x / width));
        r += sample.data[i] * layerAlpha + theme[0] * (1 - layerAlpha);
        g += sample.data[i + 1] * layerAlpha + theme[1] * (1 - layerAlpha);
        b += sample.data[i + 2] * layerAlpha + theme[2] * (1 - layerAlpha);
        count += 1;
      }
    }
    if (!count) continue;
    const luminance = relativeLuminance([r / count, g / count, b / count]);
    node.dataset.logTone = coverToneForLuminance(luminance, sample.tone);
  }
}

/**
 * Pick the average-marker ink from the cover, contrasting it with the bars. The
 * artwork fades into the theme surface, so the halo—not a per-card light/dark
 * surface—keeps the marker readable where it crosses the wash.
 */
function applyCoverInk() {
  const el = content.value;
  const sample = coverSample;
  if (!el || !sample || sample.url !== coverUrl.value) return;
  const width = el.offsetWidth;
  const height = el.offsetHeight;
  if (!width || !height) return;

  const imageAspect = sample.width / sample.height;
  const boxAspect = width / height;
  let displayWidth: number;
  let displayHeight: number;
  let offsetX = 0;
  let offsetY = 0;
  if (imageAspect > boxAspect) {
    displayHeight = height;
    displayWidth = height * imageAspect;
  } else {
    displayWidth = width;
    displayHeight = width / imageAspect;
    offsetY = (height - displayHeight) / 2;
  }

  // The chart is in the left column; sample where the artwork is strongest and
  // resolve the accent from the chart itself, so a locally lightened bar colour
  // also gets a matching high-contrast marker ink.
  const coverEnd = width * (2 / 3);
  const chart = el.querySelector<HTMLElement>('.dash-chart');
  const ink = chart && pickCoverInk(
    sample,
    clamp01((0 - offsetX) / displayWidth),
    clamp01((coverEnd - offsetX) / displayWidth),
    clamp01((0 - offsetY) / displayHeight),
    clamp01((height - offsetY) / displayHeight),
    chart,
  );
  if (ink) {
    el.style.setProperty('--log-cover-ink', ink.color);
    el.style.setProperty('--log-cover-halo', ink.halo);
  } else {
    el.style.removeProperty('--log-cover-ink');
    el.style.removeProperty('--log-cover-halo');
  }
}

/**
 * Each card shows the same content-sized artwork through its own clipped window.
 * Storing the card's offset lets the shared image line up across cards, while the
 * card-sized theme wash fades every card to the current theme surface.
 */
function updateCoverLayout() {
  const el = content.value;
  const cards = el ? [...el.querySelectorAll<HTMLElement>('.dash-card')] : [];
  if (!el || !coverUrl.value) {
    el?.style.removeProperty('--log-cover-ink');
    el?.style.removeProperty('--log-cover-halo');
    applyCoverTones();
    return;
  }
  // Layout offsets instead of rects: the opening animation scales the window, and
  // rects would capture that transient 0.98 scale and leave the artwork misaligned.
  el.style.setProperty('--cover-width', `${el.offsetWidth}px`);
  el.style.setProperty('--cover-height', `${el.offsetHeight}px`);
  for (const card of cards) {
    card.style.setProperty(
      '--cover-left',
      `${-card.offsetLeft - card.clientLeft}px`,
    );
    card.style.setProperty(
      '--cover-top',
      `${-card.offsetTop - card.clientTop}px`,
    );
  }
  // A new task can arrive before its cover has been sampled. Drop the previous
  // task's ink rather than briefly painting it over the new content.
  if (!coverSample || coverSample.url !== coverUrl.value) {
    el.style.removeProperty('--log-cover-ink');
    el.style.removeProperty('--log-cover-halo');
    applyCoverTones();
    return;
  }
  applyCoverTones();
  applyCoverInk();
}

async function loadCoverSample(url: string) {
  if (coverSample?.url === url) {
    updateCoverLayout();
    return;
  }

  const token = ++coverSampleToken;
  coverSample = null;
  updateCoverLayout();

  const image = new Image();
  image.crossOrigin = 'anonymous';
  image.decoding = 'async';
  image.src = url;
  try {
    await image.decode();
  } catch (error) {
    console.warn('Unable to load cover for contrast analysis', error);
    if (token === coverSampleToken) {
      updateCoverLayout();
    }
    return;
  }
  if (token !== coverSampleToken) return;

  if (!image.naturalWidth || !image.naturalHeight) {
    if (token === coverSampleToken) {
      updateCoverLayout();
    }
    return;
  }

  const maxSide = 160;
  const scale = Math.min(1, maxSide / Math.max(image.naturalWidth, image.naturalHeight));
  const sampleWidth = Math.max(1, Math.round(image.naturalWidth * scale));
  const sampleHeight = Math.max(1, Math.round(image.naturalHeight * scale));
  const canvas = document.createElement('canvas');
  canvas.width = sampleWidth;
  canvas.height = sampleHeight;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  if (!context) {
    if (token === coverSampleToken) {
      updateCoverLayout();
    }
    return;
  }
  try {
    context.drawImage(image, 0, 0, sampleWidth, sampleHeight);
  } catch (error) {
    console.warn('Unable to draw cover for contrast analysis', error);
    if (token === coverSampleToken) {
      updateCoverLayout();
    }
    return;
  }

  let data: Uint8ClampedArray;
  try {
    data = context.getImageData(0, 0, sampleWidth, sampleHeight).data;
  } catch (error) {
    console.warn('Unable to inspect cover colours', error);
    if (token === coverSampleToken) {
      updateCoverLayout();
    }
    return;
  }
  if (token !== coverSampleToken) return;

  const average = averageCoverLuminance(data);
  coverSample = {
    url,
    width: sampleWidth,
    height: sampleHeight,
    data,
    tone: average < 0.42 ? 'dark' : 'light',
  };
  updateCoverLayout();
}

const ENCODER_NAMES: Record<string, string> = {
  h264_nvenc: 'NVIDIA NVENC',
  hevc_nvenc: 'NVIDIA NVENC',
  av1_nvenc: 'NVIDIA NVENC',
  h264_qsv: 'Intel Quick Sync',
  hevc_qsv: 'Intel Quick Sync',
  av1_qsv: 'Intel Quick Sync',
  h264_amf: 'AMD AMF',
  hevc_amf: 'AMD AMF',
  av1_amf: 'AMD AMF',
  h264_vulkan: 'Vulkan Video',
  hevc_vulkan: 'Vulkan Video',
  av1_vulkan: 'Vulkan Video',
  h264_d3d12va: 'Direct3D 12',
  hevc_d3d12va: 'Direct3D 12',
  av1_d3d12va: 'Direct3D 12',
  libx264: 'x264',
  libx265: 'x265',
  libsvtav1: 'SVT-AV1',
  libaom_av1: 'libaom AV1',
  librav1e: 'rav1e',
};

const encoderName = computed(() => {
  const raw = insights.value.encoder;
  if (!raw) return null;
  if (/vulkan video/i.test(raw)) return 'Vulkan Video';
  const name = raw.match(/\b(h264|hevc|av1)_(?:nvenc|qsv|amf|vulkan|d3d12va)\b/i)?.[0];
  return name ? ENCODER_NAMES[name.toLowerCase()] ?? name : raw;
});

const qualityHint = computed(() =>
  /vulkan/i.test(insights.value.encoder ?? '') ? t('log.dash.quality-hint') : '',
);

const acceleratorLabel = computed(() => {
  const kind = insights.value.hardwareKind;
  if (kind === 'disabled' || kind === 'software') return 'CPU';
  return encoderName.value ?? t('log.dash.unknown');
});

const lastProgress = computed(() => {
  const samples = insights.value.progress;
  return samples.length ? samples[samples.length - 1] : null;
});
const progressPercent = computed(() => lastProgress.value?.percent ?? 0);

const perThousandMax = computed(() =>
  Math.max(0.001, ...insights.value.progress.map((sample) => sample.perThousand)),
);
const perThousandAverage = computed(() => {
  const samples = insights.value.progress;
  if (!samples.length) return 0;
  const frames = samples[samples.length - 1].frames;
  const total = samples.reduce((sum, sample) => sum + sample.interval, 0);
  return frames > 0 ? (total / frames) * 1000 : 0;
});
const perThousandMin = computed(() => {
  const samples = insights.value.progress;
  return samples.length ? Math.min(...samples.map((sample) => sample.perThousand)) : 0;
});
const averageLine = computed(() =>
  Math.min(100, (perThousandAverage.value / perThousandMax.value) * 100),
);

function barHeight(sample: ProgressSample): string {
  return `${Math.max(8, (sample.perThousand / perThousandMax.value) * 100)}%`;
}

function stageLabel(stage: StageTime): string {
  const key = stage.key ? `log.dash.stage.${stage.key}` : '';
  return key && te(key) ? t(key) : stage.label;
}

/** Tint each stage legend dot a little differently down the render group. */
function stageColor(index: number, group: string): string {
  if (group === 'render') return 'var(--log-accent)';
  const mix = [0, 25, 50, 75, 100][index % 5];
  return `color-mix(in srgb, var(--log-accent-2) ${100 - mix}%, var(--log-accent) ${mix}%)`;
}

function chartHint(sample: ProgressSample): string {
  return t('log.dash.chart-hint', {
    percent: sample.percent,
    time: formatSeconds(sample.perThousand),
  });
}

interface ChartTipState {
  text: string;
  left: number;
  top: number;
  below: boolean;
}

// The hover readout lives outside the cards so their overflow clipping (used by
// the shared cover artwork) cannot cut it off at the card edge.
const chartTip = ref<ChartTipState | null>(null);
const chartTipEl = ref<HTMLElement | null>(null);

function hideChartTip() {
  chartTip.value = null;
}

async function showChartTip(event: MouseEvent, sample: ProgressSample) {
  const column = event.currentTarget as HTMLElement;
  const rect = column.getBoundingClientRect();
  chartTip.value = {
    text: chartHint(sample),
    left: rect.left + rect.width / 2,
    top: rect.top - 8,
    below: false,
  };
  await nextTick();
  if (!chartTip.value || !chartTipEl.value) return;

  const tipRect = chartTipEl.value.getBoundingClientRect();
  const margin = 10;
  const halfWidth = tipRect.width / 2;
  const left = Math.min(
    Math.max(rect.left + rect.width / 2, margin + halfWidth),
    window.innerWidth - margin - halfWidth,
  );
  let below = false;
  let top = rect.top - 8;
  if (top - tipRect.height < margin) {
    below = true;
    top = rect.bottom + 8;
  }
  chartTip.value = { ...chartTip.value, left, top, below };
}

function formatSeconds(seconds: number | null | undefined, digits = 2): string {
  if (seconds == null || !Number.isFinite(seconds)) return '—';
  if (seconds < 0.001) return '<1 ms';
  if (seconds < 1) return `${Math.round(seconds * 1000)} ms`;
  if (seconds < 60) return `${seconds.toFixed(digits)} s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${(seconds - minutes * 60).toFixed(1)}s`;
}

function formatCount(value: number | null | undefined): string {
  return value == null ? '—' : value.toLocaleString();
}

function levelLabel(value: LogLevel): string {
  return value === 'warn' ? 'WARN' : value.toUpperCase();
}

function highlightParts(text: string): Array<{ text: string; hit: boolean }> {
  const keyword = query.value.trim();
  if (!keyword) return [{ text, hit: false }];

  const haystack = text.toLowerCase();
  const needle = keyword.toLowerCase();
  const parts: Array<{ text: string; hit: boolean }> = [];
  let start = 0;
  let index = haystack.indexOf(needle);
  while (index !== -1) {
    if (index > start) parts.push({ text: text.slice(start, index), hit: false });
    parts.push({ text: text.slice(index, index + needle.length), hit: true });
    start = index + needle.length;
    index = haystack.indexOf(needle, start);
  }
  if (start < text.length) parts.push({ text: text.slice(start), hit: false });
  return parts;
}

function onLevelChange(e: Event) {
  const value = (e.target as HTMLElement & { value?: string }).value;
  if (value === 'all' || value === 'info' || value === 'warn' || value === 'error') {
    level.value = value;
  }
}

function onModeChange(e: Event) {
  const target = e.target as HTMLElement & { value?: string };
  if (target.value === 'simple' || target.value === 'detail') {
    mode.value = target.value;
    return;
  }
  // Fallback if the group is cleared by something other than a segment click.
  target.value = mode.value;
}

function onModeSegmentClick(event: MouseEvent, value: ViewMode) {
  // mdui toggles the active segment off when it is clicked a second time. A
  // segmented control must always have one active option, so swallow that click
  // before it reaches the group's selection handler.
  if (mode.value === value) event.stopPropagation();
}

function selectLevel(value: LevelFilter) {
  level.value = value;
  mode.value = 'detail';
}

function clearSearch() {
  query.value = '';
}

function selectLogLine(index: number) {
  const selection = logSelection.value;
  logSelection.value = selection
    ? { anchor: selection.anchor, end: index }
    : { anchor: index, end: index };
}

function isLogLineSelected(index: number) {
  const selection = logSelection.value;
  if (!selection) return false;
  return (
    index >= Math.min(selection.anchor, selection.end) &&
    index <= Math.max(selection.anchor, selection.end)
  );
}

function clearLogSelection(event?: MouseEvent) {
  if (!logSelection.value) return;
  event?.preventDefault();
  logSelection.value = null;
}

function logFileName() {
  return (props.task?.name || 'output').replace(/[\\/:*?"<>|]/g, '_');
}

/**
 * A screenshot is rendered from the live dashboard, including the shared cover
 * layer. The cover is temporarily embedded as a data URL because pseudo-element
 * background images are not inlined by html-to-image.
 */
async function imageAsDataUrl(url: string): Promise<string> {
  const image = new Image();
  image.crossOrigin = 'anonymous';
  image.decoding = 'async';
  image.src = url;
  try {
    await image.decode();
  } catch (error) {
    console.warn('Unable to load the dashboard cover for the screenshot', error);
    throw new Error('The dashboard cover could not be loaded');
  }
  if (!image.naturalWidth || !image.naturalHeight) {
    throw new Error('The dashboard cover has no pixels');
  }

  const maxSide = 2048;
  const scale = Math.min(1, maxSide / Math.max(image.naturalWidth, image.naturalHeight));
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
  const context = canvas.getContext('2d');
  if (!context) throw new Error('The dashboard cover could not be prepared');
  try {
    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL('image/png');
  } catch (error) {
    console.warn('Unable to embed the dashboard cover in the screenshot', error);
    throw new Error('The dashboard cover could not be embedded');
  }
}

async function captureSimpleDashboard(): Promise<Blob> {
  updateCoverLayout();
  await nextTick();
  const el = content.value;
  if (!el) throw new Error('The simplified dashboard is not open');

  const coverData = coverUrl.value ? await imageAsDataUrl(coverUrl.value) : null;
  const previousCover = el.style.getPropertyValue('--log-cover');
  if (coverData) el.style.setProperty('--log-cover', `url("${coverData}")`);

  const background = el.parentElement
    ? getComputedStyle(el.parentElement).backgroundColor
    : getComputedStyle(document.body).backgroundColor;
  try {
    const blob = await toBlob(el, {
      backgroundColor:
        background && background !== 'rgba(0, 0, 0, 0)' ? background : '#ffffff',
      pixelRatio: Math.max(1, Math.min(2, window.devicePixelRatio || 1)),
    });
    if (!blob) throw new Error('The dashboard image could not be encoded');
    return blob;
  } finally {
    if (coverData) {
      if (previousCover) el.style.setProperty('--log-cover', previousCover);
      else el.style.removeProperty('--log-cover');
    }
  }
}

async function copyLog() {
  if (exporting.value) return;
  exporting.value = true;
  try {
    if (mode.value === 'simple') {
      if (typeof ClipboardItem === 'undefined' || !navigator.clipboard?.write) {
        throw new Error('Image clipboard is not supported');
      }
      const blob = await captureSimpleDashboard();
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
      toast(t('log.image-copied'), 'success');
    } else {
      await navigator.clipboard.writeText(fullLogText.value);
      toast(t('log.copied'), 'success');
    }
  } catch (e) {
    console.error(e);
    toast(
      t(mode.value === 'simple' ? 'log.image-copy-failed' : 'log.copy-failed'),
      'warning',
    );
  } finally {
    exporting.value = false;
  }
}

async function saveLog() {
  if (exporting.value) return;
  exporting.value = true;
  try {
    const name = logFileName();
    if (mode.value === 'simple') {
      const blob = await captureSimpleDashboard();
      const path = await save({
        defaultPath: `${name}.png`,
        filters: [{ name: 'PNG image', extensions: ['png'] }],
      });
      if (!path) return;
      await invoke('save_binary_file', {
        path,
        data: Array.from(new Uint8Array(await blob.arrayBuffer())),
      });
      toast(t('log.image-saved'), 'success');
      return;
    }

    const path = await save({
      defaultPath: `${name}.log`,
      filters: [{ name: 'Log', extensions: ['log', 'txt'] }],
    });
    if (!path) return;
    await invoke('save_text_file', { path, content: fullLogText.value });
    toast(t('log.saved'), 'success');
  } catch (e) {
    console.error(e);
    toast(
      t(mode.value === 'simple' ? 'log.image-save-failed' : 'log.save-failed'),
      'warning',
    );
  } finally {
    exporting.value = false;
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    e.stopPropagation();
    emit('close');
  }
}

watch(
  () => props.open,
  async (open) => {
    if (!open) return;
    level.value = 'all';
    query.value = '';
    logSelection.value = null;
    // A failed run usually has no dashboard data, so the raw error is the useful view.
    mode.value = props.task?.status.type === 'failed' ? 'detail' : 'simple';
    await nextTick();
    if (body.value) body.value.scrollTop = 0;
    root.value?.focus();
  },
);

watch([level, query], async () => {
  await nextTick();
  if (body.value) body.value.scrollTop = 0;
});

watch([() => props.open, mode, level, query], () => {
  hideChartTip();
  logSelection.value = null;
});

watch(
  [() => props.open, mode, coverUrl],
  async () => {
    await nextTick();
    updateCoverLayout();
    if (props.open && mode.value === 'simple' && coverUrl.value) {
      void loadCoverSample(coverUrl.value);
    }
  },
);

watch(content, (el, previous) => {
  if (previous) coverObserver?.unobserve(previous);
  if (!el) return;
  coverObserver?.observe(el);
  nextTick(() => {
    updateCoverLayout();
    if (coverUrl.value) void loadCoverSample(coverUrl.value);
  });
});

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  window.addEventListener('resize', hideChartTip);
  coverObserver = new ResizeObserver(() => updateCoverLayout());
  // MD3 promotes the active colour tokens onto <html>; re-sample the local text
  // tones when the theme (and therefore the wash colour) changes.
  themeObserver = new MutationObserver(() => updateCoverLayout());
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['style', 'class', 'data-theme'],
  });
  if (content.value) {
    coverObserver.observe(content.value);
    updateCoverLayout();
    if (coverUrl.value) void loadCoverSample(coverUrl.value);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
  window.removeEventListener('resize', hideChartTip);
  coverObserver?.disconnect();
  themeObserver?.disconnect();
  coverSampleToken += 1;
});
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="log-backdrop" @click.self="emit('close')">
      <section
        ref="root"
        class="log-window"
        tabindex="-1"
        role="dialog"
        aria-modal="true"
        :aria-label="t('log.title')">
        <header class="log-header">
          <h2 class="log-title" :title="task?.name">{{ task?.name }}</h2>
          <mdui-button-icon class="log-close" :title="t('log.close')" @click="emit('close')">
            <mdui-icon-close></mdui-icon-close>
          </mdui-button-icon>
        </header>

        <div class="log-toolbar">
          <mdui-segmented-button-group
            class="log-mode"
            selects="single"
            :value="mode"
            @change="onModeChange">
            <mdui-segmented-button
              value="simple"
              @click="onModeSegmentClick($event, 'simple')">
              {{ t('log.simple') }}
            </mdui-segmented-button>
            <mdui-segmented-button
              value="detail"
              @click="onModeSegmentClick($event, 'detail')">
              {{ t('log.detail') }}
            </mdui-segmented-button>
          </mdui-segmented-button-group>

          <label v-if="mode === 'detail'" class="log-search">
            <mdui-icon-search class="log-search-icon"></mdui-icon-search>
            <input
              v-model="query"
              type="search"
              :placeholder="t('log.search')"
              :aria-label="t('log.search')" />
            <button
              v-if="query"
              type="button"
              class="log-search-clear"
              :aria-label="t('log.clear-search')"
              @click="clearSearch">
              <mdui-icon-close></mdui-icon-close>
            </button>
          </label>
        </div>

        <div v-if="!allLines.length" class="log-empty">
          <mdui-icon-description--outlined class="log-empty-icon"></mdui-icon-description--outlined>
          <p>{{ t('log.empty') }}</p>
        </div>

        <template v-else>
          <!-- Simplified: a graphical summary instead of a wall of monospaced lines. -->
          <div v-if="mode === 'simple'" class="log-simple" @scroll="hideChartTip">
            <div
              ref="content"
              class="log-simple-content"
              :class="{ 'has-cover': Boolean(coverStyle) }"
              :style="coverStyle">
              <div v-if="issueLine" class="log-issue" :class="`is-${issueLine.level}`">
                <mdui-icon-error v-if="issueLine.level === 'error'" class="log-issue-icon"></mdui-icon-error>
                <mdui-icon-warning v-else class="log-issue-icon"></mdui-icon-warning>
                <div class="log-issue-text">
                  <div class="log-issue-title">
                    {{ t('log.dash.issues', { errors: counts.error, warnings: counts.warn }) }}
                  </div>
                  <div class="log-issue-preview">{{ issueLine.text }}</div>
                </div>
                <mdui-button variant="text" @click="selectLevel(issueLine.level)">
                  {{ t('log.view-detail') }}
                </mdui-button>
              </div>

              <div class="dashboard">
                <div class="dashboard-top">
                  <div class="dashboard-stack">
                    <section class="dash-card dash-card-compact">
                      <div class="dash-card-head">
                        <span class="dash-icon"><mdui-icon-memory--outlined></mdui-icon-memory--outlined></span>
                        <h3 class="dash-card-title">{{ t('log.dash.hardware') }}</h3>
                      </div>
                      <div class="dash-metric">{{ acceleratorLabel }}</div>
                    </section>

                    <section class="dash-card dash-card-compact">
                      <div class="dash-card-head">
                        <span class="dash-icon is-tertiary"><mdui-icon-hd--outlined></mdui-icon-hd--outlined></span>
                        <h3 class="dash-card-title">{{ t('log.dash.resolution') }}</h3>
                      </div>
                      <div class="dash-metric">{{ insights.resolution ?? t('log.dash.unknown') }}</div>
                      <div class="dash-meta">
                        <span v-if="insights.sourceResolution && insights.sourceResolution !== insights.resolution">
                          {{ t('log.dash.source', { value: insights.sourceResolution }) }}
                        </span>
                        <span v-if="insights.aspect">{{ t('log.dash.aspect', { value: insights.aspect }) }}</span>
                      </div>
                    </section>
                  </div>

                  <section class="dash-card dash-config-card">
                    <div class="dash-card-head">
                      <span class="dash-icon is-secondary">
                        <mdui-icon-video-settings--outlined></mdui-icon-video-settings--outlined>
                      </span>
                      <h3 class="dash-card-title">{{ t('log.dash.config') }}</h3>
                    </div>
                    <dl class="dash-config-grid">
                      <div>
                        <dt>{{ t('log.dash.codec') }}</dt>
                        <dd>{{ insights.codec ?? t('log.dash.not-recorded') }}</dd>
                      </div>
                      <div>
                        <dt>{{ t('log.dash.quality') }}</dt>
                        <dd :title="qualityHint || undefined">{{ insights.quality ?? t('log.dash.not-recorded') }}</dd>
                      </div>
                      <div>
                        <dt>{{ t('log.dash.preset') }}</dt>
                        <dd>{{ insights.preset ?? (insights.presetBuiltin ? t('log.dash.builtin') : t('log.dash.not-recorded')) }}</dd>
                      </div>
                      <div class="is-wide">
                        <dt>{{ t('log.dash.ffmpeg') }}</dt>
                        <dd class="dash-path" :title="insights.ffmpegPath ?? ''">
                          {{ insights.ffmpegPath ?? t('log.dash.not-recorded') }}
                        </dd>
                      </div>
                    </dl>
                  </section>
                </div>

                <div class="dashboard-bottom">
                  <section class="dash-card dash-progress-card">
                    <div class="dash-card-head">
                      <span class="dash-icon"><mdui-icon-data-usage--outlined></mdui-icon-data-usage--outlined></span>
                      <h3 class="dash-card-title">{{ t('log.dash.progress') }}</h3>
                    </div>

                    <template v-if="insights.progress.length">
                      <div class="dash-progress-meta">
                        <span>{{ t('log.dash.frames') }} {{ formatCount(insights.totalFrames) }}</span>
                        <span v-if="insights.averageFps != null">{{ insights.averageFps }} FPS</span>
                      </div>

                      <div class="dash-chart-wrap">
                        <div class="dash-chart">
                          <div class="dash-chart-plot" aria-hidden="true">
                            <div
                              class="dash-chart-average"
                              :style="{ bottom: `${averageLine}%` }"
                              :data-label="t('log.dash.average')"></div>
                          </div>
                          <div
                            v-for="(sample, index) in insights.progress"
                            :key="index"
                            class="dash-chart-col"
                            :aria-label="chartHint(sample)"
                            @mouseenter="showChartTip($event, sample)"
                            @mouseleave="hideChartTip">
                            <div class="dash-chart-bar" :style="{ height: barHeight(sample) }"></div>
                          </div>
                        </div>
                        <div class="dash-chart-axis">
                          <span>0%</span>
                          <span>{{ t('log.dash.per-thousand') }}</span>
                          <span>{{ progressPercent }}%</span>
                        </div>
                      </div>

                      <div class="dash-stats">
                        <div>
                          <span>{{ t('log.dash.average') }}</span>
                          <strong>{{ formatSeconds(perThousandAverage) }}</strong>
                        </div>
                        <div>
                          <span>{{ t('log.dash.fastest') }}</span>
                          <strong>{{ formatSeconds(perThousandMin) }}</strong>
                        </div>
                        <div>
                          <span>{{ t('log.dash.slowest') }}</span>
                          <strong>{{ formatSeconds(perThousandMax) }}</strong>
                        </div>
                      </div>
                    </template>

                    <div v-else class="dash-empty">
                      <mdui-icon-data-usage--outlined></mdui-icon-data-usage--outlined>
                      <span>{{ t('log.dash.no-progress') }}</span>
                    </div>
                  </section>

                  <section class="dash-card dash-stages-card">
                    <div class="dash-card-head">
                      <span class="dash-icon is-tertiary"><mdui-icon-timer--outlined></mdui-icon-timer--outlined></span>
                      <div class="dash-card-heading">
                        <h3 class="dash-card-title">{{ t('log.dash.stages') }}</h3>
                        <p class="dash-card-sub">{{ t('log.dash.stages-sub') }}</p>
                      </div>
                    </div>

                    <template v-if="insights.stages.length">
                      <div class="dash-totals">
                        <div>
                          <span>{{ t('log.dash.end-to-end') }}</span>
                          <strong>{{ formatSeconds(insights.endToEnd) }}</strong>
                        </div>
                        <div>
                          <span>{{ t('log.dash.render-time') }}</span>
                          <strong>{{ formatSeconds(insights.renderTime) }}</strong>
                        </div>
                      </div>

                      <div class="stage-legend">
                        <div
                          v-for="(stage, index) in insights.stages"
                          :key="`${stage.group}-${stage.label}-${index}`"
                          class="stage-legend-item">
                          <span
                            class="stage-dot"
                            :class="stage.group"
                            :style="{ backgroundColor: stageColor(index, stage.group) }"></span>
                          <span class="stage-name" :title="stageLabel(stage)">{{ stageLabel(stage) }}</span>
                          <span class="stage-time">{{ formatSeconds(stage.seconds) }}</span>
                        </div>
                      </div>
                    </template>

                    <div v-else class="dash-empty">
                      <mdui-icon-timer--outlined></mdui-icon-timer--outlined>
                      <span>{{ t('log.dash.no-stages') }}</span>
                    </div>
                  </section>
                </div>
              </div>
            </div>
          </div>

          <!-- Detailed: the complete raw log, with colour-coded levels and stages. -->
          <div v-else ref="body" class="log-body" @contextmenu="clearLogSelection">
            <div v-if="!filteredLines.length" class="log-empty is-inline">
              <mdui-icon-search class="log-empty-icon"></mdui-icon-search>
              <p>{{ t('log.no-match') }}</p>
            </div>
            <div
              v-for="(line, index) in filteredLines"
              :key="line.no"
              class="log-line"
              :class="[`is-${line.level}`, { 'is-selected': isLogLineSelected(index) }]"
              @click="selectLogLine(index)">
              <span class="log-no">{{ line.no }}</span>
              <span class="log-level" :class="`is-${line.level}`">{{ levelLabel(line.level) }}</span>
              <span class="log-stage" :class="`is-${line.stage}`">{{ t(`log.stage.${line.stage}`) }}</span>
              <span class="log-text">
                <template v-for="(part, index) in highlightParts(line.text)" :key="index">
                  <mark v-if="part.hit">{{ part.text }}</mark>
                  <template v-else>{{ part.text }}</template>
                </template>
              </span>
            </div>
          </div>
        </template>

        <footer class="log-footer" :class="{ 'is-simple': mode === 'simple' }">
          <div v-if="mode === 'detail'" class="log-footer-stats">
            <button
              v-for="item in levelStats"
              :key="item.value"
              type="button"
              class="log-stat"
              :class="{ active: level === item.value }"
              @click="selectLevel(item.value)">
              <span class="log-dot" :class="`is-${item.value}`"></span>
              <span>{{ item.label }}</span>
              <strong>{{ item.count }}</strong>
            </button>
          </div>

          <span
            v-if="mode === 'detail'"
            class="log-line-count"
            :class="{ 'is-selected': logSelection }"
            :title="logSelection ? t('log.selection-hint', { count: selectedLineCount }) : undefined">
            {{ lineCountLabel }}
          </span>

          <div class="log-footer-actions">
            <mdui-button variant="tonal" :disabled="exporting" @click="copyLog">
              <mdui-icon-content-copy slot="icon"></mdui-icon-content-copy>
              {{ t(mode === 'simple' ? 'log.copy-image' : 'log.copy-log') }}
            </mdui-button>
            <mdui-button variant="tonal" :disabled="exporting" @click="saveLog">
              <mdui-icon-save--outlined slot="icon"></mdui-icon-save--outlined>
              {{ t(mode === 'simple' ? 'log.save-image' : 'log.save-log') }}
            </mdui-button>
          </div>
        </footer>
      </section>

      <div
        v-if="chartTip"
        ref="chartTipEl"
        class="chart-tooltip"
        :class="{ 'is-below': chartTip.below }"
        :style="{ left: `${chartTip.left}px`, top: `${chartTip.top}px` }">{{ chartTip.text }}</div>
    </div>
  </Teleport>
</template>

<style scoped lang="scss">
@use '../styles/breakpoints' as bp;

.log-backdrop {
  position: fixed;
  inset: 0;
  /* mdui's navigation rail sits at 2000; the log surface is modal and must clear it. */
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4vh 4vw;
  background-color: rgba(0, 0, 0, 0.45);
}

.log-window {
  display: flex;
  flex-direction: column;
  width: min(1160px, 100%);
  height: min(820px, 100%);
  overflow: hidden;
  border-radius: var(--mdui-shape-corner-extra-large);
  background-color: rgb(var(--mdui-color-surface-container-high));
  color: rgb(var(--mdui-color-on-surface));
  box-shadow: var(--mdui-elevation-level4);
  outline: none;
  animation: log-window-in var(--app-motion-duration-spatial-default, 220ms) var(--mdui-motion-easing-emphasized-decelerate);
}

@keyframes log-window-in {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

/* ===== Header ===== */
.log-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 16px 16px 20px;
  border-bottom: 1px solid rgba(var(--mdui-color-outline-variant), 0.5);
}

.log-title {
  flex: 1;
  min-width: 0;
  margin: 0;
  font-size: var(--mdui-typescale-title-large-size);
  font-weight: var(--mdui-typescale-title-large-weight);
  line-height: var(--mdui-typescale-title-large-line-height);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-close {
  flex-shrink: 0;
}

/* ===== Toolbar ===== */
.log-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid rgba(var(--mdui-color-outline-variant), 0.5);
}

.log-mode {
  flex-shrink: 0;
}

/* The selected check mark makes the compact two-option switch look busy; the
   tinted segment already communicates the active mode. */
.log-mode mdui-segmented-button::part(selected-icon) {
  display: none;
}

.log-mode mdui-segmented-button::part(label) {
  padding-right: 0;
  padding-left: 0;
}

.log-search {
  display: flex;
  align-items: center;
  gap: 6px;
  width: min(280px, 100%);
  height: 38px;
  margin-left: auto;
  padding: 0 8px 0 12px;
  border: 1px solid rgb(var(--mdui-color-outline-variant));
  border-radius: var(--mdui-shape-corner-full);
  background-color: rgb(var(--mdui-color-surface-container-highest));
  transition-property: border-color, box-shadow;
  transition-duration: var(--app-motion-duration-effects-fast, 120ms);
  transition-timing-function: var(--mdui-motion-easing-standard);
}

.log-search:focus-within {
  border-color: rgb(var(--mdui-color-primary));
  box-shadow: 0 0 0 2px rgba(var(--mdui-color-primary), 0.16);
}

.log-search-icon {
  flex-shrink: 0;
  font-size: 18px;
  color: rgb(var(--mdui-color-on-surface-variant));
}

.log-search input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: rgb(var(--mdui-color-on-surface));
  font-family: inherit;
  font-size: var(--mdui-typescale-body-small-size);
}

.log-search input::placeholder {
  color: rgb(var(--mdui-color-on-surface-variant));
}

.log-search input::-webkit-search-cancel-button {
  display: none;
}

.log-search-clear {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: rgb(var(--mdui-color-on-surface-variant));
  cursor: pointer;
}

.log-search-clear:hover {
  background-color: rgba(var(--mdui-color-on-surface), 0.08);
}

.log-search-clear mdui-icon-close {
  font-size: 16px;
}

/* ===== Empty state ===== */
.log-empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 48px 20px;
  color: rgb(var(--mdui-color-on-surface-variant));
}

.log-empty.is-inline {
  flex: none;
  padding: 64px 20px;
}

.log-empty-icon {
  font-size: 3rem;
  color: rgb(var(--mdui-color-outline));
}

.log-empty p {
  margin: 0;
  font-size: var(--mdui-typescale-body-medium-size);
}

/* ===== Simplified dashboard ===== */
.log-simple {
  flex: 1;
  min-height: 0;
  overflow: auto;
  background-color: rgb(var(--mdui-color-surface-container));
}

.log-simple-content {
  position: relative;
  padding: 20px;

  /* Defaults match the current MD3 theme. A card that sits on artwork overrides
     these with a palette derived from its own slice of the illustration. */
  --log-fg: rgb(var(--mdui-color-on-surface));
  --log-muted: rgb(var(--mdui-color-on-surface-variant));
  --log-faint: rgb(var(--mdui-color-outline));
  --log-accent: rgb(var(--mdui-color-primary));
  --log-accent-2: rgb(var(--mdui-color-tertiary));
  --log-tile-bg: rgb(var(--mdui-color-surface-container));
  --log-tile-fg: rgb(var(--mdui-color-on-surface));
  --log-tile-muted: rgb(var(--mdui-color-on-surface-variant));
  --log-tile-line: rgba(var(--mdui-color-outline-variant), 0.55);
  --log-grid: rgba(var(--mdui-color-outline-variant), 0.3);
  --log-line: rgba(var(--mdui-color-outline-variant), 0.62);
  --log-cover-ink: rgb(var(--mdui-color-on-surface));
  --log-cover-halo: transparent;
  --log-text-shadow: none;
}

.log-simple-content.has-cover .dash-card {
  position: relative;
  overflow: hidden;
  isolation: isolate;
}

/* The artwork is one shared, content-sized layer: every card reveals its slice
   of the same image, and a single window-wide gradient fades it into the theme
   surface from left to right. Because the gradient lives on the shared layer,
   the fade continues across the card grid instead of restarting per card. */
.log-simple-content.has-cover .dash-card::before {
  content: '';
  position: absolute;
  top: var(--cover-top, 0);
  left: var(--cover-left, 0);
  z-index: -1;
  width: var(--cover-width, 100%);
  height: var(--cover-height, 100%);
  background-image:
    linear-gradient(
      to right,
      rgba(var(--mdui-color-surface-container-high), 0) 0%,
      rgba(var(--mdui-color-surface-container-high), 0) 40%,
      rgba(var(--mdui-color-surface-container-high), 0.35) 58%,
      rgba(var(--mdui-color-surface-container-high), 0.72) 78%,
      rgb(var(--mdui-color-surface-container-high)) 100%
    ),
    var(--log-cover);
  background-position: left top, left top;
  background-repeat: no-repeat;
  background-size: 100% 100%, cover;
  pointer-events: none;
}

/* Text sitting directly on the artwork follows the artwork, not the theme:
   dark covers get light type and light covers get dark type. The whole-cover
   tone is the fallback; applyCoverTones() refines blocks that cross the wash. */
.log-simple-content.has-cover[data-cover-tone='dark'],
.log-simple-content.has-cover .dash-card [data-log-tone='dark'] {
  --log-fg: #fff;
  --log-muted: rgba(255, 255, 255, 0.8);
  --log-faint: rgba(255, 255, 255, 0.62);
  --log-grid: rgba(255, 255, 255, 0.22);
  --log-line: rgba(255, 255, 255, 0.36);
  --log-accent: color-mix(in srgb, rgb(var(--mdui-color-primary)) 68%, white);
  --log-accent-2: color-mix(in srgb, rgb(var(--mdui-color-tertiary)) 68%, white);
  --log-text-shadow: 0 1px 2px rgba(0, 0, 0, 0.62), 0 0 7px rgba(0, 0, 0, 0.36);
}

.log-simple-content.has-cover[data-cover-tone='light'],
.log-simple-content.has-cover .dash-card [data-log-tone='light'] {
  --log-fg: #17161b;
  --log-muted: rgba(23, 22, 27, 0.78);
  --log-faint: rgba(23, 22, 27, 0.6);
  --log-grid: rgba(23, 22, 27, 0.16);
  --log-line: rgba(23, 22, 27, 0.28);
  --log-accent: color-mix(in srgb, rgb(var(--mdui-color-primary)) 78%, black);
  --log-accent-2: color-mix(in srgb, rgb(var(--mdui-color-tertiary)) 78%, black);
  --log-text-shadow: 0 1px 2px rgba(255, 255, 255, 0.55);
}

.log-simple-content.has-cover .dash-card :is(
    .dash-card-title,
    .dash-card-sub,
    .dash-metric,
    .dash-meta,
    .dash-config-grid dt,
    .dash-config-grid dd:not(.dash-path),
    .dash-progress-meta,
    .dash-chart-axis,
    .stage-name,
    .stage-time,
    .dash-empty
  ),
.log-simple-content.has-cover .dash-card [data-log-tone] {
  text-shadow: var(--log-text-shadow, none);
}

/* These labels carry their tone per span, because the left and right ends of
   the same row can sit on opposite sides of the wash. */
.log-simple-content.has-cover .dash-card .dash-progress-meta > span,
.log-simple-content.has-cover .dash-card .dash-chart-axis > span {
  color: var(--log-muted);
}

.log-issue {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px 16px;
  border: 1px solid transparent;
  border-radius: var(--mdui-shape-corner-large);
}

.log-issue.is-error {
  border-color: rgba(var(--mdui-color-error), 0.28);
  background-color: rgb(var(--mdui-color-error-container));
  color: rgb(var(--mdui-color-on-error-container));
}

.log-issue.is-warn {
  border-color: rgba(var(--mdui-color-warning), 0.34);
  background-color: rgb(var(--mdui-color-warning-container));
  color: rgb(var(--mdui-color-on-warning-container));
}

.log-issue-icon {
  flex-shrink: 0;
  font-size: 22px;
}

.log-issue-text {
  flex: 1;
  min-width: 0;
}

.log-issue-title {
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: 600;
  line-height: var(--mdui-typescale-title-small-line-height);
}

.log-issue-preview {
  display: -webkit-box;
  margin-top: 2px;
  overflow: hidden;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: 11.5px;
  line-height: 1.5;
  opacity: 0.85;
}

.dashboard {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.dashboard-top,
.dashboard-bottom {
  display: grid;
  grid-template-columns: minmax(0, 0.92fr) minmax(0, 1.08fr);
  gap: 16px;
}

.dashboard-stack {
  display: grid;
  grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
  gap: 16px;
  min-width: 0;
}

.dash-card {
  min-width: 0;
  padding: 16px 18px;
  border: 1px solid rgba(var(--mdui-color-outline-variant), 0.62);
  border-radius: var(--mdui-shape-corner-large);
  background-color: rgb(var(--mdui-color-surface-container-high));
}

.dash-card-compact {
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.dash-card-head {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  margin-bottom: 14px;
}

.dash-card-compact .dash-card-head {
  margin-bottom: 10px;
}

.dash-icon {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  border-radius: var(--mdui-shape-corner-small);
  background-color: rgb(var(--mdui-color-primary-container));
  color: rgb(var(--mdui-color-on-primary-container));
}

.dash-icon.is-tertiary {
  background-color: rgb(var(--mdui-color-tertiary-container));
  color: rgb(var(--mdui-color-on-tertiary-container));
}

.dash-icon.is-secondary {
  background-color: rgb(var(--mdui-color-secondary-container));
  color: rgb(var(--mdui-color-on-secondary-container));
}

.dash-icon mdui-icon-memory--outlined,
.dash-icon mdui-icon-hd--outlined,
.dash-icon mdui-icon-video-settings--outlined,
.dash-icon mdui-icon-data-usage--outlined,
.dash-icon mdui-icon-timer--outlined {
  font-size: 19px;
}

.dash-card-heading {
  min-width: 0;
}

.dash-card-title {
  margin: 0;
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: 600;
  line-height: var(--mdui-typescale-title-small-line-height);
  color: var(--log-fg);
}

.dash-card-sub {
  margin: 1px 0 0;
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  color: var(--log-muted);
}

.dash-metric {
  font-size: var(--mdui-typescale-title-large-size);
  font-weight: 600;
  line-height: 1.2;
  color: var(--log-fg);
  overflow-wrap: anywhere;
}

.dash-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: var(--log-muted);
}

.dash-config-card {
  display: flex;
  flex-direction: column;
}

.dash-config-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px 24px;
  margin: 0;
}

.dash-config-grid > div {
  min-width: 0;
}

.dash-config-grid .is-wide {
  grid-column: 1 / -1;
}

.dash-config-grid dt {
  margin-bottom: 2px;
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  color: var(--log-muted);
}

.dash-config-grid dd {
  margin: 0;
  overflow: hidden;
  color: var(--log-fg);
  font-size: var(--mdui-typescale-body-medium-size);
  font-weight: 500;
  line-height: var(--mdui-typescale-body-medium-line-height);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dash-config-grid .dash-path {
  /* The path runs across both the artwork and the theme wash, so it gets the
     same frosted surface as the tiles instead of a single cover tone. */
  display: inline-block;
  max-width: 100%;
  padding: 3px 7px;
  border: 1px solid var(--log-tile-line);
  border-radius: var(--mdui-shape-corner-extra-small);
  background-color: color-mix(in srgb, var(--log-tile-bg) 76%, transparent);
  -webkit-backdrop-filter: blur(10px) saturate(1.3);
  backdrop-filter: blur(10px) saturate(1.3);
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: 11.5px;
  font-weight: 400;
  color: var(--log-tile-muted);
}

/* ===== Progress card ===== */
.dash-progress-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: var(--mdui-typescale-label-medium-size);
  font-weight: 500;
  line-height: var(--mdui-typescale-label-medium-line-height);
  font-variant-numeric: tabular-nums;
  color: var(--log-muted);
  white-space: nowrap;
}

.dash-chart-wrap {
  margin-top: 20px;
}

.dash-chart {
  --chart-pad-top: 9px;

  position: relative;
  display: flex;
  align-items: flex-end;
  gap: 0;
  height: 124px;
  padding: var(--chart-pad-top) 0 0;
  border-radius: var(--mdui-shape-corner-small) var(--mdui-shape-corner-small) 0 0;
  background:
    linear-gradient(to top, color-mix(in srgb, var(--log-accent) 9%, transparent), transparent 72%),
    repeating-linear-gradient(
      to top,
      transparent 0,
      transparent calc(25% - 1px),
      var(--log-grid) calc(25% - 1px),
      var(--log-grid) 25%
    );
  border-bottom: 1px solid var(--log-line);
}

/* Overlays the exact area the bars are scaled against, so a percentage along
   either axis means the same thing for the bars and the average marker. */
.dash-chart-plot {
  position: absolute;
  inset: var(--chart-pad-top) 0 0;
  pointer-events: none;
}

.dash-chart-col {
  position: relative;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  flex: 1;
  height: 100%;
  min-width: 0;
}

.dash-chart-bar {
  width: 100%;
  min-height: 5px;
  border-radius: 1px 1px 0 0;
  background: linear-gradient(
    to top,
    var(--log-accent),
    color-mix(in srgb, var(--log-accent) 82%, transparent)
  );
  transition-property: filter, transform;
  transition-duration: var(--app-motion-duration-effects-fast, 120ms);
  transition-timing-function: var(--mdui-motion-easing-standard);
}

.dash-chart-col:hover .dash-chart-bar {
  filter: brightness(1.12);
}

/* The marker and its label take the cover colour that contrasts most with the
   chart bars; the halo flips to the opposite ink for the lighter background. */
.dash-chart-average {
  position: absolute;
  right: 0;
  left: 0;
  z-index: 1;
  border-top: 1px dashed var(--log-cover-ink);
  filter: drop-shadow(0 0 1px var(--log-cover-halo));
  pointer-events: none;
}

.dash-chart-average::after {
  content: attr(data-label);
  position: absolute;
  top: -15px;
  right: 0;
  padding: 0 4px;
  border-radius: 2px;
  color: var(--log-cover-ink);
  font-size: 9.5px;
  font-weight: 600;
  line-height: 1.3;
  text-shadow:
    0 0 2px var(--log-cover-halo),
    0 0 5px var(--log-cover-halo);
}

/* Floating hover readout. It is fixed (and painted above the cards) so a card's
   overflow clip cannot cut it off, with the same frosted treatment as the tiles. */
.chart-tooltip {
  position: fixed;
  z-index: 3100;
  padding: 5px 9px;
  border: 1px solid rgba(var(--mdui-color-outline-variant), 0.55);
  border-radius: var(--mdui-shape-corner-extra-small);
  background-color: rgba(var(--mdui-color-surface-container-high), 0.68);
  color: rgb(var(--mdui-color-on-surface));
  font-size: 11px;
  line-height: 1.4;
  white-space: nowrap;
  box-shadow: var(--mdui-elevation-level2);
  pointer-events: none;
  transform: translate(-50%, -100%);
  -webkit-backdrop-filter: blur(14px) saturate(1.4);
  backdrop-filter: blur(14px) saturate(1.4);
}

.chart-tooltip.is-below {
  transform: translate(-50%, 0);
}

.dash-chart-axis {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-top: 6px;
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  color: var(--log-muted);
}

.dash-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
  margin-top: 16px;
}

/* Stat tiles are frosted glass: the cover artwork stays visible underneath,
   softened by a blur so the numbers keep a quiet backdrop. */
.dash-stats > div {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border: 1px solid var(--log-tile-line);
  border-radius: var(--mdui-shape-corner-small);
  background-color: color-mix(in srgb, var(--log-tile-bg) 82%, transparent);
  -webkit-backdrop-filter: blur(14px) saturate(1.4);
  backdrop-filter: blur(14px) saturate(1.4);
}

.dash-stats span {
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  color: var(--log-tile-muted);
}

.dash-stats strong {
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: 600;
  line-height: var(--mdui-typescale-title-small-line-height);
  font-variant-numeric: tabular-nums;
  color: var(--log-tile-fg);
}

/* ===== Stage card ===== */
.dash-totals {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  margin-bottom: 16px;
}

.dash-totals > div {
  padding: 10px 12px;
  border: 1px solid var(--log-tile-line);
  border-radius: var(--mdui-shape-corner-small);
  background-color: color-mix(in srgb, var(--log-tile-bg) 82%, transparent);
  -webkit-backdrop-filter: blur(14px) saturate(1.4);
  backdrop-filter: blur(14px) saturate(1.4);
}

.dash-totals span {
  display: block;
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  color: var(--log-tile-muted);
}

.dash-totals strong {
  display: block;
  margin-top: 2px;
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: 600;
  line-height: var(--mdui-typescale-title-medium-line-height);
  font-variant-numeric: tabular-nums;
  color: var(--log-tile-fg);
}

.stage-legend {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 5px 18px;
}

.stage-legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  max-width: 100%;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
}

.stage-dot {
  width: 6px;
  height: 6px;
  flex-shrink: 0;
  border-radius: 50%;
}

.stage-dot.setup {
  background-color: var(--log-accent-2);
}

.stage-dot.render {
  background-color: var(--log-accent);
}

.stage-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  color: var(--log-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stage-time {
  flex-shrink: 0;
  color: var(--log-faint);
  font-size: var(--mdui-typescale-label-medium-size);
  font-variant-numeric: tabular-nums;
}

.dash-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 150px;
  color: var(--log-muted);
  font-size: var(--mdui-typescale-body-small-size);
}

.dash-empty mdui-icon-data-usage--outlined,
.dash-empty mdui-icon-timer--outlined {
  font-size: 2rem;
  color: var(--log-faint);
}

/* ===== Detailed log ===== */
.log-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 8px 0;
  background-color: rgb(var(--mdui-color-surface-container-lowest));
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: 12.5px;
  line-height: 1.55;
}

.log-line {
  display: grid;
  grid-template-columns: 42px 52px 62px minmax(0, 1fr);
  gap: 10px;
  align-items: start;
  padding: 2px 20px 2px 14px;
  border-left: 2px solid transparent;
  cursor: pointer;
}

.log-line:hover {
  background-color: rgba(var(--mdui-color-on-surface), 0.045);
}

.log-line.is-info {
  border-left-color: rgba(var(--mdui-color-primary), 0.2);
}

.log-line.is-warn {
  border-left-color: rgb(var(--mdui-color-warning));
  background-color: rgba(var(--mdui-color-warning), 0.055);
}

.log-line.is-error {
  border-left-color: rgb(var(--mdui-color-error));
  background-color: rgba(var(--mdui-color-error), 0.07);
}

.log-line.is-selected,
.log-line.is-selected:hover {
  background-color: rgba(var(--mdui-color-primary), 0.16);
}

.log-no {
  color: rgb(var(--mdui-color-outline));
  text-align: right;
  user-select: none;
  font-variant-numeric: tabular-nums;
}

.log-level,
.log-stage {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 20px;
  margin-top: 1px;
  border-radius: var(--mdui-shape-corner-full);
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.02em;
  line-height: 1;
  white-space: nowrap;
}

.log-level {
  padding: 0 7px;
}

.log-level.is-info {
  background-color: rgba(var(--mdui-color-primary), 0.12);
  color: rgb(var(--mdui-color-primary));
}

.log-level.is-warn {
  background-color: rgb(var(--mdui-color-warning-container));
  color: rgb(var(--mdui-color-on-warning-container));
}

.log-level.is-error {
  background-color: rgb(var(--mdui-color-error-container));
  color: rgb(var(--mdui-color-on-error-container));
}

.log-stage {
  padding: 0 6px;
  font-weight: 500;
}

.log-stage.is-progress {
  background-color: rgba(var(--mdui-color-primary), 0.1);
  color: rgb(var(--mdui-color-primary));
}

.log-stage.is-summary {
  background-color: rgba(var(--mdui-color-tertiary), 0.12);
  color: rgb(var(--mdui-color-tertiary));
}

.log-stage.is-env {
  background-color: rgba(var(--mdui-color-secondary), 0.13);
  color: rgb(var(--mdui-color-secondary));
}

.log-stage.is-plain {
  background-color: rgb(var(--mdui-color-surface-container-high));
  color: rgb(var(--mdui-color-on-surface-variant));
}

.log-text {
  min-width: 0;
  color: rgb(var(--mdui-color-on-surface));
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.log-line.is-error .log-text {
  color: rgb(var(--mdui-color-error));
}

.log-text mark {
  padding: 0 1px;
  border-radius: 2px;
  background-color: rgb(var(--mdui-color-warning-container));
  color: rgb(var(--mdui-color-on-warning-container));
}

/* ===== Footer ===== */
.log-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  padding: 9px 16px;
  border-top: 1px solid rgba(var(--mdui-color-outline-variant), 0.5);
  background-color: rgb(var(--mdui-color-surface-container-high));
}

.log-footer-stats {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-wrap: wrap;
}

.log-stat {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--mdui-shape-corner-full);
  background: transparent;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-family: inherit;
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  cursor: pointer;
  transition-property: background-color, color;
  transition-duration: var(--app-motion-duration-effects-fast, 120ms);
  transition-timing-function: var(--mdui-motion-easing-standard);
}

.log-stat:hover {
  background-color: rgba(var(--mdui-color-on-surface), 0.06);
}

.log-stat.active {
  background-color: rgb(var(--mdui-color-surface-container-highest));
  color: rgb(var(--mdui-color-on-surface));
}

.log-stat strong {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.log-dot {
  width: 7px;
  height: 7px;
  flex-shrink: 0;
  border-radius: 50%;
  background-color: rgb(var(--mdui-color-outline));
}

.log-dot.is-info {
  background-color: rgb(var(--mdui-color-primary));
}

.log-dot.is-warn {
  background-color: rgb(var(--mdui-color-warning));
}

.log-dot.is-error {
  background-color: rgb(var(--mdui-color-error));
}

.log-line-count {
  margin-left: auto;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-size: var(--mdui-typescale-label-small-size);
  line-height: var(--mdui-typescale-label-small-line-height);
  white-space: nowrap;
}

.log-line-count.is-selected {
  color: rgb(var(--mdui-color-primary));
  font-weight: 600;
}

.log-footer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* Tonal buttons use the secondary container, which can collapse to white in
   some themes. Paint a primary tint directly so they still read as buttons. */
.log-footer-actions mdui-button:not([disabled]) {
  color: rgb(var(--mdui-color-primary));
  background-color: color-mix(
    in srgb,
    rgb(var(--mdui-color-primary)) 12%,
    rgb(var(--mdui-color-surface-container-high))
  );
}

.log-footer.is-simple .log-footer-actions {
  margin-left: auto;
}

/* ===== Responsive ===== */
@include bp.below-expanded {
  .dashboard-top,
  .dashboard-bottom {
    grid-template-columns: minmax(0, 1fr);
  }

  .dashboard-stack {
    grid-template-rows: auto auto;
  }
}

@include bp.compact {
  .log-backdrop {
    padding: 0;
  }

  .log-window {
    width: 100%;
    height: 100%;
    border-radius: 0;
  }

  .log-header {
    padding: 12px;
  }

  .log-title {
    font-size: var(--mdui-typescale-title-medium-size);
  }

  .log-toolbar {
    flex-wrap: wrap;
    padding: 10px 12px;
  }

  .log-mode {
    width: 100%;
  }

  .log-search {
    width: 100%;
    margin-left: 0;
  }

  .log-simple-content {
    padding: 12px;
  }

  .log-issue {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .dash-card {
    padding: 14px;
  }

  .dash-config-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .dash-config-grid .is-wide {
    grid-column: auto;
  }

  .dash-stats {
    gap: 6px;
  }

  .log-line {
    grid-template-columns: 34px 48px minmax(0, 1fr);
    gap: 7px;
    padding: 2px 12px 2px 10px;
  }

  .log-stage {
    display: none;
  }

  .log-footer {
    padding: 8px 12px;
  }

  .log-line-count {
    order: 3;
    width: 100%;
    margin-left: 0;
  }

  .log-footer-actions {
    margin-left: auto;
  }
}
</style>
