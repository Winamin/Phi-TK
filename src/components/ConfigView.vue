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

  preset-new: New
  preset-save: Save
  preset-delete: Delete
  refresh: Refresh
  open-folder: Open folder

  settings: Settings
  cat:
    output: Output
    recording: Recording
    game: Game
    graphics: Effects
    player: Player
    debug: Debug

  group:
    resolution-fps: Resolution & Frame Rate
    encoder: Encoder
    bitrate: Bitrate
    performance: Performance
    volume: Volume
    audio-format: Audio Format
    chart-scale: Chart Scale & Note Speed
    text-ending: Text & Ending
    respack: Resource Pack
    note-scale: Note Scale
    effects: Effects
    ui: UI Display
    profile: Profile
    challenge: Challenge Mode
    debug: Debug Options

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
  encoder-cpu: CPU 软编码
  fps: 帧率
  hw-accel: 硬件加速
  hw-accel-tips: 提升速度，略微降低质量
  fxaa: FXAA 抗锯齿
  fxaa-tips: 会导致画面模糊
  sample-count: 采样数
  sample-count-tips: 非1启用MSAA
  bitrate-control: 码率控制
  bitrate: 码率/CRF
  ffmpeg-thread: 线程优化
  video-format: 视频格式
  video-format-flac-error: FLAC 音频下不可用
  player-avatar: 头像
  player-name: 名称
  player-rks: RKS
  challenge-color: 课题颜色
  challenge-colors: 白,绿,蓝,红,金,彩
  challenge-rank: 课题等级
  respack: 资源包
  respack-default: '[默认]'
  note-scale: 音符缩放
  render: UI 显示
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
  combo: COMBO 文本
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
  preset-cannot-use-default: 不能使用 default
  default-preset: 默认
  back: 返回

  preset-new: 新建
  preset-save: 保存
  preset-delete: 删除
  refresh: 刷新
  open-folder: 打开文件夹

  settings: 设置
  cat:
    output: 输出
    recording: 录制
    game: 游戏
    graphics: 效果
    player: 玩家
    debug: 调试

  group:
    resolution-fps: 分辨率与帧率
    encoder: 编码器
    bitrate: 码率
    performance: 性能
    volume: 音量
    audio-format: 音频格式
    chart-scale: 谱面缩放与流速
    text-ending: 文本与结算
    respack: 资源包
    note-scale: 音符缩放
    effects: 效果开关
    ui: UI 显示
    profile: 个人信息
    challenge: 课题模式
    debug: 调试选项
</i18n>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { toast, anyFilter, toastError, validateFields } from '@/common';
import type { RenderConfig } from '@/model';
import TipSwitch from './TipSwitch.vue';
import MdCombobox from './md/MdCombobox.vue';
import MdSelect from './md/MdSelect.vue';
import MdSlider from './md/MdSlider.vue';
import MdTextField from './md/MdTextField.vue';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
defineProps<{ initAspectRatio?: number }>();
const RESOLUTIONS = ['1280x720', '1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080', '7680x4320'];
const FFMPEG_PRESETS = [
  { value: 'veryfast', title: 'VeryFast' },
  { value: 'faster', title: 'Faster' },
  { value: 'fast', title: 'Fast' },
  { value: 'medium', title: 'Medium' },
  { value: 'slow', title: 'Slow' },
  { value: 'slower', title: 'Slower' },
  { value: 'veryslow', title: 'VerySlow' },
];

const VIDEO_FORMAT = [
  { value: 'mp4', title: 'MP4' },
  { value: 'mov', title: 'MOV' },
];
const VIDEO_CODECS = [
  { value: 'h264', title: 'H.264' },
  { value: 'hevc', title: 'HEVC' },
  { value: 'av1', title: 'AV1' },
];
// Titles come from the `encoder-*` i18n keys, which already existed but were bypassed by a
// hardcoded Chinese list. Computed so the labels follow the interface language.
const ENCODERS = computed(() => [
  { value: 'auto', title: t('encoder-auto') },
  { value: 'nvenc', title: t('encoder-nvenc') },
  { value: 'qsv', title: t('encoder-qsv') },
  { value: 'amf', title: t('encoder-amf') },
  { value: 'vulkan', title: t('encoder-vulkan') },
  { value: 'cpu', title: t('encoder-cpu') },
]);
const AUDIO_FORMATS = ['flac', 'mp3', 'aac', 'opus', 'wav'];
const AUDIO_BITS = [16, 24, 32];
const SAMPLE_RATES = [44100, 48000, 96000, 192000, 384000, 768000];
const BITRATE_CONTROLS = ['CRF', 'CBR'];
const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

const form = ref<HTMLFormElement>();

/**
 * Icons are `@mdui/icons` custom-element tag names, rendered through `<component :is>`.
 *
 * This is the destination list for the landscape navigation rail — the same
 * primary-navigation pattern a native Android tablet settings app uses
 * (rail on the leading edge, one destination's content in the pane beside it).
 */
const categories = [
  { key: 'output', icon: 'mdui-icon-videocam--outlined', activeIcon: 'mdui-icon-videocam' },
  { key: 'recording', icon: 'mdui-icon-music-note--outlined', activeIcon: 'mdui-icon-music-note' },
  { key: 'game', icon: 'mdui-icon-sports-esports--outlined', activeIcon: 'mdui-icon-sports-esports' },
  { key: 'graphics', icon: 'mdui-icon-palette--outlined', activeIcon: 'mdui-icon-palette' },
  { key: 'player', icon: 'mdui-icon-person--outlined', activeIcon: 'mdui-icon-person' },
  { key: 'debug', icon: 'mdui-icon-bug-report--outlined', activeIcon: 'mdui-icon-bug-report' },
] as const;

const activeCategory = ref<(typeof categories)[number]['key']>('output');

const resolution = ref('1920x1080');
const fps = ref('60');
const sampleCount = ref('1');
const videoCodec = ref('h264');
const encoder = ref('auto');
const ffmpegPreset = ref('medium');

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

const playerAvatar = ref<string>();
const playerName = ref('');
const playerRks = ref('15.0');
const challengeRank = ref('45');
const challengeColor = ref(t('challenge-colors').split(',')[4]);
const challengeColors = computed(() => t('challenge-colors').split(','));
const chartDebug = ref(false);
const flidX = ref(false);
const handSplit = ref(false);
const disableLoading = ref(false);

/** `<v-img>` used the raw path, which the webview cannot load — Tauri needs an asset URL. */
const avatarUrl = computed(() => {
  if (!playerAvatar.value) return '';
  try {
    return convertFileSrc(playerAvatar.value);
  } catch {
    return '';
  }
});

const videoFormat = ref('mp4');
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

const pctLabel = (value: number) => `${Math.round(value * 100)}%`;
const numLabel = (value: number) => value.toFixed(2);

/* A `type` rather than an `interface` so it satisfies `MdSelect`'s `items` prop: TypeScript
   only gives object *literal* types an implicit index signature, never interfaces. */
type Respack = {
  name: string;
  path: string | null;
  index: number;
};
const DEFAULT_RESPACK: Respack = { name: t('respack-default'), path: null, index: 0 };
const respacks = ref([DEFAULT_RESPACK]);
const respack = ref(DEFAULT_RESPACK);
async function updateRespacks() {
  const list = (await invoke('get_respacks')) as { name: string; path: string }[];
  respacks.value = [DEFAULT_RESPACK, ...list.map((obj, i) => ({ name: obj.name, path: obj.path, index: i + 1 }))];
  respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0];
}
updateRespacks();

/**
 * The select carries `index`, not the object: the old `<v-select>` had no `item-value`, so
 * Vuetify fell back to the *title* and stored the name string — leaving `respack.path`
 * undefined and silently dropping `resPackPath` from every render.
 */
function onRespackChange(value: unknown) {
  respack.value = respacks.value.find((x) => x.index === Number(value)) ?? respacks.value[0];
}

/* See `Respack`: a `type`, not an `interface`, for `MdSelect`'s `items` prop. */
type Preset = {
  name: string;
  key: string;
  config: RenderConfig;
};
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

/** Replaces `<v-select return-object>` — the select carries `key`, the object is looked up. */
function onPresetChange(value: unknown) {
  const found = presets.value.find((x) => x.key === String(value));
  if (!found) return;
  preset.value = found;
  applyConfig(found.config);
}

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

async function buildConfig(): Promise<RenderConfig | null> {
  // Native constraint validation over the mdui fields, replacing `<v-form>.validate()`.
  if (!validateFields(form.value)) {
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
    challengeColor: STD_CHALLENGE_COLORS[challengeColors.value.indexOf(challengeColor.value)],
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
  challengeColor.value = challengeColors.value[STD_CHALLENGE_COLORS.indexOf(c.challengeColor)];
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
  <form ref="form" class="config-root" @submit.prevent>
    <!-- ==================================================================
         Landscape tablet layout — MD3 adaptive "list-detail".

         A fixed navigation rail on the leading edge lists the six
         destinations; the pane beside it shows exactly one destination's
         settings. This is the same primary-navigation shell a native
         Android tablet settings app uses at the `expanded` window size
         class, and it is why the rail never scrolls with the content.
         ================================================================== -->
    <nav class="config-rail" :aria-label="t('settings')">
      <ul class="rail-list">
        <li v-for="cat in categories" :key="cat.key">
          <button
            type="button"
            class="rail-item"
            :class="{ 'is-active': activeCategory === cat.key }"
            :aria-current="activeCategory === cat.key ? 'page' : undefined"
            @click="activeCategory = cat.key">
            <!-- The active destination swaps to its filled icon — MD3's selected
                 state for navigation items, not just a colour change. -->
            <span class="rail-icon"><component :is="activeCategory === cat.key ? cat.activeIcon : cat.icon" /></span>
            <span class="rail-label">{{ t(`cat.${cat.key}`) }}</span>
          </button>
        </li>
      </ul>

      <!-- Preset management lives at the rail's foot: it acts on the whole
           config rather than on any one destination. -->
      <div class="rail-foot">
        <MdSelect
          class="preset-select"
          variant="outlined"
          :model-value="preset.key"
          :items="presets"
          item-title="name"
          item-value="key"
          :label="t('presets')"
          @update:model-value="onPresetChange" />
        <div class="preset-actions">
          <mdui-button variant="tonal" class="preset-btn" @click="createPreset">
            <mdui-icon-add slot="icon"></mdui-icon-add>
            {{ t('preset-new') }}
          </mdui-button>
          <mdui-button variant="tonal" class="preset-btn" :disabled="preset.key === 'default'" @click="replacePreset">
            <mdui-icon-save--outlined slot="icon"></mdui-icon-save--outlined>
            {{ t('preset-save') }}
          </mdui-button>
          <mdui-button variant="tonal" class="preset-btn btn-danger" :disabled="preset.key === 'default'" @click="deletePreset">
            <mdui-icon-delete--outlined slot="icon"></mdui-icon-delete--outlined>
            {{ t('preset-delete') }}
          </mdui-button>
        </div>
      </div>
    </nav>

    <!-- ==================== Detail pane ====================
         No pane header: the rail already marks the active destination, so an
         icon + title repeating it here was duplication. The settings grid is the
         first thing in the pane. -->
    <div class="config-pane">

      <div class="settings-scroll">
        <!-- ==================== OUTPUT ==================== -->
        <div v-show="activeCategory === 'output'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.resolution-fps') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdCombobox v-model="resolution" :items="RESOLUTIONS" :label="t('resolution')" />
              </div>
              <div class="field-row">
                <MdTextField v-model="fps" variant="outlined" inputmode="numeric" :label="t('fps')" />
                <MdTextField v-model="sampleCount" variant="outlined" inputmode="numeric" :label="t('sample-count')" :helper="t('sample-count-tips')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.encoder') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdSelect v-model="videoCodec" variant="outlined" :items="VIDEO_CODECS" :label="t('video-codec')" />
                <MdSelect v-model="encoder" variant="outlined" :items="ENCODERS" :label="t('encoder-select')" />
              </div>
              <div class="field-row">
                <MdSelect v-model="ffmpegPreset" variant="outlined" :items="FFMPEG_PRESETS" :label="t('ffmpeg-preset')" />
                <MdSelect v-model="videoFormat" variant="outlined" :items="VIDEO_FORMAT" :label="t('video-format')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.bitrate') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdSelect v-model="bitrateControl" variant="outlined" :items="BITRATE_CONTROLS" :label="t('bitrate-control')" />
                <MdTextField v-model="bitrate" variant="outlined" :label="t('bitrate')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.performance') }}</h2>
            <div class="setting-card">
              <div class="switch-grid">
                <TipSwitch v-model="hwAccel" :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" />
                <TipSwitch v-model="fxaa" :label="t('fxaa')" :tooltip="t('fxaa-tips')" />
                <TipSwitch v-model="ffmpegThread" :label="t('ffmpeg-thread')" />
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== RECORDING ==================== -->
        <div v-show="activeCategory === 'recording'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.volume') }}</h2>
            <div class="setting-card">
              <div class="setting-item">
                <span class="md3-icon-badge item-icon"><mdui-icon-music-note></mdui-icon-music-note></span>
                <span class="item-label">{{ t('volume-music') }}</span>
                <!-- The readout sits in a fixed-width slot so the two sliders'
                     tracks line up, which a bare percentage would not guarantee. -->
                <span class="item-value">{{ pctLabel(volumeMusic) }}</span>
              </div>
              <MdSlider v-model="volumeMusic" class="item-slider" :min="0" :max="2" :step="0.01" :labelFormatter="pctLabel" />
              <mdui-divider></mdui-divider>
              <div class="setting-item">
                <span class="md3-icon-badge item-icon"><mdui-icon-volume-up></mdui-icon-volume-up></span>
                <span class="item-label">{{ t('volume-sfx') }}</span>
                <span class="item-value">{{ pctLabel(volumeSfx) }}</span>
              </div>
              <MdSlider v-model="volumeSfx" class="item-slider" :min="0" :max="2" :step="0.01" :labelFormatter="pctLabel" />
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.audio-format') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdSelect v-model="audioFormat" variant="outlined" :items="AUDIO_FORMATS" :label="t('audio-format')" />
                <MdSelect v-model="audioBit" variant="outlined" :items="AUDIO_BITS" :label="t('audio-bit-depth')" :disabled="audioFormat !== 'wav'" />
              </div>
              <div class="field-row">
                <MdSelect v-model="targetAudio" variant="outlined" :items="SAMPLE_RATES" :label="t('target_audio')" />
                <MdTextField v-model="bufferSize" variant="outlined" :number="true" inputmode="numeric" :label="t('buffer_size')" />
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== GAME ==================== -->
        <div v-show="activeCategory === 'game'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.chart-scale') }}</h2>
            <div class="setting-card">
              <div class="setting-item">
                <span class="item-label">{{ t('chart_ratio') }}</span>
                <span class="item-value">{{ numLabel(chartRatio) }}</span>
              </div>
              <MdSlider v-model="chartRatio" class="item-slider" :min="0.05" :max="1" :step="0.01" :labelFormatter="numLabel" />
              <mdui-divider></mdui-divider>
              <div class="setting-item">
                <span class="item-label">{{ t('note-speed-factor') }}</span>
                <span class="item-value">{{ numLabel(noteSpeedFactor) }}</span>
              </div>
              <MdSlider v-model="noteSpeedFactor" class="item-slider" :min="0.1" :max="2" :step="0.01" :labelFormatter="numLabel" />
              <mdui-divider></mdui-divider>
              <div class="switch-grid">
                <TipSwitch v-model="background" :label="t('background')" />
                <TipSwitch v-model="bar" :label="t('bar')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.text-ending') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdTextField v-model="combo" variant="outlined" :label="t('combo')" />
                <MdTextField v-model="watermark" variant="outlined" :label="t('watermark')" />
              </div>
              <div class="field-row">
                <MdTextField v-model="endingLength" variant="outlined" inputmode="decimal" :label="t('ending-length')" />
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== GRAPHICS ==================== -->
        <div v-show="activeCategory === 'graphics'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.respack') }}</h2>
            <div class="setting-card">
              <div class="respack-row">
                <MdSelect
                  class="respack-select"
                  variant="outlined"
                  :model-value="respack.index"
                  :items="respacks"
                  item-title="name"
                  item-value="index"
                  :label="t('respack')"
                  @update:model-value="onRespackChange" />
                <mdui-tooltip :content="t('refresh')">
                  <mdui-button-icon @click="updateRespacks">
                    <mdui-icon-refresh></mdui-icon-refresh>
                  </mdui-button-icon>
                </mdui-tooltip>
                <mdui-tooltip :content="t('open-folder')">
                  <mdui-button-icon @click="openRespackFolder">
                    <mdui-icon-folder-open></mdui-icon-folder-open>
                  </mdui-button-icon>
                </mdui-tooltip>
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.note-scale') }}</h2>
            <div class="setting-card">
              <div class="setting-item">
                <span class="item-label">{{ t('note-scale') }}</span>
                <span class="item-value">{{ numLabel(noteScale) }}</span>
              </div>
              <MdSlider v-model="noteScale" class="item-slider" :min="0" :max="5" :step="0.05" :labelFormatter="numLabel" />
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.effects') }}</h2>
            <div class="setting-card">
              <div class="switch-grid">
                <TipSwitch v-model="doubleHint" :label="t('double-hint')" />
                <TipSwitch v-model="aggressive" :label="t('aggressive')" :tooltip="t('aggressive-tips')" />
                <TipSwitch v-model="disableParticle" :label="t('disable-particle')" />
                <TipSwitch v-model="disableEffect" :label="t('disable-effect')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.ui') }}</h2>
            <div class="setting-card">
              <!-- Filter chips are `<mdui-chip variant="filter">`'s only correct use:
                   a multi-select set of independent display toggles. -->
              <div class="ui-grid">
                <mdui-chip
                  v-for="item in renderList"
                  :key="item"
                  variant="filter"
                  selectable
                  :selected="render.includes(item)"
                  @change="(e: Event) => { (e.target as any).selected ? render.push(item) : render = render.filter(x => x !== item) }">
                  {{ item }}
                </mdui-chip>
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== PLAYER ==================== -->
        <div v-show="activeCategory === 'player'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.profile') }}</h2>
            <div class="setting-card">
              <div class="avatar-row">
                <div class="avatar">
                  <img v-if="avatarUrl" :src="avatarUrl" alt="" class="avatar-img" />
                  <mdui-icon-person v-else class="avatar-fallback"></mdui-icon-person>
                </div>
                <div class="avatar-text">
                  <span class="md3-title-medium">{{ t('player-avatar') }}</span>
                  <span class="md3-caption">{{ playerName || t('player-name') }}</span>
                </div>
                <mdui-button variant="outlined" @click="chooseAvatar">
                  <mdui-icon-image--outlined slot="icon"></mdui-icon-image--outlined>
                  {{ t('player-avatar') }}
                </mdui-button>
              </div>
              <div class="field-row">
                <MdTextField v-model="playerName" variant="outlined" :label="t('player-name')" />
                <MdTextField v-model="playerRks" variant="outlined" inputmode="decimal" :label="t('player-rks')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.challenge') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdTextField v-model="challengeRank" variant="outlined" inputmode="numeric" :label="t('challenge-rank')" />
                <MdSelect v-model="challengeColor" variant="outlined" :items="challengeColors" :label="t('challenge-color')" />
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== DEBUG ==================== -->
        <div v-show="activeCategory === 'debug'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.debug') }}</h2>
            <div class="setting-card">
              <div class="switch-grid">
                <TipSwitch v-model="chartDebug" :label="t('chart_debug')" />
                <TipSwitch v-model="flidX" :label="t('flid_x')" />
                <TipSwitch v-model="handSplit" :label="t('hand-split')" />
                <TipSwitch v-model="disableLoading" :label="t('disable-loading')" :tooltip="t('disable-loading-tips')" />
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  </form>
</template>

<style scoped lang="scss">
@use '../styles/breakpoints' as bp;
@use '../styles/states' as st;
@use '../styles/motion' as mo;

/* ============================================================
   ConfigView — landscape tablet MD3

   Shell: MD3 adaptive list-detail. A fixed navigation rail (the leading
   edge) owns destination switching, and the pane beside it renders exactly
   one destination. Neither column scrolls as a whole — only `.settings-scroll`
   does — so the rail and the pane header stay pinned the way a native
   Android settings app behaves on a landscape tablet.

   Spacing follows MD3's padding/gap model (md32.png): 16dp horizontal pane
   padding, 24dp between groups, 8dp between a label and its control.
   ============================================================ */

.config-root {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  width: 100%;
  height: 100%;
  min-height: 0;
  background-color: rgb(var(--mdui-color-surface));
}

/* ============================================================
   Navigation rail — primary destinations
   ============================================================ */
.config-rail {
  display: flex;
  flex-direction: column;
  width: 240px;
  flex-shrink: 0;
  min-height: 0;
  padding: 16px 12px;
  gap: 8px;
  /* `surface-container` is the MD3 container tone for a rail that shares the
     screen with content, rather than `surface` which it would blend into. */
  background-color: rgb(var(--mdui-color-surface-container));
  border-radius: 0 var(--mdui-shape-corner-extra-large) var(--mdui-shape-corner-extra-large) 0;
}

/* Destination list.

   The rail is narrow, so the global 6px scrollbar reads as a heavy bar against
   the pill-shaped items. It is slimmed to 3px and kept invisible until the rail
   is actually hovered — the scroll affordance is still there, it just does not
   compete with the navigation items. */
.rail-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: thin;
}

.rail-list::-webkit-scrollbar {
  width: 3px;
}

.rail-list::-webkit-scrollbar-thumb {
  background: transparent;
}

.rail-list:hover::-webkit-scrollbar-thumb {
  background: rgba(var(--mdui-color-outline), 0.5);
}

/* MD3 navigation item: 56dp tall (well above the 48dp minimum), full pill radius
   on the selected state. The state layer composites over the selected fill, so a
   press on the active row still reads as a press. */
.rail-item {
  @include st.interactive;

  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  height: 56px;
  padding: 0 16px;
  border: none;
  border-radius: var(--mdui-shape-corner-full);
  background-color: transparent;
  color: rgb(var(--mdui-color-on-surface-variant));
  font: inherit;
  font-size: var(--mdui-typescale-label-large-size);
  font-weight: var(--mdui-typescale-label-large-weight);
  line-height: var(--mdui-typescale-label-large-line-height);
  text-align: start;
  transition: color var(--mdui-motion-duration-short4) var(--mdui-motion-easing-standard);
}

.rail-item.is-active {
  background-color: rgb(var(--mdui-color-secondary-container));
  color: rgb(var(--mdui-color-on-secondary-container));
  font-weight: 600;
}

.rail-icon {
  display: flex;
  flex-shrink: 0;
  font-size: 1.375rem;
}

/* MD3's rail indicator is a shape that appears on selection. The selected item
   swaps to its filled icon, so animating that swap on the bouncy spring reads as
   the indicator arriving rather than the icon simply being replaced. */
.rail-item.is-active .rail-icon {
  animation: rail-icon-pop var(--app-motion-duration-bouncy)
    var(--app-motion-spring-bouncy, var(--mdui-motion-easing-emphasized-decelerate)) both;
}

@keyframes rail-icon-pop {
  from {
    transform: scale(0.7);
    opacity: 0;
  }
  to {
    transform: none;
    opacity: 1;
  }
}

.rail-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Preset management — acts on the whole config, so it sits below the
   destination list rather than inside any one destination. */
.rail-foot {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid rgba(var(--mdui-color-outline-variant), 0.6);
  flex-shrink: 0;
}

.preset-select {
  width: 100%;
}

.preset-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

/* "Save" spans the full width — it is the primary preset action. */
.preset-actions .preset-btn:nth-child(2) {
  grid-column: span 2;
  order: -1;
}

.preset-actions .preset-btn {
  --mdui-button-height: 2.5rem;
}

.btn-danger {
  --mdui-color-secondary-container: var(--mdui-color-error-container);
  --mdui-color-on-secondary-container: var(--mdui-color-on-error-container);
}

/* ============================================================
   Detail pane
   ============================================================ */
.config-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

/* ---------- Scrolling settings column ----------
   A bounded, centred measure with a *stable* two-column grid.

   Two columns is not arbitrary: every destination has an even number of groups
   (output 4, recording 2, game 2, graphics 4, player 2, debug 1), so the rows
   always fill completely. The previous `auto-fit` track cap let the count drift
   between two and four as the window changed, which produced half-empty rows
   and cards ending at different heights — the ragged look this replaces.

   `max-width` is what pins the count: 340px tracks inside 1000px can never fit
   three columns, so the layout is the same at 1100px and at 2400px. */
.settings-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: thin;
  padding: 8px 24px 32px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
  align-content: start;
  max-width: 1000px;
  margin: 0 auto;
  width: 100%;
  /* Row gap larger than column gap: groups stacked vertically need more air
     between them than neighbours sitting side by side. */
  gap: 28px 24px;
  scroll-behavior: smooth;
}

.settings-stack {
  display: contents;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;

  /* The groups rise in sequence as the pane appears. `.settings-stack` is
     `display: contents`, so each visible stack's sections form their own
     1..N sequence and the stagger restarts per destination. */
  @include mo.enter-rise(10px);
  @include mo.stagger(6, 40ms, 40ms);
}

/* ---------- Section title ----------
   Flush with the card edge below it. It used to carry `padding: 0 4px`, which
   put the header 4px right of the card and 12px left of the card's contents —
   three different left edges in one group. */
.section-title {
  margin: 0;
  padding: 0;
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: 600;
  line-height: var(--mdui-typescale-title-small-line-height);
  letter-spacing: 0.01em;
  color: rgb(var(--mdui-color-primary));
}

/* ---------- Setting card ----------
   Row spacing is a single `gap` on the card rather than a padding on each row
   type. The old scheme gave `.field-row`, `.switch-grid`, `.setting-item` and
   `.item-slider` four different paddings, so the vertical rhythm inside a card
   depended on which kinds of row it happened to contain. Now rows only carry
   their 16dp side padding and the card owns all vertical spacing. */
.setting-card {
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-extra-large);
  padding: 14px 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
  /* The fill is an effect and the corner radius is a shape, so they run on
     separate MD3 clocks — the radius settles on the slower spatial spring. */
  transition-property: background-color, border-radius;
  transition-duration: var(--app-motion-duration-effects-default), var(--app-motion-duration-spatial-slow);
  transition-timing-function: var(--app-motion-spring-effects-default), var(--app-motion-spring-spatial-slow);
}

/* Focus morphs the card from an extra-large rounded rectangle to a tighter one —
   a shape change, which is exactly what MD3 reserves spatial springs for. */
.setting-card:focus-within {
  background-color: rgb(var(--mdui-color-surface-container));
  border-radius: var(--mdui-shape-corner-large);
}

/* ---------- Form rows ----------
   Side padding only; the card's `gap` supplies all vertical spacing. */
.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 0 16px;
}

/* A lone control spans the row rather than sitting in a half-width column. */
.field-row > :only-child {
  grid-column: 1 / -1;
}

.switch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px 12px;
  padding: 0 16px;
}

/* ---------- Slider rows ---------- */
.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
}

.item-icon {
  width: 2.25rem;
  height: 2.25rem;
  font-size: 1.1rem;
}

.item-label {
  flex: 1;
  min-width: 0;
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
}

/* Fixed-width readout so stacked sliders share one track alignment. */
.item-value {
  min-width: 3.5rem;
  text-align: right;
  flex-shrink: 0;
  color: rgb(var(--mdui-color-primary));
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: var(--mdui-typescale-body-medium-size);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.item-slider {
  display: block;
  padding: 0 16px;
}

/* ---------- Resource pack row ---------- */
.respack-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
}

.respack-select {
  flex: 1;
  min-width: 0;
}

/* ---------- UI display filter chips ---------- */
.ui-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 16px;
}

/* ---------- Avatar ---------- */
.avatar-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 16px;
}

.avatar {
  width: 64px;
  height: 64px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--mdui-shape-corner-full);
  overflow: hidden;
  background-color: rgb(var(--mdui-color-surface-variant));
  color: rgb(var(--mdui-color-on-surface-variant));
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-fallback {
  font-size: 2rem;
}

.avatar-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

/* ============================================================
   Breakpoints — MD3 window size classes

   `large` (>=1080px) shows the rail with a measured, centred grid; `medium`
   (600–1079px) narrows the rail and lets the grid fill the pane; `compact`
   (<600px) moves the rail to a horizontal scroller above the pane, because a
   fixed side rail would eat most of a phone-width window.

   The previous `max-width: 839px` block was labelled "compact" but 839px is
   actually the top of `medium`, so 600–839px windows incorrectly got the
   horizontal rail. The named mixins make that class of mistake impossible.
   ============================================================ */

/* Compact and medium: narrow the rail and let the grid fill the pane instead of
   floating in the middle of it. */
@include bp.below-large {
  .config-rail {
    width: 200px;
    padding: 12px 10px;
    gap: 6px;
  }
  /* Shorter rows so six destinations plus the preset foot fit without the rail
     growing its own scrollbar on a short window. */
  .rail-item {
    height: 48px;
    padding: 0 12px;
    gap: 10px;
    font-size: var(--mdui-typescale-label-medium-size);
  }
  .rail-icon {
    font-size: 1.25rem;
  }
  .rail-foot {
    padding-top: 8px;
  }
  /* Only the gutter narrows. The track width is deliberately NOT overridden:
     lowering it to 260px would let a third column fit inside the 1000px cap and
     reintroduce the uneven rows the base rule is sized to prevent. */
  .settings-scroll {
    padding: 16px 20px 28px;
  }
}

/* Compact: rail moves to the top as a horizontal scroller and the pane stacks
   under it. Only here does the settings grid collapse to a single column. */
@include bp.compact {
  .config-root {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
  }
  .config-rail {
    width: 100%;
    padding: 8px 12px;
    gap: 6px;
    border-radius: 0 0 var(--mdui-shape-corner-large) var(--mdui-shape-corner-large);
  }
  /* A horizontal rail: destinations scroll sideways, presets stay pinned below. */
  .rail-list {
    flex-direction: row;
    overflow-x: auto;
    overflow-y: hidden;
    flex: none;
    gap: 4px;
    padding-bottom: 4px;
  }
  .rail-item {
    width: auto;
    /* MD3 minimum touch target is 48dp; 40dp was below it. */
    height: 48px;
    padding: 0 12px;
    white-space: nowrap;
    flex-shrink: 0;
    font-size: var(--mdui-typescale-label-medium-size);
  }
  .rail-icon {
    font-size: 1.125rem;
  }
  .rail-foot {
    flex-direction: row;
    align-items: center;
    gap: 8px;
    padding-top: 8px;
  }
  .preset-select {
    width: auto;
    flex: 1;
    min-width: 0;
  }
  .preset-actions {
    display: flex;
    flex-shrink: 0;
  }
  /* One compact action survives; the rest would not fit beside the select. */
  .preset-actions .preset-btn:nth-child(1),
  .preset-actions .preset-btn:nth-child(3) {
    display: none;
  }
  .preset-actions .preset-btn:nth-child(2) {
    order: 0;
  }
  .settings-scroll {
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
    padding: 12px 16px 20px;
  }
  .field-row {
    grid-template-columns: minmax(0, 1fr);
    gap: 8px;
  }
  .setting-card {
    border-radius: var(--mdui-shape-corner-large);
  }
}
</style>
