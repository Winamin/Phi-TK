<i18n>
en:
  title:
    output: Output
    player: Player
    graphics: Graphics
    audio: Audio

  resolution: Resolution
  ffmpeg-preset: Preset
  fps: FPS

  hw-accel: Hardware Acceleration
  hw-accel-tips: Improve rendering speed, slightly reduce quality

  fxaa: FXAA
  fxaa-tips: FXAA, as a low-cost anti-aliasing method, will cause the picture to be blurred, and it is not recommended to turn it on

  sample-count: Sample Count
  sample-count-tips: Must be a power of 2. A non-1 sample count enables MSAA, which can improve the quality of the picture while increasing the performance cost

  bitrate-control: Bitrate Control
  bitrate: Quantization parameters/Bitrate
  bitrate-tips: CRF-This is the CRF level. CBR-This is the bitrate

  player-avatar: Player Avatar
  player-name: Player Name
  player-rks: Player Rks.

  image-filter: Image

  challenge-color: Challenge Mode Color
  challenge-colors: White,Green,Blue,Red,Golden,Rainbow

  challenge-rank: Challenge Mode Rank

  respack: Resource Pack
  respack-default: '[Default]'
  respack-refresh: Refresh
  respack-open: Open Folder

  note-scale: Note Scale

  double-hint: Double Hit Hint

  aggressive: Aggressive Optimization
  aggressive-tips: Hide off-screen note, Improve rendering speed, but may cause some notes to disappear

  disable-particle: Disable Particle
  disable-effect: Disable Effect

  volume-music: Music Volume
  volume-sfx: SFX Volume

  ending-length: Result Screen Duration
  disable-loading: Loading disabled
  disable-loading-tips: It's a bit problematic and not recommended
  chart_debug: Debug Mode
  chart_ratio: Chart Zoom
  buffer_size: Adjust Buffer Size
  buffer_size-tips: Not very useful
  target_audio: Target sample rate
  combo: Customize COMBO Text
  watermark: Watermark Text
  flid_x: Mirror Mode
  background: Remove background rendering

  render-list: Total score,name,judgment line,level,combo,progress bar,Progress bar 1,Progress bar 2,Pause

  presets: Presets
  preset-refresh: Refresh
  preset-create: Create
  preset-create-title: Preset name
  preset-created: Preset created
  preset-delete: Delete
  preset-deleted: Preset deleted
  preset-replace: Replace
  preset-replaced: Preset replaced
  preset-cannot-use-default: Cannot use 'default' as preset name
  default-preset: Default

zh-CN:
  title:
    output: 输出
    player: 玩家
    graphics: 图像
    audio: 音频

  resolution: 分辨率
  ffmpeg-preset: 预设
  fps: FPS

  hw-accel: 硬件加速
  hw-accel-tips: 提升渲染速度，略微降低质量

  fxaa: FXAA
  fxaa-tips: FXAA 以低成本实现抗锯齿，但会导致画面模糊，不建议开启

  hevc: HEVC编码
  hevc-tips: 使用 HEVC 编码，压缩率更高，渲染速度更慢

  sample-count: 采样数
  sample-count-tips: 非 1 的采样数(必须为 2 的幂)会启用 MSAA(若开头无画面请关闭此项)

  bitrate-control: 码率控制
  bitrate: 量化参数/码率
  bitrate-tips: CRF-该项为CRF级别 CBR-该项为码率

  player-avatar: 玩家头像
  player-name: 玩家名
  player-rks: 玩家 RKS

  image-filter: 图像

  challenge-color: 课题模式颜色
  challenge-colors: 白,绿,蓝,红,金,彩

  challenge-rank: 课题模式等级

  respack: 资源包
  respack-default: '[默认]'
  respack-refresh: 刷新
  respack-open: 打开文件夹

  note-scale: 音符缩放

  double-hint: 双押提示

  aggressive: 激进优化
  aggressive-tips: 剔除屏幕外按键，提升渲染速度，但可能会导致部分音符消失

  disable-particle: 禁用粒子
  disable-effect: 禁用特效

  volume-music: 音乐音量
  volume-sfx: 音效音量

  ending-length: 结算画面时长
  disable-loading: 禁用加载
  disable-loading-tips: 有点问题，不建议使用
  chart_debug: 调试模式
  chart_ratio: 谱面缩放
  buffer_size: 音频Buffer Size
  buffer_size-tips: 用处不大
  target_audio: 目标采样率
  combo: 自定义COMBO名称
  watermark: 水印
  flid_x: 镜像模式
  background: 只显示背景

  render-list: 判定线,分数,连击,等级,名字,进度条,时间显示,百分比显示,暂停

  presets: 预设配置
  preset-refresh: 刷新
  preset-create: 创建
  preset-create-title: 预设配置名
  preset-created: 预设配置已创建
  preset-delete: 删除
  preset-deleted: 预设配置已删除
  preset-replace: 替换
  preset-replaced: 预设配置已替换
  preset-cannot-use-default: 不能使用 'default' 作为配置名
  default-preset: 默认

</i18n>

<script setup lang="ts">
import { ref, h, computed } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import { VDivider, VForm } from 'vuetify/components';

import { RULES, isNumeric, toast, anyFilter, toastError } from '../common';
import type { RenderConfig } from '../model';

import TipSwitch from './TipSwitch.vue';
import TipTextField from './TipTextField.vue';

const props = defineProps<{ initAspectRatio?: number }>();

const RESOLUTIONS = ['1280x720', '1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080', '7680x4320'];
const ffmpegPresetPresetList = ['veryfast p1 speed', 'faster p2 speed', 'fast p3 balanced', 'medium p4 balanced', 'slow p5 balanced', 'slower p6 quality', 'veryslow p7 quality'];
const bitrateControlList = ['CRF', 'CBR'];
const targetAudioOptions = [96000, 44100, 48000, 192000, 384000, 768000, 1536000, 3072000, 6144000, 12288000];
const targetAudio = ref(targetAudioOptions[0]);

const DEFAULT_CONFIG: RenderConfig = {
  resolution: [1920, 1080],
  ffmpegPreset: 'medium p4 balanced',
  endingLength: -2.0,
  disableLoading: true,
  chartDebug: false,
  flidX: false,
  chartRatio: 1,
  bufferSize: 256,
  fps: 60,
  hardwareAccel: true,
  hevc: false,
  bitrateControl: 'CRF',
  bitrate: '28',
  targetAudio: 96000,
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
  showProgressText: false,
  showTimeText: false,

  //ui [prpr]
  uiLine: true,
  uiScore: true,
  uiCombo: true,
  uiLevel: true,
  uiName: true,
  uiPb: true,
  uiPause: true,
};

function parseResolution(resolution: string): [number, number] | null {
  let parts = resolution.split(/[xX]/g);
  if (parts.length !== 2) return null;
  let ws = parts[0].trim(),
    hs = parts[1].trim();
  if (!isNumeric(ws) || !isNumeric(hs)) return null;
  let w = parseInt(ws),
    h = parseInt(hs);
  if (w <= 0 || h <= 0) return null;
  return [w, h];
}
const resolutionRule = (value: string) => parseResolution(value) !== null || t('rules.resolution');
const sampleCountRule = (value: string) => (isNumeric(value) && Math.log2(Number(value)) % 1 === 0) || t('rules.sample-count');

const form = ref<VForm>();

const resolution = ref('1920x1080'),
  ffmpegPreset = ref('medium p4 balanced'),
  fps = ref('60'),
  hwAccel = ref(true),
  hevc = ref(false);

const fxaa = ref(false),
  sampleCount = ref('1'),
  bitrateControl = ref('CRF'),
  bitrate = ref('28');

const playerAvatar = ref<string>(),
  playerName = ref(''),
  playerRks = ref('15.0');

const combo = ref('AUTOPLAY');
const watermark = ref('');

async function chooseAvatar() {
  let file = await open({
    filters: [
      {
        name: t('image-filter'),
        extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'],
      },
      anyFilter(),
    ],
  });
  if (file) {
    playerAvatar.value = file as string;
  }
}

const challengeColor = ref(t('challenge-colors').split(',')[4]),
  challengeRank = ref('45');

interface Respack {
  name: string;
  path: string | null;
  index: number;
}
const DEFAULT_RESPACK: Respack = {
  name: t('respack-default'),
  path: null,
  index: 0,
};
async function getRespacks() {
  return [DEFAULT_RESPACK, ...((await invoke('get_respacks')) as { name: string; path: string }[])].map((obj, index) => ({
    name: obj.name,
    path: obj.path,
    index: index + 1,
  }));
}
const respacks = ref([DEFAULT_RESPACK]);
const respack = ref(DEFAULT_RESPACK);
async function updateRespacks() {
  respacks.value = await getRespacks();
  respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0];
}
updateRespacks();

const noteScale = ref(1);

const doubleHint = ref(true),
  aggressive = ref(false),
  disableParticle = ref(false),
  disableEffect = ref(false);

const volumeMusic = ref(1),
  volumeSfx = ref(1);

const endingLength = ref('-2.0');
const activeSection = ref('output');
const disableLoading = ref(false);
const flidX = ref(false);
const chartDebug = ref(false);
const chartRatio = ref(1.0);
const bufferSize = ref(256);
const showProgressText = ref(false);
const showTimeText = ref(false);
const background = ref(false);


//prpr [ui]
const renderList = ref(t('render-list').split(','))
const render = ref<string[]>([])
render.value.push(...renderList.value.slice(0, 8));

const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

async function buildConfig(): Promise<RenderConfig | null> {
  if (!(await form.value!.validate()).valid) {
    toast(t('has-error'), 'error');
    return null;
  }
  return {
    resolution: (() => {
      let parts = resolution.value.split('x');
      return [parseInt(parts[0]), parseInt(parts[1])];
    })(),
    ffmpegPreset: ffmpegPreset.value,
    endingLength: parseFloat(endingLength.value),
    disableLoading: disableLoading.value,
    chartDebug: chartDebug.value,
    flidX: flidX.value,
    chartRatio: chartRatio.value,
    bufferSize: bufferSize.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    hevc: hevc.value,
    bitrateControl: bitrateControl.value,
    bitrate: bitrate.value,
    targetAudio: targetAudio.value,
    background: background.value,

    aggressive: aggressive.value,
    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)],
    challengeRank: parseInt(challengeRank.value),
    disableEffect: disableEffect.value,
    doubleHint: doubleHint.value,
    fxaa: fxaa.value,
    noteScale: noteScale.value,
    particle: !disableParticle.value,
    playerAvatar: playerAvatar.value ? (playerAvatar.value.length ? playerAvatar.value : null) : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value),
    sampleCount: parseInt(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    combo: combo.value,
    watermark: watermark.value,

    //ui [prpr]
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

function onEnter() {
  if (preset.value.key !== 'default') return;
  resolution.value = RESOLUTIONS[1];
  if (props.initAspectRatio) {
    for (let res of RESOLUTIONS) {
      let [w, h] = parseResolution(res)!;
      if (Math.abs(w / h - props.initAspectRatio) < 0.01) {
        resolution.value = res;
        break;
      }
    }
  }
}

defineExpose({ buildConfig, onEnter });

function StickyLabel(props: { title: string }) {
  return h('div', { class: 'mb-4 bg-surface sticky-label', style: 'position: sticky; top: 0; z-index: 2' }, [h('h3', { class: 'pa-1' }, props.title)]);
}

function applyConfig(config: RenderConfig) {
  resolution.value = config.resolution.join('x');
  ffmpegPreset.value = config.ffmpegPreset;
  endingLength.value = String(config.endingLength);
  disableLoading.value = config.disableLoading;
  chartDebug.value = config.chartDebug;
  flidX.value = config.flidX;
  chartRatio.value = config.chartRatio;
  bufferSize.value = config.bufferSize;
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  hevc.value = config.hevc;
  bitrateControl.value = config.bitrateControl;
  bitrate.value = config.bitrate;
  targetAudio.value = config.targetAudio;
  background.value = config.background;

  aggressive.value = config.aggressive;
  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(config.challengeColor)];
  challengeRank.value = String(config.challengeRank);
  disableEffect.value = config.disableEffect;
  doubleHint.value = config.doubleHint;
  fxaa.value = config.fxaa;
  noteScale.value = config.noteScale;
  disableParticle.value = !config.particle;
  playerAvatar.value = config.playerAvatar || undefined;
  playerName.value = config.playerName;
  playerRks.value = String(config.playerRks);
  sampleCount.value = String(config.sampleCount);
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0];
  volumeMusic.value = config.volumeMusic;
  volumeSfx.value = config.volumeSfx;
  combo.value = config.combo;
  watermark.value = config.watermark;

  //ui [prpr]
  render.value = []
  if (config.uiLine) render.value.push(renderList.value[0]);
  if (config.uiScore) render.value.push(renderList.value[1]);
  if (config.uiCombo) render.value.push(renderList.value[2]);
  if (config.uiLevel) render.value.push(renderList.value[3]);
  if (config.uiName) render.value.push(renderList.value[4]);
  if (config.uiPb) render.value.push(renderList.value[5]);
  if (config.showProgressText) render.value.push(renderList.value[6]);
  if (config.showTimeText) render.value.push(renderList.value[7]);
  if (config.uiPause) render.value.push(renderList.value[8]);
}

interface Preset {
  name: string;
  key: string;
  config: RenderConfig;
}
const DEFAULT_PRESET: Preset = {
  name: t('default-preset'),
  key: 'default',
  config: DEFAULT_CONFIG,
};

async function getPresets() {
  let result = [DEFAULT_PRESET];
  let pairs = (await invoke('get_presets')) as Record<string, RenderConfig>;
  for (let key of Object.keys(pairs).sort()) {
    result.push({
      name: key,
      key,
      config: pairs[key],
    });
  }
  return result;
}
const presets = ref([DEFAULT_PRESET]);
const preset = ref(DEFAULT_PRESET);
async function updatePresets() {
  presets.value = await getPresets();
  preset.value = presets.value.find((x) => x.key === preset.value.key) || presets.value[0];
}
updatePresets();

async function openRespackFolder() {
  try {
    await invoke('open_respack_folder');
  } catch (e) {
    toastError(e);
  }
}

async function createPreset() {
  let config = await buildConfig();
  if (!config) return;
  let name = prompt(t('preset-create-title'));
  if (!name || !name.length) return;
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
  let config = await buildConfig();
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
  <v-form ref="form" class="settings-form">
    <v-btn-toggle
      v-model="activeSection"
      mandatory
      class="section-switcher mb-4"
      color="primary"
      density="compact"
    >
      <v-btn value="output" variant="text">
        <v-icon icon="mdi-export" size="24"/>
      </v-btn>
      <v-btn value="player" variant="text">
        <v-icon icon="mdi-account" size="24"/>
      </v-btn>
      <v-btn value="graphics" variant="text">
        <v-icon icon="mdi-palette" size="24"/>
      </v-btn>
      <v-btn value="audio" variant="text">
        <v-icon icon="mdi-speaker" size="24"/>
      </v-btn>
    </v-btn-toggle>

    <v-row class="mb-4" dense>
      <v-col cols="12" sm="8" md="6" lg="5">
        <v-combobox @update:model-value="(val) => applyConfig(val.config)" class="mx-2" :label="t('presets')" :items="presets" item-title="name" v-model="preset"></v-combobox>
      </v-col>
      <v-col cols="12" sm="4" md="6" lg="7" class="d-flex align-center gap">
        <v-btn
          variant="tonal"
          color="primary"
          size="small"
          v-t="'preset-refresh'"
          @click="updatePresets"
        />
        <v-btn
          variant="tonal"
          color="primary"
          size="small"
          v-t="'preset-create'"
          @click="createPreset"
        />
        <v-btn
          variant="tonal"
          color="error"
          size="small"
          v-t="'preset-delete'"
          :disabled="preset.key === 'default'"
          @click="deletePreset"
        />
        <v-btn
          variant="tonal"
          color="warning"
          size="small"
          v-t="'preset-replace'"
          :disabled="preset.key === 'default'"
          @click="replacePreset"
        />
      </v-col>
    </v-row>

    <v-expand-transition>
      <div v-if="activeSection === 'output'" class="section-content">
        <v-row dense class="mb-2">
          <v-col cols="12" md="6" lg="4">
            <v-combobox
              :label="t('resolution')"
              :items="RESOLUTIONS"
              :rules="[resolutionRule]"
              density="compact"
              variant="outlined"
              v-model="resolution"
            />
          </v-col>
          <v-col cols="12" md="6" lg="4">
            <v-combobox
              :label="t('ffmpeg-preset')"
              :items="ffmpegPresetPresetList"
              density="compact"
              variant="outlined"
              v-model="ffmpegPreset"
            />
          </v-col>
          <v-col cols="12" md="6" lg="4">
            <v-text-field
              :label="t('fps')"
              type="number"
              :rules="[RULES.positiveInt]"
              density="compact"
              variant="outlined"
              v-model="fps"
            />
          </v-col>
        </v-row>

        <v-row dense>
          <v-col cols="12" md="6" lg="4">
            <v-combobox
              :label="t('bitrateControl')"
              :items="['CRF', 'CBR']"
              density="compact"
              variant="outlined"
              v-model="bitrateControl"
            />
          </v-col>
          <v-col cols="12" md="6" lg="4">
            <v-text-field
              :label="t('bitrate')"
              type="number"
              :rules="[RULES.positiveInt]"
              density="compact"
              variant="outlined"
              v-model="bitrate"
            />
          </v-col>
          <v-col cols="12" md="6" lg="4">
            <v-text-field
              :label="t('sample-count')"
              type="number"
              :rules="[sampleCountRule]"
              density="compact"
              variant="outlined"
              v-model="sampleCount"
            />
          </v-col>
        </v-row>

        <v-row class="mt-2" dense>
          <v-col cols="6" sm="4" md="3">
            <TipSwitch
              :label="t('hw-accel')"
              :tooltip="t('hw-accel-tips')"
              color="primary"
              v-model="hwAccel"
            />
          </v-col>
          <v-col cols="6" sm="4" md="3">
            <TipSwitch
              :label="t('fxaa')"
              :tooltip="t('fxaa-tips')"
              color="primary"
              v-model="fxaa"
            />
          </v-col>
          <v-col cols="6" sm="4" md="3">
            <TipSwitch
              :label="t('hevc')"
              :tooltip="t('hevc-tips')"
              color="primary"
              v-model="hevc"
            />
          </v-col>
        </v-row>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'player'" class="section-content">
        <v-row align="center" class="mb-4">
          <v-col cols="auto">
            <v-avatar size="64" color="surface-variant">
              <v-img v-if="playerAvatar" :src="playerAvatar"/>
              <v-icon v-else icon="mdi-account" size="40"/>
            </v-avatar>
          </v-col>
          <v-col>
            <v-btn @click="chooseAvatar" variant="tonal" block>
              {{ t('player-avatar') }}
            </v-btn>
          </v-col>
        </v-row>

        <v-row dense>
          <v-col cols="12" md="6">
            <v-text-field
              :label="t('player-name')"
              density="compact"
              variant="outlined"
              v-model="playerName"
            />
          </v-col>
          <v-col cols="12" md="6">
            <v-text-field
              :label="t('player-rks')"
              :rules="[RULES.positive]"
              type="number"
              density="compact"
              variant="outlined"
              v-model="playerRks"
            />
          </v-col>
          <v-col cols="12">
            <v-combobox
              :label="t('challenge-color')"
              :items="t('challenge-colors').split(',')"
              density="compact"
              variant="outlined"
              v-model="challengeColor"
            />
          </v-col>
        </v-row>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'graphics'" class="section-content">
        <v-row align="center" class="mb-4">
          <v-col cols="8">
            <v-combobox
              :label="t('respack')"
              :items="respacks"
              item-title="name"
              density="compact"
              variant="outlined"
              v-model="respack"
            />
          </v-col>
          <v-col cols="4" class="d-flex justify-end gap">
            <v-btn icon="mdi-refresh" variant="text" @click="updateRespacks"/>
            <v-btn icon="mdi-folder-open" variant="text" @click="openRespackFolder"/>
          </v-col>
        </v-row>

        <v-slider
          :label="t('note-scale')"
          thumb-label="always"
          :min="0"
          :max="5"
          :step="0.05"
          color="primary"
          class="mb-4"
          v-model="noteScale"
        />

        <v-row dense>
          <v-col cols="6" md="3">
            <TipSwitch :label="t('double-hint')" v-model="doubleHint" color="primary"/>
          </v-col>
          <v-col cols="6" md="3">
            <TipSwitch
              :label="t('aggressive')"
              :tooltip="t('aggressive-tips')"
              v-model="aggressive"
              color="primary"
            />
          </v-col>
          <v-col cols="6" md="3">
            <TipSwitch :label="t('disable-particle')" v-model="disableParticle" color="primary"/>
          </v-col>
          <v-col cols="6" md="3">
            <TipSwitch :label="t('disable-effect')" v-model="disableEffect" color="primary"/>
          </v-col>
          <v-col cols="6" md="6">
            <v-select v-model="render" :items="renderList" :label="t('render')" multiple></v-select>
          </v-col>
        </v-row>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'audio'" class="section-content">
        <div class="volume-controls mb-4">
          <v-row align="center">
            <v-col cols="12">
              <div class="d-flex align-center">
                <v-icon icon="mdi-music" class="mr-2"/>
                <v-slider
                  v-model="volumeMusic"
                  :min="0"
                  :max="2"
                  :step="0.01"
                  hide-details
                  color="primary"
                />
                <span class="text-caption ml-2">{{ (volumeMusic * 100).toFixed(0) }}%</span>
              </div>
            </v-col>

            <v-col cols="12">
              <div class="d-flex align-center">
                <v-icon icon="mdi-volume-high" class="mr-2"/>
                <v-slider
                  v-model="volumeSfx"
                  :min="0"
                  :max="2"
                  :step="0.01"
                  hide-details
                  color="primary"
                />
                <span class="text-caption ml-2">{{ (volumeSfx * 100).toFixed(0) }}%</span>
              </div>
            </v-col>
          </v-row>
        </div>

        <v-row dense>
          <v-col cols="4" md="4">
            <v-text-field
              :label="t('ending-length')"
              type="number"
              density="compact"
              variant="outlined"
              v-model="endingLength"
            />
          </v-col>
          <v-col cols="4" md="4">
            <v-text-field
              :label="t('watermark')"
              density="compact"
              variant="outlined"
              v-model="watermark"
            />
          </v-col>
          <v-col cols="4" md="4">
            <v-text-field
              :label="t('combo')"
              density="compact"
              variant="outlined"
              v-model="combo"
            />
          </v-col>
        </v-row>

        <v-combobox
          :label="t('target_audio')"
          :items="targetAudioOptions"
          density="compact"
          variant="outlined"
          v-model="targetAudio"
          class="mb-4"
        />
        <v-slider
          :label="t('buffer_size')"
          thumb-label="always"
          :min="128"
          :max="2048"
          class="my-4"
          v-model="bufferSize"
        />

        <v-row dense>
          <v-col cols="4" md="4">
            <v-switch
              :label="t('disable-loading')"
              density="compact"
              color="primary"
              v-model="disableLoading"
            />
          </v-col>
          <v-col cols="4" md="4">
            <v-switch
              :label="t('background')"
              density="compact"
              color="primary"
              v-model="background"
            />
          </v-col>
          <v-col cols="4" md="4">
            <v-switch
              :label="t('chart_debug')"
              density="compact"
              color="primary"
              v-model="chartDebug"
            />
          </v-col>
          <v-col cols="4" md="4">
            <v-switch
              :label="t('flid_x')"
              density="compact"
              color="primary"
              v-model="flidX"
            />
          </v-col>
        </v-row>

        <v-slider
          :label="t('chart_ratio')"
          thumb-label="always"
          :min="0.05"
          :max="1"
          :step="0.01"
          class="mt-4"
          v-model="chartRatio"
        />
      </div>
    </v-expand-transition>
  </v-form>
</template>

<style scoped>
.gap {
  gap: 0.5rem;
}

.section-content {
  padding: 1rem;
  background: rgba(var(--v-theme-surface-variant), 0.1);
  border-radius: 8px;
  border: 1px solid rgba(var(--v-theme-primary), 0.1);
}

.volume-controls {
  background: rgba(var(--v-theme-surface-variant), 0.05);
  padding: 1rem;
  border-radius: 8px;
}
</style>