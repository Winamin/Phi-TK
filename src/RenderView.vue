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
    can-also-drop: You can also drag & drop the file to here
    drop: DROP CHART HERE

  chart-file: Chart file

  chart-name: Chart name
  charter: Charter
  illustrator: Illustrator
  level: Level
  aspect: Aspect ratio
  dim: Background dim

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

  render-started: Rendering has started!
  see-tasks: See tasks

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
    can-also-drop: 也可以直接拖谱面到窗口哦
    drop: 拖放谱面到这

  chart-file: 谱面文件

  chart-name: 谱面名
  charter: 谱师
  composer: 曲师
  illustrator: 画师
  level: 难度
  aspect: 宽高比
  dim: 背景昏暗程度

  tip: Tip
  tip-placeholder: 留空则随机选择

  width: 宽
  height: 高

  preview: 演示
  render: 渲染

  render-started: 视频开始渲染了！
  see-tasks: 查看任务列表

  ffmpeg-not-found: 笨蛋怎么没安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。

</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke, event, dialog, shell } from '@tauri-apps/api';

import { toastError, RULES, toast, anyFilter, isString } from './common';
import type { ChartInfo } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import moment from 'moment';

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

  // noexcept
  await loadChart(file as string);

  choosingChart.value = false;
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

const fileHovering = ref(false);
event.listen('tauri://file-drop-hover', (_event) => (fileHovering.value = step.value === 'choose'));
event.listen('tauri://file-drop-cancelled', (_event) => (fileHovering.value = false));
event.listen('tauri://file-drop', async (event) => {
  if (step.value === 'choose') {
    fileHovering.value = false;
    await loadChart((event.payload as string[])[0]);
  }
});

const form = ref<VForm>();

const configView = ref<typeof ConfigView>();
async function buildParams() {
  let config = await configView.value!.buildConfig();
  if (!config) return null;
  if (!chartInfo.value!.tip?.trim().length) chartInfo.value!.tip = null;
  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  };
}

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
    await invoke('post_render', { params });
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
  
  const offsetX = (e.clientX - rect.left - centerX) * 0.15;
  const offsetY = (e.clientY - rect.top - centerY) * 0.15;
  
  moveOffset.value = { 
    x: offsetX * (window.innerWidth / 1280),
    y: offsetY * (window.innerHeight / 720) 
  };
}
const archiveStyle = computed(() => ({
  transform: `translate(
    ${moveOffset.value.x * 1.2}px, 
    ${moveOffset.value.y * 1.2}px
  ) rotate3d(
    ${-moveOffset.value.y / 20}, 
    ${moveOffset.value.x / 20}, 
    0, 
    ${Math.sqrt(moveOffset.value.x**2 + moveOffset.value.y**2) / 4}deg
  )`,
  filter: `drop-shadow(${-moveOffset.value.x/4}px ${-moveOffset.value.y/4}px 6px rgba(99, 102, 241, 0.2))`
}));

const folderStyle = computed(() => ({
  transform: `translate(
    ${-moveOffset.value.x * 0.8}px, 
    ${-moveOffset.value.y * 0.8}px
  ) rotate3d(
    ${moveOffset.value.y / 20}, 
    ${-moveOffset.value.x / 20}, 
    0, 
    ${Math.sqrt(moveOffset.value.x**2 + moveOffset.value.y**2) / 6}deg
  )`,
  filter: `drop-shadow(${moveOffset.value.x/4}px ${moveOffset.value.y/4}px 6px rgba(99, 102, 241, 0.2))`
}));
</script>

<template>
  <div class="app-container">
    <div class="background-glow"></div>
    
    <div class="pa-8 w-100 h-100" style="max-width: 1280px">
      <v-stepper 
        alt-labels 
        v-model="stepIndex" 
        hide-actions 
        :items="steps.map((x) => t('steps.' + x))" 
        class="glass-stepper"
      >
        <div v-if="step === 'config' || step === 'options'" class="d-flex flex-row pa-6 pb-4 pt-0">
          <v-btn 
            variant="text" 
            @click="stepIndex && stepIndex--" 
            class="nav-button"
            v-t="'prev-step'"
          ></v-btn>
          <div class="flex-grow-1"></div>
          <v-btn 
            v-if="step === 'options'" 
            variant="tonal" 
            @click="previewChart" 
            class="mr-2 preview-button"
            v-t="'preview'"
          ></v-btn>
          <v-btn 
            variant="tonal" 
            @click="moveNext" 
            class="primary-action-button"
          >
            {{ step === 'options' ? t('render') : t('next-step') }}
            <template v-slot:loader>
              <v-progress-circular indeterminate size="20"></v-progress-circular>
            </template>
          </v-btn>
        </div>

        <template v-slot:item.1>
          <div 
            class="selection-container"
            @mousemove="onHoverMove"
            @mouseleave="resetHover"
            ref="hoverContainer"
          >
            <div class="selection-option left-option">
              <v-btn 
                :style="archiveStyle"
                class="selection-button archive-button" 
                size="x-large" 
                @click="chooseChart(false)" 
                prepend-icon="mdi-folder-zip"
              >
                {{ t('choose.archive') }}
                <div class="button-glow"></div>
              </v-btn>
            </div>
            
            <v-divider vertical class="divider-style"></v-divider>
            
            <div class="selection-option right-option">
              <v-btn 
                :style="folderStyle"
                class="selection-button folder-button" 
                size="x-large" 
                @click="chooseChart(true)" 
                prepend-icon="mdi-folder"
              >
                {{ t('choose.folder') }}
                <div class="button-glow"></div>
              </v-btn>
            </div>
          </div>
          
          <p class="drop-hint">
            <v-icon icon="mdi-arrow-down" class="bounce-icon"></v-icon>
            {{ t('choose.can-also-drop') }}
          </p>
          
          <v-overlay 
            v-model="parsingChart" 
            contained 
            class="loading-overlay" 
            persistent 
            :close-on-content-click="false"
          >
            <div class="loading-content">
              <v-progress-circular 
                indeterminate 
                size="64"
                width="4"
                color="primary"
              ></v-progress-circular>
              <p class="loading-text">Loading Chart...</p>
            </div>
          </v-overlay>
        </template>

        <template v-slot:item.2>
          <v-form ref="form" v-if="chartInfo" class="config-form">
            <v-row no-gutters class="form-row">
              <v-col cols="8">
                <v-text-field 
                  class="form-field" 
                  :label="t('chart-name')" 
                  :rules="[RULES.non_empty]" 
                  v-model="chartInfo.name"
                  variant="outlined"
                  color="primary"
                ></v-text-field>
              </v-col>
              <v-col cols="4">
                <v-text-field 
                  class="form-field" 
                  :label="t('level')" 
                  :rules="[RULES.non_empty]" 
                  v-model="chartInfo.level"
                  variant="outlined"
                  color="primary"
                ></v-text-field>
              </v-col>
            </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('charter')" :rules="[RULES.non_empty]" v-model="chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('composer')" v-model="chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('illustrator')" v-model="chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1 align-center">
            <v-col cols="4">
              <div class="mx-2 d-flex flex-column">
                <p class="text-caption" v-t="'aspect'"></p>
                <div class="d-flex flex-row align-center justify-center">
                  <v-text-field type="number" class="mr-2" :rules="[RULES.positive]" :label="t('width')" v-model="aspectWidth"></v-text-field>
                  <p>:</p>
                  <v-text-field type="number" class="ml-2" :rules="[RULES.positive]" :label="t('height')" v-model="aspectHeight"></v-text-field>
                </div>
              </div>
            </v-col>
            <v-col cols="8" class="px-6">
              <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.05" v-model="chartInfo.backgroundDim"></v-slider>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12">
              <v-text-field class="mx-2" :label="t('tip')" :placeholder="t('tip-placeholder')" v-model="chartInfo.tip"></v-text-field>
            </v-col>
          </v-row>
        </v-form>
      </template>

      <template v-slot:item.3>
          <ConfigView 
            ref="configView" 
            :init-aspect-ratio="tryParseAspect()"
            class="config-view"
          ></ConfigView>
        </template>

      <template v-slot:item.4>
          <div class="completion-screen">
            <div class="confetti-animation"></div>
            <v-icon icon="mdi-check-circle" size="100" color="success" class="success-icon"></v-icon>
            <h2 class="completion-title">{{ t('render-started') }}</h2>
            <v-btn 
              @click="router.push({ name: 'tasks' })" 
              class="task-button"
              v-t="'see-tasks'"
            ></v-btn>
          </div>
        </template>
      </v-stepper>
      
      <v-overlay 
        v-model="fileHovering" 
        contained 
        class="drop-zone-overlay" 
        persistent 
        :close-on-content-click="false"
      >
        <div class="drop-zone-content">
          <div class="drop-pulse"></div>
          <h1>{{ t('choose.drop') }}</h1>
          <div class="drop-zone-glow"></div>
        </div>
      </v-overlay>
    </div>
  </div>
</template>

<style scoped>
/* 基础布局 */
.app-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%);
}

.background-glow {
  position: fixed;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle at center, rgba(99, 102, 241, 0.1) 0%, transparent 70%);
  animation: rotate-glow 60s linear infinite;
  z-index: -1;
}

@keyframes rotate-glow {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 玻璃拟态效果 */
.glass-stepper {
  border-radius: 24px !important;
  background: rgba(23, 9, 99, 0.6) !important;
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 
    0 8px 32px 0 rgba(31, 38, 135, 0.2),
    0 0 40px -10px rgb(99 102 241 / 0.3),
    inset 0 0 20px rgba(255, 255, 255, 0.05);
  overflow: hidden;
  position: relative;
}

.glass-stepper::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.05) 0%,
    rgba(255, 255, 255, 0) 50%,
    rgba(255, 255, 255, 0.05) 100%
  );
  pointer-events: none;
}

/* 选择谱面区域 */
.selection-container {
  display: flex;
  margin-top: 2rem;
  gap: 1rem;
  height: 300px;
  align-items: center;
  justify-content: center;
  position: relative;
}

.selection-option {
  flex-grow: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  position: relative;
}

.divider-style {
  height: 60%;
  align-self: center;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.selection-button {
  width: 75%;
  height: 120px;
  border-radius: 16px !important;
  font-size: 1.1rem;
  font-weight: 500;
  letter-spacing: 0.5px;
  transition: all 0.6s cubic-bezier(0.23, 1, 0.32, 1);
  position: relative;
  overflow: hidden;
  z-index: 1;
}

.archive-button {
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
}

.folder-button {
  background: linear-gradient(45deg, #ec4899, #f97316) !important;
}

.button-glow {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    45deg,
    transparent 25%,
    rgba(255, 255, 255, 0.1) 50%,
    transparent 75%
  );
  background-size: 200% 200%;
  opacity: 0;
  transition: opacity 0.3s;
  z-index: -1;
}

.selection-button:hover .button-glow {
  opacity: 1;
  animation: shine 1.5s infinite;
}

.selection-button:hover {
  transform: 
    translateY(-6px) 
    scale(1.05) 
    rotateZ(1deg) !important;
  box-shadow: 
    0 12px 24px -6px rgba(0, 0, 0, 0.3),
    0 8px 16px -8px rgba(0, 0, 0, 0.2) !important;
}

@keyframes shine {
  0% { background-position: 150% 150%; }
  100% { background-position: -50% -50%; }
}

/* 拖放区域 */
.drop-zone-overlay {
  background: rgba(99, 102, 241, 0.2) !important;
  backdrop-filter: blur(12px);
}

.drop-zone-content {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.drop-pulse {
  width: 200px;
  height: 200px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  border: 2px dashed rgba(255, 255, 255, 0.3);
  animation: 
    pulse 2s infinite,
    float 3s ease-in-out infinite;
  position: absolute;
}

.drop-zone-content h1 {
  font-size: 2.5rem;
  font-weight: 600;
  color: white;
  text-shadow: 0 2px 10px rgba(99, 102, 241, 0.5);
  z-index: 1;
}

.drop-zone-glow {
  position: absolute;
  width: 300px;
  height: 300px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(99, 102, 241, 0.3) 0%, transparent 70%);
  animation: pulse-glow 3s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { transform: scale(1); opacity: 0.7; }
  50% { transform: scale(1.2); opacity: 0.3; }
}

/* 加载动画 */
.loading-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.loading-text {
  color: white;
  font-size: 1.2rem;
  font-weight: 500;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* 按钮样式 */
.primary-action-button {
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
  color: white !important;
  font-weight: 500;
  letter-spacing: 0.5px;
  padding: 0 24px;
  height: 48px;
  border-radius: 12px !important;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3) !important;
  transition: all 0.3s ease;
}

.primary-action-button:hover {
  transform: translateY(-2px) !important;
  box-shadow: 0 8px 16px rgba(99, 102, 241, 0.4) !important;
}

.preview-button {
  background: rgba(255, 255, 255, 0.1) !important;
  border: 1px solid rgba(255, 255, 255, 0.2) !important;
  color: white !important;
  transition: all 0.3s ease;
}

.preview-button:hover {
  background: rgba(255, 255, 255, 0.2) !important;
  transform: translateY(-2px) !important;
}

/* 完成页面 */
.completion-screen {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  position: relative;
  padding: 2rem;
}

.success-icon {
  margin-bottom: 1.5rem;
  filter: drop-shadow(0 4px 12px rgba(76, 175, 80, 0.4));
}

.completion-title {
  font-size: 1.8rem;
  font-weight: 600;
  color: white;
  text-align: center;
  margin-bottom: 2rem;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.task-button {
  background: rgba(255, 255, 255, 0.1) !important;
  border: 1px solid rgba(255, 255, 255, 0.2) !important;
  color: white !important;
  padding: 0 24px;
  height: 48px;
  border-radius: 12px !important;
  transition: all 0.3s ease;
}

.task-button:hover {
  background: rgba(255, 255, 255, 0.2) !important;
  transform: translateY(-2px) !important;
}

/* 表单样式 */
.config-form {
  padding: 1rem;
}

.form-field {
  margin: 0.5rem;
}

.form-field :deep(.v-field) {
  background: rgba(255, 255, 255, 0.05) !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
  color: white !important;
  border-radius: 12px !important;
  transition: all 0.3s ease;
}

.form-field :deep(.v-field--focused) {
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.5) !important;
  background: rgba(255, 255, 255, 0.1) !important;
}

.form-field :deep(.v-label) {
  color: rgba(255, 255, 255, 0.7) !important;
}

.form-field :deep(.v-field__input) {
  color: white !important;
}

/* 动画效果 */
@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.1); opacity: 1; }
}

.bounce-icon {
  animation: bounce 2s infinite;
  margin-right: 8px;
}

@keyframes bounce {
  0%, 20%, 50%, 80%, 100% { transform: translateY(0); }
  40% { transform: translateY(-10px); }
  60% { transform: translateY(-5px); }
}

.drop-hint {
  text-align: center;
  margin: 1rem 0 2rem;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 步骤条样式 */
:deep(.v-stepper-header) {
  background: transparent !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

:deep(.v-stepper-header__item) {
  transition: all 0.4s cubic-bezier(0.23, 1, 0.32, 1);
  position: relative;
  overflow: hidden;
}

:deep(.v-stepper-header__item)::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 50%;
  width: 0;
  height: 2px;
  background: linear-gradient(90deg, #6366f1, #8b5cf6);
  transition: all 0.3s ease;
  transform: translateX(-50%);
}

:deep(.v-stepper-header__item--active)::after {
  width: 80%;
}

:deep(.v-stepper-header__item:hover) {
  transform: translateY(-2px);
  background: rgba(255, 255, 255, 0.05);
}

:deep(.v-stepper-header__item:hover)::after {
  width: 60%;
}

:deep(.v-stepper-header__title) {
  color: rgba(255, 255, 255, 0.8) !important;
  font-weight: 500;
}

:deep(.v-stepper-header__title.v-stepper-header__title--active) {
  color: white !important;
  text-shadow: 0 0 8px rgba(99, 102, 241, 0.3);
}

/* 内容过渡动画 */
:deep(.v-stepper__content) {
  transition: 
    opacity 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    transform 0.8s cubic-bezier(0.23, 1, 0.32, 1);
}

:deep(.v-stepper__content-leave-active) {
  transition: 
    opacity 0.3s ease,
    transform 0.4s ease;
}

:deep(.v-stepper__content-leave-to) {
  opacity: 0;
  transform: 
    translateX(-20px) 
    perspective(500px) 
    rotateY(10deg) 
    scale(0.98);
}

:deep(.v-stepper__content-enter-from) {
  opacity: 0;
  transform: 
    translateX(20px) 
    perspective(500px) 
    rotateY(-10deg) 
    scale(0.98);
}
</style>