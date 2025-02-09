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
    archive: Archive (.zip, .pez)
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
    archive: 压缩包 (.zip, .pez)
    folder: 文件夹
    can-also-drop: 也可以直接拖谱面到窗口哦～
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

  preview: 预览
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
</script>

<template>
  <div class="render-container glass-background">
    <v-stepper 
      alt-labels 
      v-model="stepIndex" 
      hide-actions 
      :items="steps.map((x) => t('steps.' + x))"
      class="stepper-glow"
    >
      <div v-if="step === 'config' || step === 'options'" class="step-controls">
        <v-btn 
          variant="text" 
          @click="stepIndex && stepIndex--" 
          class="nav-btn hover-scale"
        >
          <v-icon>mdi-arrow-left</v-icon>
          {{ t('prev-step') }}
        </v-btn>
        <div class="flex-grow-1"></div>
        <v-btn 
          v-if="step === 'options'" 
          variant="tonal" 
          @click="previewChart" 
          class="preview-btn hover-scale"
        >
          <v-icon>mdi-eye-outline</v-icon>
          {{ t('preview') }}
        </v-btn>
        <v-btn 
          variant="tonal" 
          @click="moveNext" 
          class="next-btn gradient-btn hover-scale"
        >
          {{ step === 'options' ? t('render') : t('next-step') }}
          <v-icon>mdi-arrow-right</v-icon>
        </v-btn>
      </div>

      <template v-slot:item.1>
        <div class="step-choose">
          <div class="file-selector">
            <div class="file-option-card hover-scale" @click="chooseChart(false)">
              <div class="option-icon gradient-bg">
                <v-icon size="48">mdi-folder-zip</v-icon>
              </div>
              <h3 class="option-title">{{ t('choose.archive') }}</h3>
            </div>
            
            <v-divider vertical class="divider-glow"></v-divider>
            
            <div class="file-option-card hover-scale" @click="chooseChart(true)">
              <div class="option-icon gradient-bg">
                <v-icon size="48">mdi-folder</v-icon>
              </div>
              <h3 class="option-title">{{ t('choose.folder') }}</h3>
            </div>
          </div>
          
          <p class="drop-hint text-glow">{{ t('choose.can-also-drop') }}</p>
          
          <v-overlay v-model="parsingChart" class="loading-overlay">
            <v-progress-circular 
              indeterminate 
              size="64"
              width="4"
              class="glow-spinner"
            ></v-progress-circular>
          </v-overlay>
        </div>
      </template>

      <template v-slot:item.2>
        <v-form ref="form" v-if="chartInfo" class="step-config">
        </v-form>
      </template>

      <template v-slot:item.4>
        <div class="render-complete">
          <div class="particle-effect"></div>
          <span class="emoji-glow">🎉</span>
          <h1 class="complete-title text-gradient">{{ t('render-started') }}</h1>
          <v-btn 
            @click="router.push({ name: 'tasks' })" 
            class="task-btn hover-scale"
            variant="flat"
          >
            <v-icon>mdi-playlist-check</v-icon>
            {{ t('see-tasks') }}
          </v-btn>
        </div>
      </template>
    </v-stepper>

    <v-overlay v-model="fileHovering" class="drop-overlay">
      <div class="drop-container">
        <div class="drop-glow"></div>
        <h1 class="drop-text neon-text">{{ t('choose.drop') }}</h1>
      </div>
    </v-overlay>
  </div>
</template>

<style scoped>
.render-container {
  padding: 2rem;
  width: 100%;
  height: 100%;
  max-width: 1280px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(16px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.stepper-glow {
  :deep(.v-stepper__step) {
    &::after {
      background: linear-gradient(90deg, #2196f3, #e91e63);
      filter: drop-shadow(0 0 8px rgba(33, 150, 243, 0.3));
    }
    &.v-stepper__step--active {
      .v-stepper__step__title {
        text-shadow: 0 0 12px rgba(33, 150, 243, 0.4);
      }
    }
  }
}

.file-option-card {
  width: 300px;
  height: 200px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  
  &:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.3);
    
    &::before {
      opacity: 1;
    }
  }
  
  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(45deg, #2196f333, #e91e6333);
    opacity: 0;
    transition: opacity 0.3s ease;
  }
}

.option-icon {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 1rem;
}

.gradient-bg {
  background: linear-gradient(45deg, #2196F3, #E91E63);
}

.text-gradient {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.text-glow {
  text-shadow: 0 0 12px rgba(33, 150, 243, 0.4);
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #2196F3);
}

.hover-scale {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  
  &:hover {
    transform: scale(1.05);
  }
}

.gradient-btn {
  background: linear-gradient(45deg, #2196F3, #E91E63) !important;
  color: white !important;
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.3) !important;
}

.drop-overlay {
  background: rgba(0, 0, 0, 0.9) !important;
  
  .drop-container {
    position: relative;
    padding: 2rem;
    
    .drop-glow {
      position: absolute;
      inset: -20px;
      background: linear-gradient(45deg, #2196F3, #E91E63);
      filter: blur(40px);
      opacity: 0.3;
      animation: pulse 2s infinite;
    }
  }
}

@keyframes pulse {
  0% { opacity: 0.3; }
  50% { opacity: 0.6; }
  100% { opacity: 0.3; }
}

.neon-text {
  color: #fff;
  text-shadow: 
    0 0 10px #2196F3,
    0 0 20px #2196F3,
    0 0 30px #2196F3;
}

/* 完成页面特效 */
.render-complete {
  position: relative;
  .particle-effect {
    position: absolute;
    inset: 0;
    background: radial-gradient(circle, rgba(33,150,243,0.1) 0%, transparent 70%);
  }
  
  .emoji-glow {
    font-size: 96px;
    filter: drop-shadow(0 0 12px rgba(33,150,243,0.5));
    animation: float 3s ease-in-out infinite;
  }
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-20px); }
}
</style>
