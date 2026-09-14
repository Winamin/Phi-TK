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
      folder-desc: Unpacked chart folder
      filter-name: Chart file

    chart-file: Chart file
    choose-drop: Drag and drop chart file here
    parsing: Parsing chart…

    hero:
      tagline: Turn any Phigros chart into a shareable video.
    cards:
      open: Open a chart
      open-desc: Archive or an unpacked folder
    recent:
      title: Recent charts
      empty: Charts you open will show up here
      clear: Clear
    close: Close
    shortcuts:
      title: Keyboard shortcuts
      label: Shortcuts
      close: Close the current page
      next: Next step
      back: Previous step
      open: Open a chart
      help: Toggle this help
      hint: Shortcuts are disabled while typing in a field

    chart-name: Chart name
    charter: Charter
    composer: Composer
    illustrator: Illustrator
    level: Level
    aspect: Aspect ratio
    dim: Background dim
    hold_cover: Hold Head Partial Cover

    tip: Tip
    tip-placeholder: Leave empty to choose randomly

    preview: Preview
    render: Render
    play: Play

    render-started: Rendering has started!
    see-tasks: See tasks
    next-chart: Render Next Chart
    duration: Elapsed
    render-status: 'Progress: {progress}% | FPS: {fps} | ETA: {estimate}'

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
      folder-desc: 已解压的谱面文件夹
      filter-name: 谱面文件

    chart-file: 谱面文件
    choose-drop: 拖拽谱面文件到此处
    parsing: 解析谱面中…

    hero:
      tagline: 把任意 Phigros 谱面变成可以分享的视频。
    cards:
      open: 打开谱面
      open-desc: 压缩包或已解压的文件夹
    recent:
      title: 最近打开
      empty: 打开过的谱面会出现在这里
      clear: 清空
    close: 关闭
    shortcuts:
      title: 键盘快捷键
      label: 快捷键
      close: 关闭当前页面
      next: 下一步
      back: 上一步
      open: 打开谱面
      help: 显示 / 隐藏本帮助
      hint: 输入框聚焦时快捷键自动停用

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

    preview: 演示
    render: 渲染
    play: 游玩

    render-started: 视频开始渲染了！
    see-tasks: 查看任务列表
    next-chart: 渲染下一个谱面
    duration: 耗时
    render-status: '进度：{progress}% | FPS: {fps} | 预计：{estimate}'

    ffmpeg-not-found: 笨蛋怎么没安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。

  </i18n>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { toastError, toast, anyFilter, isString, validateFields } from './common';
import type { ChartInfo, FileDropEvent, Task } from './model';
import ConfigView from './components/ConfigView.vue';
import MdTextField from './components/md/MdTextField.vue';
import MdSlider from './components/md/MdSlider.vue';
import MdSwitch from './components/md/MdSwitch.vue';
import moment from 'moment';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as shell from '@tauri-apps/plugin-shell';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import gsap from 'gsap';
const { t } = useI18n();

// Tauri's `listen` resolves asynchronously, so a component unmounted before it settles
// must still release the handle — hence the `disposed` guard.
let disposed = false;
const unlistenFns: UnlistenFn[] = [];

function track(pending: Promise<UnlistenFn>) {
  pending.then((unlisten) => (disposed ? unlisten() : unlistenFns.push(unlisten))).catch((e) => console.error('Failed to register listener:', e));
}

onUnmounted(() => {
  disposed = true;
  unlistenFns.forEach((unlisten) => unlisten());
  unlistenFns.length = 0;
});

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

const flipCardRef = ref<HTMLElement>();

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
    renderCover.value = null;
    chartInfo.value = (await invoke('parse_chart', { path: file })) as ChartInfo;
    rememberChart(chartInfo.value, file);
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

    await nextTick();
    performFlip();
  } catch (e) {
    toastError(e);
  } finally {
    parsingChart.value = false;
  }
}

function performFlip() {
  stepIndex.value = 2;
  nextTick(() => {
    if (flipCardRef.value) {
      gsap.from(flipCardRef.value, {
        scale: 0.96,
        opacity: 0,
        y: 24,
        duration: 0.45,
        ease: 'power2.out',
      });
    }
  });
}

function unflipCard() {
  stepIndex.value = 1;
}

const aspectWidth = ref('0'),
  aspectHeight = ref('0');

const form = ref<HTMLFormElement>();

const configView = ref<typeof ConfigView>();
async function buildParams() {
  let config = await configView.value!.buildConfig();
  if (!config) return null;
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
      await shell.open('https://github.com/BtbN/FFmpeg-Builds/releases');
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
const renderCover = ref<string | null>(null);
let coverPollInterval: ReturnType<typeof setInterval> | null = null;
track(listen('render-msg', (msg) => (renderMsg.value = msg.payload as string)));
track(
  listen('render-progress', (msg) => {
    let payload = msg.payload as { progress: number; fps: number; estimate: number };
    renderMsg.value = t('render-status', {
      progress: (payload.progress * 100).toFixed(2),
      fps: payload.fps,
      estimate: moment.duration(payload.estimate, 'seconds').humanize(true, { ss: 1 }),
    });
    renderProgress.value = payload.progress * 100;
  }),
);
// Deliberately does *not* advance `stepIndex`: 'render' is the last step, so incrementing
// left `step` undefined and blanked the whole view once the render finished. The elapsed
// time now lands on the render surface instead.
track(listen('render-done', (msg) => (renderDuration.value = Math.round(msg.payload as number))));

async function fetchRenderCover() {
  try {
    const tasks = await invoke<Task[]>('get_tasks');
    if (tasks && tasks.length > 0) {
      const task = tasks.find((t) => t.path === chartPath);
      if (task?.cover) {
        renderCover.value = task.cover;
        // The cover never changes once resolved — no reason to keep hitting the backend.
        stopCoverPolling();
      }
    }
  } catch (e) {
    console.error('Failed to fetch render cover:', e);
  }
}

function startCoverPolling() {
  stopCoverPolling();
  fetchRenderCover();
  coverPollInterval = setInterval(fetchRenderCover, 700);
}

function stopCoverPolling() {
  if (coverPollInterval) {
    clearInterval(coverPollInterval);
    coverPollInterval = null;
  }
}

watch(step, (s) => {
  if (s === 'render') startCoverPolling();
  else stopCoverPolling();
});

onUnmounted(() => stopCoverPolling());

async function moveNext() {
  if (step.value === 'config') {
    // Native constraint validation over the mdui fields, replacing `<v-form>.validate()`.
    if (validateFields(form.value)) {
      stepIndex.value++;
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

function goBack() {
  if (step.value === 'config') {
    unflipCard();
    return;
  }
  if (stepIndex.value > 1) {
    stepIndex.value--;
  }
}

/* ============================================================================
   Step chrome.

   Steps above `choose` are plain full-area pages, so `onStepPage` gates which
   of the two top-level branches renders — the launcher or a step page.
   ============================================================================ */

/** Whether a step page is showing (i.e. the launcher is unmounted). */
const onStepPage = computed(() => step.value !== 'choose');

/**
 * The config and render-options steps are dense forms that want the full window,
 * so the app rail folds away for them. The render-result step is read-only, and
 * the user is likely to head for "tasks" next — the rail comes back there.
 */
const wantsFullWidth = computed(() => step.value === 'config' || step.value === 'options');

function emitRail(hidden: boolean) {
  window.dispatchEvent(new CustomEvent('rail-hidden', { detail: hidden }));
}

watch(wantsFullWidth, emitRail, { immediate: true });
// A step flow can end by navigation rather than by the user stepping back, so the
// rail must be restored on the way out or it would stay hidden for the whole app.
onUnmounted(() => emitRail(false));

const surfaceTitle = computed(() => t(`steps.${step.value}`));

function closeSurfaces() {
  stepIndex.value = 1;
}

const showShortcuts = ref(false);

function onKeydown(e: KeyboardEvent) {
  const target = e.target as HTMLElement | null;
  // Never hijack keys while the user is typing into a field.
  const typing = !!target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable);

  if (e.key === 'Escape') {
    if (showShortcuts.value) {
      showShortcuts.value = false;
      return;
    }
    if (onStepPage.value && !typing) closeSurfaces();
    return;
  }

  if (e.key === '?' && !typing) {
    showShortcuts.value = !showShortcuts.value;
    return;
  }

  if (e.ctrlKey && e.key === 'o') {
    e.preventDefault();
    closeSurfaces();
    chooseChart(e.shiftKey);
    return;
  }

  // Arrow-driven stepping mirrors a modal wizard: only active on floating layers.
  if (onStepPage.value && !typing) {
    if (e.key === 'ArrowRight' && step.value === 'config') moveNext();
    if (e.key === 'ArrowLeft') goBack();
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));

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

/** The dim slider stores 0–1 but reads better as a percentage on the handle label. */
const dimLabel = (value: number) => `${Math.round(value * 100)}%`;

const durationText = computed(() =>
  renderDuration.value === undefined ? '' : moment.duration(renderDuration.value, 'seconds').humanize(),
);

const fileHovering = ref(false);
track(listen('tauri://drag-over', () => (fileHovering.value = step.value === 'choose')));
track(listen('tauri://drag-leave', () => (fileHovering.value = false)));
track(
  listen('tauri://drag-drop', async (payload) => {
    const files = (payload.payload as FileDropEvent).paths;
    fileHovering.value = false;
    // A drop anywhere returns to the launcher and loads the new chart, so the
    // wizard never keeps stale options from the previous chart on screen.
    stepIndex.value = 1;
    await loadChart(files[0]);
  }),
);

function resetAndGoChoose() {
  stepIndex.value = 1;
  chartInfo.value = undefined;
  chartPath = '';
  renderCover.value = null;
}

/* ============================================================================
   Start screen — data sources.

   Everything here is read-only chrome around the two "open" actions, so each
   accessor is defensive: a missing `localStorage` key or a corrupt stored blob
   must degrade to a placeholder rather than break the landing surface.
   ============================================================================ */

type RecentChart = {
  path: string;
  name: string;
  level?: string;
  charter?: string;
  at: number;
};

const RECENT_KEY = 'recentCharts';
const RECENT_LIMIT = 5;

const recentCharts = ref<RecentChart[]>([]);

function readRecent(): RecentChart[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((x): x is RecentChart => !!x && typeof (x as RecentChart).path === 'string');
  } catch {
    return [];
  }
}

function rememberChart(info: ChartInfo | undefined, path: string) {
  const entry: RecentChart = {
    path,
    name: info?.name?.trim() || path.split(/[\\/]/).pop() || path,
    level: info?.level,
    charter: info?.charter,
    at: Date.now(),
  };
  // De-duplicate by path so re-opening the same chart moves it to the top
  // instead of filling the list with copies.
  const next = [entry, ...recentCharts.value.filter((x) => x.path !== path)].slice(0, RECENT_LIMIT);
  recentCharts.value = next;
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(next));
  } catch (e) {
    console.error('Failed to persist recent charts:', e);
  }
}

function clearRecent() {
  recentCharts.value = [];
  localStorage.removeItem(RECENT_KEY);
}

onMounted(() => (recentCharts.value = readRecent()));

/** `WxH` → a compact "16:9"-style label; used on the render page. */
function aspectText(width: string, height: string) {
  const w = parseFloat(width);
  const h = parseFloat(height);
  if (isNaN(w) || isNaN(h) || h === 0) return '';
  return `${w}:${h}`;
}
</script>

<template>
  <div class="render-container">
    <!-- ==================================================================
         Launcher.

         A step page replaces this entirely rather than stacking over it, so the
         launcher is unmounted while a step is active — `v-if`, not `v-show`.
         ================================================================== -->
    <section v-if="!onStepPage" class="start-layer">
      <div class="start-scroll">
        <div class="start-inner">
          <!-- ---------- Hero ---------- -->
          <header class="hero">
            <div class="hero-text">
              <h1 class="hero-title">{{ t('steps.choose') }}</h1>
              <p class="hero-tagline">{{ t('hero.tagline') }}</p>
            </div>
          </header>

          <!-- ---------- Two cells: open, and what was open before ---------- -->
          <div class="bento">
            <!-- Primary: the two open actions, sized as the visual anchor. -->
            <div class="bento-cell cell-open">
              <h2 class="cell-title">{{ t('cards.open') }}</h2>
              <p class="cell-sub">{{ t('cards.open-desc') }}</p>
              <div class="open-row">
                <button class="open-tile" type="button" @click="chooseChart(false)">
                  <span class="md3-icon-badge md3-icon-badge-primary open-tile-icon">
                    <mdui-icon-folder-zip--outlined></mdui-icon-folder-zip--outlined>
                  </span>
                  <span class="open-tile-text">
                    <span class="md3-title-medium">{{ t('choose.archive') }}</span>
                    <span class="md3-caption">.zip · .pez</span>
                  </span>
                </button>
                <button class="open-tile" type="button" @click="chooseChart(true)">
                  <span class="md3-icon-badge open-tile-icon">
                    <mdui-icon-folder--outlined></mdui-icon-folder--outlined>
                  </span>
                  <span class="open-tile-text">
                    <span class="md3-title-medium">{{ t('choose.folder') }}</span>
                    <span class="md3-caption">{{ t('choose.folder-desc') }}</span>
                  </span>
                </button>
              </div>
            </div>

            <!-- Recent charts: the highest-value content on a returning launch. -->
            <div class="bento-cell cell-recent">
              <div class="cell-head">
                <h2 class="cell-title">{{ t('recent.title') }}</h2>
                <mdui-button-icon v-if="recentCharts.length" @click="clearRecent">
                  <mdui-tooltip :content="t('recent.clear')">
                    <mdui-icon-delete-sweep--outlined></mdui-icon-delete-sweep--outlined>
                  </mdui-tooltip>
                </mdui-button-icon>
              </div>

              <ul v-if="recentCharts.length" class="recent-list">
                <li v-for="item in recentCharts" :key="item.path">
                  <button class="recent-row" type="button" @click="loadChart(item.path)">
                    <span class="md3-icon-badge recent-icon">
                      <mdui-icon-description--outlined></mdui-icon-description--outlined>
                    </span>
                    <span class="recent-text">
                      <span class="recent-name">{{ item.name }}</span>
                      <span class="recent-meta">{{ [item.level, item.charter].filter(Boolean).join(' · ') || item.path }}</span>
                    </span>
                    <mdui-icon-chevron-right class="recent-chevron"></mdui-icon-chevron-right>
                  </button>
                </li>
              </ul>

              <div v-else class="recent-empty">
                <mdui-icon-history--outlined class="recent-empty-icon"></mdui-icon-history--outlined>
                <span class="md3-caption">{{ t('recent.empty') }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ==================================================================
         Steps 1..3 — plain pages.

         These used to be modal sheets stacked over the launcher. They are now
         ordinary full-area pages: the launcher is unmounted while a step is
         active, so there is no scrim, no depth sliver and no elevation to
         maintain — each step owns the whole content area.
         ================================================================== -->
    <Transition name="page">
      <section v-if="onStepPage" :key="step" class="step-page" :class="'is-' + step">
        <!-- ---------- Page app bar ---------- -->
        <header class="page-bar">
          <mdui-button-icon @click="goBack">
            <mdui-tooltip :content="t('prev-step')">
              <mdui-icon-arrow-back></mdui-icon-arrow-back>
            </mdui-tooltip>
          </mdui-button-icon>

          <div class="page-heading">
            <h2 class="page-title md3-title-large">{{ surfaceTitle }}</h2>
            <!-- MD3 has no stepper; the current step is a pill and the rest are dots. -->
            <div class="step-dots">
              <template v-for="(s, i) in steps" :key="s">
                <span v-if="step === s" class="step-pill"></span>
                <span v-else class="step-dot" :class="{ 'is-done': stepIndex > i + 1 }"></span>
              </template>
            </div>
          </div>

          <div class="page-actions">
            <template v-if="step === 'options'">
              <mdui-button variant="text" @click="playChart">
                <mdui-icon-sports-esports slot="icon"></mdui-icon-sports-esports>
                {{ t('play') }}
              </mdui-button>
              <mdui-button variant="text" @click="previewChart">
                <mdui-icon-visibility slot="icon"></mdui-icon-visibility>
                {{ t('preview') }}
              </mdui-button>
            </template>
            <mdui-button v-if="step === 'config'" variant="filled" @click="moveNext">
              {{ t('next-step') }}
              <mdui-icon-arrow-forward slot="end-icon"></mdui-icon-arrow-forward>
            </mdui-button>
            <mdui-button v-else-if="step === 'options'" variant="filled" @click="moveNext">
              {{ t('render') }}
              <mdui-icon-arrow-forward slot="end-icon"></mdui-icon-arrow-forward>
            </mdui-button>
          </div>
        </header>

        <mdui-divider></mdui-divider>

        <!-- ---------- Page body ---------- -->
        <div class="page-body">
          <!-- Step 1: configure the parsed chart -->
          <div v-if="step === 'config'" class="config-body" ref="flipCardRef">
            <form v-if="chartInfo" ref="form" class="config-form" @submit.prevent>
              <div class="config-grid">
                <div class="config-col">
                  <h3 class="col-title">{{ t('chart-file') }}</h3>
                  <MdTextField v-model="chartInfo.name" variant="outlined" required :label="t('chart-name') + ' *'" />
                  <MdTextField v-model="chartInfo.level" variant="outlined" required :label="t('level') + ' *'" />
                  <MdTextField v-model="chartInfo.charter" variant="outlined" required :label="t('charter') + ' *'" />
                  <MdTextField v-model="chartInfo.composer" variant="outlined" :label="t('composer')" />
                  <MdTextField v-model="chartInfo.illustrator" variant="outlined" :label="t('illustrator')" />
                </div>
                <div class="config-col">
                  <h3 class="col-title">{{ t('aspect') }}</h3>
                  <div class="field-group">
                    <span class="field-label">{{ t('aspect') }}</span>
                    <div class="aspect-row">
                      <!-- Deliberately not `type="number"`: an input with no usable `step`
                           reports a stepMismatch for ratios like 16/9 = 1.777…, which
                           `validateFields()` would then treat as a blocking error. -->
                      <MdTextField v-model="aspectWidth" variant="outlined" inputmode="decimal" class="aspect-field" />
                      <span class="aspect-sep">:</span>
                      <MdTextField v-model="aspectHeight" variant="outlined" inputmode="decimal" class="aspect-field" />
                    </div>
                  </div>
                  <div class="field-group">
                    <span class="field-label">{{ t('dim') }} — {{ Math.round(chartInfo.backgroundDim * 100) }}%</span>
                    <!-- Numeric bindings, not literals: Vue assigns these as DOM properties
                         (`'min' in el`), and Lit's number converter only runs attribute →
                         property, so `min="0"` would leave the string "0" on the element. -->
                    <MdSlider v-model="chartInfo.backgroundDim" :min="0" :max="1" :step="0.01" :labelFormatter="dimLabel" />
                  </div>
                  <div class="field-group field-toggle">
                    <span class="field-label">{{ t('hold_cover') }}</span>
                    <MdSwitch v-model="chartInfo.HoldPartialCover" />
                  </div>
                  <MdTextField v-model="chartInfo.tip" variant="outlined" :label="t('tip')" :placeholder="t('tip-placeholder')" />
                </div>
              </div>
            </form>
          </div>

          <!-- Step 2: render options -->
          <div v-else-if="step === 'options'" class="options-body">
            <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()" />
          </div>

          <!-- Step 3: render summary -->
          <div v-else-if="step === 'render' && chartInfo" class="render-body">
            <div class="render-hero">
              <div class="render-cover">
                <img v-if="renderCover" :src="convertFileSrc(renderCover)" class="cover-img" alt="" />
                <div v-else class="cover-placeholder">
                  <mdui-icon-music-note--outlined class="cover-icon"></mdui-icon-music-note--outlined>
                </div>
              </div>
              <div class="render-summary">
                <div class="render-heading">
                  <mdui-icon-videocam class="render-heading-icon"></mdui-icon-videocam>
                  <h3 class="md3-title-large">{{ t('render-started') }}</h3>
                </div>
                <p class="render-name">{{ chartInfo.name }}</p>
                <div class="render-chips">
                  <span class="meta-chip">{{ t('level') }} {{ chartInfo.level }}</span>
                  <span class="meta-chip">{{ aspectText(aspectWidth, aspectHeight) }}</span>
                  <span v-if="durationText" class="meta-chip">{{ t('duration') }} {{ durationText }}</span>
                </div>
              </div>
            </div>

            <div class="info-rows">
              <div class="info-row">
                <span class="info-label">{{ t('charter') }}</span>
                <span class="info-value">{{ chartInfo.charter }}</span>
              </div>
              <div v-if="chartInfo.composer" class="info-row">
                <span class="info-label">{{ t('composer') }}</span>
                <span class="info-value">{{ chartInfo.composer }}</span>
              </div>
              <div v-if="chartInfo.illustrator" class="info-row">
                <span class="info-label">{{ t('illustrator') }}</span>
                <span class="info-value">{{ chartInfo.illustrator }}</span>
              </div>
            </div>

            <!-- `<mdui-linear-progress>` runs 0–1, while `renderProgress` is a percentage. -->
            <div v-if="renderProgress !== undefined" class="render-progress">
              <mdui-linear-progress :value="renderProgress / 100"></mdui-linear-progress>
              <p class="progress-text md3-caption">{{ renderMsg }}</p>
            </div>

            <div class="render-actions">
              <mdui-button variant="tonal" @click="router.push({ name: 'tasks' })">
                <mdui-icon-view-list slot="icon"></mdui-icon-view-list>
                {{ t('see-tasks') }}
              </mdui-button>
              <mdui-button variant="filled" @click="resetAndGoChoose">
                <mdui-icon-add slot="icon"></mdui-icon-add>
                {{ t('next-chart') }}
              </mdui-button>
            </div>
          </div>
        </div>
      </section>
    </Transition>

    <!-- ==================================================================
         Transient layers — drag affordance, parse spinner, shortcut help.
         ================================================================== -->

    <!-- Deliberately not an `<mdui-dialog>`: this is a transient hover affordance,
         and a modal would trap focus and animate on every drag event. -->
    <Transition name="fade">
      <div v-if="fileHovering" class="overlay">
        <div class="overlay-card drop-zone">
          <mdui-icon-download class="drop-icon"></mdui-icon-download>
          <p class="drop-text">{{ t('choose-drop') }}</p>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="parsingChart" class="overlay">
        <div class="overlay-card parse-card">
          <mdui-circular-progress class="parse-spinner"></mdui-circular-progress>
          <span>{{ t('parsing') }}</span>
        </div>
      </div>
    </Transition>

    <!-- Shortcut reference. Its own floating layer, so it stacks above the wizard. -->
    <Transition name="fade">
      <div v-if="showShortcuts" class="overlay" @click.self="showShortcuts = false">
        <div class="overlay-card shortcut-card">
          <div class="shortcut-head">
            <h3 class="md3-title-large">{{ t('shortcuts.title') }}</h3>
            <mdui-button-icon @click="showShortcuts = false">
              <mdui-icon-close></mdui-icon-close>
            </mdui-button-icon>
          </div>
          <ul class="shortcut-list">
            <li>
              <span>{{ t('shortcuts.close') }}</span>
              <kbd>Esc</kbd>
            </li>
            <li>
              <span>{{ t('shortcuts.next') }}</span>
              <kbd>→</kbd>
            </li>
            <li>
              <span>{{ t('shortcuts.back') }}</span>
              <kbd>←</kbd>
            </li>
            <li>
              <span>{{ t('shortcuts.open') }}</span>
              <span class="key-group"><kbd>Ctrl</kbd><kbd>O</kbd></span>
            </li>
            <li>
              <span>{{ t('shortcuts.help') }}</span>
              <kbd>?</kbd>
            </li>
          </ul>
          <p class="md3-caption shortcut-hint">{{ t('shortcuts.hint') }}</p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped lang="scss">
@use './styles/breakpoints' as bp;
@use './styles/states' as st;
@use './styles/motion' as mo;

/* ============================================================
   RenderView — MD3 page architecture

   .start-layer   the launcher (mounted only while no step is active)
   .step-page     config / options / render, each a full-area page
   .overlay       transient drag / parse / help layers

   The three steps are peers, not stacked modal sheets: switching between them
   unmounts the launcher rather than dimming it, so there is no scrim and no
   elevation to maintain. Only the transient layers float.
   ============================================================ */

.render-container {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  overflow: hidden;
}

/* ============================================================
   Layer 0 — Start screen
   ============================================================ */
.start-layer {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.start-scroll {
  height: 100%;
  overflow-y: auto;
  scroll-behavior: smooth;
}

/* Bounded measure, matching the settings pane so switching between the
   launcher and ConfigView does not shift the content edges. */
.start-inner {
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding: 32px 32px 40px;
  max-width: 1200px;
  margin: 0 auto;
  min-height: 100%;
  width: 100%;
}

/* ---------- Hero ---------- */
.hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  flex-wrap: wrap;
}

.hero-text {
  min-width: 0;
}

.hero-title {
  margin: 0;
  font-size: var(--mdui-typescale-display-small-size);
  font-weight: var(--mdui-typescale-display-small-weight);
  line-height: var(--mdui-typescale-display-small-line-height);
  letter-spacing: -0.01em;
  color: rgb(var(--mdui-color-on-surface));
}

.hero-tagline {
  margin: 6px 0 0;
  max-width: 46ch;
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}

/* ---------- Bento grid ----------
   Two columns, equal height.

   `stretch` rather than `start`: a row whose two cards end at different heights
   reads as unfinished, and the reference layout keeps its blocks rectangular.
   The shorter card centres its own content instead of stretching its controls —
   `.cell-open` below — so equal height never means an oversized button. */
.bento {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr);
  align-items: stretch;
  gap: 20px;
  min-height: 0;
}

/* The open cell is the shorter of the two, so its stack centres in the row's
   height. Its tiles keep their intrinsic size — only the whitespace grows. */
.cell-open {
  justify-content: center;
}

.bento-cell {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-extra-large);
  min-width: 0;
  /* Colour is an effect and the corner radius is a shape, so they run on
     different MD3 clocks. */
  transition-property: background-color, border-radius;
  transition-duration: var(--app-motion-duration-effects-default), var(--app-motion-duration-spatial-slow);
  transition-timing-function: var(--app-motion-spring-effects-default), var(--app-motion-spring-spatial-slow);

  /* The two cells rise in sequence behind the hero. */
  @include mo.enter-rise(16px);
  @include mo.stagger(2, 60ms, 40ms);
}

.bento-cell:hover {
  background-color: rgb(var(--mdui-color-surface-container));
}

.cell-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.cell-title {
  margin: 0;
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-title-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
}

.cell-sub {
  margin: 0;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}

/* ---------- Open tiles ----------
   Plain <button>s rather than `<mdui-card clickable>`: these are the single most
   important controls on the screen, and a real button keeps the keyboard and
   screen-reader semantics (mdui's card only fakes the ripple).

   Deliberately *not* `flex: 1`: the tiles are sized to their content so the card
   hugs them. Letting them stretch made two short labels occupy ~700px of empty
   height, which read as a broken layout rather than as a large target.

   No `margin-top` either: the cell centres its whole stack, and an extra offset
   here would push the centred block off-centre. */
.open-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.open-tile {
  @include st.interactive;

  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  /* MD3 minimum touch target. */
  min-height: 48px;
  border: none;
  border-radius: var(--mdui-shape-corner-large);
  background-color: rgb(var(--mdui-color-surface-container-high));
  color: inherit;
  font: inherit;
  text-align: start;

  /* Per-property timing rather than one shared transition: `transform` carries
     the press/release bounce so it runs on the bouncy spring, while the hover
     lift's shadow settles on the calmer spatial one. A single transition would
     have forced both onto the same curve, and the bouncier of the two would win
     on `transform` alone — flattening the press feedback. */
  transform: scale(1);
  transition-property: transform, box-shadow;
  transition-duration: var(--app-motion-duration-bouncy), var(--app-motion-duration-spatial-default);
  transition-timing-function: var(--app-motion-spring-bouncy, var(--mdui-motion-easing-emphasized-decelerate)),
    var(--app-motion-spring-spatial-default, var(--mdui-motion-easing-emphasized-decelerate));
}

/* The lift is the tile's own hover feedback; the state layer adds the MD3
   overlay on top of it. */
.open-tile:hover {
  transform: translateY(-2px);
  box-shadow: var(--mdui-elevation-level2);
}

/* Pressing contracts quickly on the accelerate curve; releasing hands back to
   the bouncy spring declared above, which is what makes the tile pop. */
.open-tile:active {
  transform: scale(0.97);
  transition-duration: var(--mdui-motion-duration-short3), var(--app-motion-duration-effects-fast);
  transition-timing-function: var(--mdui-motion-easing-standard-accelerate), var(--mdui-motion-easing-standard);
}

.open-tile-icon {
  width: 2.75rem;
  height: 2.75rem;
  font-size: 1.375rem;
}

.open-tile-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

/* ---------- Recent charts ----------
   Capped so the card cannot grow without bound as history accumulates; beyond a
   handful of entries the list scrolls inside the card. */
.recent-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 340px;
  overflow-y: auto;
  min-height: 0;
}

.recent-row {
  @include st.interactive;
  @include mo.press-scale(0.98);

  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  /* MD3 two-line list item: 56dp tall with 16dp leading padding. Hand-rolled
     rather than `<mdui-list-item>` because that renders a non-focusable <div>,
     and these rows are the primary way back into a chart — a real <button>
     keeps them keyboard-operable. */
  min-height: 56px;
  padding: 8px 16px;
  border: none;
  border-radius: var(--mdui-shape-corner-large);
  background-color: transparent;
  color: inherit;
  font: inherit;
  text-align: start;
}

/* Recent charts rise in sequence, so the list reads as assembling itself rather
   than blinking into place. Delays stop at 8 entries; beyond that the rows enter
   together rather than trailing in. */
.recent-list li {
  @include mo.enter-rise(8px);
  @include mo.stagger(8, 45ms, 60ms);
}

.recent-icon {
  width: 2.25rem;
  height: 2.25rem;
  font-size: 1.1rem;
}

.recent-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.recent-name {
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-meta {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-chevron {
  flex-shrink: 0;
  color: rgb(var(--mdui-color-outline));
}

.recent-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  flex: 1;
  min-height: 140px;
  text-align: center;
}

.recent-empty-icon {
  font-size: 2.5rem;
  color: rgb(var(--mdui-color-outline));
}

/* ============================================================
   Step pages

   Plain full-area pages, not modal sheets: no scrim, no elevation, no depth
   sliver. The page owns the whole content area beside the app rail.
   ============================================================ */
.step-page {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background-color: rgb(var(--mdui-color-surface));
}

/* ---------- Page app bar ---------- */
.page-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  height: 64px;
  padding: 0 16px 0 8px;
  background-color: rgb(var(--mdui-color-surface));
}

.page-heading {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.page-title {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.page-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

/* mdui sizes icon buttons at 2.5rem (40dp), below MD3's 48dp minimum touch
   target, and its `overflow: hidden` blocks the pseudo-element trick. Growing
   the box and pulling it back with a negative margin raises the target without
   letting the bigger box reflow the bar. */
.page-bar mdui-button-icon {
  min-width: 48px;
  min-height: 48px;
  margin: -4px;
}

.step-dots {
  display: flex;
  align-items: center;
  gap: 6px;
}

.step-pill {
  width: 24px;
  height: 8px;
  border-radius: var(--mdui-shape-corner-full);
  background-color: rgb(var(--mdui-color-primary));
  transition: width var(--mdui-motion-duration-medium2) var(--mdui-motion-easing-emphasized);
}

.step-dot {
  width: 8px;
  height: 8px;
  border-radius: var(--mdui-shape-corner-full);
  background-color: rgb(var(--mdui-color-outline-variant));
  transition: background-color var(--mdui-motion-duration-short4) var(--mdui-motion-easing-standard);
}

.step-dot.is-done {
  background-color: rgb(var(--mdui-color-primary));
}

/* ---------- Page body ---------- */
.page-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ============================================================
   Config surface
   ============================================================ */
.config-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 32px;
}

.config-col {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.col-title {
  margin: 0 0 4px;
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: var(--mdui-typescale-title-small-weight);
  line-height: var(--mdui-typescale-title-small-line-height);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: rgb(var(--mdui-color-primary));
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: var(--mdui-typescale-label-large-size);
  font-weight: var(--mdui-typescale-label-large-weight);
  line-height: var(--mdui-typescale-label-large-line-height);
  letter-spacing: var(--mdui-typescale-label-large-tracking);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.aspect-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.aspect-field {
  flex: 1;
  min-width: 0;
}

.aspect-sep {
  font-size: var(--mdui-typescale-title-large-size);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.field-toggle {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

/* ============================================================
   Options surface — ConfigView fills it edge to edge
   ============================================================ */
.options-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* ============================================================
   Render surface
   ============================================================ */
.render-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 24px;
}

.render-hero {
  display: flex;
  gap: 20px;
  align-items: center;
}

.render-cover {
  width: 168px;
  height: 168px;
  flex-shrink: 0;
  border-radius: var(--mdui-shape-corner-extra-large);
  overflow: hidden;
  background-color: rgb(var(--mdui-color-surface-container-highest));
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.cover-icon {
  font-size: 2.5rem;
  color: rgb(var(--mdui-color-outline));
}

.render-summary {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.render-heading {
  display: flex;
  align-items: center;
  gap: 10px;
}

.render-heading-icon {
  font-size: 1.5rem;
  color: rgb(var(--mdui-color-primary));
}

.render-heading h3 {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface));
}

.render-name {
  margin: 0;
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow: hidden;
  text-overflow: ellipsis;
}

.render-chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* Static metadata pills — the same shape as a chip without the interaction. */
.meta-chip {
  padding: 4px 12px;
  border-radius: var(--mdui-shape-corner-small);
  border: 1px solid rgb(var(--mdui-color-outline-variant));
  font-size: var(--mdui-typescale-label-large-size);
  line-height: var(--mdui-typescale-label-large-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.info-rows {
  display: flex;
  flex-direction: column;
  background-color: rgb(var(--mdui-color-surface-container));
  border-radius: var(--mdui-shape-corner-large);
  padding: 4px 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(var(--mdui-color-outline-variant), 0.5);
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  color: rgb(var(--mdui-color-on-surface-variant));
}

.info-value {
  color: rgb(var(--mdui-color-on-surface));
  text-align: right;
  word-break: break-word;
}

.render-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-text {
  margin: 0;
  text-align: center;
}

.render-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: auto;
  padding-top: 8px;
}

/* ============================================================
   Layer 2 — Transient overlays
   ============================================================ */
.overlay {
  position: absolute;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(var(--mdui-color-scrim), 0.4);
}

.overlay-card {
  background-color: rgb(var(--mdui-color-surface-container-high));
  border-radius: var(--mdui-shape-corner-extra-large);
  box-shadow: var(--mdui-elevation-level3);
  color: rgb(var(--mdui-color-on-surface));
}

.drop-zone {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px;
  border: 2px dashed rgb(var(--mdui-color-primary));
}

.drop-icon {
  font-size: 3rem;
  color: rgb(var(--mdui-color-primary));
}

.drop-text {
  margin: 0;
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-title-medium-line-height);
}

.parse-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px 32px;
}

.parse-spinner {
  width: 2rem;
  height: 2rem;
}

/* ---------- Shortcut help ---------- */
.shortcut-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: min(420px, 90vw);
  padding: 20px;
  background-color: rgb(var(--mdui-color-surface-container-high));
  box-shadow: var(--mdui-elevation-level5);
}

.shortcut-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.shortcut-head h3 {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface));
}

.shortcut-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.shortcut-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 8px 0;
  color: rgb(var(--mdui-color-on-surface));
  font-size: var(--mdui-typescale-body-medium-size);
}

.key-group {
  display: inline-flex;
  gap: 4px;
}

.shortcut-list kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  padding: 3px 8px;
  border-radius: var(--mdui-shape-corner-small);
  background-color: rgb(var(--mdui-color-surface-container-highest));
  color: rgb(var(--mdui-color-on-surface-variant));
  font-family: 'Roboto Mono', Consolas, monospace;
  font-size: var(--mdui-typescale-label-medium-size);
}

.shortcut-hint {
  margin: 0;
}

/* ============================================================
   Motion

   Step pages are peers along the horizontal axis, so they slide (shared-axis X)
   on a spatial spring; the transient overlays are not spatially related to what
   they cover, so they use MD3 fade-through instead of a slide.
   ============================================================ */
.page-enter-active {
  @include mo.spatial((opacity, transform));
}

.page-leave-active {
  transition-property: opacity, transform;
  transition-duration: var(--mdui-motion-duration-short4);
  transition-timing-function: var(--mdui-motion-easing-emphasized-accelerate);
}

.page-enter-from {
  opacity: 0;
  transform: translateX(16px);
}

.page-leave-to {
  opacity: 0;
  transform: translateX(-16px);
}

/* MD3 fade-through: the outgoing card shrinks away, the incoming one fades in
   from slightly oversized. */
.fade {
  @include mo.fade-through(0.92);
}

/* The scrim itself only needs an opacity change — scaling it would reveal the
   edges of the surface behind. */
.fade-enter-active .overlay,
.fade-leave-active .overlay {
  transition: opacity var(--app-motion-duration-effects-fast) var(--app-motion-spring-effects-fast);
}

/* ============================================================
   Responsive — MD3 window size classes
   ============================================================ */

/* Compact (< 600px): single column. */
@include bp.compact {
  .start-inner {
    padding: 16px 16px 24px;
    gap: 16px;
  }
  .bento {
    grid-template-columns: 1fr;
  }
  .cell-open {
    grid-column: span 1;
  }
  .open-row {
    grid-template-columns: 1fr;
  }
  .hero {
    align-items: flex-start;
  }
  .hero-title {
    font-size: var(--mdui-typescale-headline-large-size);
    line-height: var(--mdui-typescale-headline-large-line-height);
  }
  .page-bar {
    height: 56px;
  }
  /* Icon-only primary actions; the icons carry the meaning in a cramped bar. */
  .page-actions mdui-button::part(label) {
    display: none;
  }
  .config-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }
  .render-hero {
    flex-direction: column;
    align-items: flex-start;
  }
  .render-cover {
    width: 100%;
    height: 140px;
  }
  .config-body,
  .render-body {
    padding: 16px;
  }
}

/* Medium (600–839px): two columns. */
@include bp.medium {
  .bento {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .cell-open {
    grid-column: span 2;
  }
}

/* Expanded (≥ 840px): roomier gutters. */
@include bp.expanded {
  .start-inner {
    padding: 36px 40px 48px;
    gap: 24px;
  }
  .page-bar {
    padding: 0 24px 0 12px;
  }
}
</style>
