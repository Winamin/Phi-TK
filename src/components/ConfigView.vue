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
  combo: Customize COMBO Text
  flid_x: Mirror Mode
  show_progress_text: Progress bar 1
  show_progress_text-tips: Percent scale display
  show_time_text: Progress bar 2
  show_time_text-tips: Time display

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
  combo: 自定义COMBO名称
  flid_x: 镜像模式
  show_progress_text: 进度条1
  show_progress_text-tips: 百分之比例显示
  show_time_text: 进度条2
  show_time_text-tips: 时间显示

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
import { ref, h } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/dialog';

import { VDivider, VForm } from 'vuetify/components';

import { RULES, isNumeric, toast, anyFilter, toastError } from '../common';
import type { RenderConfig } from '../model';

import TipSwitch from './TipSwitch.vue';
import TipTextField from './TipTextField.vue';

const props = defineProps<{ initAspectRatio?: number }>();

const RESOLUTIONS = [ '1280x720','1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080', '7680x4320'];
const ffmpegPresetPresetList = ['veryfast p1 speed', 'faster p2 speed','fast p3 balanced', 'medium p4 balanced', 'slow p5 balanced', 'slower p6 quality', 'veryslow p7 quality'];
const bitrateControlList = ['CRF','CBR'];

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

const combo = ref('AUTOPLAY')

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
const disableLoading = ref(false)
const flidX = ref(false)
const chartDebug = ref(false)
const chartRatio = ref(1.0)
const bufferSize = ref(256)
const showProgressText = ref(false)
const showTimeText = ref(false)

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
    showProgressText: showProgressText.value,
    showTimeText: showTimeText.value,
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
  showProgressText.value = config.showProgressText;
  showTimeText.value = config.showTimeText;
}

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
  showProgressText: false,
  showTimeText: false,
};
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
  <v-form ref="form" class="settings-form">_
    <v-btn-toggle
      v-model="activeSection"
      mandatory
      class="section-switcher"
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

    <v-row no-gutters class="mx-n2 align-center">
      <v-col cols="8">
        <v-combobox @update:model-value="(val) => applyConfig(val.config)" class="mx-2" :label="t('presets')" :items="presets" item-title="name" v-model="preset"></v-combobox>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2 pink lighten-1 rounded-pill" v-t="'preset-refresh'" @click="updatePresets"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2 pink lighten-1 rounded-pill" v-t="'preset-create'" @click="createPreset"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2 pink lighten-1 rounded-pill" v-t="'preset-delete'" :disabled="preset.key === 'default'" @click="deletePreset"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2 pink lighten-1 rounded-pill" v-t="'preset-replace'" :disabled="preset.key === 'default'" @click="replacePreset"></v-btn>
      </v-col>
    </v-row>

    <v-expand-transition>
      <div v-if="activeSection === 'output'" class="section-content">
        <div class="grid-3col">
          <v-combobox :label="t('resolution')" :items="RESOLUTIONS" :rules="[resolutionRule]" density="compact" variant="outlined" v-model="resolution"/>
          <v-combobox :label="t('ffmpeg-preset')" :items="ffmpegPresetPresetList" density="compact" variant="outlined" v-model="ffmpegPreset"/>
          <v-text-field :label="t('fps')" type="number" :rules="[RULES.positiveInt]" density="compact" variant="outlined" v-model="fps"/>
          <v-combobox :label="t('bitrateControl')" :items="['CRF', 'CBR']" density="compact" variant="outlined" v-model="bitrateControl"/>
          <v-text-field :label="t('bitrate')" type="number" :rules="[RULES.positiveInt]" density="compact" variant="outlined" v-model="bitrate"/>
          <v-text-field :label="t('sample-count')" type="number" :rules="[sampleCountRule]" density="compact" variant="outlined" v-model="sampleCount"/>
        </div>
        
        <div class="switch-group">
          <TipSwitch :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" color="primary" v-model="hwAccel"/>
          <TipSwitch :label="t('fxaa')" :tooltip="t('fxaa-tips')" color="primary" v-model="fxaa"/>
          <TipSwitch :label="t('hevc')" :tooltip="t('hevc-tips')" color="primary" v-model="hevc"/>
        </div>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'player'" class="section-content">
        <div class="avatar-section">
          <v-avatar size="64" class="avatar-preview">
            <v-img v-if="playerAvatar" :src="playerAvatar"/>
            <v-icon v-else icon="mdi-account" size="40"/>
          </v-avatar>
          <v-btn @click="chooseAvatar" variant="tonal" class="ml-4">
            {{ t('player-avatar') }}
          </v-btn>
        </div>

        <div class="grid-2col">
          <v-text-field :label="t('player-name')" variant="outlined" density="compact" v-model="playerName"/>
          <v-text-field :label="t('player-rks')" :rules="[RULES.positive]" type="number" variant="outlined" density="compact" v-model="playerRks"/>
        </div>
        
        <v-combobox
            :label="t('challenge-color')"
            :items="t('challenge-colors').split(',')"
            variant="outlined"
            density="compact"
            v-model="challengeColor"
          />
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'graphics'" class="section-content">
        <div class="respack-controls">
          <v-combobox :label="t('respack')" :items="respacks" item-title="name" variant="outlined" density="compact" v-model="respack"/>
          <v-btn @click="updateRespacks" icon="mdi-refresh" variant="text"/>
          <v-btn @click="openRespackFolder" icon="mdi-folder-open" variant="text"/>
        </div>

        <v-slider :label="t('note-scale')" thumb-label="always" :min="0" :max="5" :step="0.05" color="primary" track-color="secondary" v-model="noteScale"/>

        <div class="switch-grid">
          <TipSwitch :label="t('double-hint')" v-model="doubleHint" color="primary"/>
          <TipSwitch :label="t('aggressive')" :tooltip="t('aggressive-tips')" v-model="aggressive" color="primary"/>
          <TipSwitch :label="t('disable-particle')" v-model="disableParticle" color="primary"/>
          <TipSwitch :label="t('disable-effect')" v-model="disableEffect" color="primary"/>
        </div>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="activeSection === 'audio'" class="section-content">
        <div class="volume-controls">
          <div class="volume-item">
            <v-icon icon="mdi-music" size="20"/>
            <v-slider :model-value="volumeMusic" @update:model-value="v => volumeMusic = v" :min="0" :max="2" :step="0.1" color="primary" hide-details/>
            <span class="volume-value">{{ (volumeMusic * 100).toFixed(0) }}%</span>
          </div>
          
          <div class="volume-item">
            <v-icon icon="mdi-volume-high" size="20"/>
            <v-slider :model-value="volumeSfx" @update:model-value="v => volumeSfx = v" :min="0" :max="2" :step="0.1" color="primary" hide-details/>
            <span class="volume-value">{{ (volumeSfx * 100).toFixed(0) }}%</span>
          </div>
        </div>

        <div class="advanced-controls">
          <v-text-field :label="t('ending-length')" variant="outlined" density="compact" type="number" v-model="endingLength"/>
          
          <div class="mini-switches">
            <v-tooltip :text="t('disable-loading-tips')">
              <template v-slot:activator="{ props }">
                <v-switch :label="t('disable-loading')" v-model="disableLoading" v-bind="props" color="primary" density="compact"/>
              </template>
            </v-tooltip>
            <v-tooltip :text="t('show_progress_text-tips')">
              <template v-slot:activator="{ props }">
                <v-switch :label="t('show_progress_text')" v-model="showProgressText" v-bind="props" color="primary" density="compact"/>
              </template>
            </v-tooltip>
            <v-tooltip :text="t('show_time_text-tips')">
              <template v-slot:activator="{ props }">
                <v-switch :label="t('show_time_text')" v-model="showTimeText" v-bind="props" color="primary" density="compact"/>
              </template>
            </v-tooltip>
          </div>
          <v-text-field :label="t('combo')" variant="outlined" density="compact" v-model="combo"/>
          <v-slider :label="t('buffer_size')" :tooltip="t('buffer_size-tips')" thumb-label="always" :min="128" :max="2048" :step="1" v-model="bufferSize"> </v-slider>
          <v-switch :label="t('chart_debug')" v-model="chartDebug" color="primary" density="compact"/>
          <v-switch :label="t('flid_x')" v-model="flidX" color="primary" density="compact"/>
          <v-slider :label="t('chart_ratio')" thumb-label="always" :min="0.05" :max="1" :step="0.01" color="primary" track-color="secondary" v-model="chartRatio"/>
        </div>
      </div>
    </v-expand-transition>
  </v-form>
</template>

<style scoped>
.settings-form {
  background: linear-gradient(135deg, rgba(16, 16, 32, 0.95), rgba(24, 24, 48, 0.95));
  backdrop-filter: blur(12px);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.section-switcher {
  margin-bottom: 24px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 4px;
}

.section-content {
  transition: all 0.3s ease;
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
}

.grid-3col {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.switch-group {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
  padding: 12px 0;
}

.avatar-section {
  display: flex;
  align-items: center;
  margin-bottom: 24px;
}

.avatar-preview {
  border: 2px solid rgba(255, 255, 255, 0.1);
  transition: transform 0.3s ease;
}

.avatar-preview:hover {
  transform: scale(1.05);
}

.respack-controls {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 20px;
}

.switch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.volume-controls {
  display: grid;
  gap: 24px;
  margin-bottom: 28px;
}

.volume-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.volume-value {
  min-width: 48px;
  text-align: right;
  color: #c7c0ff;
}

.advanced-controls {
  display: grid;
  gap: 16px;
}

.mini-switches {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.v-btn--active {
  background: rgba(99, 102, 241, 0.15) !important;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.3);
}

.v-switch {
  --v-switch-track-color: rgba(255, 255, 255, 0.1);
}

.v-slider-thumb {
  background: #6366f1 !important;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
}
</style>
