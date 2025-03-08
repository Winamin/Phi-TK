<i18n>
zh-CN:
  already-running: Phi TK 已经在运行

  prev-step: 上一步
  next-step: 下一步

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
  <div class="pa-8 w-100 h-100" style="max-width: 1280px">
    <v-stepper alt-labels v-model="stepIndex" hide-actions :items="['','','','']" class="elevated-stepper">
      <template v-slot:item.1>
        <div 
          class="d-flex" 
          style="gap: 2rem; margin-top: -40px; padding: 0 2rem"
          @mousemove="onHoverMove"
          @mouseleave="resetHover"
          ref="hoverContainer"
        >
          <div class="flex-grow-1 d-flex align-center justify-center w-0 py-8">
            <v-btn 
              :style="archiveStyle"
              class="w-100 gradient-primary hover-movable" 
              style="
                height: 140px;
                font-size: 1.8rem;
                letter-spacing: 3px;
                border-radius: 16px;
              " 
              @click="chooseChart(false)" 
              prepend-icon="mdi-folder-zip"
            >{{ t('choose.archive') }}</v-btn>
          </div>
          <v-divider vertical thickness="2"></v-divider>
          <div class="flex-grow-1 d-flex align-center justify-center w-0">
            <v-btn 
              :style="folderStyle"
              class="w-100 gradient-primary hover-movable" 
              style="
                height: 140px;
                font-size: 1.8rem;
                letter-spacing: 3px;
                border-radius: 16px;
              " 
              @click="chooseChart(true)" 
              prepend-icon="mdi-folder"
            >{{ t('choose.folder') }}</v-btn>
          </div>
        </div>
        <p class="mb-8 w-100 text-center mt-4 text-disabled" v-t="'choose.can-also-drop'"></p>
        <v-overlay v-model="parsingChart" contained class="align-center justify-center" persistent :close-on-content-click="false">
          <v-progress-circular indeterminate size="64"></v-progress-circular>
        </v-overlay>
      </template>

      <template v-slot:item.2>
        <v-form ref="form" v-if="chartInfo">
          <!-- 保持原有表单内容不变 -->
        </v-form>
      </template>

      <template v-slot:item.3>
        <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()"></ConfigView>
      </template>

      <template v-slot:item.4>
        <div class="d-flex flex-column justify-center align-center mb-2" style="gap: 1rem">
          <span style="font-size: 84px">✨</span>
          <h2>{{ t('render-started') }}</h2>
          <v-btn @click="router.push({ name: 'tasks' })" v-t="'see-tasks'"></v-btn>
        </div>
      </template>
    </v-stepper>
    
    <v-overlay v-model="fileHovering" contained class="align-center justify-center drop-zone-overlay" persistent :close-on-content-click="false">
      <div class="drop-pulse">
        <h1 v-t="'choose.drop'"></h1>
      </div>
    </v-overlay>
  </div>
</template>

<style scoped>
.gradient-primary {
  background: linear-gradient(
    45deg, 
    #6366f1, 
    #8b5cf6, 
    #ec4899
  ) !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.3);
  transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  position: relative;
  overflow: hidden;
}

.gradient-primary::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    45deg,
    transparent 25%,
    rgba(255,255,255,0.1) 50%,
    transparent 75%
  );
  background-size: 200% 200%;
  opacity: 0;
  transition: opacity 0.3s;
}

.gradient-primary:hover::after {
  opacity: 1;
  animation: shine 1.5s infinite;
}

@keyframes shine {
  0% { background-position: 150% 150%; }
  100% { background-position: -50% -50%; }
}

.hover-movable {
  transition: 
    transform 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    filter 0.4s ease,
    box-shadow 0.4s ease;
  will-change: transform, filter;
}

.v-btn:hover {
  transform: 
    translateY(-2px) 
    scale(1.05) 
    rotateZ(1deg) !important;
  box-shadow: 
    0 12px 24px -6px rgb(99 102 241 / 0.4),
    0 4px 12px -4px rgb(99 102 241 / 0.3) !important;
}

.elevated-stepper {
  border-radius: 24px !important;
  box-shadow: 
    0 25px 50px -12px rgb(0 0 0 / 0.3),
    0 0 40px -10px rgb(99 102 241 / 0.3) !important;
  background: rgba(23, 9, 99, 0.9) !important;
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255,255,255,0.1);
}

.drop-zone-overlay {
  background: rgba(99, 102, 241, 0.2) !important;
  backdrop-filter: blur(8px);
}

.drop-pulse {
  animation: 
    pulse 2s infinite,
    float 3s ease-in-out infinite;
  text-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

:deep(.v-stepper-header__item) {
  transition: all 0.4s ease;
}

:deep(.v-stepper-header__item):hover {
  transform: translateY(-2px);
}

.v-stepper__content {
  transition: 
    opacity 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    transform 0.8s cubic-bezier(0.23, 1, 0.32, 1);
}

.v-stepper__content-leave-active {
  transition: 
    opacity 0.3s ease,
    transform 0.4s ease;
}

.v-stepper__content-leave-to {
  opacity: 0;
  transform: 
    translateX(-20px) 
    perspective(500px) 
    rotateY(10deg) 
    scale(0.98);
}

.v-stepper__content-enter-from {
  opacity: 0;
  transform: 
    translateX(20px) 
    perspective(500px) 
    rotateY(-10deg) 
    scale(0.98);
}

.v-text-field :deep(.v-field) {
  transition: 
    box-shadow 0.4s ease,
    transform 0.3s ease;
}

.v-text-field :deep(.v-field--focused) {
  box-shadow: 
    0 0 0 2px rgb(99 102 241 / 0.3),
    0 8px 24px -6px rgb(99 102 241 / 0.2) !important;
  transform: scale(1.02);
}

.v-slider__thumb {
  transition: 
    transform 0.2s ease,
    box-shadow 0.3s ease !important;
}

.v-slider__thumb:hover {
  transform: scale(1.2);
  box-shadow: 
    0 0 0 6px rgb(99 102 241 / 0.15) !important;
}

.v-divider--vertical {
  height: 160px !important;
  align-self: center;
}
</style>
