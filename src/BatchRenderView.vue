<i18n>
en:
  title: Batch Render
  chart-file: Chart file
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
  hold-keep: Keep
  hold-on: On
  hold-off: Off
zh-CN:
  title: 批量渲染
  chart-file: 谱面文件
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
  hold-keep: 保持
  hold-on: 开启
  hold-off: 关闭
</i18n>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import moment from 'moment';

import { toast, toastError, validateFields } from './common';
import type { ChartInfo, RenderConfig } from './model';
import ConfigView from '@/components/ConfigView.vue';
import MdCheckbox from '@/components/md/MdCheckbox.vue';
import MdSlider from '@/components/md/MdSlider.vue';
import MdSwitch from '@/components/md/MdSwitch.vue';
import MdTextField from '@/components/md/MdTextField.vue';

const { t } = useI18n();

interface BatchChart {
  id: string;
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
const editingChartId = ref<string | null>(null);
const editingChart = computed(() => (editingChartId.value ? charts.value.find((c) => c.id === editingChartId.value) : undefined));
const editForm = ref<HTMLFormElement>();

const currentRenderingId = ref<string | null>(null);
const isRenderingQueue = ref(false);
const renderMsg = ref('');
const renderProgress = ref<number>(0);

const bulkEditData = ref({
  aspectWidth: '',
  aspectHeight: '',
  backgroundDim: null as number | null,
  holdCover: null as boolean | null,
});

const isAddingFiles = ref(false);
const isAddingFolder = ref(false);
const searchQuery = ref('');

const defaultConfig = ref<RenderConfig>(loadDefaultConfig());

const holdCoverValue = computed({
  get: () => bulkEditData.value.holdCover === null ? 'keep' : bulkEditData.value.holdCover ? 'on' : 'off',
  set: (v: string) => {
    if (v === 'keep') bulkEditData.value.holdCover = null;
    else bulkEditData.value.holdCover = v === 'on';
  },
});

function generateId() {
  return Math.random().toString(36).substring(2, 9);
}

async function previewChart(id: string) {
  const chart = charts.value.find((c) => c.id === id);
  if (!chart?.chartInfo) return toast(t('chart-info-missing'), 'error');
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
  const saved = localStorage.getItem('defaultRenderConfig');
  if (saved) {
    try { return JSON.parse(saved) as RenderConfig; } catch { /* empty */ }
  }
  return {
    resolution: [1920, 1080], ffmpegPreset: 'medium', endingLength: -2.0,
    disableLoading: true, chartDebug: false, flidX: false, chartRatio: 1,
    bufferSize: 256, fps: 60, hardwareAccel: true, videoCodec: 'h264',
    encoder: 'auto', bitrateControl: 'CRF', bitrate: '28', targetAudio: 48000,
    video: false, audioBit: undefined, audioFormat: 'flac', background: false,
    aggressive: false, challengeColor: 'golden', challengeRank: 45,
    disableEffect: false, doubleHint: true, fxaa: false, noteScale: 1,
    particle: true, playerAvatar: null, playerName: '', playerRks: 15,
    sampleCount: 1, resPackPath: null, speed: 1, volumeMusic: 1, volumeSfx: 1,
    combo: 'AUTOPLAY', watermark: '', handSplit: false, noteSpeedFactor: 1.0,
    ffmpegThread: false, showProgressText: false, showTimeText: false,
    uiLine: true, uiScore: true, uiCombo: true, uiLevel: true, uiName: true,
    uiPb: true, uiPause: true, bar: false,
  };
}

function saveDefaultConfig(config: RenderConfig) {
  defaultConfig.value = config;
  localStorage.setItem('defaultRenderConfig', JSON.stringify(config));
  toast(t('config-saved'), 'success');
}

async function getPresets() {
  try {
    const m = (await invoke('get_presets')) as Record<string, any>;
    presets.value = [{ name: 'default' }, ...Object.keys(m).map((name) => ({ name }))];
    selectedPreset.value = presets.value[0].name;
  } catch { /* empty */ }
}

async function processNewPaths(paths: string[]) {
  const unique = [...new Set(paths)];
  const existing = new Set(charts.value.map((c) => c.path));
  const fresh = unique.filter((p) => !existing.has(p));
  if (!fresh.length) { toast(t('no-charts-found'), 'warning'); return; }
  for (const p of fresh) await addChart(p);
  toast(t('charts-added', { count: fresh.length }), 'success');
}

async function addFiles() {
  if (isAddingFiles.value) return;
  isAddingFiles.value = true;
  try {
    const files = await open({ multiple: true, filters: [{ name: t('chart-file'), extensions: ['zip', 'json', 'pez'] }] });
    if (!files) return;
    const paths = (Array.isArray(files) ? files : [files]).map((f) => typeof f === 'string' ? f : (f as any).path);
    await processNewPaths(paths);
  } catch { toast(t('add-files-failed'), 'error'); }
  finally { isAddingFiles.value = false; }
}

async function addFolder() {
  if (isAddingFolder.value) return;
  isAddingFolder.value = true;
  try {
    const folder = await open({ directory: true });
    if (!folder) return;
    const folderPath = typeof folder === 'string' ? folder : (folder as any).path;
    const files = (await invoke('list_chart_files', { path: folderPath })) as string[];
    if (!files?.length) return toast(t('no-charts-found'), 'warning');
    const valid = files.filter((f) => ['.json', '.zip', '.pez'].includes(f.toLowerCase().slice(f.lastIndexOf('.'))));
    await processNewPaths(valid);
  } catch { toast(t('add-folder-failed'), 'error'); }
  finally { isAddingFolder.value = false; }
}

async function addChart(path: string) {
  if (charts.value.some((c) => c.path === path)) return;
  const placeholder: BatchChart = { id: generateId(), path, name: t('adding-charts'), level: '...', charter: '...', status: 'pending', selected: true };
  const idx = charts.value.push(placeholder) - 1;
  try {
    const info = (await invoke('parse_chart', { path })) as ChartInfo;
    let aW = String(info.aspectRatio), aH = '1.0';
    for (const asp of [[16, 9], [4, 3], [8, 5], [3, 2]]) {
      if (Math.abs(asp[0] / asp[1] - info.aspectRatio) < 1e-4) { aW = String(asp[0]); aH = String(asp[1]); break; }
    }
    charts.value[idx] = { ...placeholder, name: info.name, level: info.level, charter: info.charter, chartInfo: info, aspectWidth: aW, aspectHeight: aH };
  } catch (e: any) {
    let msg = t('invalid-chart-file');
    if (e.message?.includes('as zip archive')) msg = t('file-type-error', { message: 'Not a ZIP archive' });
    else if (e.message?.includes('central directory')) msg = t('file-type-error', { message: 'Invalid ZIP format' });
    charts.value[idx] = { ...placeholder, name: t('failed'), status: 'failed', selected: false, error: msg };
  }
}

function clearList() { if (confirm(t('clear-list') + '?')) charts.value = []; }
function clearDone() { charts.value = charts.value.filter((c) => c.status !== 'done'); }
function retryFailed() { charts.value.forEach((c) => { if (c.status === 'failed') c.status = 'pending'; }); }
function removeChart(id: string) { charts.value = charts.value.filter((c) => c.id !== id); }

async function buildRenderParams() {
  if (!(await invoke('test_ffmpeg'))) throw new Error(t('ffmpeg-not-found'));
  let cfg = selectedPreset.value === 'default' ? defaultConfig.value : ((await invoke('get_presets')) as any)[selectedPreset.value];
  if (!cfg) cfg = defaultConfig.value;
  if (!cfg.resolution) throw new Error('Resolution missing');
  return cfg;
}

async function saveConfig() {
  const cfg = await configViewRef.value?.buildConfig();
  if (cfg) { saveDefaultConfig(cfg); configDialog.value = false; }
}

async function startRender() {
  const pending = filteredCharts.value.filter((c) => c.selected && c.status === 'pending');
  if (!pending.length) return toast(t('no-charts-selected'), 'warning');
  try {
    const cfg = await buildRenderParams();
    isRenderingQueue.value = true;
    for (const chart of charts.value) {
      if (!isRenderingQueue.value) break;
      if (!chart.selected || chart.status !== 'pending') continue;
      currentRenderingId.value = chart.id;
      chart.status = 'rendering';
      renderProgress.value = 0;
      try {
        if (!chart.chartInfo) throw new Error(t('chart-info-missing'));
        await invoke('post_render', { params: { path: chart.path, info: chart.chartInfo, config: cfg } });
        chart.status = 'done';
      } catch (err: any) {
        chart.status = 'failed';
        chart.error = err.message || String(err);
        toastError(err);
      }
    }
    if (isRenderingQueue.value) toast(t('batch-completed', { count: pending.length }), 'success');
  } catch (err) { toastError(err); }
  finally {
    isRenderingQueue.value = false;
    currentRenderingId.value = null;
    renderMsg.value = '';
    renderProgress.value = 0;
  }
}

function stopRender() { isRenderingQueue.value = false; toast(t('batch-stopped'), 'info'); }

const filteredCharts = computed(() => {
  if (!searchQuery.value.trim()) return charts.value;
  const q = searchQuery.value.toLowerCase();
  return charts.value.filter((c) => c.name.toLowerCase().includes(q) || c.level.toLowerCase().includes(q) || c.charter.toLowerCase().includes(q));
});

const selectedCount = computed(() => charts.value.filter((c) => c.selected).length);
const allSelected = computed(() => charts.value.length > 0 && selectedCount.value === charts.value.length);

function toggleSelectAll() {
  const target = !allSelected.value;
  charts.value.forEach((c) => { if (c.status !== 'rendering') c.selected = target; });
}

let disposed = false;
const unlistenFns: UnlistenFn[] = [];
function track(p: Promise<UnlistenFn>) { p.then((fn) => disposed ? fn() : unlistenFns.push(fn)).catch((e) => console.error('Failed to register listener:', e)); }

onUnmounted(() => { disposed = true; unlistenFns.forEach((fn) => fn()); unlistenFns.length = 0; });

track(listen('render-msg', (msg) => { renderMsg.value = msg.payload as string; }));
track(listen('render-progress', (msg) => {
  const p = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = `FPS: ${p.fps} | ${t('eta')}: ${moment.duration(p.estimate, 'seconds').humanize(true, { ss: 1 })}`;
  renderProgress.value = p.progress * 100;
}));

function openEditDialog(id: string) { editingChartId.value = id; editDialog.value = true; }

async function saveChartInfo() {
  if (!editForm.value || !editingChartId.value) return;
  if (!validateFields(editForm.value)) return;
  const chart = charts.value.find((c) => c.id === editingChartId.value);
  if (!chart) return;
  const asp = tryParseAspect(chart.aspectWidth, chart.aspectHeight);
  if (asp && chart.chartInfo) chart.chartInfo.aspectRatio = asp;
  editDialog.value = false;
}

function openBulkEditDialog() {
  bulkEditData.value = { aspectWidth: '', aspectHeight: '', backgroundDim: null, holdCover: null };
  bulkEditDialog.value = true;
}

function saveBulkEdit() {
  const asp = tryParseAspect(bulkEditData.value.aspectWidth, bulkEditData.value.aspectHeight);
  charts.value.filter((c) => c.selected && c.chartInfo).forEach((chart) => {
    if (!chart.chartInfo) return;
    if (asp) { chart.aspectWidth = bulkEditData.value.aspectWidth; chart.aspectHeight = bulkEditData.value.aspectHeight; chart.chartInfo.aspectRatio = asp; }
    if (bulkEditData.value.backgroundDim !== null) chart.chartInfo.backgroundDim = bulkEditData.value.backgroundDim;
    if (typeof bulkEditData.value.holdCover === 'boolean') chart.chartInfo.HoldPartialCover = bulkEditData.value.holdCover;
  });
  bulkEditDialog.value = false;
  toast(t('config-saved'), 'success');
}

function tryParseAspect(w?: string, h?: string) {
  if (!w || !h) return undefined;
  const nW = parseFloat(w), nH = parseFloat(h);
  return isNaN(nW) || isNaN(nH) ? undefined : nW / nH;
}

const STORAGE_KEY = 'batch_render_charts_v2';
onMounted(() => {
  getPresets();
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try { charts.value = JSON.parse(saved).map((c: any) => ({ ...c, status: c.status === 'rendering' ? 'failed' : c.status })); } catch { /* empty */ }
  }
});

watch(charts, (val) => { localStorage.setItem(STORAGE_KEY, JSON.stringify(val)); }, { deep: true });

function onHoldCoverChange(e: Event) {
  const v = (e.target as HTMLElement & { value?: string }).value;
  if (v === 'keep') bulkEditData.value.holdCover = null;
  else bulkEditData.value.holdCover = v === 'on';
}
</script>

<template>
  <div class="batch-layout">
    <!-- Header -->
    <div class="batch-header">
      <div class="header-row">
        <div class="search-wrap">
          <mdui-icon-search class="search-icon"></mdui-icon-search>
          <input v-model="searchQuery" :placeholder="t('search-placeholder')" class="search-input" />
        </div>
        <div class="header-actions">
          <mdui-button variant="tonal" :disabled="isAddingFiles" @click="addFiles">
            <mdui-icon-note-add--outlined slot="icon"></mdui-icon-note-add--outlined>
            <span class="btn-label">{{ t('add-files') }}</span>
          </mdui-button>
          <mdui-button variant="tonal" :disabled="isAddingFolder" @click="addFolder">
            <mdui-icon-create-new-folder--outlined slot="icon"></mdui-icon-create-new-folder--outlined>
            <span class="btn-label">{{ t('add-folder') }}</span>
          </mdui-button>
          <mdui-button variant="tonal" @click="configDialog = true">
            <mdui-icon-settings--outlined slot="icon"></mdui-icon-settings--outlined>
            <span class="btn-label">{{ t('configure') }}</span>
          </mdui-button>
        </div>
      </div>
    </div>

    <!-- Selection bar -->
    <div class="selection-bar">
      <div class="sel-left">
        <MdCheckbox :model-value="allSelected" @update:model-value="toggleSelectAll" />
        <span class="sel-count">{{ t('selected', { count: selectedCount }) }} / {{ t('total-charts', { count: charts.length }) }}</span>
        <mdui-button v-if="selectedCount > 1" variant="text" @click="openBulkEditDialog">
          <mdui-icon-edit-note slot="icon"></mdui-icon-edit-note>
          {{ t('bulk-edit') }}
        </mdui-button>
      </div>
      <div class="sel-right">
        <mdui-button v-if="charts.some((c) => c.status === 'failed')" variant="text" @click="retryFailed">
          <mdui-icon-refresh slot="icon"></mdui-icon-refresh>
          {{ t('retry-failed') }}
        </mdui-button>
        <mdui-button v-if="charts.some((c) => c.status === 'done')" variant="text" @click="clearDone">
          <mdui-icon-done-all slot="icon"></mdui-icon-done-all>
          {{ t('clear-done') }}
        </mdui-button>
        <mdui-button variant="text" :disabled="!charts.length" @click="clearList">
          <mdui-icon-delete--outlined slot="icon"></mdui-icon-delete--outlined>
          {{ t('clear-list') }}
        </mdui-button>
      </div>
    </div>

    <!-- Render bar -->
    <div class="render-bar">
      <template v-if="isRenderingQueue">
        <mdui-chip variant="elevated" class="rendering-chip">
          <mdui-circular-progress slot="icon" class="spin-anim"></mdui-circular-progress>
          {{ t('rendering') }}
        </mdui-chip>
        <mdui-button variant="text" @click="stopRender" style="color: var(--mdui-color-error)">
          <mdui-icon-stop slot="icon"></mdui-icon-stop>
          {{ t('stop-render') }}
        </mdui-button>
      </template>
      <template v-else>
        <mdui-button variant="filled" :disabled="!selectedCount" @click="startRender">
          <mdui-icon-play-arrow slot="icon"></mdui-icon-play-arrow>
          {{ t('start-render') }} ({{ selectedCount }})
        </mdui-button>
      </template>
    </div>

    <!-- Chart list -->
    <div class="chart-list">
      <div v-if="!charts.length" class="empty-state">
        <mdui-icon-inbox--outlined class="empty-icon"></mdui-icon-inbox--outlined>
        <p class="md3-body">{{ t('no-charts') }}</p>
      </div>
      <div v-else-if="!filteredCharts.length" class="empty-state">
        <mdui-icon-search class="empty-icon"></mdui-icon-search>
        <p class="md3-body">{{ t('no-results') }}</p>
      </div>
      <div v-else class="list-scroll">
        <div v-for="item in filteredCharts" :key="item.id" class="chart-row" :class="{ 'is-rendering': item.status === 'rendering', 'is-failed': item.status === 'failed' }">
          <div v-if="item.status === 'rendering' && item.id === currentRenderingId" class="progress-bg" :style="{ width: `${renderProgress}%` }"></div>
          <div class="row-content">
            <MdCheckbox v-model="item.selected" :disabled="item.status === 'rendering'" />
            <div class="row-info">
              <div class="row-title">
                <span class="row-name" :title="item.name">{{ item.name }}</span>
                <mdui-chip variant="outlined" class="level-chip">{{ item.level }}</mdui-chip>
              </div>
              <div class="row-sub">
                <mdui-icon-person class="sub-icon"></mdui-icon-person>
                <span>{{ item.charter }}</span>
                <template v-if="item.status === 'rendering' && item.id === currentRenderingId">
                  <span class="render-status">{{ renderProgress.toFixed(1) }}% - {{ renderMsg }}</span>
                </template>
                <template v-else-if="item.error">
                  <span class="error-status" :title="item.error">{{ item.error }}</span>
                </template>
              </div>
            </div>
            <mdui-chip
              :variant="item.status === 'pending' ? 'outlined' : 'elevated'"
              class="status-chip"
              :class="`status-${item.status}`">
              {{ t(item.status) }}
            </mdui-chip>
            <div class="row-actions">
              <mdui-button-icon :disabled="!item.chartInfo || item.status === 'rendering'" @click="openEditDialog(item.id)" :title="t('edit')">
                <mdui-icon-edit--outlined></mdui-icon-edit--outlined>
              </mdui-button-icon>
              <mdui-button-icon :disabled="!item.chartInfo || item.status === 'rendering'" @click="previewChart(item.id)" :title="t('preview')">
                <mdui-icon-play-circle--outlined></mdui-icon-play-circle--outlined>
              </mdui-button-icon>
              <mdui-button-icon :disabled="item.status === 'rendering'" @click="removeChart(item.id)" :title="t('close')">
                <mdui-icon-close></mdui-icon-close>
              </mdui-button-icon>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Config Dialog -->
    <mdui-dialog :open="configDialog" @close="configDialog = false" class="config-dialog">
      <div class="dialog-toolbar">
        <mdui-button-icon @click="configDialog = false">
          <mdui-icon-close></mdui-icon-close>
        </mdui-button-icon>
        <span class="dialog-title">{{ t('configure') }}</span>
        <div class="toolbar-spacer"></div>
        <mdui-button variant="tonal" @click="saveConfig">
          <mdui-icon-save slot="icon"></mdui-icon-save>
          {{ t('save') }}
        </mdui-button>
      </div>
      <div class="config-dialog-body">
        <ConfigView ref="configViewRef" />
      </div>
    </mdui-dialog>

    <!-- Edit Dialog -->
    <mdui-dialog :open="editDialog" @close="editDialog = false" class="edit-dialog">
      <div class="dialog-toolbar">
        <mdui-button-icon @click="editDialog = false">
          <mdui-icon-close></mdui-icon-close>
        </mdui-button-icon>
        <span class="dialog-title">{{ t('chart-info') }}</span>
        <div class="toolbar-spacer"></div>
        <mdui-button variant="tonal" @click="saveChartInfo">
          <mdui-icon-save slot="icon"></mdui-icon-save>
          {{ t('save') }}
        </mdui-button>
      </div>
      <div class="dialog-body" v-if="editingChart?.chartInfo">
        <form ref="editForm" class="edit-form" @submit.prevent>
          <div class="form-grid">
            <MdTextField v-model="editingChart.chartInfo.name" variant="outlined" required :label="t('chart-name')" />
            <MdTextField v-model="editingChart.chartInfo.level" variant="outlined" required :label="t('level')" />
            <MdTextField v-model="editingChart.chartInfo.charter" variant="outlined" required :label="t('charter')" />
            <MdTextField v-model="editingChart.chartInfo.composer" variant="outlined" :label="t('composer')" />
            <MdTextField v-model="editingChart.chartInfo.illustrator" variant="outlined" :label="t('illustrator')" />
          </div>
          <mdui-divider></mdui-divider>
          <div class="form-grid">
            <div class="aspect-row">
              <MdTextField v-model="editingChart.aspectWidth" variant="outlined" inputmode="decimal" :label="t('width')" />
              <span class="aspect-sep">:</span>
              <MdTextField v-model="editingChart.aspectHeight" variant="outlined" inputmode="decimal" :label="t('height')" />
            </div>
            <div class="slider-block">
              <div class="slider-header">
                <span>{{ t('dim') }}</span>
                <span class="slider-val">{{ Math.round(editingChart.chartInfo.backgroundDim * 100) }}%</span>
              </div>
              <MdSlider v-model="editingChart.chartInfo.backgroundDim" :min="0" :max="1" :step="0.01" />
            </div>
            <div class="field-toggle">
              <span class="field-label">{{ t('hold_cover') }}</span>
              <MdSwitch v-model="editingChart.chartInfo.HoldPartialCover" />
            </div>
            <MdTextField v-model="editingChart.chartInfo.tip" variant="outlined" :label="t('tip')" />
          </div>
        </form>
      </div>
    </mdui-dialog>

    <!-- Bulk Edit Dialog -->
    <mdui-dialog :open="bulkEditDialog" @close="bulkEditDialog = false" class="bulk-edit-dialog">
      <div class="dialog-toolbar">
        <mdui-button-icon @click="bulkEditDialog = false">
          <mdui-icon-close></mdui-icon-close>
        </mdui-button-icon>
        <span class="dialog-title">{{ t('bulk-edit-title') }}</span>
        <div class="toolbar-spacer"></div>
        <mdui-button variant="tonal" @click="saveBulkEdit">
          <mdui-icon-save slot="icon"></mdui-icon-save>
          {{ t('save') }}
        </mdui-button>
      </div>
      <div class="dialog-body">
        <div class="bulk-hint">
          <mdui-icon-info--outlined class="hint-icon"></mdui-icon-info--outlined>
          <span>{{ t('bulk-edit-hint') }}</span>
        </div>
        <div class="form-grid">
          <div class="aspect-row">
            <MdTextField v-model="bulkEditData.aspectWidth" variant="outlined" inputmode="decimal" :label="t('width')" />
            <span class="aspect-sep">:</span>
            <MdTextField v-model="bulkEditData.aspectHeight" variant="outlined" inputmode="decimal" :label="t('height')" />
          </div>
          <div class="slider-block">
            <div class="slider-header">
              <span>{{ t('dim') }}</span>
              <span class="slider-val">{{ bulkEditData.backgroundDim !== null ? Math.round(bulkEditData.backgroundDim * 100) + '%' : '—' }}</span>
            </div>
            <MdSlider :model-value="bulkEditData.backgroundDim ?? 0" @update:model-value="(v: number) => bulkEditData.backgroundDim = v" :min="0" :max="1" :step="0.01" />
          </div>
          <div class="field-toggle">
            <span class="field-label">{{ t('hold_cover') }}</span>
            <mdui-segmented-button-group selects="single" :value="holdCoverValue" @change="onHoldCoverChange">
              <mdui-segmented-button value="keep">{{ t('hold-keep') }}</mdui-segmented-button>
              <mdui-segmented-button value="on">{{ t('hold-on') }}</mdui-segmented-button>
              <mdui-segmented-button value="off">{{ t('hold-off') }}</mdui-segmented-button>
            </mdui-segmented-button-group>
          </div>
        </div>
      </div>
    </mdui-dialog>
  </div>
</template>

<style scoped lang="scss">
@use './styles/breakpoints' as bp;
@use './styles/motion' as mo;

.batch-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  padding: 20px;
  box-sizing: border-box;
}

/* ===== Header ===== */
.batch-header {
  flex-shrink: 0;
  background-color: rgb(var(--mdui-color-surface-container));
  border-radius: var(--mdui-shape-corner-extra-large);
  padding: 14px 18px;
  margin-bottom: 12px;
}

.header-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.search-wrap {
  position: relative;
  flex: 1;
  max-width: 320px;
  min-width: 160px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: rgb(var(--mdui-color-on-surface-variant));
  pointer-events: none;
  font-size: 1.25rem;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 40px;
  background-color: rgb(var(--mdui-color-surface-container-high));
  border: 1px solid rgb(var(--mdui-color-outline-variant));
  border-radius: var(--mdui-shape-corner-full);
  color: rgb(var(--mdui-color-on-surface));
  font-size: var(--mdui-typescale-body-medium-size);
  font-family: inherit;
  transition: border-color var(--mdui-motion-duration-short4) var(--mdui-motion-easing-standard);
}
.search-input:focus {
  outline: none;
  border-color: rgb(var(--mdui-color-primary));
}
.search-input::placeholder {
  color: rgb(var(--mdui-color-on-surface-variant));
}

.header-actions {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

/* ===== Selection Bar ===== */
.selection-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  flex-shrink: 0;
}

.sel-left, .sel-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.sel-count {
  font-size: var(--mdui-typescale-body-small-size);
  color: rgb(var(--mdui-color-on-surface-variant));
}

/* ===== Render Bar ===== */
.render-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
  flex-shrink: 0;
}

.rendering-chip {
  animation: pulse-glow 2s infinite;
}
@keyframes pulse-glow {
  0% { box-shadow: 0 0 0 0 rgba(var(--mdui-color-tertiary), 0.4); }
  70% { box-shadow: 0 0 0 8px rgba(var(--mdui-color-tertiary), 0); }
  100% { box-shadow: 0 0 0 0 rgba(var(--mdui-color-tertiary), 0); }
}

.spin-anim {
  width: 1rem;
  height: 1rem;
  animation: spin 1.5s linear infinite;
}
@keyframes spin { 100% { transform: rotate(360deg); } }

/* ===== Chart List ===== */
.chart-list {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.list-scroll {
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 4px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
}

.empty-icon {
  font-size: 4rem;
  color: rgb(var(--mdui-color-outline));
}

/* ===== Chart Row ===== */
.chart-row {
  position: relative;
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-large);
  overflow: hidden;
  @include mo.spatial(box-shadow);

  @include mo.enter-rise(14px);
  @include mo.stagger(10, 45ms, 40ms);
}

.chart-row:hover {
  box-shadow: var(--mdui-elevation-level1);
}

.chart-row.is-failed {
  border-left: 3px solid rgb(var(--mdui-color-error));
}

.chart-row.is-rendering {
  border: 1px solid rgb(var(--mdui-color-tertiary));
}

.progress-bg {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(90deg, rgba(var(--mdui-color-tertiary-container), 0.3), rgba(var(--mdui-color-tertiary-container), 0.6));
  z-index: 0;
  /* A determinate progress fill reads as decelerating into its target rather
     than stopping dead, so it eases out instead of running linear. */
  transition: width var(--app-motion-duration-effects-slow) var(--mdui-motion-easing-standard-decelerate);
}

.row-content {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  padding: 12px 14px;
  gap: 12px;
}

.row-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.row-name {
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  color: rgb(var(--mdui-color-on-surface));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.level-chip {
  --shape-corner: var(--mdui-shape-corner-small);
  flex-shrink: 0;
}

.row-sub {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--mdui-typescale-body-small-size);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.sub-icon {
  font-size: 1rem;
}

.render-status {
  color: rgb(var(--mdui-color-primary));
  font-weight: 500;
}

.error-status {
  color: rgb(var(--mdui-color-error));
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-chip {
  flex-shrink: 0;
  --shape-corner: var(--mdui-shape-corner-full);
}

.status-pending { color: rgb(var(--mdui-color-tertiary)); }
.status-rendering { color: rgb(var(--mdui-color-primary)); }
.status-done { color: rgb(var(--mdui-color-success)); }
.status-failed { color: rgb(var(--mdui-color-error)); }

.row-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

/* ===== Dialogs ===== */
.config-dialog::part(panel) {
  width: 100vw;
  height: 100vh;
  max-width: none;
  max-height: none;
  border-radius: 0;
  display: flex;
  flex-direction: column;
}

.edit-dialog::part(panel),
.bulk-edit-dialog::part(panel) {
  max-width: min(600px, 90vw);
}

.dialog-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  flex-shrink: 0;
  border-bottom: 1px solid rgb(var(--mdui-color-outline-variant));
}

.dialog-title {
  font-size: var(--mdui-typescale-title-large-size);
  font-weight: var(--mdui-typescale-title-large-weight);
  color: rgb(var(--mdui-color-on-surface));
}

.toolbar-spacer {
  flex: 1;
}

.config-dialog-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.dialog-body {
  padding: 20px;
  overflow-y: auto;
}

/* ===== Form ===== */
.edit-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.aspect-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.aspect-sep {
  font-size: var(--mdui-typescale-title-large-size);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.slider-block {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.slider-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-size: var(--mdui-typescale-body-medium-size);
}

.slider-val {
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: var(--mdui-typescale-body-small-size);
  font-weight: var(--mdui-typescale-label-large-weight);
  color: rgb(var(--mdui-color-on-surface));
}

.field-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.field-label {
  font-size: var(--mdui-typescale-body-medium-size);
  color: rgb(var(--mdui-color-on-surface));
}

.bulk-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-radius: var(--mdui-shape-corner-medium);
  background-color: rgb(var(--mdui-color-secondary-container));
  color: rgb(var(--mdui-color-on-secondary-container));
  font-size: var(--mdui-typescale-body-small-size);
  margin-bottom: 16px;
}

.hint-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

/* ===== Responsive ===== */
@include bp.below-expanded {
  .batch-layout { padding: 12px; }
  .header-actions { flex-wrap: wrap; }
  .btn-label { display: none; }
}
</style>
