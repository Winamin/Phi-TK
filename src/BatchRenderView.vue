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
const editingChartIndex = ref(-1);
const form = ref<any>();
const bulkForm = ref<any>();

const currentRenderingIndex = ref<number>(-1);
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

function generateId() {
  return Math.random().toString(36).substring(2, 9);
}

async function previewChart(index: number) {
  const chart = charts.value[index];
  if (!chart.chartInfo) return toast(t('chart-info-missing'), 'error');
  try {
    const config = await buildRenderParams();
    await invoke('preview_chart', {
      params: { path: chart.path, info: chart.chartInfo, config: { ...config, autoplay: true } },
    });
  } catch (error) { toastError(error); }
}

function loadDefaultConfig(): RenderConfig {
  const savedConfig = localStorage.getItem('defaultRenderConfig');
  if (savedConfig) {
    try { return JSON.parse(savedConfig) as RenderConfig; } catch (e) { console.error('Failed to parse saved config', e); }
  }
  return {
    resolution: [1920, 1080], ffmpegPreset: 'medium p4 balanced', endingLength: -2.0, disableLoading: true,
    chartDebug: false, flidX: false, chartRatio: 1, bufferSize: 256, fps: 60, hardwareAccel: true,
    videoCodec: 'h264', encoder: 'auto', bitrateControl: 'CRF', bitrate: '28', targetAudio: 48000,
    video: false, audioBit: undefined, audioFormat: 'flac', background: false, aggressive: false,
    challengeColor: 'golden', challengeRank: 45, disableEffect: false, doubleHint: true, fxaa: false,
    noteScale: 1, particle: true, playerAvatar: null, playerName: '', playerRks: 15, sampleCount: 1,
    resPackPath: null, speed: 1, volumeMusic: 1, volumeSfx: 1, combo: 'AUTOPLAY', watermark: '',
    handSplit: false, noteSpeedFactor: 1.0, ffmpegThread: false, showProgressText: false,
    showTimeText: false, uiLine: true, uiScore: true, uiCombo: true, uiLevel: true, uiName: true,
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
    const presetsMap = (await invoke('get_presets')) as Record<string, any>;
    presets.value = [{ name: 'default' }, ...Object.keys(presetsMap).map((name) => ({ name }))];
    selectedPreset.value = presets.value[0].name;
  } catch (error) { console.error('Failed to get presets', error); }
}

async function processNewPaths(paths: string[]) {
  const uniquePaths = [...new Set(paths)];
  const existingPaths = new Set(charts.value.map((c) => c.path));
  const newPaths = uniquePaths.filter((path) => !existingPaths.has(path));
  if (newPaths.length === 0) { toast(t('no-charts-found'), 'warning'); return; }
  for (const path of newPaths) { await addChart(path); }
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
  } catch (error) { toast(t('add-files-failed'), 'error'); } finally { isAddingFiles.value = false; }
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
  } catch (error) { toast(t('add-folder-failed'), 'error'); } finally { isAddingFolder.value = false; }
}

async function addChart(path: string) {
  if (charts.value.some((c) => c.path === path)) return;
  const newChart: BatchChart = { id: generateId(), path, name: t('adding-charts'), level: '...', charter: '...', status: 'pending', selected: true };
  const placeholderIndex = charts.value.push(newChart) - 1;
  try {
    const chartInfo = (await invoke('parse_chart', { path })) as ChartInfo;
    let aW = String(chartInfo.aspectRatio);
    let aH = '1.0';
    for (const asp of [[16, 9], [4, 3], [8, 5], [3, 2]]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.aspectRatio) < 1e-4) { aW = String(asp[0]); aH = String(asp[1]); break; }
    }
    charts.value[placeholderIndex] = { ...newChart, name: chartInfo.name, level: chartInfo.level, charter: chartInfo.charter, chartInfo, aspectWidth: aW, aspectHeight: aH };
  } catch (error: any) {
    let errorMessage = t('invalid-chart-file');
    if (error.message?.includes('as zip archive')) errorMessage = t('file-type-error', { message: 'Not a ZIP archive' });
    else if (error.message?.includes('central directory')) errorMessage = t('file-type-error', { message: 'Invalid ZIP format' });
    charts.value[placeholderIndex] = { ...newChart, name: t('failed'), status: 'failed', selected: false, error: errorMessage };
  }
}

function clearList() { if (confirm(t('clear-list') + '?')) charts.value = []; }
function clearDone() { charts.value = charts.value.filter(c => c.status !== 'done'); }
function retryFailed() { charts.value.forEach(c => { if (c.status === 'failed') c.status = 'pending'; }); }
function removeChart(id: string) { charts.value = charts.value.filter(c => c.id !== id); }

async function buildRenderParams() {
  if (!(await invoke('test_ffmpeg'))) throw new Error(t('ffmpeg-not-found'));
  let config = selectedPreset.value === 'default' ? defaultConfig.value : ((await invoke('get_presets')) as any)[selectedPreset.value];
  if (!config) config = defaultConfig.value;
  if (!config.resolution) throw new Error('Resolution missing');
  return config;
}

async function saveConfig() {
  const config = await configViewRef.value?.buildConfig();
  if (config) { saveDefaultConfig(config); configDialog.value = false; }
}

async function startRender() {
  const pendingCount = filteredCharts.value.filter((c) => c.selected && c.status === 'pending').length;
  if (pendingCount === 0) return toast(t('no-charts-selected'), 'warning');
  try {
    const config = await buildRenderParams();
    isRenderingQueue.value = true;
    for (let i = 0; i < charts.value.length; i++) {
      if (!isRenderingQueue.value) break;
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
    if (isRenderingQueue.value) toast(t('batch-completed', { count: pendingCount }), 'success');
  } catch (error) { toastError(error); } finally {
    isRenderingQueue.value = false;
    currentRenderingIndex.value = -1;
    renderMsg.value = '';
    renderProgress.value = 0;
  }
}

function stopRender() { isRenderingQueue.value = false; toast(t('batch-stopped'), 'info'); }

const filteredCharts = computed(() => {
  if (!searchQuery.value.trim()) return charts.value;
  const q = searchQuery.value.toLowerCase();
  return charts.value.filter(c =>
    c.name.toLowerCase().includes(q) || c.level.toLowerCase().includes(q) || c.charter.toLowerCase().includes(q)
  );
});

const selectedCount = computed(() => charts.value.filter(c => c.selected).length);
const allSelected = computed(() => charts.value.length > 0 && selectedCount.value === charts.value.length);
const isIndeterminate = computed(() => selectedCount.value > 0 && selectedCount.value < charts.value.length);

function toggleSelectAll() {
  const target = !allSelected.value;
  charts.value.forEach((chart) => { if (chart.status !== 'rendering') chart.selected = target; });
}

event.listen('render-msg', (msg) => { renderMsg.value = msg.payload as string; });
event.listen('render-progress', (msg) => {
  const p = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = `FPS: ${p.fps} | ${t('eta')}: ${p.estimate}s`;
  renderProgress.value = p.progress * 100;
});

function openEditDialog(index: number) { editingChartIndex.value = index; editDialog.value = true; }

async function saveChartInfo() {
  if (!form.value || editingChartIndex.value === -1) return;
  const { valid } = await form.value.validate();
  if (!valid) return;
  const chart = charts.value[editingChartIndex.value];
  const aspect = tryParseAspect(chart.aspectWidth, chart.aspectHeight);
  if (aspect && chart.chartInfo) chart.chartInfo.aspectRatio = aspect;
  if (chart.chartInfo) { chart.name = chart.chartInfo.name; chart.level = chart.chartInfo.level; chart.charter = chart.chartInfo.charter; }
  editDialog.value = false;
}

function openBulkEditDialog() {
  bulkEditData.value = { aspectWidth: '', aspectHeight: '', backgroundDim: null, holdCover: null };
  bulkEditDialog.value = true;
}

function saveBulkEdit() {
  const aspect = tryParseAspect(bulkEditData.value.aspectWidth, bulkEditData.value.aspectHeight);
  const selectedCharts = charts.value.filter(c => c.selected && c.chartInfo);
  selectedCharts.forEach(chart => {
    if (!chart.chartInfo) return;
    if (aspect) { chart.aspectWidth = bulkEditData.value.aspectWidth; chart.aspectHeight = bulkEditData.value.aspectHeight; chart.chartInfo.aspectRatio = aspect; }
    if (bulkEditData.value.backgroundDim !== null) chart.chartInfo.backgroundDim = bulkEditData.value.backgroundDim;
    if (bulkEditData.value.holdCover !== null) chart.chartInfo.HoldPartialCover = bulkEditData.value.holdCover;
  });
  bulkEditDialog.value = false;
  toast(t('config-saved'), 'success');
}

function tryParseAspect(w?: string, h?: string) {
  if (!w || !h) return undefined;
  const numW = parseFloat(w), numH = parseFloat(h);
  return (isNaN(numW) || isNaN(numH)) ? undefined : numW / numH;
}

const STORAGE_KEY = 'batch_render_charts_v2';
onMounted(() => {
  getPresets();
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try { charts.value = JSON.parse(saved).map((c: any) => ({ ...c, status: c.status === 'rendering' ? 'failed' : c.status })); }
    catch (e) { }
  }
});

watch(charts, (val) => { localStorage.setItem(STORAGE_KEY, JSON.stringify(val)); }, { deep: true });
</script>

<template>
  <div class="batch-layout">
    <!-- Top Bar -->
    <div class="batch-header">
      <div class="header-row">
        <button class="md3-btn md3-btn-text" @click="router.go(-1)">
          <v-icon icon="mdi-arrow-left" size="20" />
          <span>{{ t('back') }}</span>
        </button>

        <div class="search-wrap">
          <v-icon icon="mdi-magnify" size="18" class="search-icon" />
          <input v-model="searchQuery" :placeholder="t('search-placeholder')" class="md3-search" />
        </div>

        <div class="header-actions">
          <button class="md3-btn md3-btn-tonal" :disabled="isAddingFiles" @click="addFiles">
            <v-icon icon="mdi-file-plus-outline" size="18" />
            <span>{{ t('add-files') }}</span>
          </button>
          <button class="md3-btn md3-btn-tonal" :disabled="isAddingFolder" @click="addFolder">
            <v-icon icon="mdi-folder-plus-outline" size="18" />
            <span>{{ t('add-folder') }}</span>
          </button>
          <button class="md3-btn md3-btn-tonal" @click="configDialog = true">
            <v-icon icon="mdi-cog-outline" size="18" />
            <span>{{ t('configure') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Selection bar -->
    <div class="selection-bar">
      <div class="sel-left">
        <v-checkbox-btn :model-value="allSelected" :indeterminate="isIndeterminate" @click="toggleSelectAll" color="primary" />
        <span class="sel-count">{{ t('selected', { count: selectedCount }) }} / {{ t('total-charts', { count: charts.length }) }}</span>

        <button v-if="selectedCount > 1" class="md3-btn md3-btn-text md3-btn-sm" @click="openBulkEditDialog">
          <v-icon icon="mdi-pencil-box-multiple-outline" size="16" />
          <span>{{ t('bulk-edit') }}</span>
        </button>
      </div>
      <div class="sel-right">
        <button v-if="charts.some(c => c.status === 'failed')" class="md3-btn md3-btn-text md3-btn-sm" @click="retryFailed">
          <v-icon icon="mdi-refresh" size="16" /><span>{{ t('retry-failed') }}</span>
        </button>
        <button v-if="charts.some(c => c.status === 'done')" class="md3-btn md3-btn-text md3-btn-sm" @click="clearDone">
          <v-icon icon="mdi-check-all" size="16" /><span>{{ t('clear-done') }}</span>
        </button>
        <button class="md3-btn md3-btn-text md3-btn-sm" @click="clearList" :disabled="charts.length === 0">
          <v-icon icon="mdi-delete-outline" size="16" /><span>{{ t('clear-list') }}</span>
        </button>
      </div>
    </div>

    <!-- Render action -->
    <div class="render-bar">
      <template v-if="isRenderingQueue">
        <v-chip color="warning" variant="flat" class="rendering-chip">
          <v-icon start icon="mdi-loading" class="spin-anim" />
          {{ t('rendering') }}
        </v-chip>
        <button class="md3-btn md3-btn-text" @click="stopRender" style="color: #ff5252;">
          <v-icon icon="mdi-stop" size="18" /><span>{{ t('stop-render') }}</span>
        </button>
      </template>
      <template v-else>
        <button class="md3-btn md3-btn-filled" :disabled="selectedCount === 0" @click="startRender">
          <v-icon icon="mdi-play" size="18" />
          <span>{{ t('start-render') }} ({{ selectedCount }})</span>
        </button>
      </template>
    </div>

    <!-- List -->
    <div class="chart-list">
      <div v-if="charts.length === 0" class="empty-state">
        <v-icon icon="mdi-text-box-plus-outline" size="64" color="rgba(255,255,255,0.15)" />
        <p>{{ t('no-charts') }}</p>
      </div>

      <div v-else class="list-scroll">
        <div
          v-for="item in filteredCharts"
          :key="item.id"
          class="chart-row"
          :class="{ 'is-rendering': item.status === 'rendering', 'is-failed': item.status === 'failed' }"
        >
          <div v-if="item.status === 'rendering'" class="progress-bg" :style="{ width: `${renderProgress}%` }"></div>

          <div class="row-content">
            <v-checkbox-btn v-model="item.selected" :disabled="item.status === 'rendering'" color="primary" />

            <div class="row-info">
              <div class="row-title">
                <span class="row-name" :title="item.name">{{ item.name }}</span>
                <v-chip size="x-small" variant="outlined" color="primary">{{ item.level }}</v-chip>
              </div>
              <div class="row-sub">
                <v-icon icon="mdi-account-music" size="14" />
                <span>{{ item.charter }}</span>
                <template v-if="item.status === 'rendering'">
                  <span class="render-status">{{ renderProgress.toFixed(1) }}% - {{ renderMsg }}</span>
                </template>
                <template v-else-if="item.error">
                  <span class="error-status" :title="item.error">{{ item.error }}</span>
                </template>
              </div>
            </div>

            <v-chip
              :color="item.status === 'done' ? 'success' : item.status === 'failed' ? 'error' : item.status === 'rendering' ? 'warning' : 'default'"
              :variant="item.status === 'pending' ? 'tonal' : 'flat'"
              size="small"
            >{{ t(item.status) }}</v-chip>

            <div class="row-actions">
              <button class="icon-btn" :disabled="!item.chartInfo || item.status === 'rendering'" @click="openEditDialog(charts.indexOf(item))" :title="t('edit')">
                <v-icon icon="mdi-pencil-outline" size="18" />
              </button>
              <button class="icon-btn" :disabled="!item.chartInfo || item.status === 'rendering'" @click="previewChart(charts.indexOf(item))" :title="t('preview')">
                <v-icon icon="mdi-play-circle-outline" size="18" />
              </button>
              <button class="icon-btn icon-btn-danger" :disabled="item.status === 'rendering'" @click="removeChart(item.id)" :title="t('close')">
                <v-icon icon="mdi-close" size="18" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Config Dialog -->
    <v-dialog v-model="configDialog" fullscreen transition="dialog-bottom-transition">
      <v-card class="md3-dialog-full h-100">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title>{{ t('configure') }}</v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-close" variant="text" @click="configDialog = false" />
        </v-toolbar>
        <v-card-text class="pa-0 h-100">
          <ConfigView ref="configViewRef" />
        </v-card-text>
        <v-card-actions class="pa-4 border-t">
          <v-spacer />
          <button class="md3-btn md3-btn-text" @click="configDialog = false">{{ t('close') }}</button>
          <button class="md3-btn md3-btn-filled" @click="saveConfig">{{ t('save') }}</button>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Edit Dialog -->
    <v-dialog v-model="editDialog" max-width="600">
      <v-card class="md3-dialog" v-if="editingChartIndex >= 0 && charts[editingChartIndex]?.chartInfo">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title>{{ t('chart-info') }}</v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-close" variant="text" @click="editDialog = false" />
        </v-toolbar>
        <v-card-text class="pa-6">
          <v-form ref="form" @submit.prevent>
            <v-row dense>
              <v-col cols="8"><v-text-field :label="t('chart-name')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.name" variant="outlined" density="comfortable" /></v-col>
              <v-col cols="4"><v-text-field :label="t('level')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.level" variant="outlined" density="comfortable" /></v-col>
              <v-col cols="4"><v-text-field :label="t('charter')" :rules="[RULES.non_empty]" v-model="charts[editingChartIndex].chartInfo!.charter" variant="outlined" density="comfortable" /></v-col>
              <v-col cols="4"><v-text-field :label="t('composer')" v-model="charts[editingChartIndex].chartInfo!.composer" variant="outlined" density="comfortable" /></v-col>
              <v-col cols="4"><v-text-field :label="t('illustrator')" v-model="charts[editingChartIndex].chartInfo!.illustrator" variant="outlined" density="comfortable" /></v-col>
            </v-row>
            <v-divider class="my-4" />
            <v-row dense align="center">
              <v-col cols="12" sm="5">
                <div class="field-label-sm">{{ t('aspect') }}</div>
                <div class="d-flex align-center gap-2">
                  <v-text-field type="number" :rules="[RULES.positive]" v-model="charts[editingChartIndex].aspectWidth" hide-details variant="outlined" density="compact" />
                  <span>:</span>
                  <v-text-field type="number" :rules="[RULES.positive]" v-model="charts[editingChartIndex].aspectHeight" hide-details variant="outlined" density="compact" />
                </div>
              </v-col>
              <v-col cols="12" sm="7">
                <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.01" v-model="charts[editingChartIndex].chartInfo!.backgroundDim" hide-details class="mt-4" />
              </v-col>
              <v-col cols="12">
                <v-switch color="primary" :label="t('hold_cover')" v-model="charts[editingChartIndex].chartInfo!.HoldPartialCover" :true-value="1" :false-value="0" hide-details />
              </v-col>
              <v-col cols="12">
                <v-text-field :label="t('tip')" v-model="charts[editingChartIndex].chartInfo!.tip" variant="outlined" density="comfortable" hide-details class="mt-2" />
              </v-col>
            </v-row>
          </v-form>
        </v-card-text>
        <v-card-actions class="pa-4 border-t">
          <v-spacer />
          <button class="md3-btn md3-btn-text" @click="editDialog = false">{{ t('close') }}</button>
          <button class="md3-btn md3-btn-filled" @click="saveChartInfo">{{ t('save') }}</button>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Bulk Edit Dialog -->
    <v-dialog v-model="bulkEditDialog" max-width="500">
      <v-card class="md3-dialog">
        <v-toolbar color="transparent" class="border-b">
          <v-toolbar-title>{{ t('bulk-edit-title') }}</v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-close" variant="text" @click="bulkEditDialog = false" />
        </v-toolbar>
        <v-card-text class="pa-6">
          <v-alert color="info" variant="tonal" class="mb-4" icon="mdi-information-outline">
            {{ t('bulk-edit-hint') }}
          </v-alert>
          <v-form ref="bulkForm" @submit.prevent>
            <div class="field-label-sm mb-1">{{ t('aspect') }}</div>
            <div class="d-flex align-center gap-2 mb-4">
              <v-text-field type="number" placeholder="Original" v-model="bulkEditData.aspectWidth" hide-details variant="outlined" density="comfortable" />
              <span>:</span>
              <v-text-field type="number" placeholder="Original" v-model="bulkEditData.aspectHeight" hide-details variant="outlined" density="comfortable" />
            </div>
            <div class="field-label-sm mb-1">{{ t('dim') }}</div>
            <v-slider thumb-label :min="0" :max="1" :step="0.01" :model-value="bulkEditData.backgroundDim ?? 0" @update:model-value="bulkEditData.backgroundDim = $event" hide-details color="primary" class="mb-2">
              <template v-slot:append>
                <v-btn icon="mdi-close-circle" size="small" variant="text" color="error" v-if="bulkEditData.backgroundDim !== null" @click="bulkEditData.backgroundDim = null" />
              </template>
            </v-slider>
            <div class="d-flex align-center justify-space-between mt-4">
              <span class="field-label-sm">{{ t('hold_cover') }}</span>
              <v-btn-toggle v-model="bulkEditData.holdCover" color="primary" variant="outlined" density="compact">
                <v-btn :value="null">Keep</v-btn>
                <v-btn :value="1">On</v-btn>
                <v-btn :value="0">Off</v-btn>
              </v-btn-toggle>
            </div>
          </v-form>
        </v-card-text>
        <v-card-actions class="pa-4 border-t">
          <v-spacer />
          <button class="md3-btn md3-btn-text" @click="bulkEditDialog = false">{{ t('close') }}</button>
          <button class="md3-btn md3-btn-filled" @click="saveBulkEdit">{{ t('save') }}</button>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
.batch-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  padding: 20px;
  box-sizing: border-box;
  font-family: 'Segoe UI', 'Microsoft YaHei', 'PingFang SC', sans-serif;
}

/* ===== MD3 Buttons ===== */
.md3-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.2, 0, 0, 1);
  white-space: nowrap;
  font-family: inherit;
}
.md3-btn:disabled { opacity: 0.5; cursor: default; }
.md3-btn-sm { padding: 6px 12px; font-size: 12px; }
.md3-btn-text { background: transparent; color: rgba(255, 255, 255, 0.7); }
.md3-btn-text:hover:not(:disabled) { background: rgba(255, 255, 255, 0.08); }
.md3-btn-tonal { background: rgba(130, 177, 255, 0.12); color: #82b1ff; }
.md3-btn-tonal:hover:not(:disabled) { background: rgba(130, 177, 255, 0.2); }
.md3-btn-filled { background: #82b1ff; color: #002f65; font-weight: 600; }
.md3-btn-filled:hover:not(:disabled) { background: #a0c4ff; box-shadow: 0 2px 8px rgba(130, 177, 255, 0.3); }

/* ===== Header ===== */
.batch-header {
  flex-shrink: 0;
  background: rgba(24, 24, 24, 0.85);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 20px;
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
  color: rgba(255, 255, 255, 0.4);
  pointer-events: none;
}

.md3-search {
  width: 100%;
  padding: 8px 12px 8px 36px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 13px;
  font-family: inherit;
  transition: border-color 0.2s ease;
}
.md3-search:focus {
  outline: none;
  border-color: rgba(130, 177, 255, 0.5);
}
.md3-search::placeholder {
  color: rgba(255, 255, 255, 0.35);
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
  padding: 6px 8px;
  flex-shrink: 0;
}

.sel-left, .sel-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.sel-count {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.55);
}

/* ===== Render Bar ===== */
.render-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  flex-shrink: 0;
}

.rendering-chip {
  animation: pulse-glow 2s infinite;
}
@keyframes pulse-glow {
  0% { box-shadow: 0 0 0 0 rgba(255, 152, 0, 0.4); }
  70% { box-shadow: 0 0 0 8px rgba(255, 152, 0, 0); }
  100% { box-shadow: 0 0 0 0 rgba(255, 152, 0, 0); }
}

.spin-anim { animation: spin 1.5s linear infinite; }
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
  color: rgba(255, 255, 255, 0.35);
  font-size: 14px;
}

/* ===== Chart Row (MD3 Card) ===== */
.chart-row {
  position: relative;
  background: rgba(30, 30, 30, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.chart-row:hover {
  background: rgba(40, 40, 40, 0.9);
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.chart-row.is-failed {
  border-left: 3px solid #ff5252;
}

.chart-row.is-rendering {
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.progress-bg {
  position: absolute;
  top: 0; left: 0;
  height: 100%;
  background: linear-gradient(90deg, rgba(130, 177, 255, 0.08), rgba(130, 177, 255, 0.18));
  z-index: 0;
  transition: width 0.3s linear;
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
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-sub {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.45);
}

.render-status {
  color: #82b1ff;
  font-weight: 500;
}

.error-status {
  color: #ff5252;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  width: 32px; height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 50%;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.15s ease;
}
.icon-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}
.icon-btn:disabled {
  opacity: 0.3;
  cursor: default;
}
.icon-btn-danger:hover:not(:disabled) {
  background: rgba(255, 82, 82, 0.15);
  color: #ff5252;
}

.field-label-sm {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* ===== Dialogs ===== */
.md3-dialog {
  background: rgba(28, 28, 28, 0.95) !important;
  backdrop-filter: blur(20px) !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  border-radius: 28px !important;
}

.md3-dialog-full {
  background: rgba(20, 20, 20, 0.98) !important;
  backdrop-filter: blur(20px) !important;
}

.border-b { border-bottom: 1px solid rgba(255, 255, 255, 0.06) !important; }
.border-t { border-top: 1px solid rgba(255, 255, 255, 0.06) !important; }

/* ===== Responsive ===== */
@media (max-width: 768px) {
  .batch-layout { padding: 12px; }
  .header-actions { flex-wrap: wrap; }
  .header-actions .md3-btn span { display: none; }
  .sel-right .md3-btn span { display: none; }
}
</style>
