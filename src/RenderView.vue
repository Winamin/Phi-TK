<i18n>
  en:
    already-running: Phi TK is already running
  
    prev-step: Previous
    next-step: Next
    steps:
      choose: 'Choose the chart'
      config: 'Configure chart'
      options: 'Render options'
      render: 'Render'
  
    choose:
      archive: Archive
      folder: Folder
  
    chart-file: Chart file
    choose-drop: Drag and drop chart file here
  
    chart-name: Chart name
    charter: Charter
    illustrator: Illustrator
    level: Level
    aspect: Aspect ratio
    dim: Background dim
    hold_cover: Hold Head Partial Cover
  
    tip: Tip
    tip-placeholder: Leave empty to choose randomly
  
    width: Width
    height: Height
  
    file:
      title: File
      chart: Chart file (empty for default)
      music: Music (empty for default)
      illustration: Illustration (empty for default)
  
    preview: Preview
    render: Render
    play: Play
  
    render-started: Rendering has started!
    see-tasks: See tasks
    next-chart: Render Next Chart
  
    ffmpeg-not-found: You haven't installed ffmpeg yet. Please download FFmpeg.exe and put it in the specific folder.
  
  zh-CN:
    already-running: Phi TK 已经在运行
  
    prev-step: 上一步
    next-step: 下一步
    steps:
      choose: '选择谱面'
      config: '配置谱面'
      options: '渲染参数'
      render: '渲染视频'
  
    choose:
      archive: 压缩包
      folder: 文件夹
  
    chart-file: 谱面文件
    choose-drop: 拖拽谱面文件到此处
  
    chart-name: 谱面名
    charter: 谱师
    composer: 曲师
    illustrator: 画师
    level: 难度
    aspect: 宽高比
    dim: 背景昏暗程度
    hold_cover: Hold 头部遮罩
  
    tip: Tip
    tip-placeholder: 留空则随机选择
  
    width: 宽
    height: 高
  
    preview: 演示
    render: 渲染
    play: 游玩
  
    render-started: 视频开始渲染了！
    see-tasks: 查看任务列表
    next-chart: 渲染下一个谱面
  
    ffmpeg-not-found: 笨蛋怎么没安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。
  
  </i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { watch } from 'vue';

import { invoke } from '@tauri-apps/api/core';
import { event } from '@tauri-apps/api';

import { toastError, RULES, toast, anyFilter, isString } from './common';
import type { ChartInfo, FileDropEvent } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import moment from 'moment';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as shell from '@tauri-apps/plugin-shell';
import { listen } from '@tauri-apps/api/event';

if (!(await invoke('is_the_only_instance'))) {
  await dialog.message(t('already-running'));
  await invoke('exit_program');
}

const router = useRouter();

const steps = ['choose', 'config', 'options', 'render'];
const stepIndex = ref(1),
  step = computed(() => steps[stepIndex.value - 1]);

const chartInfo = ref<ChartInfo>();

let chartPath = '';

const choosingChart = ref(false),
  parsingChart = ref(false);
async function chooseChart(folder?: boolean) {
  if (choosingChart.value) return;
  choosingChart.value = true;
  try {
    let file = folder
      ? await dialog.open({ directory: true })
      : await dialog.open({
          filters: [
            {
              name: t('choose.filter-name'),
              extensions: ['zip', 'pez'],
            },
            anyFilter(),
          ],
        });
    if (!file) return;
    await loadChart(file as string);
  } finally {
    choosingChart.value = false;
  }
}

async function loadChart(file: string) {
  try {
    parsingChart.value = true;
    chartPath = file;
    chartInfo.value = (await invoke('parse_chart', { path: file })) as ChartInfo;
    stepIndex.value++;
    aspectWidth.value = String(chartInfo.value.aspectRatio);
    aspectHeight.value = '1.0';
    for (let asp of [
      [16, 9],
      [4, 3],
      [8, 5],
      [3, 2],
    ]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.value.aspectRatio) < 1e-4) {
        aspectWidth.value = String(asp[0]);
        aspectHeight.value = String(asp[1]);
        break;
      }
    }
  } catch (e) {
    toastError(e);
  } finally {
    parsingChart.value = false;
  }
}

const aspectWidth = ref('0'),
  aspectHeight = ref('0');

const form = ref<VForm>();

const configView = ref<typeof ConfigView>();
async function buildParams() {
  let config = await configView.value!.buildConfig();
  if (!config) return null;

  // 同步宽高比到 chartInfo
  const aspect = tryParseAspect();
  if (aspect !== undefined) {
    chartInfo.value!.aspectRatio = aspect;
  }

  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  };
}

watch([aspectWidth, aspectHeight], ([newWidth, newHeight]) => {
  try {
    const width = parseFloat(newWidth);
    const height = parseFloat(newHeight);
    if (!isNaN(width) && !isNaN(height) && chartInfo.value) {
      chartInfo.value.aspectRatio = width / height;
    }
  } catch (e) {
    console.error('Failed to update aspect ratio:', e);
  }
});

async function postRender() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      await dialog.message(t('ffmpeg-not-found'));
      await invoke('open_app_folder');
      await shell.open('https://mivik.moe/ffmpeg-windows/');
      return false;
    }
    let params = await buildParams();
    if (!params) return false;
    const outputPath = localStorage.getItem('outputPath');
    await invoke('post_render', { params, outputPath: outputPath || null });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function previewChart() {
  try {
    let params = await buildParams();
    if (!params) return false;
    params.config.autoplay = true;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function playChart() {
  try {
    let params = await buildParams();
    if (!params) return false;
    params.config.autoplay = false;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

const renderMsg = ref(''),
  renderProgress = ref<number>(),
  renderDuration = ref<number>();
event.listen('render-msg', (msg) => (renderMsg.value = msg.payload as string));
event.listen('render-progress', (msg) => {
  let payload = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = t('render-status', {
    progress: (payload.progress * 100).toFixed(2),
    fps: payload.fps,
    estimate: moment.duration(payload.estimate, 'seconds').humanize(true, { ss: 1 }),
  });
  renderProgress.value = payload.progress * 100;
  console.log(renderProgress.value);
});
event.listen('render-done', (msg) => {
  stepIndex.value++;
  renderDuration.value = Math.round(msg.payload as number);
});

async function moveNext() {
  if (step.value === 'config') {
    if ((await form.value!.validate()).valid) {
      stepIndex.value++;
      configView.value!.onEnter();
    } else {
      toast(t('has-error'), 'error');
    }
    return;
  }
  if (step.value === 'options') {
    if (await postRender()) {
      stepIndex.value++;
    }
    return;
  }
}

let chartInQuery = router.currentRoute.value.query.chart;
if (isString(chartInQuery)) {
  onMounted(() => loadChart(chartInQuery as string));
}

function tryParseAspect(): number | undefined {
  try {
    let width = parseFloat(aspectWidth.value);
    let height = parseFloat(aspectHeight.value);
    if (isNaN(width) || isNaN(height)) return undefined;
    return width / height;
  } catch (e) {
    return undefined;
  }
}

const fileHovering = ref(false);

listen('tauri://drag-over', () => (fileHovering.value = step.value === 'choose'));
listen('tauri://drag-leave', () => (fileHovering.value = false));

listen('tauri://drag-drop', async (event) => {
  const files = (event.payload as FileDropEvent).paths;

  if (step.value === 'choose') {
    fileHovering.value = false;
    await loadChart(files[0]);
  } else if (step.value === 'config' || step.value === 'options' || step.value === 'render') {
    fileHovering.value = false;
    stepIndex.value = 1;
    await loadChart(files[0]);
  }
});

const hoverContainer = ref<HTMLElement>();
const moveOffset = ref({ x: 0, y: 0 });

function resetHover() {
  moveOffset.value = { x: 0, y: 0 };
}

function onHoverMove(e: MouseEvent) {
  if (!hoverContainer.value) return;

  const rect = hoverContainer.value.getBoundingClientRect();
  const centerX = rect.width / 2;
  const centerY = rect.height / 2;

  const offsetX = (e.clientX - rect.left - centerX) * 0.01;
  const offsetY = (e.clientY - rect.top - centerY) * 0.01;

  moveOffset.value = {
    x: offsetX * (window.innerWidth / 1280),
    y: offsetY * (window.innerHeight / 720),
  };
}
const archiveStyle = computed(() => ({
  transform: `translate(
      ${moveOffset.value.x * 1.2}px,
      ${moveOffset.value.y * 1.2}px
    ) rotate3d(
      ${moveOffset.value.y / 20},
      ${-moveOffset.value.x / 20},
      0,
      ${Math.sqrt(moveOffset.value.x ** 2 + moveOffset.value.y ** 2) / 4}deg
    )`,
  filter: `drop-shadow(${moveOffset.value.x / 4}px ${moveOffset.value.y / 4}px 6px rgba(99, 102, 241, 0.2))`,
}));

const folderStyle = computed(() => ({
  transform: `translate(
      ${-moveOffset.value.x * 0.8}px,
      ${-moveOffset.value.y * 0.8}px
    ) rotate3d(
      ${-moveOffset.value.y / 20},
      ${moveOffset.value.x / 20},
      0,
      ${Math.sqrt(moveOffset.value.x ** 2 + moveOffset.value.y ** 2) / 6}deg
    )`,
  filter: `drop-shadow(${-moveOffset.value.x / 4}px ${-moveOffset.value.y / 4}px 6px rgba(99, 102, 241, 0.2))`,
}));
</script>

<template>
  <div class="pa-8 w-100 h-100" style="max-width: 960px">
    <v-stepper 
      alt-labels 
      v-model="stepIndex" 
      hide-actions 
      :items="steps.map((x) => t('steps.' + x))" 
      class="glass-stepper"
    >
      <div v-if="step === 'config' || step === 'options'" class="d-flex flex-row pa-6 pb-4 pt-0">
        <v-btn variant="text" @click="stepIndex && stepIndex--" v-t="'prev-step'"></v-btn>
        <div class="flex-grow-1"></div>
        <v-btn v-if="step === 'options'" variant="tonal" @click="playChart" class="mr-2 glass-btn" v-t="'play'"></v-btn>
        <v-btn v-if="step === 'options'" variant="tonal" @click="previewChart" class="mr-2 glass-btn" v-t="'preview'"></v-btn>
        <v-btn variant="tonal" @click="moveNext" class="gradient-primary glass-btn">
          {{ step === 'options' ? t('render') : t('next-step') }}
        </v-btn>
      </div>

      <template v-slot:[`item.1`]>
        <div class="mt-1 d-flex" style="gap: 1rem" @mousemove="onHoverMove" @mouseleave="resetHover" ref="hoverContainer">
          <div class="flex-grow-1 d-flex align-center justify-center w-0 py-8">
            <v-btn
              :style="archiveStyle"
              class="w-75 gradient-primary hover-movable"
              style="overflow: hidden"
              size="large"
              @click="chooseChart(false)"
              prepend-icon="mdi-folder-zip"
              >{{ t('choose.archive') }}</v-btn
            >
          </div>
          <div class="flex-grow-1 d-flex align-center justify-center w-0">
            <v-btn :style="folderStyle" class="w-75 gradient-primary hover-movable" size="large" @click="chooseChart(true)" prepend-icon="mdi-folder">{{
              t('choose.folder')
            }}</v-btn>
          </div>
        </div>
        <v-overlay v-model="parsingChart" contained class="align-center justify-center custom-loading-overlay" persistent :close-on-content-click="false">
          <div class="loading-content text-center p-8">
            <v-progress-circular
              indeterminate
              color="primary"
              size="80"
              width="6"
              class="mb-4 animate-pulse"
            ></v-progress-circular>
          </div>
        </v-overlay>
      </template>

      <template v-slot:[`item.2`]>
        <v-form ref="form" v-if="chartInfo" class="glass-form">
          <v-row no-gutters class="mx-n2">
            <v-col cols="8">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('chart-name')" 
                :rules="[RULES.non_empty]" 
                v-model="chartInfo.name"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
            <v-col cols="4">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('level')" 
                :rules="[RULES.non_empty]" 
                v-model="chartInfo.level"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12" sm="4">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('charter')" 
                :rules="[RULES.non_empty]" 
                v-model="chartInfo.charter"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('composer')" 
                v-model="chartInfo.composer"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('illustrator')" 
                v-model="chartInfo.illustrator"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1 align-center">
            <v-col cols="4">
              <div class="mx-2 d-flex flex-column">
                <p class="text-caption text-white" v-t="'aspect'"></p>
                <div class="d-flex flex-row align-center justify-center">
                  <v-text-field 
                    type="number" 
                    class="mr-2 glass-input" 
                    :rules="[RULES.positive]" 
                    :label="t('width')" 
                    v-model="aspectWidth"
                    variant="solo-filled"
                    density="compact"
                  ></v-text-field>
                  <p class="text-white">:</p>
                  <v-text-field 
                    type="number" 
                    class="ml-2 glass-input" 
                    :rules="[RULES.positive]" 
                    :label="t('height')" 
                    v-model="aspectHeight"
                    variant="solo-filled"
                    density="compact"
                  ></v-text-field>
                </div>
              </div>
            </v-col>
            <v-col cols="8" class="px-6">
              <v-slider 
                :label="t('dim')" 
                thumb-label="always" 
                :min="0" 
                :max="1" 
                :step="0.01" 
                v-model="chartInfo.backgroundDim"
                color="#b19dff"
                thumb-color="#8a6cff"
              ></v-slider>
              <v-row no-gutters class="mx-n2 mt-1 align-center">
                <v-col cols="12" class="px-6">
                  <v-switch 
                    :label="t('hold_cover')" 
                    v-model="chartInfo.HoldPartialCover" 
                    :true-value="1" 
                    :false-value="0" 
                    color="#8a6cff" 
                    persistent-hint
                  ></v-switch>
                </v-col>
              </v-row>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12">
              <v-text-field 
                class="mx-2 glass-input" 
                :label="t('tip')" 
                :placeholder="t('tip-placeholder')" 
                v-model="chartInfo.tip"
                variant="solo-filled"
                density="compact"
              ></v-text-field>
            </v-col>
          </v-row>
        </v-form>
      </template>

      <template v-slot:[`item.3`]>
        <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()" class="glass-form"></ConfigView>
      </template>

      <template v-slot:[`item.4`]>
        <div class="d-flex flex-column justify-center align-center mb-2" style="gap: 1rem">
          <v-card class="glass-card" style="width: 60%">
            <v-card-title class="text-center text-gradient">
              {{ t('render-started') }}
            </v-card-title>
            <v-divider class="divider"></v-divider>
            <v-card-text>
              <v-row>
                <v-col cols="6">
                  <div class="info-item">
                    <span class="info-label">{{ t('chart-name') }}:</span>
                    <span class="info-value">{{ chartInfo?.name }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('level') }}:</span>
                    <span class="info-value">{{ chartInfo?.level }}</span>
                  </div>
                </v-col>
                <v-col cols="6">
                  <div class="info-item">
                    <span class="info-label">{{ t('charter') }}:</span>
                    <span class="info-value">{{ chartInfo?.charter }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('composer') }}:</span>
                    <span class="info-value">{{ chartInfo?.composer }}</span>
                  </div>
                </v-col>
              </v-row>
              <div class="info-item">
                <span class="info-label">{{ t('illustrator') }}:</span>
                <span class="info-value">{{ chartInfo?.illustrator }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">{{ t('aspect') }}:</span>
                <span class="info-value">{{ aspectWidth }}:{{ aspectHeight }}</span>
              </div>
            </v-card-text>
          </v-card>

          <div class="d-flex justify-center mt-4" style="gap: 16px">
            <v-btn @click="router.push({ name: 'tasks' })" class="glass-btn" color="#b19dff" style="scale: 1">
              {{ t('see-tasks') }}
            </v-btn>
            <v-btn @click="stepIndex = 1" class="glass-btn gradient-primary" style="scale: 1">
              {{ t('next-chart') }}
            </v-btn>
          </div>
        </div>
      </template>
    </v-stepper>
    <v-overlay v-model="fileHovering" contained class="align-center justify-center drop-zone-overlay" persistent :close-on-content-click="false">
      <div class="drop-pulse">
        {{ t('choose-drop') }}
      </div>
    </v-overlay>
  </div>
</template>

<style scoped>
.gradient-primary {
  background: linear-gradient(45deg, #6a5acd, #8a6cff, #5d4b9a) !important;
  transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  position: relative;
  overflow: hidden;
  color: white;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(106, 90, 205, 0.3);
}

.gradient-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(106, 90, 205, 0.4);
}

.glass-stepper {
  border-radius: 16px !important;
  background: rgba(50, 42, 90, 0.6) !important;
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
}

.glass-form {
  padding: 24px;
  background: rgba(60, 52, 100, 0.4);
  border-radius: 16px;
  margin: 16px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.glass-input {
  background: rgba(70, 60, 120, 0.3) !important;
  border-radius: 10px !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.glass-input :deep(.v-field) {
  background: transparent !important;
}

.glass-card {
  background: rgba(60, 52, 100, 0.5) !important;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
}

.divider {
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.info-item {
  display: flex;
  margin-bottom: 12px;
  align-items: center;
}

.info-label {
  font-weight: 500;
  margin-right: 8px;
  color: rgba(255, 255, 255, 0.7);
  min-width: 100px;
}

.info-value {
  color: white;
  flex-grow: 1;
}

.custom-loading-overlay {
  backdrop-filter: blur(16px);
  background-color: rgba(17, 32, 89, 0.7);
  border-radius: 1.5rem;
}

.drop-zone-overlay {
  backdrop-filter: blur(10px);
  background: rgba(70, 57, 141, 0.4);
}

.drop-pulse {
  padding: 40px 60px;
  background: rgba(80, 70, 150, 0.6);
  border: 2px dashed rgba(255, 255, 255, 0.3);
  border-radius: 16px;
  color: white;
  font-size: 1.5rem;
  font-weight: 500;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(138, 108, 255, 0.4);
  }
  70% {
    box-shadow: 0 0 0 20px rgba(138, 108, 255, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(138, 108, 255, 0);
  }
}

.glass-btn {
  background: rgba(70, 60, 120, 0.4) !important;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.3s ease;
  color: white;
}

.glass-btn:hover {
  background: rgba(90, 80, 150, 0.5) !important;
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(106, 90, 205, 0.3);
}
</style>