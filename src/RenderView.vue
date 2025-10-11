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
  <div class="hmcl-container">
    <!-- 简化的侧边栏 -->
    <div class="hmcl-sidebar">
      <!-- 简化的步骤导航 -->
      <div class="step-nav">
        <div 
          v-for="(stepName, index) in steps" 
          :key="index" 
          class="step-nav-item"
          :class="{ 'active': stepIndex === index + 1, 'completed': stepIndex > index + 1 }"
          @click="stepIndex > index + 1 && (stepIndex = index + 1)"
        >
          <div class="step-icon">
            <v-icon v-if="stepIndex > index + 1" size="20">mdi-check</v-icon>
            <span v-else>{{ index + 1 }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区域 -->
    <div class="hmcl-main">
      <!-- 顶部操作栏 -->
      <div class="hmcl-header">
        <div class="header-title">
          <h3>{{ t('steps.' + step) }}</h3>
        </div>
        <div class="header-actions">
          <v-btn 
            v-if="step === 'config' || step === 'options'" 
            variant="text" 
            @click="stepIndex && stepIndex--" 
            class="header-btn"
            size="small"
          >
            <v-icon class="mr-1">mdi-arrow-left</v-icon>
            {{ t('prev-step') }}
          </v-btn>
          <v-btn 
            v-if="step === 'options'" 
            variant="outlined" 
            @click="playChart" 
            class="header-btn ml-2"
            size="small"
          >
            <v-icon class="mr-1">mdi-play</v-icon>
            {{ t('play') }}
          </v-btn>
          <v-btn 
            v-if="step === 'options'" 
            variant="outlined" 
            @click="previewChart" 
            class="header-btn ml-2"
            size="small"
          >
            <v-icon class="mr-1">mdi-eye</v-icon>
            {{ t('preview') }}
          </v-btn>
          <v-btn 
            v-if="step === 'config' || step === 'options'" 
            variant="flat" 
            @click="moveNext" 
            class="header-btn primary-btn ml-2"
            size="small"
          >
            {{ step === 'options' ? t('render') : t('next-step') }}
            <v-icon class="ml-1">mdi-arrow-right</v-icon>
          </v-btn>
        </div>
      </div>

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
          <div class="config-content" v-if="chartInfo">
            <v-form ref="form" class="config-form">
              <!-- 左右两栏布局 -->
              <div class="config-layout">
                <!-- 左栏 - 基本信息 -->
                <div class="config-column">
                  <div class="field-group">
                    <div class="field-title">基本信息</div>
                    <div class="field-list">
                      <div class="field-item">
                        <label class="field-label">{{ t('chart-name') }} *</label>
                        <v-text-field 
                          v-model="chartInfo.name"
                          :rules="[RULES.non_empty]"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('level') }} *</label>
                        <v-text-field 
                          v-model="chartInfo.level"
                          :rules="[RULES.non_empty]"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('charter') }} *</label>
                        <v-text-field 
                          v-model="chartInfo.charter"
                          :rules="[RULES.non_empty]"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('composer') }}</label>
                        <v-text-field 
                          v-model="chartInfo.composer"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('illustrator') }}</label>
                        <v-text-field 
                          v-model="chartInfo.illustrator"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 右栏 - 显示设置 -->
                <div class="config-column">
                  <div class="field-group">
                    <div class="field-title">显示设置</div>
                    <div class="field-list">
                      <div class="field-item">
                        <label class="field-label">{{ t('aspect') }}</label>
                        <div class="aspect-inputs">
                          <v-text-field 
                            type="number"
                            v-model="aspectWidth"
                            :rules="[RULES.positive]"
                            :label="t('width')"
                            variant="outlined"
                            density="compact"
                            hide-details
                            class="aspect-input"
                          ></v-text-field>
                          <span class="aspect-separator">:</span>
                          <v-text-field 
                            type="number"
                            v-model="aspectHeight"
                            :rules="[RULES.positive]"
                            :label="t('height')"
                            variant="outlined"
                            density="compact"
                            hide-details
                            class="aspect-input"
                          ></v-text-field>
                        </div>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('dim') }}</label>
                        <v-slider 
                          v-model="chartInfo.backgroundDim"
                          :min="0"
                          :max="1"
                          :step="0.01"
                          color="primary"
                          thumb-label="always"
                          density="compact"
                          hide-details
                          class="field-slider"
                        ></v-slider>
                      </div>
                      <div class="field-item">
                        <v-switch 
                          v-model="chartInfo.HoldPartialCover"
                          :true-value="1"
                          :false-value="0"
                          :label="t('hold_cover')"
                          color="primary"
                          hide-details
                          class="field-switch"
                        ></v-switch>
                      </div>
                      <div class="field-item">
                        <label class="field-label">{{ t('tip') }}</label>
                        <v-text-field 
                          v-model="chartInfo.tip"
                          :placeholder="t('tip-placeholder')"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="field-input"
                        ></v-text-field>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </v-form>
          </div>
        </div>

        <!-- 步骤3: 渲染选项 -->
        <div class="step-content" :class="{ 'active': step === 'options' }">
          <div class="options-content">
            <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()" class="hmcl-config"></ConfigView>
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
  background-color: #1e1e1e;
  font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;
  color: #e0e0e0;
}

.hmcl-sidebar {
  width: 80px;
  background-color: #252526;
  color: #cccccc;
  display: flex;
  flex-direction: column;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
  border-right: 1px solid #3e3e42;
}

/* 步骤导航 */
.step-nav {
  padding: 20px 0;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.step-nav-item {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: rgba(60, 52, 100, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  overflow: hidden;
}

.step-nav-item::before {
  content: '';
  position: absolute;
  left: -100%;
  top: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255,255,255,0.15),
    transparent
  );
  transition: 0.5s;
}

.step-nav-item:hover {
  transform: translateY(-2px) scale(1.05);
  background: linear-gradient(
    135deg,
    rgba(96, 67, 140, 0.4) 0%,
    rgba(118, 64, 193, 0.4) 100%
  ) !important;
  box-shadow: 0 4px 20px rgba(118, 64, 193, 0.4);
  color: #fff;
}

.step-nav-item:hover::before {
  left: 100%;
}

.step-nav-item.active {
  background: linear-gradient(
    135deg,
    rgba(118, 64, 193, 0.6) 0%,
    rgba(156, 105, 217, 0.6) 100%
  ) !important;
  color: #fff;
  box-shadow: 0 0 20px rgba(118, 64, 193, 0.6);
}

.step-nav-item.active::after {
  content: '';
  position: absolute;
  left: -12px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 24px;
  background-color: #007acc;
  border-radius: 2px;
}

.step-nav-item.completed .step-icon {
  color: #4caf50;
}

.step-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 主内容区 */
.hmcl-main {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  background-color: #1e1e1e;
}

.hmcl-header {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  background-color: #252526;
  border-bottom: 1px solid #3e3e42;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.header-title h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
  color: #cccccc;
}

.header-actions {
  display: flex;
  align-items: center;
}

.header-btn {
  margin-left: 8px;
  border-radius: 4px;
  text-transform: none;
  font-weight: 400;
  background-color: #3c3c3c !important;
  color: #cccccc !important;
  border: 1px solid #3e3e42 !important;
}

.header-btn:hover {
  background-color: #464647 !important;
}

.primary-btn {
  background-color: #007acc !important;
  color: white !important;
  border-color: #007acc !important;
}

.primary-btn:hover {
  background-color: #1e6ba8 !important;
}

/* 内容区域 */
/* 步骤内容 */
.hmcl-content {
  flex-grow: 1;
  padding: 24px;
  overflow-y: auto;
  background-color: #1e1e1e;
  position: relative;
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
}

.choose-cards {
  display: flex;
  gap: 24px;
  margin-top: 40px;
}

.choose-card {
  width: 220px;
  height: 260px;
  background-color: #252526;
  border: 1px solid #3e3e42;
  border-radius: 8px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.choose-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
  border-color: #007acc;
  background-color: #2a2d2e;
}

.card-icon {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background-color: #3c3c3c;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
  color: #cccccc;
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

/* 配置内容 */
.config-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.config-form {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
}

/* 配置布局 */
.config-layout {
  display: flex;
  gap: 24px;
  flex-grow: 1;
}

.config-column {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.field-group {
  background-color: #252526;
  border-radius: 8px;
  border: 1px solid #3e3e42;
  padding: 16px;
  height: 100%;
}

.field-title {
  font-size: 16px;
  font-weight: 500;
  color: #cccccc;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #3e3e42;
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
  background-color: #3c3c3c !important;
}

.field-input :deep(.v-field) {
  border-radius: 4px;
  background-color: #3c3c3c !important;
  color: #cccccc !important;
  border-color: #3e3e42 !important;
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
  background-color: #3c3c3c !important;
  color: #cccccc !important;
  border-color: #3e3e42 !important;
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
  background-color: #3c3c3c !important;
}

.field-slider :deep(.v-slider-track__fill) {
  background-color: #007acc !important;
}

.field-slider :deep(.v-slider-thumb) {
  background-color: #7640c1 !important;
  border-radius: 50% !important;
  width: 20px !important;
  height: 20px !important;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3) !important;
  transition: all 0.2s ease !important;
}

.field-slider :deep(.v-slider-thumb:hover) {
  transform: scale(1.2) !important;
  box-shadow: 0 3px 6px rgba(118, 64, 193, 0.5) !important;
}

.field-switch :deep(.v-switch__track) {
  background-color: #3c3c3c !important;
}

.field-switch :deep(.v-switch__thumb) {
  background-color: #666 !important;
}

.field-switch--selected :deep(.v-switch__track) {
  background-color: #007acc !important;
}

.field-switch--selected :deep(.v-switch__thumb) {
  background-color: #fff !important;
}

/* 渲染内容 */
.render-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.hmcl-card {
  width: 100%;
  max-width: 600px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  background-color: #252526;
  border: 1px solid #3e3e42;
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
  border-bottom: 1px solid #3e3e42;
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
  background-color: #2a2d2e;
}

.card-actions .v-btn {
  background-color: #3c3c3c !important;
  color: #cccccc !important;
  border: 1px solid #3e3e42 !important;
}

.card-actions .v-btn.v-btn--flat {
  background-color: #007acc !important;
  color: white !important;
  border-color: #007acc !important;
}

/* 加载和拖放覆盖层 */
.hmcl-loading {
  background-color: rgba(0, 0, 0, 0.7);
}

.loading-card {
  border-radius: 8px;
  overflow: hidden;
  background-color: #252526;
}

.hmcl-drop-overlay {
  background-color: rgba(0, 0, 0, 0.7);
}

.drop-zone {
  background-color: #252526;
  border-radius: 8px;
  padding: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  color: #cccccc;
  border: 2px dashed #3e3e42;
}

.drop-zone p {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
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