<i18n>
en:
  title: Batch Render
  select-charts: Select Charts
  select-preset: Select Preset
  add-files: Add Files
  add-folder: Add Folder
  clear-list: Clear All
  clear-done: Clear Completed
  start-render: Start Render
  stop-render: Stop
  retry-failed: Retry Failed
  bulk-edit: Bulk Edit
  back: Back
  name: Name
  level: Level
  charter: Charter
  status: Status
  pending: Ready
  rendering: Rendering
  done: Done
  failed: Failed
  total-selected: "Total selected: {count}"
  all: All
  none: None
  configure: Global Config
  save: Save
  actions: Actions
  close: Close
  total-charts: "Total: {count}"
  select-all: Select All
  deselect-all: Deselect All
  progress: Progress
  eta: ETA
  no-charts: Drag and drop chart folders/files here, or use the buttons above to add them.
  no-results: No results found
  search-placeholder: Search charts, charter or level...
  selected: "Selected: {count}"
  filtered-results: "Filtered: {count}"
  add-files-failed: Failed to add files
  no-charts-found: No charts found in folder
  charts-added: "{count} charts added"
  add-folder-failed: Failed to add folder
  no-charts-selected: No charts selected
  batch-completed: "Batch completed: {count} rendered"
  batch-stopped: "Batch queue stopped"
  ffmpeg-not-found: FFmpeg not found
  chart-info-missing: Chart info missing
  adding-charts: Adding...
  invalid-chart-file: Invalid chart file
  file-type-error: "File type error: {message}"
  config-saved: "Configuration saved"
  chart-name: Chart Name
  composer: Composer
  illustrator: Illustrator
  background-dim: Background Dim
  hold_cover: Hold Partial Cover
  tip: Tip
  aspect: Aspect Ratio
  width: Width
  height: Height
  dim: Background Dim
  preview: Preview
  edit: Edit
  chart-info: Chart Info
  bulk-edit-title: Bulk Edit Settings
  bulk-edit-hint: Leave fields blank to keep their original values.
zh-CN:
  title: 批量渲染
  select-charts: 选择谱面
  select-preset: 选择预设
  add-files: 添加文件
  add-folder: 添加文件夹
  clear-list: 清空全部
  clear-done: 清除已完成
  start-render: 开始渲染
  stop-render: 停止渲染
  retry-failed: 重试失败项
  bulk-edit: 批量编辑
  back: 返回上一级
  name: 名称
  level: 难度
  charter: 谱师
  status: 状态
  pending: 已就绪
  rendering: 渲染中
  done: 已完成
  failed: 失败
  actions: 操作
  total-selected: "已选择: {count}"
  all: 全选
  none: 取消全选
  configure: 全局渲染配置
  close: 关闭
  total-charts: "总计: {count}"
  select-all: 全选
  deselect-all: 取消全选
  progress: 进度
  eta: 预计
  no-charts: 将谱面文件或文件夹拖拽至此，或使用上方按钮添加
  no-results: 未找到相关谱面
  search-placeholder: 搜索谱面名称、谱师或难度...
  selected: "已选择 {count} 项"
  filtered-results: "筛选结果: {count}"
  save: 保存
  add-files-failed: 添加文件失败
  no-charts-found: 未找到有效谱面文件
  charts-added: "成功添加 {count} 个谱面"
  add-folder-failed: 添加文件夹失败
  no-charts-selected: 未选择任何谱面
  batch-completed: "批量任务完成，共处理 {count} 个谱面"
  batch-stopped: "已停止队列"
  ffmpeg-not-found: 未找到 FFmpeg 环境
  chart-info-missing: 谱面信息缺失
  adding-charts: 解析中...
  invalid-chart-file: 无效的谱面文件
  file-type-error: "文件类型错误: {message}"
  config-saved: "配置已保存"
  chart-name: 谱面名称
  composer: 作曲家
  illustrator: 插画师
  background-dim: 背景暗度
  hold_cover: Hold 头部遮罩
  aspect: 宽高比
  tip: 提示信息 (Tip)
  width: 宽
  height: 高
  dim: 背景昏暗程度
  preview: 预览
  edit: 编辑
  chart-info: 谱面信息
  bulk-edit-title: 批量编辑属性
  bulk-edit-hint: 未填写的项将保持每个谱面原有的设置不变。
</i18n>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { event } from '@tauri-apps/api';

import { toast, toastError, RULES } from './common';
import type { ChartInfo, RenderConfig } from './model';
import ConfigView from '@/components/ConfigView.vue';

const { t } = useI18n();
const router = useRouter();

interface BatchChart {
  id: string; // Add unique ID for better list rendering
  path: string;
  name: string;
  level: string;
  charter: string;
  status: 'pending' | 'rendering' | 'done' | 'failed';
  selected: boolean;
  error?: string;
  chartInfo?: ChartInfo;
  aspectWidth?: string;
  aspectHeight?: string;
}

const charts = ref<BatchChart[]>([]);
const selectedPreset = ref<string>('default');
const presets = ref<{ name: string }[]>([]);
const configViewRef = ref<InstanceType<typeof ConfigView> | null>(null);
const configDialog = ref(false);
const editDialog = ref(false);
const bulkEditDialog = ref(false);
const editingChartIndex = ref(-1);
const form = ref<any>();
const bulkForm = ref<any>();

// 渲染队列控制
const currentRenderingIndex = ref<number>(-1);
const isRenderingQueue = ref(false);
const renderMsg = ref('');
const renderProgress = ref<number>(0);

// 批量编辑数据模型
const bulkEditData = ref({
  aspectWidth: '',
  aspectHeight: '',
  backgroundDim: null as number | null,
  holdCover: null as boolean | null,
});

// 添加状态
const isAddingFiles = ref(false);
const isAddingFolder = ref(false);
const searchQuery = ref('');

// 默认配置
const defaultConfig = ref<RenderConfig>(loadDefaultConfig());

// UUID Generator for safe list keys
function generateId() {
  return Math.random().toString(36).substring(2, 9);
}

// 预览单个谱面
async function previewChart(index: number) {
  const chart = charts.value[index];
  if (!chart.chartInfo) return toast(t('chart-info-missing'), 'error');
  try {
    const config = await buildRenderParams();
    await invoke('preview_chart', {
      params: { path: chart.path, info: chart.chartInfo, config: { ...config, autoplay: true } },
    });
  } catch (error) {
    toastError(error);
  }
}

function loadDefaultConfig(): RenderConfig {
  const savedConfig = localStorage.getItem('defaultRenderConfig');
  if (savedConfig) {
    try {
      return JSON.parse(savedConfig) as RenderConfig;
    } catch (e) {
      console.error('Failed to parse saved config', e);
    }
  }
  return {
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
    videoCodec: 'h264',
    encoder: 'auto',
    bitrateControl: 'CRF',
    bitrate: '28',
    targetAudio: 48000,
    video: false,
    audioBit: undefined,
    audioFormat: 'flac',
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
    handSplit: false,
    noteSpeedFactor: 1.0,
    ffmpegThread: false,
    showProgressText: false,
    showTimeText: false,
    uiLine: true,
    uiScore: true,
    uiCombo: true,
    uiLevel: true,
    uiName: true,
    uiPb: true,
    uiPause: true,
    bar: false,
  };
}

function saveDefaultConfig(config: RenderConfig) {
  defaultConfig.value = config;
  localStorage.setItem('defaultRenderConfig', JSON.stringify(config));
  toast(t('config-saved'), 'success');
}

async function getPresets() {
  try {
    const presetsMap = (await invoke('get_presets')) as Record<string, any>;
    presets.value = [{ name: 'default' }, ...Object.keys(presetsMap).map((name) => ({ name }))];
    selectedPreset.value = presets.value[0].name;
  } catch (error) {
    console.error('Failed to get presets', error);
  }
}

// 提取共用的添加逻辑
async function processNewPaths(paths: string[]) {
  const uniquePaths = [...new Set(paths)];
  const existingPaths = new Set(charts.value.map((c) => c.path));
  const newPaths = uniquePaths.filter((path) => !existingPaths.has(path));

  if (newPaths.length === 0) {
    toast(t('no-charts-found'), 'warning');
    return;
  }

  for (const path of newPaths) {
    await addChart(path);
  }
  toast(t('charts-added', { count: newPaths.length }), 'success');
}

async function addFiles() {
  if (isAddingFiles.value) return;
  isAddingFiles.value = true;
  try {
    const files = await open({
      multiple: true,
      filters: [{ name: t('chart-file'), extensions: ['zip', 'json', 'pez'] }],
    });
    if (!files) return;
    const paths = (Array.isArray(files) ? files : [files]).map((f) => (typeof f === 'string' ? f : (f as any).path));
    await processNewPaths(paths);
  } catch (error) {
    toast(t('add-files-failed'), 'error');
  } finally {
    isAddingFiles.value = false;
  }
}

async function addFolder() {
  if (isAddingFolder.value) return;
  isAddingFolder.value = true;
  try {
    const folder = await open({ directory: true });
    if (!folder) return;
    const folderPath = typeof folder === 'string' ? folder : (folder as any).path;
    const files = (await invoke('list_chart_files', { path: folderPath })) as string[];

    if (!files || files.length === 0) return toast(t('no-charts-found'), 'warning');

    const validExtensions = ['.json', '.zip', '.pez'];
    const filteredFiles = files.filter((f) => validExtensions.includes(f.toLowerCase().slice(f.lastIndexOf('.'))));
    await processNewPaths(filteredFiles);
  } catch (error) {
    toast(t('add-folder-failed'), 'error');
  } finally {
    isAddingFolder.value = false;
  }
}

async function addChart(path: string) {
  if (charts.value.some((c) => c.path === path)) return;

  const newChart: BatchChart = {
    id: generateId(),
    path,
    name: t('adding-charts'),
    level: '...',
    charter: '...',
    status: 'pending',
    selected: true,
  };
  const placeholderIndex = charts.value.push(newChart) - 1;

  try {
    const chartInfo = (await invoke('parse_chart', { path })) as ChartInfo;
    let aW = String(chartInfo.aspectRatio);
    let aH = '1.0';

    for (const asp of [[16, 9], [4, 3], [8, 5], [3, 2]]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.aspectRatio) < 1e-4) {
        aW = String(asp[0]); aH = String(asp[1]); break;
      }
    }

    charts.value[placeholderIndex] = {
      ...newChart,
      name: chartInfo.name,
      level: chartInfo.level,
      charter: chartInfo.charter,
      chartInfo,
      aspectWidth: aW,
      aspectHeight: aH,
    };
  } catch (error: any) {
    let errorMessage = t('invalid-chart-file');
    if (error.message?.includes('as zip archive')) errorMessage = t('file-type-error', { message: 'Not a ZIP archive' });
    else if (error.message?.includes('central directory')) errorMessage = t('file-type-error', { message: 'Invalid ZIP format' });

    charts.value[placeholderIndex] = {
      ...newChart,
      name: t('failed'),
      status: 'failed',
      selected: false,
      error: errorMessage,
    };
  }
}

// 列表操作快捷键
function clearList() {
  if (confirm(t('clear-list') + '?')) charts.value = [];
}
function clearDone() {
  charts.value = charts.value.filter(c => c.status !== 'done');
}
function retryFailed() {
  charts.value.forEach(c => {
    if (c.status === 'failed') c.status = 'pending';
  });
}
function removeChart(id: string) {
  charts.value = charts.value.filter(c => c.id !== id);
}

// 构建渲染参数
async function buildRenderParams() {
  if (!(await invoke('test_ffmpeg'))) throw new Error(t('ffmpeg-not-found'));
  let config = selectedPreset.value === 'default' ? defaultConfig.value : ((await invoke('get_presets')) as any)[selectedPreset.value];
  if (!config) config = defaultConfig.value;
  if (!config.resolution) throw new Error('Resolution missing');
  return config;
}

async function saveConfig() {
  const config = await configViewRef.value?.buildConfig();
  if (config) {
    saveDefaultConfig(config);
    configDialog.value = false;
  }
}

// 队列执行逻辑
async function startRender() {
  const pendingCount = filteredCharts.value.filter((c) => c.selected && c.status === 'pending').length;
  if (pendingCount === 0) return toast(t('no-charts-selected'), 'warning');

  try {
    const config = await buildRenderParams();
    isRenderingQueue.value = true;

    for (let i = 0; i < charts.value.length; i++) {
      if (!isRenderingQueue.value) break; // 检查是否中止

      const chart = charts.value[i];
      if (!chart.selected || chart.status !== 'pending') continue;

      currentRenderingIndex.value = i;
      chart.status = 'rendering';
      renderProgress.value = 0;

      try {
        if (!chart.chartInfo) throw new Error(t('chart-info-missing'));
        await invoke('post_render', { params: { path: chart.path, info: chart.chartInfo, config } });
        chart.status = 'done';
      } catch (error: any) {
        chart.status = 'failed';
        chart.error = error.message || String(error);
        toastError(error);
      }
    }

    if (isRenderingQueue.value) {
      toast(t('batch-completed', { count: pendingCount }), 'success');
    }
  } catch (error) {
    toastError(error);
  } finally {
    isRenderingQueue.value = false;
    currentRenderingIndex.value = -1;
    renderMsg.value = '';
    renderProgress.value = 0;
  }
}

function stopRender() {
  isRenderingQueue.value = false;
  toast(t('batch-stopped'), 'info');
}

// 筛选与选择逻辑
const filteredCharts = computed(() => {
  if (!searchQuery.value.trim()) return charts.value;
  const q = searchQuery.value.toLowerCase();
  return charts.value.filter(c =>
    c.name.toLowerCase().includes(q) ||
    c.level.toLowerCase().includes(q) ||
    c.charter.toLowerCase().includes(q)
  );
});

const selectedCount = computed(() => charts.value.filter(c => c.selected).length);
const allSelected = computed(() => charts.value.length > 0 && selectedCount.value === charts.value.length);
const isIndeterminate = computed(() => selectedCount.value > 0 && selectedCount.value < charts.value.length);

function toggleSelectAll() {
  const target = !allSelected.value;
  charts.value.forEach((chart) => {
    if (chart.status !== 'rendering') chart.selected = target;
  });
}

// Tauri Event Listeners
event.listen('render-msg', (msg) => {
  renderMsg.value = msg.payload as string;
});
event.listen('render-progress', (msg) => {
  const p = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = `FPS: ${p.fps} | ${t('eta')}: ${p.estimate}s`;
  renderProgress.value = p.progress * 100;
});

// 编辑功能
function openEditDialog(index: number) {
  editingChartIndex.value = index;
  editDialog.value = true;
}

async function saveChartInfo() {
  if (!form.value || editingChartIndex.value === -1) return;
  const { valid } = await form.value.validate();
  if (!valid) return;

  const chart = charts.value[editingChartIndex.value];
  const aspect = tryParseAspect(chart.aspectWidth, chart.aspectHeight);
  if (aspect && chart.chartInfo) chart.chartInfo.aspectRatio = aspect;
  if (chart.chartInfo) {
    chart.name = chart.chartInfo.name;
    chart.level = chart.chartInfo.level;
    chart.charter = chart.chartInfo.charter;
  }
  editDialog.value = false;
}

// 批量编辑功能
function openBulkEditDialog() {
  bulkEditData.value = { aspectWidth: '', aspectHeight: '', backgroundDim: null, holdCover: null };
  bulkEditDialog.value = true;
}

function saveBulkEdit() {
  const aspect = tryParseAspect(bulkEditData.value.aspectWidth, bulkEditData.value.aspectHeight);
  const selectedCharts = charts.value.filter(c => c.selected && c.chartInfo);

  selectedCharts.forEach(chart => {
    if (!chart.chartInfo) return;
    if (aspect) {
      chart.aspectWidth = bulkEditData.value.aspectWidth;
      chart.aspectHeight = bulkEditData.value.aspectHeight;
      chart.chartInfo.aspectRatio = aspect;
    }
    if (bulkEditData.value.backgroundDim !== null) {
      chart.chartInfo.backgroundDim = bulkEditData.value.backgroundDim;
    }
    if (bulkEditData.value.holdCover !== null) {
      chart.chartInfo.HoldPartialCover = bulkEditData.value.holdCover;
    }
  });

  bulkEditDialog.value = false;
  toast(t('config-saved'), 'success');
}

function tryParseAspect(w?: string, h?: string) {
  if (!w || !h) return undefined;
  const numW = parseFloat(w), numH = parseFloat(h);
  return (isNaN(numW) || isNaN(numH)) ? undefined : numW / numH;
}

// 本地存储
const STORAGE_KEY = 'batch_render_charts_v2';
onMounted(() => {
  getPresets();
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      charts.value = JSON.parse(saved).map((c: any) => ({...c, status: c.status === 'rendering' ? 'failed' : c.status}));
    }
    catch(e) {}
  }
});

watch(charts, (val) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
}, { deep: true });
</script>

<template>
  <div class="batch-layout">
    <div class="header-glass mb-4">
      <div class="d-flex align-center justify-space-between mb-4">
        <v-btn @click="router.go(-1)" prepend-icon="mdi-arrow-left" variant="text" class="text-none">
          {{ t('back') }}
        </v-btn>
        <h2 class="text-h6 font-weight-bold letter-spacing-1"></h2>
        <v-btn color="secondary" variant="tonal" prepend-icon="mdi-cog" @click="configDialog = true">
          {{ t('configure') }}
        </v-btn>
      </div>

      <div class="d-flex align-center flex-wrap gap-4">
        <v-text-field
          v-model="searchQuery"
          :placeholder="t('search-placeholder')"
          prepend-inner-icon="mdi-magnify"
          variant="solo-filled"
          density="comfortable"
          hide-details
          flat
          class="search-bar"
        ></v-text-field>

        <v-divider vertical class="mx-1 d-none d-sm-block"></v-divider>

        <v-btn color="primary" variant="tonal" prepend-icon="mdi-file-plus" :loading="isAddingFiles" @click="addFiles">
          {{ t('add-files') }}
        </v-btn>
        <v-btn color="primary" variant="tonal" prepend-icon="mdi-folder-plus" :loading="isAddingFolder" @click="addFolder">
          {{ t('add-folder') }}
        </v-btn>

        <v-spacer></v-spacer>

        <template v-if="isRenderingQueue">
          <v-chip color="warning" variant="flat" class="font-weight-bold px-4 pulse-anim">
            <v-icon start icon="mdi-loading" class="spin-anim"></v-icon>
            {{ t('rendering') }}
          </v-chip>
          <v-btn color="error" variant="flat" prepend-icon="mdi-stop" @click="stopRender">
            {{ t('stop-render') }}
          </v-btn>
        </template>
        <template v-else>
          <v-btn
            color="success"
            variant="flat"
            prepend-icon="mdi-play"
            :disabled="selectedCount === 0"
            @click="startRender"
            class="glow-button"
          >
            {{ t('start-render') }} ({{ selectedCount }})
          </v-btn>
        </template>
      </div>
    </div>

    <div class="list-controls d-flex align-center justify-space-between px-2 mb-2">
      <div class="d-flex align-center gap-3">
        <v-checkbox-btn
          :model-value="allSelected"
          :indeterminate="isIndeterminate"
          @click="toggleSelectAll"
          color="primary"
        ></v-checkbox-btn>
        <span class="text-body-2 text-medium-emphasis">
          {{ t('selected', { count: selectedCount }) }} / {{ t('total-charts', { count: charts.length }) }}
        </span>

        <v-slide-x-transition>
          <div v-if="selectedCount > 1" class="d-flex gap-2 ml-4">
            <v-btn size="small" variant="tonal" color="info" prepend-icon="mdi-pencil-box-multiple" @click="openBulkEditDialog">
              {{ t('bulk-edit') }}
            </v-btn>
          </div>
        </v-slide-x-transition>
      </div>

      <div class="d-flex gap-2">
        <v-btn size="small" variant="text" color="warning" @click="retryFailed" v-if="charts.some(c => c.status === 'failed')">
          <v-icon start>mdi-refresh</v-icon>{{ t('retry-failed') }}
        </v-btn>
        <v-btn size="small" variant="text" color="success" @click="clearDone" v-if="charts.some(c => c.status === 'done')">
          <v-icon start>mdi-check-all</v-icon>{{ t('clear-done') }}
        </v-btn>
        <v-btn size="small" variant="text" color="error" @click="clearList" :disabled="charts.length === 0">
          <v-icon start>mdi-trash-can-outline</v-icon>{{ t('clear-list') }}
        </v-btn>
      </div>
    </div>

    <div class="list-container flex-grow-1">
      <v-fade-transition mode="out-in">
        <div v-if="charts.length === 0" class="empty-state w-100 h-100 d-flex flex-column align-center justify-center">
          <v-icon size="80" color="primary" class="mb-4 opacity-50">mdi-text-box-plus-outline</v-icon>
          <div class="text-h6 text-medium-emphasis mb-2">{{ t('no-charts') }}</div>
        </div>

        <v-virtual-scroll v-else :items="filteredCharts" item-height="84" class="custom-scroll px-2 pb-2">
          <template v-slot:default="{ item, index }">
            <v-card
              class="chart-row-card mb-2"
              :class="{ 'is-rendering': item.status === 'rendering', 'is-failed': item.status === 'failed' }"
              elevation="0"
            >
              <div
                v-if="item.status === 'rendering'"
                class="progress-bg"
                :style="{ width: `${renderProgress}%` }"
              ></div>

              <div class="d-flex align-center pa-3 position-relative z-1">
                <div class="d-flex align-center mr-3">
                  <v-checkbox-btn
                    v-model="item.selected"
                    :disabled="item.status === 'rendering'"
                    color="primary"
                  ></v-checkbox-btn>
                </div>

                <div class="chart-info flex-grow-1 min-w-0">
                  <div class="d-flex align-center gap-2 mb-1">
                    <span class="text-subtitle-1 font-weight-bold text-truncate" :title="item.name">{{ item.name }}</span>
                    <v-chip size="x-small" variant="outlined" color="primary">{{ item.level }}</v-chip>
                  </div>
                  <div class="text-caption text-medium-emphasis d-flex align-center text-truncate">
                    <v-icon size="14" class="mr-1">mdi-account-music</v-icon> {{ item.charter }}
                    <template v-if="item.status === 'rendering'">
                      <v-divider vertical class="mx-2"></v-divider>
                      <span class="text-primary font-weight-medium">{{ renderProgress.toFixed(1) }}% - {{ renderMsg }}</span>
                    </template>
                    <template v-else-if="item.error">
                      <v-divider vertical class="mx-2"></v-divider>
                      <span class="text-error text-truncate" style="max-width: 300px" :title="item.error">{{ item.error }}</span>
                    </template>
                  </div>
                </div>

                <div class="status-zone mx-4 min-w-[80px] text-center">
                  <v-chip
                    :color="item.status === 'done' ? 'success' : item.status === 'failed' ? 'error' : item.status === 'rendering' ? 'warning' : 'default'"
                    :variant="item.status === 'pending' ? 'tonal' : 'flat'"
                    size="small"
                    class="text-uppercase font-weight-bold"
                  >
                    {{ t(item.status) }}
                  </v-chip>
                </div>

                <div class="action-zone d-flex gap-1">
                  <v-btn icon="mdi-pencil" size="small" variant="text" color="info" :disabled="!item.chartInfo || item.status === 'rendering'" @click="openEditDialog(charts.indexOf(item))" :title="t('edit')"></v-btn>
                  <v-btn icon="mdi-play-circle-outline" size="small" variant="text" color="success" :disabled="!item.chartInfo || item.status === 'rendering'" @click="previewChart(charts.indexOf(item))" :title="t('preview')"></v-btn>
                  <v-btn icon="mdi-close" size="small" variant="text" color="error" :disabled="item.status === 'rendering'" @click="removeChart(item.id)" :title="t('close')"></v-btn>
                </div>
              </div>
            </v-card>
          </template>
        </v-virtual-scroll>
      </v-fade-transition>
    </div>

    <v-dialog v-model="configDialog" fullscreen transition="dialog-bottom-transition">
      <v-card class="glass-dialog h-100">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title class="font-weight-bold">{{ t('configure') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon="mdi-close" variant="text" @click="configDialog = false"></v-btn>
        </v-toolbar>
        <v-card-text class="pa-0 h-100">
          <ConfigView ref="configViewRef" />
        </v-card-text>
        <v-card-actions class="pa-4 bg-surface-light border-t">
          <v-spacer></v-spacer>
          <v-btn variant="plain" @click="configDialog = false">{{ t('close') }}</v-btn>
          <v-btn color="primary" variant="flat" class="px-6" @click="saveConfig">{{ t('save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="editDialog" max-width="600" transition="dialog-bottom-transition">
      <v-card rounded="xl" class="glass-dialog">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title class="font-weight-bold">{{ t('chart-info') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon="mdi-close" variant="text" @click="editDialog = false"></v-btn>
        </v-toolbar>
        <v-card-text class="pa-6">
          <v-form ref="form" v-if="editingChartIndex >= 0 && charts[editingChartIndex]?.chartInfo">
            <v-row dense>
              <v-col cols="8"><v-text-field :label="t('chart-name')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.name" variant="outlined" density="comfortable"></v-text-field></v-col>
              <v-col cols="4"><v-text-field :label="t('level')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.level" variant="outlined" density="comfortable"></v-text-field></v-col>
              <v-col cols="4"><v-text-field :label="t('charter')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.charter" variant="outlined" density="comfortable"></v-text-field></v-col>
              <v-col cols="4"><v-text-field :label="t('composer')" v-model="charts[editingChartIndex].chartInfo!.composer" variant="outlined" density="comfortable"></v-text-field></v-col>
              <v-col cols="4"><v-text-field :label="t('illustrator')" v-model="charts[editingChartIndex].chartInfo!.illustrator" variant="outlined" density="comfortable"></v-text-field></v-col>
            </v-row>
            <v-divider class="my-4"></v-divider>
            <v-row dense align="center">
              <v-col cols="12" sm="5">
                <div class="text-caption mb-1">{{ t('aspect') }}</div>
                <div class="d-flex align-center gap-2">
                  <v-text-field type="number" :rules="[RULES.positive]" v-model="charts[editingChartIndex].aspectWidth" hide-details variant="outlined" density="compact"></v-text-field>
                  <span>:</span>
                  <v-text-field type="number" :rules="[RULES.positive]" v-model="charts[editingChartIndex].aspectHeight" hide-details variant="outlined" density="compact"></v-text-field>
                </div>
              </v-col>
              <v-col cols="12" sm="7">
                <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.01" v-model="charts[editingChartIndex].chartInfo!.backgroundDim" hide-details class="mt-4"></v-slider>
              </v-col>
              <v-col cols="12">
                <v-switch color="primary" :label="t('hold_cover')" v-model="charts[editingChartIndex].chartInfo!.HoldPartialCover" :true-value="1" :false-value="0" hide-details></v-switch>
              </v-col>
              <v-col cols="12">
                <v-text-field :label="t('tip')" v-model="charts[editingChartIndex].chartInfo!.tip" variant="outlined" density="comfortable" hide-details class="mt-2"></v-text-field>
              </v-col>
            </v-row>
          </v-form>
        </v-card-text>
        <v-card-actions class="pa-4 bg-surface-light border-t">
          <v-spacer></v-spacer>
          <v-btn variant="plain" @click="editDialog = false">{{ t('close') }}</v-btn>
          <v-btn color="primary" variant="flat" class="px-6" @click="saveChartInfo">{{ t('save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="bulkEditDialog" max-width="500" transition="dialog-bottom-transition">
      <v-card rounded="xl" class="glass-dialog">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title class="font-weight-bold">{{ t('bulk-edit-title') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon="mdi-close" variant="text" @click="bulkEditDialog = false"></v-btn>
        </v-toolbar>
        <v-card-text class="pa-6">
          <v-alert color="info" variant="tonal" class="mb-4 text-body-2" icon="mdi-information-outline">
            {{ t('bulk-edit-hint') }}
          </v-alert>
          <v-form ref="bulkForm">
            <div class="text-subtitle-2 mb-2">{{ t('aspect') }}</div>
            <div class="d-flex align-center gap-2 mb-6">
              <v-text-field type="number" placeholder="Original" v-model="bulkEditData.aspectWidth" hide-details variant="outlined" density="comfortable"></v-text-field>
              <span>:</span>
              <v-text-field type="number" placeholder="Original" v-model="bulkEditData.aspectHeight" hide-details variant="outlined" density="comfortable"></v-text-field>
            </div>

            <div class="text-subtitle-2 mb-1">{{ t('dim') }}</div>
            <v-slider thumb-label :min="0" :max="1" :step="0.01" :model-value="bulkEditData.backgroundDim ?? 0" @update:model-value="bulkEditData.backgroundDim = $event" hide-details color="primary" class="mb-2">
              <template v-slot:append>
                <v-btn icon="mdi-close-circle" size="small" variant="text" color="error" v-if="bulkEditData.backgroundDim !== null" @click="bulkEditData.backgroundDim = null"></v-btn>
              </template>
            </v-slider>

            <div class="d-flex align-center justify-space-between mt-4">
              <span class="text-subtitle-2">{{ t('hold_cover') }}</span>
              <v-btn-toggle v-model="bulkEditData.holdCover" color="primary" variant="outlined" density="compact">
                <v-btn :value="null">Keep</v-btn>
                <v-btn :value="1">On</v-btn>
                <v-btn :value="0">Off</v-btn>
              </v-btn-toggle>
            </div>
          </v-form>
        </v-card-text>
        <v-card-actions class="pa-4 bg-surface-light border-t">
          <v-spacer></v-spacer>
          <v-btn variant="plain" @click="bulkEditDialog = false">{{ t('close') }}</v-btn>
          <v-btn color="primary" variant="flat" class="px-6" @click="saveBulkEdit">{{ t('save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
/* 全局布局 */
.batch-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: 16px;
  box-sizing: border-box;
  background: transparent;
}

/* 通用间距辅助类 */
.gap-1 { gap: 4px; }
.gap-2 { gap: 8px; }
.gap-3 { gap: 12px; }
.gap-4 { gap: 16px; }
.min-w-0 { min-width: 0; }

/* 玻璃态头部 */
.header-glass {
  background: rgba(30, 30, 30, 0.65);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 16px 20px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
}

.search-bar {
  max-width: 300px;
  flex-grow: 1;
}

/* 列表容器滚动定制 */
.list-container {
  position: relative;
  overflow: hidden;
  border-radius: 16px;
}
.custom-scroll {
  height: 100%;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(255,255,255,0.2) transparent;
}
.custom-scroll::-webkit-scrollbar {
  width: 6px;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.2);
  border-radius: 4px;
}

/* 单个列表项（核心视觉） */
.chart-row-card {
  background: rgba(40, 40, 40, 0.7) !important;
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px !important;
  transition: all 0.2s cubic-bezier(0.25, 0.8, 0.25, 1);
  position: relative;
  overflow: hidden;
}

.chart-row-card:hover {
  background: rgba(55, 55, 55, 0.9) !important;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2) !important;
  border-color: rgba(255, 255, 255, 0.1);
}

/* 进度条背景效果 */
.progress-bg {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(90deg, rgba(var(--v-theme-primary), 0.15) 0%, rgba(var(--v-theme-primary), 0.3) 100%);
  z-index: 0;
  transition: width 0.3s linear;
}

.z-1 { z-index: 1; }

.is-failed {
  border-left: 4px solid rgb(var(--v-theme-error)) !important;
}

.is-rendering {
  border: 1px solid rgba(var(--v-theme-warning), 0.4) !important;
}

/* 动画效果 */
.spin-anim {
  animation: spin 1.5s linear infinite;
}
@keyframes spin { 100% { transform: rotate(360deg); } }

.pulse-anim {
  animation: pulse 2s infinite;
}
@keyframes pulse {
  0% { box-shadow: 0 0 0 0 rgba(var(--v-theme-warning), 0.4); }
  70% { box-shadow: 0 0 0 10px rgba(var(--v-theme-warning), 0); }
  100% { box-shadow: 0 0 0 0 rgba(var(--v-theme-warning), 0); }
}

/* 弹窗毛玻璃 */
.glass-dialog {
  background: rgba(24, 24, 24, 0.85) !important;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.border-b { border-bottom: 1px solid rgba(255,255,255,0.08) !important; }
.border-t { border-top: 1px solid rgba(255,255,255,0.08) !important; }

.glow-button {
  box-shadow: 0 0 15px rgba(var(--v-theme-success), 0.3);
  transition: box-shadow 0.3s;
}
.glow-button:hover:not(:disabled) {
  box-shadow: 0 0 25px rgba(var(--v-theme-success), 0.6);
}
</style>