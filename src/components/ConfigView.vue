<i18n>
en:
  resolution: Resolution
  ffmpeg-preset: Preset
  video-codec: Encoder
  encoder-select: Hardware Encoder
  encoder-auto: Auto
  encoder-nvenc: NVIDIA NVENC
  encoder-qsv: Intel QSV
  encoder-amf: AMD AMF
  encoder-vulkan: Vulkan
  encoder-cpu: CPU Software
  fps: FPS
  hw-accel: Hardware Acceleration
  hw-accel-tips: Improve speed, slightly reduce quality
  fxaa: FXAA
  fxaa-tips: Causes blur, not recommended
  sample-count: Sample Count
  sample-count-tips: Power of 2 for MSAA
  bitrate-control: Bitrate Control
  bitrate: Bitrate/CRF
  ffmpeg-thread: Thread Optimization
  video-format: Video Format
  video-format-flac-error: Not available with FLAC audio
  player-avatar: Avatar
  player-name: Name
  player-rks: RKS
  challenge-color: Challenge Color
  challenge-colors: White,Green,Blue,Red,Golden,Rainbow
  challenge-rank: Challenge Rank
  respack: Resource Pack
  respack-default: '[Default]'
  note-scale: Note Scale
  render: UI Display
  double-hint: Double Hint
  aggressive: Aggressive Optimization
  aggressive-tips: May cause notes to disappear
  disable-particle: No Particles
  disable-effect: No Effects
  volume-music: Music Volume
  volume-sfx: SFX Volume
  audio-format: Audio Format
  audio-bit-depth: Bit Depth
  ending-length: Ending Duration
  disable-loading: Skip Loading
  disable-loading-tips: May have issues
  chart_debug: Debug Mode
  chart_ratio: Chart Scale
  buffer_size: Buffer Size
  target_audio: Sample Rate
  combo: COMBO Text
  watermark: Watermark
  flid_x: Mirror
  background: BG Only
  hand-split: Hand Split
  note-speed-factor: Note Speed
  bar: Judge Bar
  render-list: Judge Line,Score,Combo,Level,Name,Progress,Percent,Time,Pause
  presets: Presets
  preset-create-title: Preset name
  preset-created: Preset created
  preset-deleted: Preset deleted
  preset-replaced: Preset replaced
  preset-cannot-use-default: Cannot use 'default'
  default-preset: Default
  back: Back

zh-CN:
  resolution: 分辨率
  ffmpeg-preset: 编码预设
  video-codec: 编码器
  encoder-select: 硬件编码器
  encoder-auto: 自动选择
  encoder-nvenc: NVIDIA NVENC
  encoder-qsv: Intel QSV
  encoder-amf: AMD AMF
  encoder-vulkan: Vulkan
  encoder-cpu: CPU软编码
  fps: 帧率
  hw-accel: 硬件加速
  hw-accel-tips: 提升速度，略微降低质量
  fxaa: FXAA抗锯齿
  fxaa-tips: 会导致画面模糊
  sample-count: 采样数
  sample-count-tips: 非1启用MSAA
  bitrate-control: 码率控制
  bitrate: 码率/CRF
  ffmpeg-thread: 线程优化
  video-format: 视频格式
  video-format-flac-error: FLAC音频下不可用
  player-avatar: 头像
  player-name: 名称
  player-rks: RKS
  challenge-color: 课题颜色
  challenge-colors: 白,绿,蓝,红,金,彩
  challenge-rank: 课题等级
  respack: 资源包
  respack-default: '[默认]'
  note-scale: 音符缩放
  render: UI显示
  double-hint: 双押提示
  aggressive: 激进优化
  aggressive-tips: 可能导致音符消失
  disable-particle: 禁用粒子
  disable-effect: 禁用特效
  volume-music: 音乐音量
  volume-sfx: 音效音量
  audio-format: 音频格式
  audio-bit-depth: 位深度
  ending-length: 结算时长
  disable-loading: 渲染时跳过加载
  disable-loading-tips: 可能有问题
  chart_debug: 调试模式
  chart_ratio: 谱面缩放
  buffer_size: 缓冲区
  target_audio: 采样率
  combo: COMBO文本
  watermark: 水印
  flid_x: 镜像
  background: 仅背景显示
  hand-split: 手序拆解
  note-speed-factor: 流速
  bar: 判定条显示
  render-list: 判定线,分数,连击,等级,名字,进度,百分比,时间,暂停
  presets: 预设
  preset-create-title: 预设名称
  preset-created: 预设已创建
  preset-deleted: 预设已删除
  preset-replaced: 预设已替换
  preset-cannot-use-default: 不能使用default
  default-preset: 默认
  back: 返回
</i18n>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { VForm } from 'vuetify/components';
import { toast, anyFilter, toastError } from '../common';
import type { RenderConfig } from '../model';
import TipSwitch from './TipSwitch.vue';

const props = defineProps<{ initAspectRatio?: number }>();

const RESOLUTIONS = ['1280x720', '1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080', '7680x4320'];
const FFMPEG_PRESETS = ['veryfast', 'faster', 'fast', 'medium', 'slow', 'slower', 'veryslow'];
const VIDEO_CODECS = [
  { value: 'h264', title: 'H.264' },
  { value: 'hevc', title: 'HEVC' },
  { value: 'av1', title: 'AV1' },
];
const ENCODERS = [
  { value: 'auto', title: '自动选择', desc: '自动检测最佳编码器' },
  { value: 'nvenc', title: 'NVIDIA NVENC', desc: 'NVIDIA GPU 硬件加速' },
  { value: 'qsv', title: 'Intel QSV', desc: 'Intel Quick Sync Video' },
  { value: 'amf', title: 'AMD AMF', desc: 'AMD Advanced Media Framework' },
  { value: 'vulkan', title: 'Vulkan', desc: '跨平台 GPU 编码' },
  { value: 'cpu', title: 'CPU 软编码', desc: 'libx264/libx265/libaom-av1' },
];
const AUDIO_FORMATS = ['flac', 'mp3', 'aac', 'opus', 'wav'];
const AUDIO_BITS = [16, 24, 32];
const SAMPLE_RATES = [44100, 48000, 96000, 192000, 384000, 768000];
const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

const form = ref<VForm>();
const activeCategory = ref('output');

const categories = [
  { key: 'output', icon: 'mdi-video-outline', label: '输出' },
  { key: 'recording', icon: 'mdi-music-note', label: '录制' },
  { key: 'game', icon: 'mdi-gamepad-variant-outline', label: '游戏' },
  { key: 'graphics', icon: 'mdi-palette-outline', label: '效果' },
  { key: 'player', icon: 'mdi-account-outline', label: '玩家' },
  { key: 'debug', icon: 'mdi-bug-outline', label: '调试' },
];

const hoveredField = ref<string | null>(null);
function setHover(field: string) {
  hoveredField.value = field;
}
function clearHover() {
  hoveredField.value = null;
}

const resolution = ref('1920x1080');
const fps = ref('60');
const sampleCount = ref('1');
const videoCodec = ref('h264');
const encoder = ref('auto');
const ffmpegPreset = ref('medium');
const videoFormat = ref('mp4');
const bitrateControl = ref('CRF');
const bitrate = ref('28');
const hwAccel = ref(true);
const fxaa = ref(false);
const ffmpegThread = ref(false);
const audioFormat = ref('flac');
const audioBit = ref<number | undefined>(undefined);
const targetAudio = ref(48000);
const volumeMusic = ref(1);
const volumeSfx = ref(1);
const bufferSize = ref(256);
const combo = ref('AUTOPLAY');
const watermark = ref('');
const endingLength = ref('-2.0');
const chartRatio = ref(1.0);
const noteSpeedFactor = ref(1.0);
const background = ref(false);
const bar = ref(false);
const noteScale = ref(1);
const doubleHint = ref(true);
const aggressive = ref(false);
const disableParticle = ref(false);
const disableEffect = ref(false);
const render = ref<string[]>([]);
const DEFAULT_RENDER_LIST = ['Judge Line', 'Score', 'Combo', 'Level', 'Name', 'Progress', 'Percent', 'Time', 'Pause'];
const renderList = ref(t('render-list').split(',').length === 9 ? t('render-list').split(',') : DEFAULT_RENDER_LIST);
function toggleRender(item: string) {
  const index = render.value.indexOf(item);
  if (index > -1) render.value.splice(index, 1);
  else render.value.push(item);
}

const playerAvatar = ref<string>();
const playerName = ref('');
const playerRks = ref('15.0');
const challengeRank = ref('45');
const challengeColor = ref(t('challenge-colors').split(',')[4]);
const chartDebug = ref(false);
const flidX = ref(false);
const handSplit = ref(false);
const disableLoading = ref(false);

const video = ref(false);
watch([audioFormat, video], ([newAudio, newVideo]) => {
  if (newAudio === 'flac' && newVideo) {
    video.value = false;
    videoFormat.value = 'mp4';
    toast(t('video-format-flac-error'), 'error');
  }
});
watch(videoFormat, (val) => {
  video.value = val === 'mov';
});

interface Respack {
  name: string;
  path: string | null;
  index: number;
}
const DEFAULT_RESPACK: Respack = { name: t('respack-default'), path: null, index: 0 };
const respacks = ref([DEFAULT_RESPACK]);
const respack = ref(DEFAULT_RESPACK);
async function updateRespacks() {
  const list = (await invoke('get_respacks')) as { name: string; path: string }[];
  respacks.value = [DEFAULT_RESPACK, ...list.map((obj, i) => ({ name: obj.name, path: obj.path, index: i + 1 }))];
  respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0];
}
updateRespacks();

interface Preset {
  name: string;
  key: string;
  config: RenderConfig;
}
const DEFAULT_PRESET: Preset = {
  name: t('default-preset'),
  key: 'default',
  config: {
    resolution: [1920, 1080],
    ffmpegPreset: 'medium',
    endingLength: -2.0,
    disableLoading: true,
    chartDebug: false,
    flidX: false,
    chartRatio: 1,
    bufferSize: 256,
    fps: 60,
    hardwareAccel: true,
    videoCodec: 'h264',
    encoder: 'auto',
    bitrateControl: 'CRF',
    bitrate: '28',
    targetAudio: 48000,
    video: false,
    audioBit: undefined,
    audioFormat: 'flac',
    background: false,
    aggressive: false,
    challengeColor: 'golden',
    challengeRank: 45,
    disableEffect: false,
    doubleHint: true,
    fxaa: false,
    noteScale: 1,
    particle: true,
    playerAvatar: null,
    playerName: '',
    playerRks: 15,
    sampleCount: 1,
    resPackPath: null,
    speed: 1,
    volumeMusic: 1,
    volumeSfx: 1,
    combo: 'AUTOPLAY',
    watermark: '',
    ffmpegThread: false,
    showProgressText: false,
    showTimeText: false,
    handSplit: false,
    noteSpeedFactor: 1.0,
    uiLine: true,
    uiScore: true,
    uiCombo: true,
    uiLevel: true,
    uiName: true,
    uiPb: true,
    uiPause: true,
    bar: false,
  },
};
const presets = ref([DEFAULT_PRESET]);
const preset = ref(DEFAULT_PRESET);

async function updatePresets() {
  const currentKey = preset.value.key;
  const pairs = (await invoke('get_presets')) as Record<string, RenderConfig>;
  presets.value = [
    DEFAULT_PRESET,
    ...Object.keys(pairs)
      .sort()
      .map((key) => ({ name: key, key, config: pairs[key] })),
  ];
  const found = presets.value.find((x) => x.key === currentKey);
  preset.value = found || presets.value[0];
  applyConfig(preset.value.config);
}
updatePresets();

async function chooseAvatar() {
  const file = await open({ filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'] }, anyFilter()] });
  if (file) playerAvatar.value = file as string;
}
async function openRespackFolder() {
  try {
    await invoke('open_respack_folder');
  } catch (e) {
    toastError(e);
  }
}

// Preview computations
const aspectPreview = computed(() => {
  const [w, h] = resolution.value.split('x').map(Number);
  if (!w || !h || isNaN(w) || isNaN(h)) return { w: 16, h: 9, label: '16:9' };
  const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
  const g = gcd(w, h);
  return { w: w / g, h: h / g, label: `${w / g}:${h / g}` };
});

const pixelGrid = computed(() => {
  const [w, h] = resolution.value.split('x').map(Number);
  if (!w || !h || isNaN(w) || isNaN(h)) return { cols: 32, rows: 18, dots: 576 };
  const scale = Math.min(48 / w, 36 / h, 0.025);
  const cols = Math.max(8, Math.round(w * scale));
  const rows = Math.max(6, Math.round(h * scale));
  return { cols, rows, dots: cols * rows };
});

const presetSpeed = computed(() => {
  const speeds: Record<string, number> = { veryfast: 95, faster: 85, fast: 70, medium: 50, slow: 35, slower: 20, veryslow: 8 };
  return speeds[ffmpegPreset.value] ?? 50;
});

const presetQuality = computed(() => 100 - presetSpeed.value);

const challengeColorHex = computed(() => {
  const colors: Record<string, string> = {
    white: '#e0e0e0',
    green: '#4caf50',
    blue: '#2196f3',
    red: '#f44336',
    golden: '#ffc107',
    rainbow: 'linear-gradient(90deg, #f44336, #ff9800, #ffeb3b, #4caf50, #2196f3, #9c27b0)',
  };
  const idx = t('challenge-colors').split(',').indexOf(challengeColor.value);
  return colors[STD_CHALLENGE_COLORS[idx]] || colors.golden;
});

async function buildConfig(): Promise<RenderConfig | null> {
  if (!(await form.value!.validate()).valid) {
    toast(t('has-error'), 'error');
    return null;
  }
  const [w, h] = resolution.value.split('x').map(Number);
  return {
    resolution: [w, h],
    ffmpegPreset: ffmpegPreset.value,
    endingLength: parseFloat(endingLength.value),
    disableLoading: disableLoading.value,
    chartDebug: chartDebug.value,
    flidX: flidX.value,
    chartRatio: chartRatio.value,
    bufferSize: bufferSize.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    videoCodec: videoCodec.value,
    encoder: encoder.value,
    bitrateControl: bitrateControl.value,
    bitrate: bitrate.value,
    targetAudio: targetAudio.value,
    background: background.value,
    handSplit: handSplit.value,
    noteSpeedFactor: noteSpeedFactor.value,
    video: video.value,
    audioBit: audioFormat.value === 'wav' ? audioBit.value : undefined,
    audioFormat: audioFormat.value,
    bar: bar.value,
    aggressive: aggressive.value,
    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)],
    challengeRank: parseInt(challengeRank.value),
    disableEffect: disableEffect.value,
    doubleHint: doubleHint.value,
    fxaa: fxaa.value,
    noteScale: noteScale.value,
    particle: !disableParticle.value,
    playerAvatar: playerAvatar.value?.length ? playerAvatar.value : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value),
    sampleCount: parseInt(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    combo: combo.value,
    watermark: watermark.value,
    ffmpegThread: ffmpegThread.value,
    uiLine: render.value.includes(renderList.value[0]),
    uiScore: render.value.includes(renderList.value[1]),
    uiCombo: render.value.includes(renderList.value[2]),
    uiLevel: render.value.includes(renderList.value[3]),
    uiName: render.value.includes(renderList.value[4]),
    uiPb: render.value.includes(renderList.value[5]),
    showProgressText: render.value.includes(renderList.value[6]),
    showTimeText: render.value.includes(renderList.value[7]),
    uiPause: render.value.includes(renderList.value[8]),
  };
}

function applyConfig(c: RenderConfig) {
  resolution.value = c.resolution.join('x');
  ffmpegPreset.value = c.ffmpegPreset;
  endingLength.value = String(c.endingLength);
  disableLoading.value = c.disableLoading;
  chartDebug.value = c.chartDebug;
  flidX.value = c.flidX;
  chartRatio.value = c.chartRatio;
  bufferSize.value = c.bufferSize;
  fps.value = String(c.fps);
  hwAccel.value = c.hardwareAccel;
  videoCodec.value = c.videoCodec;
  encoder.value = c.encoder || 'auto';
  bitrateControl.value = c.bitrateControl;
  bitrate.value = c.bitrate;
  targetAudio.value = c.targetAudio;
  background.value = c.background ?? false;
  handSplit.value = c.handSplit ?? false;
  noteSpeedFactor.value = c.noteSpeedFactor;
  video.value = c.video;
  audioFormat.value = c.audioFormat || 'flac';
  audioBit.value = c.audioBit;
  aggressive.value = c.aggressive;
  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(c.challengeColor)];
  challengeRank.value = String(c.challengeRank);
  disableEffect.value = c.disableEffect;
  doubleHint.value = c.doubleHint;
  fxaa.value = c.fxaa;
  noteScale.value = c.noteScale;
  disableParticle.value = !(c.particle ?? true);
  playerAvatar.value = c.playerAvatar || undefined;
  playerName.value = c.playerName;
  playerRks.value = String(c.playerRks);
  sampleCount.value = String(c.sampleCount);
  respack.value = respacks.value.find((x) => x.path === c.resPackPath) || respacks.value[0];
  volumeMusic.value = c.volumeMusic;
  volumeSfx.value = c.volumeSfx;
  combo.value = c.combo;
  watermark.value = c.watermark;
  bar.value = c.bar ?? false;
  ffmpegThread.value = c.ffmpegThread ?? false;
  render.value = [];
  const list = renderList.value;
  if ((c.uiLine ?? true) && list[0]) render.value.push(list[0]);
  if ((c.uiScore ?? true) && list[1]) render.value.push(list[1]);
  if ((c.uiCombo ?? true) && list[2]) render.value.push(list[2]);
  if ((c.uiLevel ?? true) && list[3]) render.value.push(list[3]);
  if ((c.uiName ?? true) && list[4]) render.value.push(list[4]);
  if ((c.uiPb ?? true) && list[5]) render.value.push(list[5]);
  if ((c.showProgressText ?? false) && list[6]) render.value.push(list[6]);
  if ((c.showTimeText ?? false) && list[7]) render.value.push(list[7]);
  if ((c.uiPause ?? true) && list[8]) render.value.push(list[8]);
}
defineExpose({ buildConfig, applyConfig });

async function createPreset() {
  const config = await buildConfig();
  if (!config) return;
  const name = prompt(t('preset-create-title'));
  if (!name) return;
  if (name === 'default') {
    toast(t('preset-cannot-use-default'), 'error');
    return;
  }
  try {
    await invoke('add_preset', { name, config });
    await updatePresets();
    preset.value = presets.value.find((x) => x.key === name) || presets.value[0];
    toast(t('preset-created'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function deletePreset() {
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await updatePresets();
    toast(t('preset-deleted'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function replacePreset() {
  const config = await buildConfig();
  if (!config) return;
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await invoke('add_preset', { name: preset.value.key, config });
    await updatePresets();
    toast(t('preset-replaced'), 'success');
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <v-form ref="form" class="config-root" @submit.prevent>
    <!-- Category chips -->
    <div class="cat-bar">
      <button v-for="cat in categories" :key="cat.key" type="button" class="cat-chip" :class="{ 'is-active': activeCategory === cat.key }" @click="activeCategory = cat.key">
        <v-icon :icon="cat.icon" size="18" />
        <span>{{ cat.label }}</span>
      </button>
    </div>

    <!-- Two-column body: settings + preview -->
    <div class="config-body">
      <!-- LEFT: Settings scroll -->
      <div class="settings-col">
        <div class="settings-scroll">
          <!-- OUTPUT -->
          <div v-show="activeCategory === 'output'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">分辨率与帧率</div>
              <div class="field-row"><v-combobox v-model="resolution" :items="RESOLUTIONS" :label="t('resolution')" density="compact" variant="outlined" hide-details /></div>
              <div class="field-row two-col">
                <v-text-field v-model="fps" :label="t('fps')" density="compact" variant="outlined" hide-details />
                <v-text-field v-model="sampleCount" :label="t('sample-count')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">编码器</div>
              <div class="field-row two-col">
                <v-select
                  v-model="videoCodec"
                  :items="VIDEO_CODECS"
                  item-title="title"
                  item-value="value"
                  :label="t('video-codec')"
                  density="compact"
                  variant="outlined"
                  hide-details />
                <v-select
                  v-model="encoder"
                  :items="ENCODERS"
                  item-title="title"
                  item-value="value"
                  :label="t('encoder-select')"
                  density="compact"
                  variant="outlined"
                  hide-details />
              </div>
              <div class="field-row two-col">
                <v-select v-model="ffmpegPreset" :items="FFMPEG_PRESETS" :label="t('ffmpeg-preset')" density="compact" variant="outlined" hide-details />
                <v-select v-model="videoFormat" :items="['mp4', 'mov']" :label="t('video-format')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">码率</div>
              <div class="field-row two-col">
                <v-select v-model="bitrateControl" :items="['CRF', 'CBR']" :label="t('bitrate-control')" density="compact" variant="outlined" hide-details />
                <v-text-field v-model="bitrate" :label="t('bitrate')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">性能</div>
              <div class="switch-grid">
                <TipSwitch v-model="hwAccel" :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" density="compact" color="primary" />
                <TipSwitch v-model="fxaa" :label="t('fxaa')" :tooltip="t('fxaa-tips')" density="compact" color="primary" />
                <TipSwitch v-model="ffmpegThread" :label="t('ffmpeg-thread')" density="compact" color="primary" />
              </div>
            </div>
          </div>

          <!-- RECORDING -->
          <div v-show="activeCategory === 'recording'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">音量</div>
              <div class="slider-row">
                <v-icon icon="mdi-music-note" size="18" />
                <span class="slider-label">{{ t('volume-music') }}</span>
                <v-slider v-model="volumeMusic" :min="0" :max="2" :step="0.01" hide-details color="primary" />
                <span class="slider-val">{{ Math.round(volumeMusic * 100) }}%</span>
              </div>
              <div class="slider-row">
                <v-icon icon="mdi-volume-high" size="18" />
                <span class="slider-label">{{ t('volume-sfx') }}</span>
                <v-slider v-model="volumeSfx" :min="0" :max="2" :step="0.01" hide-details color="primary" />
                <span class="slider-val">{{ Math.round(volumeSfx * 100) }}%</span>
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">音频格式</div>
              <div class="field-row two-col">
                <v-select v-model="audioFormat" :items="AUDIO_FORMATS" :label="t('audio-format')" density="compact" variant="outlined" hide-details />
                <v-select
                  v-model="audioBit"
                  :items="AUDIO_BITS"
                  :label="t('audio-bit-depth')"
                  density="compact"
                  variant="outlined"
                  hide-details
                  :disabled="audioFormat !== 'wav'" />
              </div>
              <div class="field-row two-col">
                <v-select v-model="targetAudio" :items="SAMPLE_RATES" :label="t('target_audio')" density="compact" variant="outlined" hide-details />
                <v-text-field v-model="bufferSize" :label="t('buffer_size')" type="number" density="compact" variant="outlined" hide-details />
              </div>
            </div>
          </div>

          <!-- GAME -->
          <div v-show="activeCategory === 'game'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">谱面缩放与流速</div>
              <div class="slider-block">
                <div class="slider-header">
                  <span>{{ t('chart_ratio') }}</span
                  ><span class="slider-val">{{ chartRatio.toFixed(2) }}</span>
                </div>
                <v-slider v-model="chartRatio" :min="0.05" :max="1" :step="0.01" hide-details color="primary" />
              </div>
              <div class="slider-block">
                <div class="slider-header">
                  <span>{{ t('note-speed-factor') }}</span
                  ><span class="slider-val">{{ noteSpeedFactor.toFixed(2) }}</span>
                </div>
                <v-slider v-model="noteSpeedFactor" :min="0.1" :max="2" :step="0.01" hide-details color="primary" />
              </div>
              <div class="switch-grid">
                <v-switch v-model="background" :label="t('background')" density="compact" color="primary" hide-details />
                <v-switch v-model="bar" :label="t('bar')" density="compact" color="primary" hide-details />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">文本与结算</div>
              <div class="field-row two-col">
                <v-text-field v-model="combo" :label="t('combo')" density="compact" variant="outlined" hide-details />
                <v-text-field v-model="watermark" :label="t('watermark')" density="compact" variant="outlined" hide-details />
              </div>
              <div class="field-row">
                <v-text-field v-model="endingLength" :label="t('ending-length')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
          </div>

          <!-- GRAPHICS -->
          <div v-show="activeCategory === 'graphics'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">资源包</div>
              <div class="respack-row">
                <v-select v-model="respack" :items="respacks" item-title="name" :label="t('respack')" density="compact" variant="outlined" hide-details class="flex-grow-1" />
                <v-btn icon="mdi-refresh" size="small" variant="text" @click="updateRespacks" />
                <v-btn icon="mdi-folder-open" size="small" variant="text" @click="openRespackFolder" />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">音符缩放</div>
              <div class="slider-block">
                <div class="slider-header">
                  <span>{{ t('note-scale') }}</span
                  ><span class="slider-val">{{ noteScale.toFixed(2) }}</span>
                </div>
                <v-slider v-model="noteScale" :min="0" :max="5" :step="0.05" hide-details color="primary" />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">效果开关</div>
              <div class="switch-grid">
                <TipSwitch v-model="doubleHint" :label="t('double-hint')" density="compact" color="primary" />
                <TipSwitch v-model="aggressive" :label="t('aggressive')" :tooltip="t('aggressive-tips')" density="compact" color="primary" />
                <TipSwitch v-model="disableParticle" :label="t('disable-particle')" density="compact" color="primary" />
                <TipSwitch v-model="disableEffect" :label="t('disable-effect')" density="compact" color="primary" />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">UI 显示</div>
              <div class="ui-grid">
                <div v-for="(item, index) in renderList" :key="index" class="ui-chip" :class="{ 'is-on': render.includes(item) }" @click="toggleRender(item)">
                  <v-icon :icon="render.includes(item) ? 'mdi-checkbox-marked' : 'mdi-checkbox-blank-outline'" size="16" />
                  <span>{{ item }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- PLAYER -->
          <div v-show="activeCategory === 'player'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">个人信息</div>
              <div class="avatar-row">
                <v-avatar size="56" color="surface-variant">
                  <v-img v-if="playerAvatar" :src="playerAvatar" />
                  <v-icon v-else icon="mdi-account" size="32" />
                </v-avatar>
                <v-btn @click="chooseAvatar" variant="outlined" size="small">{{ t('player-avatar') }}</v-btn>
              </div>
              <div class="field-row two-col">
                <v-text-field v-model="playerName" :label="t('player-name')" density="compact" variant="outlined" hide-details />
                <v-text-field v-model="playerRks" :label="t('player-rks')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
            <div class="md3-card">
              <div class="card-label">课题模式</div>
              <div class="field-row two-col">
                <v-text-field v-model="challengeRank" :label="t('challenge-rank')" density="compact" variant="outlined" hide-details />
                <v-select v-model="challengeColor" :items="t('challenge-colors').split(',')" :label="t('challenge-color')" density="compact" variant="outlined" hide-details />
              </div>
            </div>
          </div>

          <!-- DEBUG -->
          <div v-show="activeCategory === 'debug'" class="settings-stack">
            <div class="md3-card">
              <div class="card-label">调试选项</div>
              <div class="switch-grid">
                <v-switch v-model="chartDebug" :label="t('chart_debug')" density="compact" color="primary" hide-details />
                <v-switch v-model="flidX" :label="t('flid_x')" density="compact" color="primary" hide-details />
                <v-switch v-model="handSplit" :label="t('hand-split')" density="compact" color="primary" hide-details />
                <TipSwitch v-model="disableLoading" :label="t('disable-loading')" :tooltip="t('disable-loading-tips')" density="compact" color="warning" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Preview panel disabled
      <div class="preview-col">
        <div class="preview-panel">
          <Transition name="preview-fade" mode="out-in">
            <div key="default" class="preview-content preview-default">
              <v-icon icon="mdi-eye-outline" size="48" color="rgba(255,255,255,0.15)" />
              <p class="preview-hint">悬停选项查看预览</p>
            </div>
          </Transition>
        </div>
      </div>
      -->
    </div>

    <!-- Preset bar -->
    <div class="preset-bar">
      <v-select
        :model-value="preset"
        @update:model-value="
          (val: Preset) => {
            preset = val;
            applyConfig(val.config);
          }
        "
        :items="presets"
        item-title="name"
        :label="t('presets')"
        density="compact"
        variant="outlined"
        hide-details
        return-object
        class="preset-select" />
      <div class="preset-actions">
        <v-btn variant="tonal" color="primary" size="small" prepend-icon="mdi-plus" @click="createPreset">新建</v-btn>
        <v-btn variant="tonal" color="warning" size="small" prepend-icon="mdi-content-save" :disabled="preset.key === 'default'" @click="replacePreset">保存</v-btn>
        <v-btn variant="tonal" color="error" size="small" prepend-icon="mdi-delete" :disabled="preset.key === 'default'" @click="deletePreset">删除</v-btn>
      </div>
    </div>
  </v-form>
</template>

<style scoped>
.config-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: transparent;
  font-family: 'Segoe UI', 'Microsoft YaHei', 'PingFang SC', sans-serif;
}

.cat-bar {
  display: flex;
  gap: 8px;
  padding: 10px 16px;
  flex-shrink: 0;
}
.cat-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  background: transparent;
  color: rgba(255, 255, 255, 0.55);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
  font-family: inherit;
}
.cat-chip:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.8);
}
.cat-chip.is-active {
  background: rgba(130, 177, 255, 0.15);
  border-color: rgba(130, 177, 255, 0.4);
  color: #82b1ff;
}

.config-body {
  flex: 1;
  display: flex;
  gap: 12px;
  min-height: 0;
  padding: 0 16px;
  overflow: hidden;
}

/* LEFT column */
.settings-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.settings-scroll {
  flex: 1;
  overflow-y: auto;
  padding-right: 4px;
}
.settings-stack {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* RIGHT preview column */
.preview-col {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}
.preview-panel {
  flex: 1;
  background: rgba(24, 24, 24, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}

.preview-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px;
  width: 100%;
}
.preview-title {
  font-size: 13px;
  font-weight: 600;
  color: rgba(130, 177, 255, 0.8);
  text-transform: uppercase;
  letter-spacing: 1px;
  align-self: flex-start;
}
.preview-default {
  opacity: 0.6;
}
.preview-hint {
  color: rgba(255, 255, 255, 0.35);
  font-size: 14px;
  margin: 8px 0 0;
}

/* Preview transition */
.preview-fade-enter-active {
  transition:
    opacity 0.25s ease,
    transform 0.25s ease;
}
.preview-fade-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}
.preview-fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}
.preview-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* ===== Cards ===== */
.md3-card {
  background: rgba(30, 30, 30, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 20px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  cursor: default;
}
.md3-card:hover {
  border-color: rgba(130, 177, 255, 0.2);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25);
}
.card-label {
  font-size: 11px;
  font-weight: 700;
  color: rgba(130, 177, 255, 0.7);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.field-row {
  display: flex;
  gap: 10px;
}
.field-row.two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
}
.switch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 10px;
  color: rgba(255, 255, 255, 0.7);
}
.slider-row .v-slider {
  flex: 1;
  min-width: 0;
}
.slider-label {
  min-width: 56px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.55);
}
.slider-val {
  min-width: 40px;
  text-align: right;
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  font-family: 'Consolas', monospace;
}

.slider-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.slider-header {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
}

.respack-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.avatar-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.ui-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 6px;
}
.ui-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  background: rgba(50, 50, 50, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  cursor: pointer;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  transition: all 0.2s;
  user-select: none;
  white-space: nowrap;
}
.ui-chip:hover {
  background: rgba(60, 60, 60, 0.6);
}
.ui-chip.is-on {
  background: rgba(130, 177, 255, 0.12);
  border-color: rgba(130, 177, 255, 0.3);
  color: #82b1ff;
}

.preset-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}
.preset-select {
  flex: 1;
  max-width: 240px;
}
.preset-actions {
  display: flex;
  gap: 8px;
}

/* ===== PREVIEW VISUALIZATIONS ===== */

/* Pixel grid demo */
.pixel-grid-wrap {
  width: 100%;
  display: flex;
  justify-content: center;
  padding: 12px 0;
}
.pixel-grid {
  display: grid;
  gap: 2px;
  width: fit-content;
  padding: 8px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}
.pixel-dot {
  width: 5px;
  height: 5px;
  border-radius: 1px;
  background: #82b1ff;
  opacity: 0;
  animation: pixelFadeIn 0.6s ease forwards;
}
@keyframes pixelFadeIn {
  0% {
    opacity: 0;
    transform: scale(0);
  }
  100% {
    opacity: 0.7;
    transform: scale(1);
  }
}
.pixel-dot:nth-child(odd) {
  animation-delay: 0.1s;
}
.pixel-dot:nth-child(3n) {
  animation-delay: 0.2s;
}
.pixel-dot:nth-child(5n) {
  animation-delay: 0.3s;
}

/* Performance bars */
.perf-bars {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.perf-bar-group {
  display: flex;
  align-items: center;
  gap: 10px;
}
.perf-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  min-width: 36px;
}
.perf-bar {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  overflow: hidden;
}
.perf-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.4s ease;
}
.perf-speed {
  background: linear-gradient(90deg, #4caf50, #8bc34a);
}
.perf-quality {
  background: linear-gradient(90deg, #2196f3, #82b1ff);
}
.perf-val {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  min-width: 32px;
  text-align: right;
  font-family: 'Consolas', monospace;
}

/* Bitrate gauge */
.bitrate-demo {
  width: 100%;
  display: flex;
  justify-content: center;
}
.bitrate-gauge {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.gauge-svg {
  width: 160px;
  height: 80px;
}
.gauge-value {
  font-size: 28px;
  font-weight: 700;
  color: #82b1ff;
  font-family: 'Consolas', monospace;
  margin-top: -8px;
}
.gauge-unit {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

/* Toggle demo */
.toggles-demo {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.toggle-demo-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 13px;
  transition: all 0.2s;
}
.toggle-demo-item.active {
  background: rgba(130, 177, 255, 0.1);
  color: #82b1ff;
}
.toggle-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  transition: background 0.2s;
}
.toggle-demo-item.active .toggle-dot {
  background: #82b1ff;
  box-shadow: 0 0 8px rgba(130, 177, 255, 0.5);
}

/* Wave demo */
.wave-demo {
  width: 100%;
}
.wave-bars {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 80px;
  padding: 4px 0;
}
.wave-bar {
  flex: 1;
  background: linear-gradient(180deg, #82b1ff, #4a7adf);
  border-radius: 2px;
  min-height: 4px;
  transition: height 0.3s ease;
  animation: wavePulse 1.5s ease-in-out infinite alternate;
}
.wave-bar-sfx {
  background: linear-gradient(180deg, #e91e63, #ad1457);
}
@keyframes wavePulse {
  0% {
    opacity: 0.7;
  }
  100% {
    opacity: 1;
  }
}
.wave-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 4px;
  display: block;
}

/* Audio quality bars */
.audio-quality-bars {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.aq-bar-group {
  display: flex;
  align-items: center;
  gap: 8px;
}
.aq-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  min-width: 36px;
  font-weight: 600;
  text-transform: uppercase;
}
.aq-label.is-active {
  color: #82b1ff;
}
.aq-bar {
  flex: 1;
  height: 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 3px;
  overflow: hidden;
}
.aq-fill {
  height: 100%;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.15);
  transition: width 0.4s ease;
}
.aq-fill.is-active {
  background: #82b1ff;
}

/* Chart preview */
.chart-preview-box {
  width: 100%;
  height: 140px;
  background: linear-gradient(180deg, rgba(20, 20, 30, 0.9), rgba(10, 10, 20, 0.95));
  border-radius: 12px;
  position: relative;
  overflow: hidden;
  transition: transform 0.3s ease;
  transform-origin: center center;
}
.chart-judge-line {
  position: absolute;
  bottom: 20px;
  left: 10%;
  right: 10%;
  height: 2px;
  background: linear-gradient(90deg, transparent, rgba(130, 177, 255, 0.6), rgba(233, 30, 99, 0.6), rgba(130, 177, 255, 0.6), transparent);
  box-shadow: 0 0 8px rgba(130, 177, 255, 0.3);
}
.chart-note {
  position: absolute;
  width: 36px;
  height: 36px;
  object-fit: contain;
  filter: drop-shadow(0 0 8px rgba(99, 102, 241, 0.4));
  animation: noteFall linear infinite;
}
.chart-note-1 {
  left: 30%;
  animation-delay: 0s;
}
.chart-note-2 {
  left: 55%;
  animation-delay: 0.6s;
}
.chart-note-3 {
  left: 75%;
  animation-delay: 1.2s;
}
@keyframes noteFall {
  0% {
    top: -30px;
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  85% {
    opacity: 1;
  }
  100% {
    top: calc(100% - 30px);
    opacity: 0;
  }
}

/* Text preview */
.text-demo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 100%;
}
.combo-preview {
  font-size: 32px;
  font-weight: 800;
  color: rgba(255, 255, 255, 0.9);
  letter-spacing: 2px;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}
.watermark-preview {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.3);
  font-style: italic;
}
.ending-preview {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  background: rgba(255, 255, 255, 0.05);
  padding: 6px 14px;
  border-radius: 8px;
}

/* Note scale demo */
.note-scale-demo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
.note-img {
  object-fit: contain;
  filter: drop-shadow(0 0 12px rgba(99, 102, 241, 0.4));
  transition: all 0.3s ease;
}
.note-scale-label {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  font-family: 'Consolas', monospace;
}

/* Effects preview */
.effects-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.fx-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 13px;
  position: relative;
  overflow: hidden;
  min-height: 44px;
}
.fx-item.off {
  opacity: 0.4;
}
.fx-particles {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.particle {
  position: absolute;
  width: 4px;
  height: 4px;
  background: rgba(130, 177, 255, 0.6);
  border-radius: 50%;
  animation: particleFloat 2s ease-in-out infinite;
}
@keyframes particleFloat {
  0% {
    transform: translateY(0) scale(1);
    opacity: 0.8;
  }
  50% {
    transform: translateY(-12px) scale(1.3);
    opacity: 0.4;
  }
  100% {
    transform: translateY(0) scale(1);
    opacity: 0.8;
  }
}
.fx-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at 30% 50%, rgba(130, 177, 255, 0.15), transparent 60%);
  pointer-events: none;
}
.fx-double {
  display: flex;
  gap: 6px;
}
.hint-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #e91e63;
  box-shadow: 0 0 6px rgba(233, 30, 99, 0.5);
}

/* UI phone mock */
.ui-phone-mock {
  width: 120px;
  height: 200px;
  background: rgba(10, 10, 10, 0.9);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  position: relative;
  overflow: hidden;
}
.phone-screen {
  width: 100%;
  height: 100%;
  position: relative;
}
.phone-ui-line {
  position: absolute;
  bottom: 30px;
  left: 10%;
  right: 10%;
  height: 2px;
  background: linear-gradient(90deg, transparent, rgba(130, 177, 255, 0.5), transparent);
}
.phone-ui-score {
  position: absolute;
  top: 12px;
  right: 8px;
  font-size: 8px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.7);
  font-family: 'Consolas', monospace;
}
.phone-ui-combo {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 10px;
  font-weight: 800;
  color: rgba(255, 255, 255, 0.5);
}
.phone-ui-name {
  position: absolute;
  top: 12px;
  left: 8px;
  font-size: 7px;
  color: rgba(255, 255, 255, 0.5);
}
.phone-ui-pb {
  position: absolute;
  bottom: 8px;
  left: 8px;
  right: 8px;
  height: 3px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}
.pb-fill {
  height: 100%;
  background: #82b1ff;
  border-radius: 2px;
}

/* Player card */
.player-card-preview {
  display: flex;
  align-items: center;
  gap: 14px;
  background: rgba(40, 40, 40, 0.6);
  padding: 14px 18px;
  border-radius: 16px;
  width: 100%;
}
.player-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.player-name-pv {
  font-size: 16px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}
.player-rks-pv {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  font-family: 'Consolas', monospace;
}

/* Challenge badge */
.challenge-badge {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 3px solid var(--badge-color, #ffc107);
  box-shadow: 0 0 20px color-mix(in srgb, var(--badge-color, #ffc107) 40%, transparent);
  transition: all 0.3s ease;
}
.badge-rank {
  font-size: 28px;
  font-weight: 800;
  color: var(--badge-color, #ffc107);
}

/* Debug demo */
.debug-demo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 100%;
}
.debug-grid {
  width: 100%;
  height: 80px;
  background-image: linear-gradient(rgba(255, 255, 255, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
  background-size: 20px 20px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}
.debug-mirror {
  width: 60px;
  height: 60px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.3s ease;
}
.debug-mirror.flipped {
  transform: scaleX(-1);
}
.mirror-arrow {
  font-size: 24px;
  color: rgba(255, 255, 255, 0.5);
}
.debug-split {
  display: flex;
  gap: 4px;
}
.split-left {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: rgba(33, 150, 243, 0.4);
}
.split-right {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: rgba(233, 30, 99, 0.4);
}
.debug-skip {
  font-size: 12px;
  font-weight: 700;
  color: rgba(255, 152, 0, 0.7);
  background: rgba(255, 152, 0, 0.1);
  padding: 6px 14px;
  border-radius: 8px;
}

/* Meta info */
.preview-meta {
  display: flex;
  gap: 16px;
  width: 100%;
  justify-content: center;
  flex-wrap: wrap;
}
.meta-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}
.meta-label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.35);
  text-transform: uppercase;
}
.meta-value {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  font-family: 'Consolas', monospace;
}

/* ===== Responsive ===== */
@media (max-width: 900px) {
  .config-body {
    flex-direction: column;
  }
  .preview-col {
    width: 100%;
    height: 200px;
    flex-shrink: 0;
  }
  .preview-panel {
    border-radius: 16px;
  }
}
@media (max-width: 600px) {
  .preview-col {
    display: none;
  }
  .ui-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
