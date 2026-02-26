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
  encoder-desc-auto: Auto detect best encoder
  encoder-desc-nvenc: NVIDIA GPU hardware acceleration
  encoder-desc-qsv: Intel Quick Sync Video
  encoder-desc-amf: AMD Advanced Media Framework
  encoder-desc-vulkan: Cross-platform GPU encoding
  encoder-desc-cpu: libx264/libx265/libaom-av1
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
  encoder-desc-auto: 自动检测最佳编码器
  encoder-desc-nvenc: NVIDIA GPU 硬件加速
  encoder-desc-qsv: Intel Quick Sync Video
  encoder-desc-amf: AMD Advanced Media Framework
  encoder-desc-vulkan: 跨平台 GPU 编码
  encoder-desc-cpu: libx264/libx265/libaom-av1
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

// Constants
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

// Navigation
const activeSection = ref<string | null>(null);

// Categories config
const categories = [
  { key: 'output', icon: 'mdi-video', title: '输出', subtitle: '分辨率、编码、格式', color: '#FF8A00', gradient: 'linear-gradient(135deg, #FF8A00, #FF5722)' },
  { key: 'recording', icon: 'mdi-music-circle', title: '录制', subtitle: '音量、格式、采样率', color: '#E91E63', gradient: 'linear-gradient(135deg, #E91E63, #9C27B0)' },
  { key: 'game', icon: 'mdi-gamepad-variant', title: '游戏', subtitle: '连击、水印、流速', color: '#4CAF50', gradient: 'linear-gradient(135deg, #4CAF50, #8BC34A)' },
  { key: 'graphics', icon: 'mdi-palette', title: '效果', subtitle: '粒子、特效、缩放', color: '#2196F3', gradient: 'linear-gradient(135deg, #2196F3, #03A9F4)' },
  { key: 'player', icon: 'mdi-account', title: '玩家', subtitle: '头像、名称、课题', color: '#9C27B0', gradient: 'linear-gradient(135deg, #9C27B0, #673AB7)' },
  { key: 'debug', icon: 'mdi-bug', title: '调试', subtitle: '镜像、手序、加载', color: '#607D8B', gradient: 'linear-gradient(135deg, #607D8B, #455A64)' },
];

const currentCategory = computed(() => categories.find((c) => c.key === activeSection.value));

// Settings
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
const activeTab = ref('display');
const showQuickEdit = ref(false);
const focusField = ref<string | null>(null);
function toggleRender(item: string) {
  const index = render.value.indexOf(item);
  if (index > -1) {
    render.value.splice(index, 1);
  } else {
    render.value.push(item);
  }
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

// Video format sync
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

// Resource Pack
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

// Presets
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
  // Always apply config
  applyConfig(preset.value.config);
}
updatePresets();

async function chooseAvatar() {
  const file = await open({
    filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'] }, anyFilter()],
  });
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
  <v-form ref="form" class="config-wrapper">
    <!-- Android 15 风格三区块布局 -->
    <div class="android-layout" :class="{ 'is-expanded': activeSection }">
      <!-- 默认视图：三区块 -->
      <template v-if="!activeSection">
        <!-- 区块1：分类卡片（较大区域） -->
        <section class="section-cards" aria-label="设置分类">
          <div class="section-header">
            <v-icon icon="mdi-tune" size="20" />
            <span class="section-title">设置分类</span>
          </div>
          <div class="cards-scroll">
            <div class="cards-list">
              <div
                v-for="cat in categories"
                :key="cat.key"
                class="category-card"
                :style="{ '--card-accent': cat.color }"
                @click="activeSection = cat.key">
                <div class="card-ripple"></div>
                <div class="card-icon-wrapper" :style="{ background: cat.gradient }">
                  <v-icon :icon="cat.icon" size="22" color="white" />
                </div>
                <div class="card-info">
                  <div class="card-title">{{ cat.title }}</div>
                  <div class="card-subtitle">{{ cat.subtitle }}</div>
                </div>
                <v-icon icon="mdi-chevron-right" size="20" class="card-arrow" />
              </div>
            </div>
          </div>
        </section>

        <!-- 区块2：快速设置（中等区域） -->
        <section class="section-quick" aria-label="快速设置">
          <div class="section-header">
            <v-icon icon="mdi-bolt" size="20" />
            <span class="section-title">快速设置</span>
            <v-btn
              icon
              size="x-small"
              variant="text"
              density="compact"
              class="expand-btn"
              :class="{ 'is-active': showQuickEdit }"
              @click="showQuickEdit = !showQuickEdit; if (!showQuickEdit) focusField = null">
              <v-icon :icon="showQuickEdit ? 'mdi-chevron-up' : 'mdi-chevron-down'" size="18" />
            </v-btn>
          </div>

          <!-- 快速设置滚动容器 -->
          <div class="quick-scroll">
            <!-- 快速设置摘要网格 -->
            <div class="quick-grid" :class="{ 'is-expanded': showQuickEdit }">
              <!-- 分辨率 -->
              <div class="quick-tile" @click="showQuickEdit = true; focusField = focusField === 'resolution' ? null : 'resolution'">
                <div class="tile-icon">
                  <v-icon icon="mdi-monitor" size="18" />
                </div>
                <div class="tile-content">
                  <span class="tile-value">{{ resolution }}</span>
                  <span class="tile-label">分辨率</span>
                </div>
                <transition name="tile-expand">
                  <div v-if="focusField === 'resolution'" class="tile-editor" @click.stop>
                    <v-combobox v-model="resolution" :items="RESOLUTIONS" density="compact" variant="outlined" hide-details />
                  </div>
                </transition>
              </div>

              <!-- 帧率 -->
              <div class="quick-tile" @click="showQuickEdit = true; focusField = focusField === 'fps' ? null : 'fps'">
                <div class="tile-icon">
                  <v-icon icon="mdi-speedometer" size="18" />
                </div>
                <div class="tile-content">
                  <span class="tile-value">{{ fps }}</span>
                  <span class="tile-label">帧率</span>
                </div>
                <transition name="tile-expand">
                  <div v-if="focusField === 'fps'" class="tile-editor" @click.stop>
                    <v-text-field v-model="fps" density="compact" variant="outlined" hide-details type="number" />
                  </div>
                </transition>
              </div>

              <!-- 编码器 -->
              <div class="quick-tile" @click="showQuickEdit = true; focusField = focusField === 'codec' ? null : 'codec'">
                <div class="tile-icon">
                  <v-icon icon="mdi-video-vintage" size="18" />
                </div>
                <div class="tile-content">
                  <span class="tile-value">{{ videoCodec.toUpperCase() }}</span>
                  <span class="tile-label">编码</span>
                </div>
                <transition name="tile-expand">
                  <div v-if="focusField === 'codec'" class="tile-editor" @click.stop>
                    <v-select v-model="videoCodec" :items="VIDEO_CODECS" item-title="title" item-value="value" density="compact" variant="outlined" hide-details />
                  </div>
                </transition>
              </div>

              <!-- 采样率 -->
              <div class="quick-tile" @click="showQuickEdit = true; focusField = focusField === 'audio' ? null : 'audio'">
                <div class="tile-icon">
                  <v-icon icon="mdi-waveform" size="18" />
                </div>
                <div class="tile-content">
                  <span class="tile-value">{{ targetAudio / 1000 }}k</span>
                  <span class="tile-label">采样率</span>
                </div>
                <transition name="tile-expand">
                  <div v-if="focusField === 'audio'" class="tile-editor" @click.stop>
                    <v-select v-model="targetAudio" :items="SAMPLE_RATES" density="compact" variant="outlined" hide-details />
                  </div>
                </transition>
              </div>
            </div>

            <!-- 展开的完整编辑面板 -->
            <div v-if="showQuickEdit" class="quick-panel">
              <div class="panel-row">
                <div class="panel-field">
                  <label>分辨率</label>
                  <v-combobox v-model="resolution" :items="RESOLUTIONS" density="compact" variant="outlined" hide-details />
                </div>
                <div class="panel-field">
                  <label>帧率</label>
                  <v-text-field v-model="fps" density="compact" variant="outlined" hide-details type="number" />
                </div>
              </div>
              <div class="panel-row">
                <div class="panel-field">
                  <label>编码器</label>
                  <v-select v-model="videoCodec" :items="VIDEO_CODECS" item-title="title" item-value="value" density="compact" variant="outlined" hide-details />
                </div>
                <div class="panel-field">
                  <label>采样率</label>
                  <v-select v-model="targetAudio" :items="SAMPLE_RATES" density="compact" variant="outlined" hide-details />
                </div>
              </div>
              <div class="panel-row">
                <div class="panel-field">
                  <label>编码预设</label>
                  <v-select v-model="ffmpegPreset" :items="FFMPEG_PRESETS" density="compact" variant="outlined" hide-details />
                </div>
                <div class="panel-field">
                  <label>码率/CRF</label>
                  <v-text-field v-model="bitrate" density="compact" variant="outlined" hide-details />
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- 区块3：预设管理（较小区域） -->
        <section class="section-preset" aria-label="预设管理">
          <div class="section-header">
            <v-icon icon="mdi-bookmark-multiple" size="20" />
            <span class="section-title">预设管理</span>
          </div>
          <div class="preset-content">
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
            <div class="preset-buttons">
              <v-btn variant="tonal" color="primary" size="small" prepend-icon="mdi-plus" @click="createPreset">创建</v-btn>
              <v-btn variant="tonal" color="warning" size="small" prepend-icon="mdi-content-save" :disabled="preset.key === 'default'" @click="replacePreset">保存</v-btn>
              <v-btn variant="tonal" color="error" size="small" prepend-icon="mdi-delete" :disabled="preset.key === 'default'" @click="deletePreset">删除</v-btn>
            </div>
          </div>
        </section>
      </template>

      <!-- 展开覆盖层：侧边栏导航 + 详细设置 -->
      <transition name="overlay-slide">
        <div v-if="activeSection" class="expanded-overlay">
          <!-- 侧边导航栏 -->
          <nav class="expanded-nav" :style="{ '--nav-accent': currentCategory?.color }">
            <div class="nav-header">
              <v-btn variant="text" density="compact" @click="activeSection = null" class="nav-back">
                <v-icon icon="mdi-arrow-left" size="20" />
              </v-btn>
              <span class="nav-title">设置</span>
            </div>
            <div class="nav-list">
              <div
                v-for="cat in categories"
                :key="cat.key"
                class="nav-item"
                :class="{ 'is-active': activeSection === cat.key }"
                :style="{ '--item-accent': cat.color }"
                @click="activeSection = cat.key">
                <div class="nav-icon-wrapper" :style="{ background: activeSection === cat.key ? cat.gradient : 'transparent' }">
                  <v-icon :icon="cat.icon" size="20" :color="activeSection === cat.key ? 'white' : cat.color" />
                </div>
                <span class="nav-label">{{ cat.title }}</span>
              </div>
            </div>
            <!-- 底部预设栏 -->
            <div class="nav-preset">
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
                density="compact"
                variant="outlined"
                hide-details
                return-object
                class="preset-mini" />
            </div>
          </nav>

          <!-- 详细设置区域 -->
          <main class="expanded-content" :style="{ '--content-accent': currentCategory?.color }">
            <div class="content-scroll">
              <div class="detail-content">
              <!-- 输出设置 -->
              <template v-if="activeSection === 'output'">
                <div class="section-group">
                  <div class="group-label">基础设置</div>
                  <v-combobox v-model="resolution" :items="RESOLUTIONS" :label="t('resolution')" density="compact" variant="outlined" hide-details />
                  <div class="field-row">
                    <v-text-field v-model="fps" :label="t('fps')" density="compact" variant="outlined" hide-details />
                    <v-text-field v-model="sampleCount" :label="t('sample-count')" density="compact" variant="outlined" hide-details />
                  </div>
                </div>
                <div class="section-group">
                  <div class="group-label">编码设置</div>
                  <div class="field-row">
                    <v-select v-model="videoCodec" :items="VIDEO_CODECS" item-title="title" item-value="value" :label="t('video-codec')" density="compact" variant="outlined" hide-details />
                    <v-select v-model="encoder" :items="ENCODERS" item-title="title" item-value="value" :label="t('encoder-select')" density="compact" variant="outlined" hide-details />
                  </div>
                  <div class="field-row">
                    <v-select v-model="ffmpegPreset" :items="FFMPEG_PRESETS" :label="t('ffmpeg-preset')" density="compact" variant="outlined" hide-details />
                    <v-select v-model="videoFormat" :items="['mp4', 'mov']" :label="t('video-format')" density="compact" variant="outlined" hide-details />
                  </div>
                  <div class="field-row">
                    <v-select v-model="bitrateControl" :items="['CRF', 'CBR']" :label="t('bitrate-control')" density="compact" variant="outlined" hide-details />
                    <v-text-field v-model="bitrate" :label="t('bitrate')" density="compact" variant="outlined" hide-details />
                  </div>
                </div>
                <div class="section-group">
                  <div class="group-label">性能优化</div>
                  <div class="switches">
                    <TipSwitch v-model="hwAccel" :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" density="compact" color="primary" />
                    <TipSwitch v-model="fxaa" :label="t('fxaa')" :tooltip="t('fxaa-tips')" density="compact" color="primary" />
                    <TipSwitch v-model="ffmpegThread" :label="t('ffmpeg-thread')" density="compact" color="primary" />
                  </div>
                </div>
              </template>

              <!-- 录制设置 -->
              <template v-if="activeSection === 'recording'">
                <div class="section-group">
                  <div class="group-label">音量控制</div>
                  <div class="volume-grid">
                    <div class="volume-item">
                      <v-icon icon="mdi-music" size="small" />
                      <span class="volume-label">{{ t('volume-music') }}</span>
                      <v-slider v-model="volumeMusic" :min="0" :max="2" :step="0.01" hide-details color="primary" />
                      <span class="volume-val">{{ Math.round(volumeMusic * 100) }}%</span>
                    </div>
                    <div class="volume-item">
                      <v-icon icon="mdi-volume-high" size="small" />
                      <span class="volume-label">{{ t('volume-sfx') }}</span>
                      <v-slider v-model="volumeSfx" :min="0" :max="2" :step="0.01" hide-details color="primary" />
                      <span class="volume-val">{{ Math.round(volumeSfx * 100) }}%</span>
                    </div>
                  </div>
                </div>

                <div class="section-group">
                  <div class="group-label">音频设置</div>

                  <!-- 第一行：音频格式和比特深度 -->
                  <div class="settings-row">
                    <v-select v-model="audioFormat" :items="AUDIO_FORMATS" :label="t('audio-format')" density="compact" variant="outlined" hide-details class="flex-item" />
                    <v-select
                      v-model="audioBit"
                      :items="AUDIO_BITS"
                      :label="t('audio-bit-depth')"
                      density="compact"
                      variant="outlined"
                      hide-details
                      :disabled="audioFormat !== 'wav'"
                      class="flex-item" />
                  </div>

                  <!-- 第二行：采样率和缓冲区大小 -->
                  <div class="settings-row">
                    <v-select v-model="targetAudio" :items="SAMPLE_RATES" :label="t('target_audio')" density="compact" variant="outlined" hide-details class="flex-item" />
                    <v-text-field v-model="bufferSize" :label="t('buffer_size')" type="number" density="compact" variant="outlined" hide-details class="flex-item" />
                  </div>
                </div>
              </template>

              <!-- 游戏设置 -->
              <template v-if="activeSection === 'game'">
                <div class="game-settings-container">
                  <!-- 选项卡 -->
                  <v-tabs v-model="activeTab" color="primary" grow class="settings-tabs">
                    <v-tab value="display">显示设置</v-tab>
                    <v-tab value="chart">谱面设置</v-tab>
                  </v-tabs>

                  <!-- 选项卡内容区域 - 可滚动 -->
                  <div class="tabs-content">
                    <v-window v-model="activeTab" class="content-window">
                      <!-- 显示设置页 -->
                      <v-window-item value="display" class="tab-panel">
                        <div class="settings-list">
                          <div class="field-row">
                            <v-text-field v-model="combo" :label="t('combo')" density="compact" variant="outlined" hide-details />
                            <v-text-field v-model="watermark" :label="t('watermark')" density="compact" variant="outlined" hide-details />
                          </div>
                          <v-text-field v-model="endingLength" :label="t('ending-length')" density="compact" variant="outlined" hide-details />
                        </div>
                      </v-window-item>

                      <!-- 谱面设置页 -->
                      <v-window-item value="chart" class="tab-panel py-2">
                        <div class="setting-item mb-6">
                          <div class="setting-header">
                            <span class="setting-label">{{ t('chart_ratio') }}</span>
                            <span class="setting-value">{{ chartRatio.toFixed(2) }}</span>
                          </div>
                          <v-slider
                            v-model="chartRatio"
                            :min="0.05"
                            :max="1"
                            :step="0.01"
                            hide-details
                            color="primary"
                            track-color="grey-lighten-3"
                            thumb-size="0" />
                        </div>

                        <div class="setting-item">
                          <div class="setting-header">
                            <span class="setting-label">{{ t('note-speed-factor') }}</span>
                            <span class="setting-value">{{ noteSpeedFactor.toFixed(2) }}</span>
                          </div>
                          <v-slider
                            v-model="noteSpeedFactor"
                            :min="0.1"
                            :max="2"
                            :step="0.01"
                            hide-details
                            color="primary"
                            track-color="grey-lighten-3"
                            thumb-size="0" />
                        </div>
                      </v-window-item>
                    </v-window>
                  </div>

                  <!-- 底部固定的开关区域 -->
                  <div class="fixed-switches">
                    <v-divider></v-divider>
                    <div class="switches">
                      <v-switch v-model="background" :label="t('background')" density="compact" color="primary" hide-details />
                      <v-switch v-model="bar" :label="t('bar')" density="compact" color="primary" hide-details />
                    </div>
                  </div>
                </div>
              </template>

              <!-- 视觉效果 -->
              <template v-if="activeSection === 'graphics'">
                <div class="section-group">
                  <div class="group-label">资源包</div>
                  <div class="respack-row">
                    <v-select v-model="respack" :items="respacks" item-title="name" :label="t('respack')" density="compact" variant="outlined" hide-details class="flex-grow-1" />
                    <v-btn icon="mdi-refresh" size="small" variant="text" @click="updateRespacks" />
                    <v-btn icon="mdi-folder-open" size="small" variant="text" @click="openRespackFolder" />
                  </div>
                </div>
                <div class="section-group">
                  <div class="group-label">音符设置</div>
                  <v-slider v-model="noteScale" :label="t('note-scale')" :min="0" :max="5" :step="0.05" thumb-label="always" hide-details />
                </div>
                <div class="section-group">
                  <div class="group-label">效果开关</div>
                  <div class="switches">
                    <TipSwitch v-model="doubleHint" :label="t('double-hint')" density="compact" color="primary" />
                    <TipSwitch v-model="aggressive" :label="t('aggressive')" :tooltip="t('aggressive-tips')" density="compact" color="primary" />
                    <TipSwitch v-model="disableParticle" :label="t('disable-particle')" density="compact" color="primary" />
                    <TipSwitch v-model="disableEffect" :label="t('disable-effect')" density="compact" color="primary" />
                  </div>
                </div>
                <div class="section-group">
                  <div class="group-label">UI显示</div>
                  <div class="ui-toggles">
                    <div v-for="(item, index) in renderList" :key="index" class="ui-toggle-item" :class="{ 'is-active': render.includes(item) }" @click="toggleRender(item)">
                      <v-icon :icon="render.includes(item) ? 'mdi-checkbox-marked' : 'mdi-checkbox-blank-outline'" size="18" />
                      <span>{{ item }}</span>
                    </div>
                  </div>
                </div>
              </template>

              <!-- 玩家信息 -->
              <template v-if="activeSection === 'player'">
                <div class="section-group">
                  <div class="group-label">个人信息</div>
                  <div class="avatar-section">
                    <v-avatar size="64" color="surface-variant">
                      <v-img v-if="playerAvatar" :src="playerAvatar" />
                      <v-icon v-else icon="mdi-account" size="40" />
                    </v-avatar>
                    <v-btn @click="chooseAvatar" variant="outlined" size="small">{{ t('player-avatar') }}</v-btn>
                  </div>
                  <div class="field-row">
                    <v-text-field v-model="playerName" :label="t('player-name')" density="compact" variant="outlined" hide-details />
                    <v-text-field v-model="playerRks" :label="t('player-rks')" density="compact" variant="outlined" hide-details />
                  </div>
                </div>
                <div class="section-group">
                  <div class="group-label">课题模式</div>
                  <div class="field-row">
                    <v-text-field v-model="challengeRank" :label="t('challenge-rank')" density="compact" variant="outlined" hide-details />
                    <v-select v-model="challengeColor" :items="t('challenge-colors').split(',')" :label="t('challenge-color')" density="compact" variant="outlined" hide-details />
                  </div>
                </div>
              </template>

              <!-- 调试工具 -->
              <template v-if="activeSection === 'debug'">
                <div class="section-group">
                  <div class="group-label">调试选项</div>
                  <div class="switches">
                    <v-switch v-model="chartDebug" :label="t('chart_debug')" density="compact" color="primary" hide-details />
                    <v-switch v-model="flidX" :label="t('flid_x')" density="compact" color="primary" hide-details />
                    <v-switch v-model="handSplit" :label="t('hand-split')" density="compact" color="primary" hide-details />
                    <TipSwitch v-model="disableLoading" :label="t('disable-loading')" :tooltip="t('disable-loading-tips')" density="compact" color="warning" />
                  </div>
                </div>
              </template>
            </div>
          </div>
        </main>
        </div>
      </transition>
    </div>
  </v-form>
</template>

<style scoped>
/* ============================================
   Android 15 Material You 风格设计系统
   ============================================ */

/* CSS 变量 - 响应式自适应 */
.config-wrapper {
  /* 间距系统 */
  --space-xs: clamp(4px, 0.4vw, 8px);
  --space-sm: clamp(8px, 0.8vw, 12px);
  --space-md: clamp(12px, 1.2vw, 20px);
  --space-lg: clamp(16px, 1.6vw, 28px);
  --space-xl: clamp(24px, 2.4vw, 40px);

  /* 字体大小 */
  --text-xs: clamp(10px, 0.75vw, 12px);
  --text-sm: clamp(11px, 0.85vw, 14px);
  --text-base: clamp(13px, 1vw, 16px);
  --text-lg: clamp(15px, 1.15vw, 18px);
  --text-xl: clamp(17px, 1.3vw, 22px);

  /* 圆角 */
  --radius-sm: clamp(6px, 0.6vw, 10px);
  --radius-md: clamp(10px, 1vw, 16px);
  --radius-lg: clamp(14px, 1.4vw, 24px);
  --radius-xl: clamp(20px, 2vw, 32px);

  /* 动画曲线 */
  --ease-out: cubic-bezier(0.33, 1, 0.68, 1);
  --ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --spring: cubic-bezier(0.34, 1.56, 0.64, 1);
}

.config-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: rgba(18, 18, 18, 0.92);
  backdrop-filter: blur(32px) saturate(180%);
  -webkit-backdrop-filter: blur(32px) saturate(180%);
  position: relative;
}

/* ============================================
   Android 15 三区块布局
   ============================================ */
.android-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  height: 100%;
  padding: var(--space-md);
  overflow: hidden;
  position: relative;
}

/* 横屏/宽屏布局 */
@media (min-width: 768px) and (min-aspect-ratio: 4/3) {
  .android-layout {
    display: grid;
    grid-template-columns: minmax(280px, 1fr) minmax(240px, 0.8fr);
    grid-template-rows: auto 1fr;
    gap: var(--space-lg);
  }
}

@media (min-width: 1200px) {
  .android-layout {
    display: grid;
    grid-template-columns: minmax(320px, 1.2fr) minmax(280px, 1fr) minmax(200px, 0.8fr);
    grid-template-rows: 1fr;
  }
}

/* ============================================
   区块通用样式
   ============================================ */
section {
  display: flex;
  flex-direction: column;
  background: rgba(30, 30, 30, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: all 0.35s var(--ease-out);
}

section:hover {
  border-color: rgba(255, 255, 255, 0.1);
}

.section-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(40, 40, 40, 0.4);
}

.section-header .v-icon {
  color: rgba(255, 255, 255, 0.6);
}

.section-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.75);
  letter-spacing: 0.3px;
}

.expand-btn {
  margin-left: auto;
  border-radius: var(--radius-sm);
  transition: all 0.2s var(--ease-out);
}

.expand-btn.is-active {
  background: rgba(80, 80, 80, 0.4);
}

/* ============================================
   区块1：分类卡片（主要区域）
   ============================================ */
.section-cards {
  flex: 1 1 auto;
  min-height: 120px;
}

@media (min-width: 768px) and (min-aspect-ratio: 4/3) {
  .section-cards {
    grid-row: 1 / 3;
    flex: none;
  }
}

@media (min-width: 1200px) {
  .section-cards {
    /* 第一列 */
    flex: none;
  }
}

.cards-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--space-sm);
  scrollbar-width: thin;
  scrollbar-color: rgba(80, 80, 80, 0.5) transparent;
}

.cards-scroll::-webkit-scrollbar {
  width: 4px;
}

.cards-scroll::-webkit-scrollbar-thumb {
  background: rgba(80, 80, 80, 0.5);
  border-radius: 4px;
}

.cards-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(clamp(140px, 45%, 180px), 1fr));
  gap: var(--space-sm);
  align-content: start;
}

@media (min-width: 768px) {
  .cards-list {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 1200px) {
  .cards-list {
    grid-template-columns: 1fr;
  }
}

/* Android 15 风格卡片 */
.category-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-md);
  background: rgba(45, 45, 45, 0.5);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.25s var(--ease-out);
  overflow: hidden;
  border: 1px solid transparent;
}

.category-card::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--card-accent);
  opacity: 0;
  transition: opacity 0.25s var(--ease-out);
}

.category-card:hover {
  background: rgba(55, 55, 55, 0.6);
  transform: scale(1.01);
  border-color: rgba(255, 255, 255, 0.08);
}

.category-card:hover::before {
  opacity: 0.06;
}

.category-card:active {
  transform: scale(0.98);
}

.category-card.is-selected {
  background: rgba(60, 60, 60, 0.7);
  border-color: var(--card-accent);
}

.category-card.is-selected::before {
  opacity: 0.12;
}

/* 卡片波纹效果 */
.card-ripple {
  position: absolute;
  inset: 0;
  overflow: hidden;
  border-radius: inherit;
  pointer-events: none;
}

.card-ripple::after {
  content: '';
  position: absolute;
  width: 100%;
  height: 100%;
  background: radial-gradient(circle at center, rgba(255, 255, 255, 0.15), transparent 70%);
  transform: scale(0);
  opacity: 0;
  transition: transform 0.5s ease, opacity 0.3s ease;
}

.category-card:active .card-ripple::after {
  transform: scale(2);
  opacity: 1;
  transition: none;
}

/* 卡片图标 */
.card-icon-wrapper {
  width: clamp(36px, 5vw, 44px);
  height: clamp(36px, 5vw, 44px);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

/* 卡片信息 */
.card-info {
  flex: 1;
  min-width: 0;
  position: relative;
  z-index: 1;
}

.card-title {
  font-size: var(--text-base);
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 2px;
}

.card-subtitle {
  font-size: var(--text-xs);
  color: rgba(255, 255, 255, 0.45);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-arrow {
  color: rgba(255, 255, 255, 0.3);
  transition: all 0.25s var(--ease-out);
  opacity: 0;
}

.card-arrow.is-visible {
  opacity: 1;
  color: var(--card-accent);
}

/* ============================================
   区块2：快速设置（中等区域）
   ============================================ */
.section-quick {
  flex: 2 1 auto;
  min-height: 180px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

@media (min-width: 768px) and (min-aspect-ratio: 4/3) {
  .section-quick {
    grid-column: 2;
    grid-row: 1;
    flex: none;
  }
}

@media (min-width: 1200px) {
  .section-quick {
    grid-column: 2;
  }
}

/* 快速设置滚动容器 */
.quick-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
  scrollbar-width: thin;
  scrollbar-color: rgba(80, 80, 80, 0.5) transparent;
}

.quick-scroll::-webkit-scrollbar {
  width: 4px;
}

.quick-scroll::-webkit-scrollbar-thumb {
  background: rgba(80, 80, 80, 0.5);
  border-radius: 4px;
}

/* 快速设置网格 */
.quick-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-xs);
  padding: var(--space-sm);
  transition: all 0.3s var(--ease-out);
}

@media (max-width: 767px) {
  .quick-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

.quick-grid.is-expanded {
  grid-template-columns: 1fr;
}

/* 快速设置瓷砖 - Android 15 风格 */
.quick-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-md) var(--space-sm);
  background: rgba(50, 50, 50, 0.5);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s var(--ease-out);
  position: relative;
  overflow: hidden;
  min-height: 80px;
}

.quick-tile:hover {
  background: rgba(60, 60, 60, 0.6);
  transform: translateY(-2px);
}

.quick-tile:active {
  transform: scale(0.96);
}

.quick-grid.is-expanded .quick-tile {
  flex-direction: row;
  justify-content: flex-start;
  min-height: auto;
  padding: var(--space-sm) var(--space-md);
}

.tile-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: rgba(70, 70, 70, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.7);
}

.tile-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.quick-grid.is-expanded .tile-content {
  flex-direction: row;
  gap: var(--space-sm);
}

.tile-value {
  font-size: var(--text-sm);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

.tile-label {
  font-size: var(--text-xs);
  color: rgba(255, 255, 255, 0.5);
}

/* 瓷砖编辑器 */
.tile-editor {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(35, 35, 35, 0.95);
  padding: var(--space-sm);
  border-radius: var(--radius-sm);
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);
}

/* 瓷砖展开动画 */
.tile-expand-enter-active,
.tile-expand-leave-active {
  transition: all 0.25s var(--ease-out);
}

.tile-expand-enter-from,
.tile-expand-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

/* 快速编辑面板 */
.quick-panel {
  padding: var(--space-md);
  overflow-y: auto;
  background: rgba(35, 35, 35, 0.4);
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.panel-row {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.panel-row:last-child {
  margin-bottom: 0;
}

.panel-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.panel-field label {
  font-size: var(--text-xs);
  font-weight: 500;
  color: rgba(255, 255, 255, 0.55);
}

/* 面板滑动动画 */
.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: all 0.35s var(--ease-out);
  transform-origin: top;
}

.panel-slide-enter-from,
.panel-slide-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.panel-slide-enter-to,
.panel-slide-leave-from {
  opacity: 1;
  max-height: 300px;
}

/* ============================================
   区块3：预设管理（较小区域）
   ============================================ */
.section-preset {
  flex-shrink: 0;
  min-height: fit-content;
}

@media (min-width: 768px) and (min-aspect-ratio: 4/3) {
  .section-preset {
    grid-column: 2;
    grid-row: 2;
  }
}

@media (min-width: 1200px) {
  .section-preset {
    grid-column: 3;
    grid-row: 1;
  }
}

.preset-content {
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.preset-select {
  flex: 1;
}

.preset-buttons {
  display: flex;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.preset-buttons .v-btn {
  flex: 1;
  min-width: 80px;
}

/* ============================================
   详细设置面板（覆盖层）
   ============================================ */
.detail-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  background: rgba(15, 15, 15, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  max-width: 480px;
  margin: 0 auto;
  background: rgba(28, 28, 28, 0.95);
  border-radius: 0;
  box-shadow: 0 0 60px rgba(0, 0, 0, 0.5);
}

@media (min-width: 768px) {
  .detail-panel {
    margin: var(--space-lg) var(--space-xl);
    border-radius: var(--radius-xl);
    height: calc(100% - var(--space-xl) * 2);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
}

/* 详细面板头部 */
.detail-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  background: rgba(40, 40, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  position: relative;
}

.detail-header::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--accent-color);
}

.back-btn {
  color: rgba(255, 255, 255, 0.7);
}

.header-info {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-left: var(--space-sm);
}

.header-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
}

/* 详细面板内容 */
.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
  scrollbar-width: thin;
  scrollbar-color: rgba(80, 80, 80, 0.5) transparent;
}

.detail-content::-webkit-scrollbar {
  width: 6px;
}

.detail-content::-webkit-scrollbar-track {
  background: transparent;
}

.detail-content::-webkit-scrollbar-thumb {
  background: rgba(80, 80, 80, 0.5);
  border-radius: 3px;
}

/* 设置分组 */
.section-group {
  margin-bottom: var(--space-lg);
  padding: var(--space-md);
  background: rgba(45, 45, 45, 0.4);
  border-radius: var(--radius-md);
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.group-label {
  font-size: var(--text-xs);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  margin-bottom: var(--space-sm);
  padding-bottom: var(--space-xs);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.field-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: var(--space-sm);
  margin-top: var(--space-sm);
}

.switches {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: var(--space-xs) var(--space-md);
}

/* 音量控制 */
.volume-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--space-sm);
}

@media (min-width: 500px) {
  .volume-grid {
    grid-template-columns: 1fr 1fr;
  }
}

.volume-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm);
  background: rgba(50, 50, 50, 0.4);
  border-radius: var(--radius-sm);
}

.volume-label {
  min-width: 50px;
  font-size: var(--text-sm);
  color: rgba(255, 255, 255, 0.6);
}

.volume-val {
  min-width: 40px;
  text-align: right;
  font-size: var(--text-sm);
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
}

.volume-item .v-slider {
  flex: 1;
  min-width: 0;
}

/* 设置行 */
.settings-row {
  display: flex;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.settings-row:last-child {
  margin-bottom: 0;
}

.flex-item {
  flex: 1;
  min-width: 0;
}

/* 资源包行 */
.respack-row {
  display: flex;
  gap: var(--space-sm);
  align-items: center;
}

/* 头像区域 */
.avatar-section {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  margin-bottom: var(--space-md);
}

/* UI 切换网格 */
.ui-toggles {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-xs);
}

@media (max-width: 400px) {
  .ui-toggles {
    grid-template-columns: repeat(2, 1fr);
  }
}

.ui-toggle-item {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm);
  background: rgba(50, 50, 50, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s var(--ease-out);
  font-size: var(--text-sm);
  color: rgba(255, 255, 255, 0.6);
  user-select: none;
}

.ui-toggle-item:hover {
  background: rgba(60, 60, 60, 0.5);
}

.ui-toggle-item.is-active {
  background: rgba(70, 70, 70, 0.6);
  border-color: rgba(100, 100, 100, 0.5);
  color: rgba(255, 255, 255, 0.9);
}

/* 游戏设置容器 */
.game-settings-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(30, 30, 30, 0.5);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.settings-tabs {
  flex-shrink: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(35, 35, 35, 0.5);
}

.tabs-content {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: var(--space-md);
}

.content-window {
  height: 100%;
}

.tab-panel {
  height: 100%;
}

/* 图形预览区域 */
.chart-preview-container {
  background: rgba(30, 30, 30, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--radius-md);
  padding: var(--space-md);
  margin-bottom: var(--space-lg);
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-sm);
}

.preview-label {
  font-size: var(--text-xs);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.preview-canvas {
  position: relative;
  height: clamp(100px, 18vw, 160px);
  background: linear-gradient(180deg, rgba(99, 102, 241, 0.08) 0%, rgba(30, 30, 30, 0.9) 100%);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.judge-line {
  position: absolute;
  bottom: clamp(16px, 2.5vw, 24px);
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, transparent, rgba(99, 102, 241, 0.8), rgba(236, 72, 153, 0.8), rgba(99, 102, 241, 0.8), transparent);
  box-shadow: 0 0 10px rgba(99, 102, 241, 0.5);
}

.note-demo {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: clamp(16px, 2.5vw, 24px);
  transform-origin: center bottom;
}

.demo-note {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  animation: noteFall linear infinite;
  opacity: 0;
}

.demo-note .note-inner {
  width: clamp(24px, 3.5vw, 36px);
  height: clamp(24px, 3.5vw, 36px);
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border-radius: 50%;
  box-shadow: 0 0 12px rgba(99, 102, 241, 0.6);
}

.drag-note .drag-body {
  width: clamp(36px, 5vw, 52px);
  height: clamp(12px, 1.8vw, 18px);
  border-radius: 10px;
  background: linear-gradient(90deg, #ec4899, #f472b6);
  box-shadow: 0 0 12px rgba(236, 72, 153, 0.6);
}

.hold-note .hold-body {
  width: clamp(18px, 2.5vw, 26px);
  height: clamp(44px, 7vw, 70px);
  border-radius: 15px;
  background: linear-gradient(180deg, #22d3ee, #06b6d4);
  box-shadow: 0 0 12px rgba(34, 211, 238, 0.6);
}

@keyframes noteFall {
  0% {
    top: -40px;
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  90% {
    opacity: 1;
  }
  100% {
    top: calc(100% - 16px);
    opacity: 0;
  }
}

.speed-indicator {
  position: absolute;
  bottom: 6px;
  left: 10px;
  right: 10px;
  display: flex;
  gap: 12px;
  font-size: 10px;
}

.speed-label {
  color: rgba(255, 255, 255, 0.45);
}

.speed-value {
  color: rgba(255, 255, 255, 0.85);
  font-weight: 600;
  font-family: 'Monaco', 'Consolas', monospace;
}

/* 设置项 */
.setting-item {
  background: rgba(50, 50, 50, 0.5);
  border-radius: var(--radius-sm);
  padding: var(--space-sm) var(--space-md);
  margin-bottom: var(--space-md);
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-xs);
}

.setting-label {
  font-size: var(--text-sm);
  color: rgba(255, 255, 255, 0.7);
}

.setting-value {
  font-size: var(--text-sm);
  font-weight: 600;
  color: #818cf8;
  font-family: 'Monaco', 'Consolas', monospace;
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.settings-list > * {
  background: rgba(50, 50, 50, 0.5);
  border-radius: var(--radius-sm);
  padding: var(--space-sm);
}

/* 固定开关区域 */
.fixed-switches {
  flex-shrink: 0;
  background: rgba(35, 35, 35, 0.6);
  padding: var(--space-sm) var(--space-md);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.fixed-switches .switches {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
  justify-content: center;
}

/* ============================================
   动画效果
   ============================================ */
.detail-enter-enter-active,
.detail-enter-leave-active {
  transition: all 0.4s var(--ease-out);
}

.detail-enter-enter-from,
.detail-enter-leave-to {
  opacity: 0;
}

.detail-enter-enter-from .detail-panel,
.detail-enter-leave-to .detail-panel {
  transform: translateY(40px) scale(0.95);
}

.detail-enter-enter-to .detail-panel,
.detail-enter-leave-from .detail-panel {
  transform: translateY(0) scale(1);
}

/* ============================================
   响应式适配
   ============================================ */
@media (max-width: 600px) {
  .android-layout {
    padding: var(--space-sm);
    gap: var(--space-sm);
  }

  .cards-list {
    grid-template-columns: 1fr 1fr;
  }

  .quick-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .panel-row {
    grid-template-columns: 1fr;
  }

  .field-row {
    grid-template-columns: 1fr;
  }

  .preset-buttons {
    flex-direction: column;
  }

  .preset-buttons .v-btn {
    width: 100%;
  }
}

/* 超宽屏幕 */
@media (min-width: 1600px) {
  .android-layout {
    grid-template-columns: minmax(360px, 1fr) minmax(320px, 1fr) minmax(240px, 0.9fr);
    padding: var(--space-xl);
  }
}

/* ============================================
   展开覆盖层：侧边栏导航 + 详细设置
   ============================================ */
.expanded-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  background: rgba(18, 18, 18, 0.97);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
}

/* 移动端：顶部导航栏 */
@media (max-width: 767px) {
  .expanded-overlay {
    flex-direction: column;
  }
}

/* 桌面端：侧边栏布局 */
@media (min-width: 768px) {
  .expanded-overlay {
    flex-direction: row;
  }
}

/* 覆盖层动画 */
.overlay-slide-enter-active,
.overlay-slide-leave-active {
  transition: all 0.35s var(--ease-out);
}

.overlay-slide-enter-from,
.overlay-slide-leave-to {
  opacity: 0;
}

.overlay-slide-enter-from .expanded-nav,
.overlay-slide-leave-to .expanded-nav {
  transform: translateX(-20px);
}

.overlay-slide-enter-from .expanded-content,
.overlay-slide-leave-to .expanded-content {
  transform: translateX(20px);
}

/* 侧边导航栏 */
.expanded-nav {
  display: flex;
  flex-direction: column;
  background: rgba(25, 25, 25, 0.95);
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
  transition: transform 0.35s var(--ease-out);
}

@media (max-width: 767px) {
  .expanded-nav {
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
    padding: var(--space-sm);
    border-right: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    gap: var(--space-sm);
  }
}

@media (min-width: 768px) {
  .expanded-nav {
    width: clamp(180px, 20vw, 240px);
    height: 100%;
  }
}

.nav-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

@media (max-width: 767px) {
  .nav-header {
    border-bottom: none;
    padding: 0;
  }
}

.nav-back {
  color: rgba(255, 255, 255, 0.7);
}

.nav-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

@media (max-width: 767px) {
  .nav-title {
    display: none;
  }
}

/* 导航列表 */
.nav-list {
  display: flex;
  flex-direction: column;
  flex: 1;
  padding: var(--space-sm);
  overflow-y: auto;
  gap: var(--space-xs);
}

@media (max-width: 767px) {
  .nav-list {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: center;
    flex: 0 0 auto;
    padding: 0;
    gap: var(--space-xs);
  }
}

/* 导航项 */
.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s var(--ease-out);
  position: relative;
  overflow: hidden;
}

@media (max-width: 767px) {
  .nav-item {
    flex-direction: column;
    padding: var(--space-xs);
    gap: 4px;
    min-width: 60px;
  }
}

.nav-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--item-accent);
  transform: scaleY(0);
  transition: transform 0.2s var(--ease-out);
}

@media (max-width: 767px) {
  .nav-item::before {
    left: 50%;
    top: auto;
    bottom: 0;
    right: auto;
    width: 100%;
    height: 2px;
    transform: translateX(-50%) scaleX(0);
  }
}

.nav-item:hover {
  background: rgba(60, 60, 60, 0.4);
}

.nav-item.is-active {
  background: rgba(70, 70, 70, 0.5);
}

.nav-item.is-active::before {
  transform: scaleY(1);
}

@media (max-width: 767px) {
  .nav-item.is-active::before {
    transform: translateX(-50%) scaleX(0.6);
  }
}

.nav-icon-wrapper {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s var(--ease-out);
}

@media (max-width: 767px) {
  .nav-icon-wrapper {
    width: 28px;
    height: 28px;
  }
}

.nav-label {
  font-size: var(--text-sm);
  color: rgba(255, 255, 255, 0.7);
  white-space: nowrap;
}

@media (max-width: 767px) {
  .nav-label {
    font-size: 10px;
  }
}

.nav-item.is-active .nav-label {
  color: rgba(255, 255, 255, 0.95);
}

/* 底部预设选择器 */
.nav-preset {
  padding: var(--space-sm);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

@media (max-width: 767px) {
  .nav-preset {
    display: none;
  }
}

.preset-mini {
  font-size: var(--text-xs);
}

/* 详细设置区域 */
.expanded-content {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  padding: var(--space-lg);
  background: rgba(20, 20, 20, 0.5);
}

@media (max-width: 767px) {
  .expanded-content {
    padding: var(--space-md);
  }
}

.expanded-content::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--content-accent);
}

@media (max-width: 767px) {
  .expanded-content::before {
    display: none;
  }
}

/* 内容区域头部 */
.content-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-lg);
  padding-bottom: var(--space-md);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.content-title {
  font-size: var(--text-xl);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
}

/* 内容滚动区域 */
.content-scroll {
  flex: 1;
  overflow-y: auto;
  height: 100%;
  scrollbar-width: thin;
  scrollbar-color: rgba(80, 80, 80, 0.5) transparent;
}

.content-scroll::-webkit-scrollbar {
  width: 6px;
}

.content-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.content-scroll::-webkit-scrollbar-thumb {
  background: rgba(80, 80, 80, 0.5);
  border-radius: 3px;
}

/* 内容动画 */
.android-layout > template {
  animation: fadeIn 0.3s var(--ease-out);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
</style>


