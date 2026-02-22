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
  filter: `drop-shadow(${moveOffset.value.x / 4}px ${moveOffset.value.y / 4}px 6px rgba(0, 0, 0, 0.3))`,
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
  filter: `drop-shadow(${-moveOffset.value.x / 4}px ${-moveOffset.value.y / 4}px 6px rgba(0, 0, 0, 0.3))`,
}));
</script>

<template>
  <div class="hmcl-container">
    <!-- 主内容区域 -->
    <div class="hmcl-main">
      <!-- 内容区域 -->
      <div class="hmcl-content">
        <!-- 步骤1: 选择谱面 -->
        <div class="step-content" :class="{ 'active': step === 'choose' }">
          <div class="choose-content">
            <div class="choose-cards">
              <div
                class="choose-card"
                @click="chooseChart(false)"
                @mousemove="onHoverMove"
                @mouseleave="resetHover"
                ref="hoverContainer"
              >
                <div class="card-icon" :style="archiveStyle">
                  <v-icon size="48">mdi-folder-zip</v-icon>
                </div>
                <h4>{{ t('choose.archive') }}</h4>
                <p>选择 .zip 或 .pez 格式的谱面压缩包</p>
              </div>
              <div
                class="choose-card"
                @click="chooseChart(true)"
              >
                <div class="card-icon" :style="folderStyle">
                  <v-icon size="48">mdi-folder</v-icon>
                </div>
                <h4>{{ t('choose.folder') }}</h4>
                <p>选择包含谱面文件的文件夹</p>
              </div>
            </div>
            <v-overlay v-model="parsingChart" contained class="align-center justify-center hmcl-loading" persistent :close-on-content-click="false">
              <v-card class="loading-card">
                <v-card-text class="text-center pa-6">
                  <v-progress-circular
                    indeterminate
                    color="primary"
                    size="40"
                    width="4"
                    class="mb-4"
                  ></v-progress-circular>
                  <p>正在解析谱面文件...</p>
                </v-card-text>
              </v-card>
            </v-overlay>
          </div>
        </div>

        <!-- 步骤2: 配置谱面 -->
        <div class="step-content" :class="{ 'active': step === 'config' }">
          <div class="config-panel" v-if="chartInfo">
            <v-form ref="form" class="config-form-neo">
              <!-- 标题区 -->
              <div class="config-header">
                <h2 class="config-title">Configure Chart</h2>
                <p class="config-subtitle">设置谱面基本信息与显示参数</p>
              </div>

              <!-- 内容网格布局 -->
              <div class="config-grid">
                <!-- 左栏 - 基本信息 -->
                <div class="config-section">
                  <div class="section-header">
                    <span class="section-number">01</span>
                    <h3 class="section-title">基本信息</h3>
                  </div>
                  <div class="section-body">
                    <div class="neo-field">
                      <label class="neo-label required">{{ t('chart-name') }}</label>
                      <input
                        v-model="chartInfo.name"
                        class="neo-input"
                        :class="{ 'has-value': chartInfo.name }"
                        placeholder=" "
                      />
                    </div>
                    <div class="neo-field">
                      <label class="neo-label required">{{ t('level') }}</label>
                      <input
                        v-model="chartInfo.level"
                        class="neo-input"
                        :class="{ 'has-value': chartInfo.level }"
                        placeholder=" "
                      />
                    </div>
                    <div class="neo-field">
                      <label class="neo-label required">{{ t('charter') }}</label>
                      <input
                        v-model="chartInfo.charter"
                        class="neo-input"
                        :class="{ 'has-value': chartInfo.charter }"
                        placeholder=" "
                      />
                    </div>
                    <div class="neo-field">
                      <label class="neo-label">{{ t('composer') }}</label>
                      <input
                        v-model="chartInfo.composer"
                        class="neo-input"
                        :class="{ 'has-value': chartInfo.composer }"
                        placeholder=" "
                      />
                    </div>
                    <div class="neo-field">
                      <label class="neo-label">{{ t('illustrator') }}</label>
                      <input
                        v-model="chartInfo.illustrator"
                        class="neo-input"
                        :class="{ 'has-value': chartInfo.illustrator }"
                        placeholder=" "
                      />
                    </div>
                  </div>
                </div>

                <!-- 右栏 - 显示设置 -->
                <div class="config-section">
                  <div class="section-header">
                    <span class="section-number">02</span>
                    <h3 class="section-title">显示设置</h3>
                  </div>
                  <div class="section-body">
                    <!-- 宽高比 -->
                    <div class="neo-field">
                      <label class="neo-label">{{ t('aspect') }}</label>
                      <div class="aspect-row">
                        <input
                          type="number"
                          v-model="aspectWidth"
                          class="neo-input aspect-input"
                        />
                        <span class="aspect-colon">:</span>
                        <input
                          type="number"
                          v-model="aspectHeight"
                          class="neo-input aspect-input"
                        />
                      </div>
                    </div>

                    <!-- 背景暗度 -->
                    <div class="neo-field">
                      <label class="neo-label">{{ t('dim') }}</label>
                      <div class="slider-container">
                        <input
                          type="range"
                          v-model="chartInfo.backgroundDim"
                          min="0"
                          max="1"
                          step="0.01"
                          class="neo-slider"
                        />
                        <span class="slider-value">{{ Math.round(chartInfo.backgroundDim * 100) }}%</span>
                      </div>
                    </div>

                    <!-- Hold遮罩开关 -->
                    <div class="neo-field toggle-field">
                      <label class="neo-label">{{ t('hold_cover') }}</label>
                      <button
                        type="button"
                        class="neo-toggle"
                        :class="{ 'active': chartInfo.HoldPartialCover }"
                        @click="chartInfo.HoldPartialCover = !chartInfo.HoldPartialCover"
                      >
                        <span class="toggle-indicator"></span>
                      </button>
                    </div>

                    <!-- Tip -->
                    <div class="neo-field">
                      <label class="neo-label">{{ t('tip') }}</label>
                      <input
                        v-model="chartInfo.tip"
                        class="neo-input"
                        :placeholder="t('tip-placeholder')"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </v-form>
          </div>
        </div>

        <!-- 步骤3: 渲染选项 -->
        <div class="step-content" :class="{ 'active': step === 'options' }">
          <div class="options-panel">
            <!-- 标题区 -->
            <div class="options-header">
              <h2 class="options-title">Render Options</h2>
              <p class="options-subtitle">配置视频输出参数与渲染设置</p>
            </div>
            <div class="options-content">
              <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()" class="hmcl-config"></ConfigView>
            </div>
          </div>
        </div>

        <!-- 步骤4: 渲染中 -->
        <div class="step-content" :class="{ 'active': step === 'render' }">
          <div class="render-content">
            <v-card class="hmcl-card">
              <v-card-title class="card-title">
                <v-icon class="mr-2">mdi-render</v-icon>
                {{ t('render-started') }}
              </v-card-title>
              <v-divider></v-divider>
              <v-card-text>
                <div class="info-list">
                  <div class="info-item">
                    <span class="info-label">{{ t('chart-name') }}</span>
                    <span class="info-value">{{ chartInfo?.name }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('level') }}</span>
                    <span class="info-value">{{ chartInfo?.level }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('charter') }}</span>
                    <span class="info-value">{{ chartInfo?.charter }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('composer') }}</span>
                    <span class="info-value">{{ chartInfo?.composer }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('illustrator') }}</span>
                    <span class="info-value">{{ chartInfo?.illustrator }}</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">{{ t('aspect') }}</span>
                    <span class="info-value">{{ aspectWidth }}:{{ aspectHeight }}</span>
                  </div>
                </div>

                <div v-if="renderProgress !== undefined" class="progress-area">
                  <v-progress-linear
                    :model-value="renderProgress"
                    color="primary"
                    height="8"
                    rounded
                  ></v-progress-linear>
                  <p class="progress-text">{{ renderMsg }}</p>
                </div>
              </v-card-text>
              <v-divider></v-divider>
              <v-card-actions class="card-actions">
                <v-btn @click="router.push({ name: 'tasks' })" variant="outlined">
                  <v-icon class="mr-1">mdi-view-list</v-icon>
                  {{ t('see-tasks') }}
                </v-btn>
                <v-btn @click="stepIndex = 1" variant="flat" color="primary">
                  <v-icon class="mr-1">mdi-plus</v-icon>
                  {{ t('next-chart') }}
                </v-btn>
              </v-card-actions>
            </v-card>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部操作栏 -->
    <div class="bottom-action-bar" v-if="step === 'config' || step === 'options' || step === 'render'">
      <div class="action-buttons-left">
        <v-btn
          v-if="step === 'config' || step === 'options'"
          variant="outlined"
          @click="stepIndex && stepIndex--"
          class="action-btn"
          size="small"
        >
          <v-icon class="mr-1">mdi-arrow-left</v-icon>
          {{ t('prev-step') }}
        </v-btn>
      </div>
      <div class="action-buttons-right">
        <v-btn
          v-if="step === 'options'"
          variant="outlined"
          @click="playChart"
          class="action-btn"
          size="small"
        >
          <v-icon class="mr-1">mdi-gamepad-variant</v-icon>
          {{ t('play') }}
        </v-btn>
        <v-btn
          v-if="step === 'options'"
          variant="outlined"
          @click="previewChart"
          class="action-btn"
          size="small"
        >
          <v-icon class="mr-1">mdi-eye</v-icon>
          {{ t('preview') }}
        </v-btn>
        <v-btn
          v-if="step === 'config' || step === 'options'"
          variant="flat"
          @click="moveNext"
          class="action-btn primary-btn"
          size="small"
        >
          {{ step === 'options' ? t('render') : t('next-step') }}
          <v-icon class="ml-1">mdi-arrow-right</v-icon>
        </v-btn>
      </div>
    </div>

    <!-- 文件拖放覆盖层 -->
    <v-overlay v-model="fileHovering" contained class="align-center justify-center hmcl-drop-overlay" persistent :close-on-content-click="false">
      <div class="drop-zone">
        <v-icon size="48" class="mb-2">mdi-download</v-icon>
        <p>{{ t('choose-drop') }}</p>
      </div>
    </v-overlay>
  </div>
</template>

<style scoped>
.hmcl-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: transparent;
  font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;
  color: #e0e0e0;
  position: relative;
  z-index: 3;
}

/* 主内容区 */
.hmcl-main {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  background-color: transparent;
  position: relative;
  z-index: 3;
}

/* 主内容区 */
/* 步骤内容 */
.hmcl-content {
  flex-grow: 1;
  padding: 24px;
  overflow-y: auto;
  background-color: transparent;
  position: relative;
  z-index: 3;
}

/* 步骤切换动画 */
.step-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 24px;
  opacity: 0;
  transform: translateX(20px);
  transition: opacity 0.3s ease, transform 0.3s ease;
  pointer-events: none;
  overflow-y: auto;
  z-index: 4;
  max-height: 100%;
  box-sizing: border-box;
}

.step-content.active {
  opacity: 1;
  transform: translateX(0);
  pointer-events: auto;
}

/* 选择内容 */
.choose-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  position: relative;
  z-index: 4;
}

.choose-cards {
  display: flex;
  gap: 24px;
  margin-top: 40px;
}

.choose-card {
  width: 220px;
  height: 260px;
  background: rgba(26, 26, 26, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  position: relative;
  z-index: 4;
}

.choose-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(34, 34, 34, 0.9);
  position: relative;
  z-index: 5;
}

.card-icon {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: rgba(42, 42, 42, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
  color: #cccccc;
  position: relative;
  z-index: 5;
}

.choose-card h4 {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 500;
  color: #cccccc;
}

.choose-card p {
  margin: 0;
  font-size: 14px;
  color: #9d9d9d;
  text-align: center;
}

/* 配置面板 - Neo-Brutalism 风格 */
.config-panel {
  width: 100%;
  height: 100%;
  max-height: calc(100vh - 180px);
  display: flex;
  flex-direction: column;
  padding: 0;
  overflow-y: auto;
}

.config-form-neo {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.config-header {
  margin-bottom: 24px;
  flex-shrink: 0;
}

.config-title {
  font-size: 28px;
  font-weight: 800;
  color: var(--text-primary, #f5f5f5);
  margin: 0 0 8px;
  letter-spacing: -0.5px;
}

.config-subtitle {
  font-size: 14px;
  color: var(--text-secondary, #a0a0a0);
  margin: 0;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
  flex: 1;
  min-height: 0;
}

.config-section {
  background: rgba(20, 20, 22, 0.9);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 2px solid rgba(255, 255, 255, 0.08);
}

.section-number {
  font-size: 14px;
  font-weight: 700;
  color: var(--brand-primary, #6366f1);
  background: rgba(99, 102, 241, 0.15);
  padding: 4px 10px;
  border-radius: 6px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #f5f5f5);
  margin: 0;
}

.section-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex: 1;
}

/* Neo 表单字段 */
.neo-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.neo-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary, #a0a0a0);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.neo-label.required::after {
  content: '*';
  color: #ef4444;
  margin-left: 4px;
}

.neo-input {
  width: 100%;
  padding: 12px 16px;
  background: rgba(40, 40, 44, 0.8);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: var(--text-primary, #f5f5f5);
  font-size: 14px;
  transition: all 0.2s ease;
}

.neo-input:focus {
  outline: none;
  border-color: var(--brand-primary, #6366f1);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
}

.neo-input::placeholder {
  color: var(--text-muted, #666);
}

/* 宽高比输入 */
.aspect-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.aspect-input {
  flex: 1;
  text-align: center;
}

.aspect-colon {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-secondary, #a0a0a0);
}

/* 滑块 */
.slider-container {
  display: flex;
  align-items: center;
  gap: 16px;
}

.neo-slider {
  flex: 1;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  appearance: none;
  cursor: pointer;
}

.neo-slider::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  background: var(--brand-primary, #6366f1);
  border: 2px solid white;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.5);
  transition: transform 0.15s ease;
}

.neo-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

.slider-value {
  min-width: 50px;
  text-align: right;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #f5f5f5);
}

/* 开关 */
.toggle-field {
  flex-direction: row !important;
  align-items: center;
  justify-content: space-between;
}

.neo-toggle {
  width: 52px;
  height: 28px;
  background: rgba(40, 40, 44, 0.8);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 14px;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
}

.neo-toggle.active {
  background: var(--brand-primary, #6366f1);
  border-color: var(--brand-primary, #6366f1);
}

.toggle-indicator {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 1px 1px 0 rgba(0, 0, 0, 0.3);
}

.neo-toggle.active .toggle-indicator {
  transform: translateX(24px);
}

/* 响应式 */
@media (max-width: 768px) {
  .config-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .config-section {
    padding: 16px;
  }

  .config-title {
    font-size: 22px;
  }

  .section-header {
    margin-bottom: 16px;
    padding-bottom: 12px;
  }
}

/* 配置内容 - 保留旧样式兼容 */
.config-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 4;
}

.config-form {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 4;
}

/* 配置布局 */
.config-layout {
  display: flex;
  gap: 24px;
  flex-grow: 1;
  position: relative;
  z-index: 4;
}

.config-column {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.field-group {
  background: rgba(26, 26, 26, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 16px;
  height: 100%;
}

.field-title {
  font-size: 16px;
  font-weight: 500;
  color: #cccccc;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.field-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.field-item {
  display: flex;
  flex-direction: column;
}

.field-label {
  font-size: 14px;
  font-weight: 500;
  color: #cccccc;
  margin-bottom: 6px;
}

.field-input {
  background-color: #2a2a2a !important;
}

.field-input :deep(.v-field) {
  border-radius: 4px;
  background-color: #2a2a2a !important;
  color: #cccccc !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.field-input :deep(.v-field-label) {
  color: #9d9d9d !important;
}

.field-input :deep(.v-input__control) {
  color: #cccccc !important;
}

/* 宽高比输入 */
.aspect-inputs {
  display: flex;
  align-items: center;
  gap: 8px;
}

.aspect-input {
  width: 80px;
}

.aspect-input :deep(.v-field) {
  border-radius: 4px;
  background-color: #2a2a2a !important;
  color: #cccccc !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.aspect-separator {
  font-size: 16px;
  font-weight: 500;
  color: #cccccc;
  margin: 0 4px;
}

/* 滑块和开关 */
.field-slider {
  margin: 8px 0;
}

.field-slider :deep(.v-slider-track__background) {
  background-color: #2a2a2a !important;
  opacity: 0.6; /* 可选：调整轨道背景透明度 */
}

.field-slider :deep(.v-slider-track__fill) {
  background-color: #505050 !important;
}

.field-slider :deep(.v-slider-thumb .v-slider-thumb__surface) {
  border-radius: 50% !important;
  width: 20px !important;
  height: 20px !important;
  background-color: #606060 !important;
  border: 2px solid white !important;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3) !important;
  transition: all 0.2s ease !important;
}

.field-slider :deep(.v-slider-thumb--focused .v-slider-thumb__surface),
.field-slider :deep(.v-slider-thumb:hover .v-slider-thumb__surface),
.field-slider :deep(.v-slider-thumb--active .v-slider-thumb__surface) {
  border-radius: 50% !important;
  background-color: #808080 !important;
  transform: scale(1.2) !important;
  box-shadow: 0 3px 6px rgba(0, 0, 0, 0.5) !important;
}

.field-switch :deep(.v-switch__track) {
  background-color: #2a2a2a !important;
  border-radius: 10px !important;
}

.field-switch :deep(.v-switch__thumb) {
  background-color: #666 !important;
  border-radius: 50% !important;
  transition: all 0.2s ease !important;
}

.field-switch :deep(.v-switch--selected .v-switch__track) {
  background-color: #505050 !important;
}

.field-switch :deep(.v-switch--selected .v-switch__thumb) {
  background-color: #fff !important;
}

/* 选项面板 - Neo-Brutalism 风格 */
.options-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 180px);
  overflow-y: auto;
}

.options-header {
  margin-bottom: 24px;
  flex-shrink: 0;
}

.options-title {
  font-size: 28px;
  font-weight: 800;
  color: var(--text-primary, #f5f5f5);
  margin: 0 0 8px;
  letter-spacing: -0.5px;
}

.options-subtitle {
  font-size: 14px;
  color: var(--text-secondary, #a0a0a0);
  margin: 0;
}

.options-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.hmcl-config {
  height: 100%;
}

/* 渲染内容 */
.render-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  position: relative;
  z-index: 4;
}

.hmcl-card {
  width: 100%;
  max-width: 600px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  background: rgba(26, 26, 26, 0.9);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  position: relative;
  z-index: 4;
}

.card-title {
  font-size: 18px;
  font-weight: 500;
  color: #cccccc;
}

.info-list {
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  width: 120px;
  font-weight: 500;
  color: #9d9d9d;
}

.info-value {
  flex-grow: 1;
  color: #cccccc;
}

.progress-area {
  margin-top: 16px;
  position: relative;
  z-index: 6;
}

.progress-text {
  text-align: center;
  margin-top: 8px;
  font-size: 14px;
  color: #9d9d9d;
}

.card-actions {
  justify-content: flex-end;
  gap: 8px;
  background: rgba(31, 31, 31, 0.85);
}

.card-actions .v-btn {
  background-color: #2a2a2a !important;
  color: #cccccc !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.card-actions .v-btn.v-btn--flat {
  background-color: #404040 !important;
  color: white !important;
  border-color: #404040 !important;
}

/* 加载和拖放覆盖层 */
.hmcl-loading {
  background-color: rgba(0, 0, 0, 0.8);
  z-index: 10;
}

.loading-card {
  border-radius: 8px;
  overflow: hidden;
  background-color: #1a1a1a;
}

.hmcl-drop-overlay {
  background-color: rgba(0, 0, 0, 0.8);
  z-index: 10;
}

.drop-zone {
  background-color: #1a1a1a;
  border-radius: 8px;
  padding: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  color: #cccccc;
  border: 2px dashed rgba(255, 255, 255, 0.2);
}

.drop-zone p {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

/* 底部操作栏 */
.bottom-action-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  background: rgba(18, 18, 18, 0.95);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(8px);
  z-index: 100;
}

.action-buttons-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-buttons-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-btn {
  border-radius: 8px;
  text-transform: none;
  font-weight: 500;
  background: rgba(50, 50, 50, 0.5) !important;
  color: #e0e0e0 !important;
  border: 1px solid rgba(80, 80, 80, 0.4) !important;
  transition: all 0.3s ease;
  padding: 0 16px;
}

.action-btn:hover {
  background: rgba(70, 70, 70, 0.6) !important;
  border-color: rgba(120, 120, 120, 0.5) !important;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.primary-btn {
  background: linear-gradient(135deg, #404040, #505050) !important;
  color: white !important;
  border: none !important;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.primary-btn:hover {
  background: linear-gradient(135deg, #505050, #606060) !important;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

@media (max-width: 768px) {
  .hmcl-sidebar {
    width: 60px;
  }

  .step-nav {
    gap: 12px;
  }

  .step-nav-item {
    width: 36px;
    height: 36px;
  }

  .step-icon {
    font-size: 14px;
  }

  .choose-cards {
    flex-direction: column;
    align-items: center;
  }

  .config-layout {
    flex-direction: column;
    gap: 16px;
  }
}

@media (max-width: 600px) {
  .hmcl-container {
    flex-direction: column;
  }

  .hmcl-sidebar {
    width: 100%;
    height: auto;
    flex-direction: row;
    border-right: none;
    border-bottom: 1px solid #3e3e42;
  }

  .step-nav {
    flex-direction: row;
    padding: 10px 0;
    gap: 8px;
  }

  .step-nav-item {
    width: 32px;
    height: 32px;
  }

  .step-icon {
    font-size: 12px;
  }

  .step-nav-item.active::after {
    left: 50%;
    top: -8px;
    transform: translateX(-50%);
    width: 24px;
    height: 4px;
  }

  .config-layout {
    flex-direction: column;
  }
}
</style>