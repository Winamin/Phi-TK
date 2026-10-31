<i18n>
en:
  resolution: Resolution
  video-codec: Video Codec
  fps: FPS
  fxaa: FXAA
  fxaa-tips: Causes blur, not recommended
  sample-count: Sample Count
  sample-count-off: Off (1x)
  sample-count-device: 'GPU: {device}, up to {max}x MSAA'
  sample-count-unknown: 'GPU capability unavailable; using 1/2/4/8'
  bitrate: Target Bitrate
  bitrate-hint: 'CBR, in bps. 8M = 8000k = 8000000 bps (8 Mbps).'
  bitrate-invalid: 'Enter a bitrate between 200000 and 2147483647 bps.'
  player-avatar: Avatar
  player-name: Name
  player-rks: RKS
  challenge-color: Challenge Color
  challenge-colors: White,Green,Blue,Red,Golden,Rainbow
  challenge-rank: Challenge Rank
  respack: Resource Pack
  respack-default: '[Default]'
  score-anim: Score Anim
  note-scale: Note Scale
  render: UI Display
  double-hint: Double Hint
  aggressive: Aggressive Optimization
  aggressive-tips: May cause notes to disappear
  disable-particle: No Particles
  disable-effect: No Effects
  volume-music: Music Volume
  volume-sfx: SFX Volume
  ending-length: Ending Duration
  disable-loading: Skip Loading
  disable-loading-tips: May have issues
  chart_debug: Debug Mode
  chart_ratio: Chart Scale
  buffer_size: Buffer Size
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
    bitrate: Bitrate
    performance: Performance
    volume: Volume
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
  video-codec: 视频编码格式
  fps: 帧率
  fxaa: FXAA 抗锯齿
  fxaa-tips: 会导致画面模糊
  sample-count: 采样数
  sample-count-off: 关闭
  sample-count-device: 显卡：{device}，最高支持 {max}x MSAA
  sample-count-unknown: 未能检测到显卡信息，使用 1/2/4/8
  bitrate: 目标码率
  bitrate-hint: 'CBR，单位 bps。8M = 8000k = 8000000 bps（8 Mbps）。'
  bitrate-invalid: '请输入 200000 至 2147483647 bps 范围内的码率。'
  player-avatar: 头像
  player-name: 名称
  player-rks: RKS
  challenge-color: 课题颜色
  challenge-colors: 白,绿,蓝,红,金,彩
  score-anim: 分数动画
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
  ending-length: 结算时长
  disable-loading: 渲染时跳过加载
  disable-loading-tips: 可能有问题
  chart_debug: 调试模式
  chart_ratio: 谱面缩放
  buffer_size: 缓冲区
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
    bitrate: 码率
    performance: 性能
    volume: 音量
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
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { toast, anyFilter, toastError, validateFields } from '@/common';
import type { RenderConfig } from '@/model';
import { normalizeRenderConfig, parseBitrateBps } from '@/render-config';
import TipSwitch from './TipSwitch.vue';
import MdCombobox from './md/MdCombobox.vue';
import MdSelect from './md/MdSelect.vue';
import MdSlider from './md/MdSlider.vue';
import MdTextField from './md/MdTextField.vue';
import MdCheckbox from './md/MdCheckbox.vue';
defineProps<{ initAspectRatio?: number; floatingActions?: boolean }>();
const RESOLUTIONS = ['1280x720', '1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080', '7680x4320'];

const VIDEO_CODECS = [
  { value: 'h264', title: 'H.264' },
  { value: 'hevc', title: 'HEVC' },
];

const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

const form = ref<HTMLFormElement>();

const categories = [
  { key: 'output', icon: 'mdui-icon-videocam--outlined', activeIcon: 'mdui-icon-videocam' },
  { key: 'recording', icon: 'mdui-icon-music-note--outlined', activeIcon: 'mdui-icon-music-note' },
  { key: 'game', icon: 'mdui-icon-sports-esports--outlined', activeIcon: 'mdui-icon-sports-esports' },
  { key: 'graphics', icon: 'mdui-icon-palette--outlined', activeIcon: 'mdui-icon-palette' },
  { key: 'player', icon: 'mdui-icon-person--outlined', activeIcon: 'mdui-icon-person' },
  { key: 'debug', icon: 'mdui-icon-bug-report--outlined', activeIcon: 'mdui-icon-bug-report' },
] as const;

const activeCategory = ref<(typeof categories)[number]['key']>('output');

function onCategoryChange(event: Event) {
  const value = (event.target as HTMLElement & { value?: string }).value;
  const category = categories.find((item) => item.key === value);
  if (category) activeCategory.value = category.key;
}

const resolution = ref('1920x1080');
const fps = ref('60');
const sampleCount = ref(1);
const FALLBACK_MSAA_COUNTS = [1, 2, 4, 8];
const msaaCounts = ref(FALLBACK_MSAA_COUNTS);
const msaaDevice = ref('');
const msaaLoaded = ref(false);

const msaaItems = computed(() =>
  msaaCounts.value.map((samples) => ({
    title: samples === 1 ? t('sample-count-off') : `${samples}x`,
    value: samples,
  })),
);
const maxMsaa = computed(() => msaaCounts.value[msaaCounts.value.length - 1] ?? 1);

function clampSampleCount(samples: number) {
  const usable = msaaCounts.value.filter((n) => n <= samples);
  return usable.length ? usable[usable.length - 1] : 1;
}

async function updateMsaaSupport() {
  try {
    const support = (await invoke('get_msaa_sample_counts')) as { device: string; counts: number[] };
    if (support.counts.length) {
      msaaCounts.value = support.counts;
      msaaDevice.value = support.device;
      sampleCount.value = clampSampleCount(sampleCount.value);
    }
  } catch {
    return;
  } finally {
    msaaLoaded.value = true;
  }
}
updateMsaaSupport();
const videoCodec = ref('h264');

const bitrate = ref('8M');
const fxaa = ref(false);
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
const scoreAmin = ref(false);

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

const avatarUrl = computed(() => {
  if (!playerAvatar.value) return '';
  try {
    return convertFileSrc(playerAvatar.value);
  } catch {
    return '';
  }
});

const pctLabel = (value: number) => `${Math.round(value * 100)}%`;
const numLabel = (value: number) => value.toFixed(2);

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

function onRespackChange(value: unknown) {
  respack.value = respacks.value.find((x) => x.index === Number(value)) ?? respacks.value[0];
}

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
    endingLength: -2.0,
    disableLoading: true,
    chartDebug: false,
    flidX: false,
    chartRatio: 1,
    bufferSize: 256,
    fps: 60,
    videoCodec: 'h264',
    bitrateControl: 'CBR',
    bitrate: '8M',
    targetAudio: 48000,
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
    sampleCount: 4,
    resPackPath: null,
    speed: 1,
    volumeMusic: 1,
    volumeSfx: 1,
    combo: 'AUTOPLAY',
    watermark: '',
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
    scoreAmin: false,
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
  if (!validateFields(form.value)) {
    toast(t('has-error'), 'error');
    return null;
  }
  const [w, h] = resolution.value.split('x').map(Number);
  if (parseBitrateBps(bitrate.value) === null) {
    toast(t('bitrate-invalid'), 'error');
    return null;
  }
  return {
    resolution: [w, h],
    endingLength: parseFloat(endingLength.value),
    disableLoading: disableLoading.value,
    chartDebug: chartDebug.value,
    flidX: flidX.value,
    chartRatio: chartRatio.value,
    bufferSize: bufferSize.value,
    fps: parseInt(fps.value),
    videoCodec: videoCodec.value,
    bitrateControl: 'CBR',
    bitrate: bitrate.value.trim(),
    targetAudio: 48000,
    background: background.value,
    handSplit: handSplit.value,
    noteSpeedFactor: noteSpeedFactor.value,
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
    sampleCount: clampSampleCount(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    combo: combo.value,
    watermark: watermark.value,
    scoreAmin: scoreAmin.value,
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
  c = normalizeRenderConfig(c);
  resolution.value = c.resolution.join('x');
  endingLength.value = String(c.endingLength);
  disableLoading.value = c.disableLoading;
  chartDebug.value = c.chartDebug;
  flidX.value = c.flidX;
  chartRatio.value = c.chartRatio;
  bufferSize.value = c.bufferSize;
  fps.value = String(c.fps);
  videoCodec.value = c.videoCodec || 'h264';
  bitrate.value = c.bitrate || '8M';
  background.value = c.background ?? false;
  handSplit.value = c.handSplit ?? false;
  noteSpeedFactor.value = c.noteSpeedFactor;
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
  sampleCount.value = clampSampleCount(c.sampleCount || 1);
  respack.value = respacks.value.find((x) => x.path === c.resPackPath) || respacks.value[0];
  volumeMusic.value = c.volumeMusic;
  volumeSfx.value = c.volumeSfx;
  combo.value = c.combo;
  watermark.value = c.watermark;
  bar.value = c.bar ?? false;
  scoreAmin.value = c.scoreAmin;
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

const presetDialog = ref(false);
const presetName = ref('');

async function createPreset() {
  const config = await buildConfig();
  if (!config) return;
  const name = presetName.value.trim();
  if (!name) return;
  if (name === 'default') {
    toast(t('preset-cannot-use-default'), 'error');
    return;
  }
  try {
    await invoke('add_preset', { name, config });
    await updatePresets();
    preset.value = presets.value.find((x) => x.key === name) || presets.value[0];
    presetDialog.value = false;
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

    <div class="config-header">
      <mdui-tabs variant="secondary" :value="activeCategory" :aria-label="t('settings')" @change="onCategoryChange">
        <mdui-tab v-for="cat in categories" :key="cat.key" :value="cat.key">
          {{ t(`cat.${cat.key}`) }}
        </mdui-tab>
      </mdui-tabs>
      <div class="preset-toolbar">
        <MdSelect
          class="preset-select"
          variant="outlined"
          :model-value="preset.key"
          :items="presets"
          item-title="name"
          item-value="key"
          :label="t('presets')"
          @update:model-value="onPresetChange" />
        <mdui-dropdown placement="bottom-end">
          <mdui-button-icon slot="trigger" :aria-label="t('presets')">
            <mdui-icon-more-vert></mdui-icon-more-vert>
          </mdui-button-icon>
          <mdui-menu>
            <mdui-menu-item @click="presetName = ''; presetDialog = true">
              <mdui-icon-add slot="icon"></mdui-icon-add>
              {{ t('preset-new') }}
            </mdui-menu-item>
            <mdui-menu-item :disabled="preset.key === 'default'" @click="replacePreset">
              <mdui-icon-save--outlined slot="icon"></mdui-icon-save--outlined>
              {{ t('preset-save') }}
            </mdui-menu-item>
            <mdui-menu-item :disabled="preset.key === 'default'" @click="deletePreset">
              <mdui-icon-delete--outlined slot="icon"></mdui-icon-delete--outlined>
              {{ t('preset-delete') }}
            </mdui-menu-item>
          </mdui-menu>
        </mdui-dropdown>
      </div>
    </div>

    <div class="config-pane">

      <div class="settings-scroll" :class="{ 'has-floating-actions': floatingActions }">

        <div v-show="activeCategory === 'output'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.resolution-fps') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdCombobox v-model="resolution" :items="RESOLUTIONS" :label="t('resolution')" />
              </div>
              <div class="field-row">
                <MdTextField v-model="fps" variant="outlined" inputmode="numeric" :label="t('fps')" />
                <MdSelect v-model="sampleCount" variant="outlined" :items="msaaItems" :label="t('sample-count')" />
              </div>
              <div v-if="msaaLoaded" class="field-hint">
                {{
                  msaaDevice
                    ? t('sample-count-device', { device: msaaDevice, max: maxMsaa })
                    : t('sample-count-unknown')
                }}
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('video-codec') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdSelect v-model="videoCodec" variant="outlined" :items="VIDEO_CODECS" :label="t('video-codec')" />
              </div>

            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.bitrate') }}</h2>
            <div class="setting-card">
              <div class="field-row">
                <MdTextField v-model="bitrate" variant="outlined" required :label="t('bitrate')" :helper="t('bitrate-hint')" />
              </div>
            </div>
          </section>

          <section class="settings-section">
            <h2 class="section-title">{{ t('group.performance') }}</h2>
            <div class="setting-card">
              <div class="switch-grid">
                <TipSwitch v-model="fxaa" :label="t('fxaa')" :tooltip="t('fxaa-tips')" />
              </div>
            </div>
          </section>
        </div>

        <div v-show="activeCategory === 'recording'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.volume') }}</h2>
            <div class="setting-card">
              <div class="setting-item">
                <span class="md3-icon-badge item-icon"><mdui-icon-music-note></mdui-icon-music-note></span>
                <span class="item-label">{{ t('volume-music') }}</span>

                <span class="item-value">{{ pctLabel(volumeMusic) }}</span>
              </div>
              <MdSlider v-model="volumeMusic" class="item-slider" :aria-label="t('volume-music')" :min="0" :max="2" :step="0.01" :labelFormatter="pctLabel" />
              <mdui-divider></mdui-divider>
              <div class="setting-item">
                <span class="md3-icon-badge item-icon"><mdui-icon-volume-up></mdui-icon-volume-up></span>
                <span class="item-label">{{ t('volume-sfx') }}</span>
                <span class="item-value">{{ pctLabel(volumeSfx) }}</span>
              </div>
              <MdSlider v-model="volumeSfx" class="item-slider" :aria-label="t('volume-sfx')" :min="0" :max="2" :step="0.01" :labelFormatter="pctLabel" />
            </div>
          </section>

        </div>

        <div v-show="activeCategory === 'game'" class="settings-stack">
          <section class="settings-section">
            <h2 class="section-title">{{ t('group.chart-scale') }}</h2>
            <div class="setting-card">
              <div class="setting-item">
                <span class="item-label">{{ t('chart_ratio') }}</span>
                <span class="item-value">{{ numLabel(chartRatio) }}</span>
              </div>
              <MdSlider v-model="chartRatio" class="item-slider" :aria-label="t('chart_ratio')" :min="0.05" :max="1" :step="0.01" :labelFormatter="numLabel" />
              <mdui-divider></mdui-divider>
              <div class="setting-item">
                <span class="item-label">{{ t('note-speed-factor') }}</span>
                <span class="item-value">{{ numLabel(noteSpeedFactor) }}</span>
              </div>
              <MdSlider v-model="noteSpeedFactor" class="item-slider" :aria-label="t('note-speed-factor')" :min="0.1" :max="2" :step="0.01" :labelFormatter="numLabel" />
              <mdui-divider></mdui-divider>
              <div class="switch-grid">
                <TipSwitch v-model="background" :label="t('background')" />
                <TipSwitch v-model="bar" :label="t('bar')" />
                <TipSwitch v-model="scoreAmin" :label="t('score-anim')" />
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
                  <mdui-button-icon :aria-label="t('refresh')" @click="updateRespacks">
                    <mdui-icon-refresh></mdui-icon-refresh>
                  </mdui-button-icon>
                </mdui-tooltip>
                <mdui-tooltip :content="t('open-folder')">
                  <mdui-button-icon :aria-label="t('open-folder')" @click="openRespackFolder">
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
              <MdSlider v-model="noteScale" class="item-slider" :aria-label="t('note-scale')" :min="0" :max="5" :step="0.05" :labelFormatter="numLabel" />
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

              <div class="ui-grid">
                <MdCheckbox
                  v-for="item in renderList"
                  :key="item"
                  :model-value="render.includes(item)"
                  @update:model-value="checked => { checked ? render.push(item) : render = render.filter(x => x !== item) }">
                  {{ item }}
                </MdCheckbox>
              </div>
            </div>
          </section>
        </div>

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
  <mdui-dialog :open="presetDialog" :headline="t('preset-create-title')" close-on-overlay-click @close="presetDialog = false">
    <MdTextField v-model="presetName" variant="outlined" required :label="t('preset-create-title')" @keydown.enter="createPreset" />
    <mdui-button slot="action" variant="text" @click="presetDialog = false">{{ t('back') }}</mdui-button>
    <mdui-button slot="action" variant="filled" :disabled="!presetName.trim()" @click="createPreset">{{ t('preset-save') }}</mdui-button>
  </mdui-dialog>
</template>

<style scoped lang="scss">
@use '../styles/motion' as mo;

.field-hint {
  margin: 0;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.config-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  background-color: rgb(var(--mdui-color-surface));
}

.config-header {
  flex-shrink: 0;
  min-width: 0;
}

.config-header mdui-tabs {
  min-width: 0;
}

.config-header mdui-tabs::part(container) {
  scrollbar-width: none;
}

.config-header mdui-tab {
  flex-shrink: 0;
  min-width: 64px;
}

.preset-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px var(--app-page-padding, 16px);
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  margin: 0 auto;
}

.preset-select {
  flex: 1;
  min-width: 0;
}

.config-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.settings-scroll {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior-y: contain;
  padding: 0 var(--app-page-padding, 16px) var(--app-section-gap, 24px);
  scroll-padding-bottom: 24px;
}

.settings-scroll.has-floating-actions {
  padding-bottom: calc(96px + env(safe-area-inset-bottom));
  scroll-padding-bottom: calc(96px + env(safe-area-inset-bottom));
}

.settings-stack {
  display: flex;
  flex-direction: column;
  gap: var(--app-section-gap, 24px);
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  margin: 0 auto;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;

  @include mo.enter-rise(10px);
  @include mo.stagger(6, 40ms, 40ms);
}

.section-title {
  margin: 0;
  padding: 0;
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: var(--mdui-typescale-title-small-weight);
  line-height: var(--mdui-typescale-title-small-line-height);
  letter-spacing: var(--mdui-typescale-title-small-tracking);
  color: rgb(var(--mdui-color-primary));
}

.setting-card {
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-medium);
  padding: var(--app-card-padding, 16px);
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.field-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 16px;
}

.field-row > :only-child {
  grid-column: 1 / -1;
}

.switch-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 8px 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 40px;
}

.item-icon {
  width: 40px;
  height: 40px;
  font-size: 24px;
}

.item-label {
  flex: 1;
  min-width: 0;
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
  overflow-wrap: anywhere;
  color: rgb(var(--mdui-color-on-surface));
}

.item-value {
  min-width: 3.5rem;
  text-align: right;
  flex-shrink: 0;
  color: rgb(var(--mdui-color-primary));
  font-size: var(--mdui-typescale-body-medium-size);
  font-weight: var(--mdui-typescale-label-large-weight);
  font-variant-numeric: tabular-nums;
}

.item-slider {
  display: block;
  padding: 0;
}

.respack-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.respack-select {
  flex: 1 1 100%;
  min-width: 0;
}

.ui-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 0 16px;
}

.avatar-row {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  align-items: center;
  gap: 12px;
}

.avatar-row mdui-button {
  grid-column: 1 / -1;
}

.avatar {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--mdui-shape-corner-full);
  overflow: hidden;
  background-color: rgb(var(--mdui-color-secondary-container));
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
  overflow-wrap: anywhere;
}

mdui-button-icon {
  width: var(--app-touch-target, 48px);
  height: var(--app-touch-target, 48px);
}

.respack-row mdui-tooltip:first-of-type {
  margin-inline-start: auto;
}

@media (min-width: 600px) {
  .field-row,
  .ui-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .respack-select {
    flex: 1;
  }

  .respack-row mdui-tooltip:first-of-type {
    margin-inline-start: 0;
  }

  .avatar-row {
    grid-template-columns: 48px minmax(0, 1fr) auto;
  }

  .avatar-row mdui-button {
    grid-column: auto;
  }
}

</style>
