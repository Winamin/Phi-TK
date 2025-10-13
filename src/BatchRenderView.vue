<i18n>
en:
  title: Batch Render
  select-charts: Select Charts
  select-preset: Select Preset
  add-files: Add Files
  add-folder: Add Folder
  clear-list: Clear List
  start-render: Start Render
  back: Back
  name: Name
  level: Level
  charter: Charter
  status: Status
  pending: Ready
  rendering: Pulling
  done: Done
  failed: Failed
  total-selected: "Total selected: {count}"
  all: All
  none: None
  configure: Configure
  save: Save
  actions: Actions
  close: Close
  total-charts: "Total charts: {count}"
  select-all: Select All
  deselect-all: Deselect All
  progress: Progress
  eta: ETA
  no-charts: No charts added
  no-results: No results found
  search-placeholder: Search charts...
  selected: "Selected: {count}"
  filtered-results: "Filtered: {count}"
  add-files-failed: Failed to add files
  no-charts-found: No charts found in folder
  charts-added: "{count} charts added"
  add-folder-failed: Failed to add folder
  no-charts-selected: No charts selected
  batch-completed: "Submission completed: {count} scores"
  ffmpeg-not-found: FFmpeg not found
  chart-info-missing: Chart info missing
  adding-charts: Adding charts...
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
zh-CN:
  title: 批量渲染
  select-charts: 选择谱面
  select-preset: 选择预设
  add-files: 添加文件
  add-folder: 添加文件夹
  clear-list: 清空列表
  start-render: 开始渲染
  back: 返回上一级
  name: 名称
  level: 难度
  charter: 谱师
  status: 状态
  pending: 已就绪
  rendering: 提交中
  done: 已完成
  failed: 失败
  actions: 操作
  total-selected: "已选择: {count}"
  all: 全选
  none: 取消全选
  configure: 渲染配置
  close: 关闭
  total-charts: "总谱面: {count}"
  select-all: 全选
  deselect-all: 取消全选
  progress: 进度
  eta: 预计
  no-charts: 未添加谱面
  no-results: 未找到结果
  search-placeholder: 搜索谱面...
  selected: "已选择: {count}"
  filtered-results: "筛选: {count}"
  save: 保存
  add-files-failed: 添加文件失败
  no-charts-found: 文件夹中未找到谱面
  charts-added: "已添加 {count} 个谱面"
  add-folder-failed: 添加文件夹失败
  no-charts-selected: 未选择谱面
  batch-completed: "提交完成: {count} 个谱面"
  ffmpeg-not-found: 未找到 FFmpeg
  chart-info-missing: 谱面信息缺失
  adding-charts: -正在添加谱面-
  invalid-chart-file: 无效的谱面文件
  file-type-error: "文件类型错误: {message}"
  config-saved: "配置已保存"
  chart-name: 谱面名称
  composer: 作曲家
  illustrator: 插画师
  background-dim: 背景暗度
  hold_cover: Hold 头部遮罩
  aspect: 宽高比
  tip: Tip
  width: 宽
  height: 高
  dim: 背景昏暗程度
  preview: 预览
  edit: 编辑
  chart-info: 谱面信息
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
const allSelected = ref(false);
const configViewRef = ref<InstanceType<typeof ConfigView> | null>(null);
const configDialog = ref(false);
const editDialog = ref(false);
const editingChartIndex = ref(-1);
const form = ref<any>();

// 当前渲染状态
const currentRenderingIndex = ref<number>(-1);
const renderMsg = ref('');
const renderProgress = ref<number>();

// 添加状态
const isAddingFiles = ref(false);
const isAddingFolder = ref(false);

// 默认配置（从localStorage加载或使用默认值）
const defaultConfig = ref<RenderConfig>(loadDefaultConfig());



// 预览单个谱面
async function previewChart(index: number) {
  const chart = charts.value[index];
  if (!chart.chartInfo) {
    toast(t('chart-info-missing'), 'error');
    return;
  }

  try {
    const config = await buildRenderParams();
    // 调用预览
    await invoke('preview_chart', {
      params: {
        path: chart.path,
        info: chart.chartInfo,
        config: {
          ...config,
          autoplay: true, // 设置为自动播放
        },
      },
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

  // 默认配置
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
    bitrateControl: 'CRF',
    bitrate: '28',
    targetAudio: 96000,
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
  };
}

// 保存默认配置到localStorage
function saveDefaultConfig(config: RenderConfig) {
  defaultConfig.value = config;
  localStorage.setItem('defaultRenderConfig', JSON.stringify(config));
  toast(t('config-saved'), 'success');
}

// 获取预设列表
async function getPresets() {
  try {
    const presetsMap = (await invoke('get_presets')) as Record<string, any>;
    presets.value = Object.keys(presetsMap).map((name) => ({ name }));
    presets.value.unshift({ name: 'default' });

    if (presets.value.length > 0) {
      selectedPreset.value = presets.value[0].name;
    }
  } catch (error) {
    console.error('Failed to get presets', error);
  }
}

// 添加文件
async function addFiles() {
  if (isAddingFiles.value) return;

  isAddingFiles.value = true;
  try {
    const files = await open({
      multiple: true,
      filters: [{ name: t('chart-file'), extensions: ['zip', 'json', 'pez'] }],
    });

    if (!files) return;

    const fileArray = Array.isArray(files) ? files : [files];
    const paths = fileArray.map((file) => (typeof file === 'string' ? file : (file as any).path));

    // 去重处理
    const uniquePaths = [...new Set(paths)];
    const existingPaths = new Set(charts.value.map((c) => c.path));
    const newPaths = uniquePaths.filter((path) => !existingPaths.has(path));

    if (newPaths.length === 0) {
      toast(t('no-charts-found'), 'warning');
      return;
    }

    // 批量添加
    for (const path of newPaths) {
      await addChart(path);
    }

    toast(t('charts-added', { count: newPaths.length }), 'success');
  } catch (error) {
    console.error('Failed to add files', error);
    toast(t('add-files-failed'), 'error');
  } finally {
    isAddingFiles.value = false;
  }
}

// 添加文件夹并解析所有谱面
async function addFolder() {
  if (isAddingFolder.value) return;

  isAddingFolder.value = true;
  try {
    const folder = await open({ directory: true });
    if (!folder) return;

    const folderPath = typeof folder === 'string' ? folder : (folder as any).path;
    const files = (await invoke('list_chart_files', { path: folderPath })) as string[];

    if (!files || files.length === 0) {
      toast(t('no-charts-found'), 'warning');
      return;
    }

    // 只处理谱面文件扩展名
    const validExtensions = ['.json', '.zip', '.pez'];
    const filteredFiles = files.filter((file) => {
      const ext = file.toLowerCase().slice(file.lastIndexOf('.'));
      return validExtensions.includes(ext);
    });

    if (filteredFiles.length === 0) {
      toast(t('no-charts-found'), 'warning');
      return;
    }

    // 去重处理
    const existingPaths = new Set(charts.value.map((c) => c.path));
    const newFiles = filteredFiles.filter((file) => !existingPaths.has(file));

    if (newFiles.length === 0) {
      toast(t('no-charts-found'), 'warning');
      return;
    }

    // 批量添加
    for (const file of newFiles) {
      await addChart(file);
    }

    toast(t('charts-added', { count: newFiles.length }), 'success');
  } catch (error) {
    console.error('Failed to add folder', error);
    toast(t('add-folder-failed'), 'error');
  } finally {
    isAddingFolder.value = false;
  }
}

// 添加单个谱面并解析完整信息
async function addChart(path: string) {
  try {
    // 检查是否已添加
    if (charts.value.some((c) => c.path === path)) {
      return;
    }

    // 先添加一个占位符
    const placeholderIndex =
      charts.value.push({
        path,
        name: t('adding-charts'),
        level: '...',
        charter: '...',
        status: 'pending',
        selected: true,
      }) - 1;

    const chartInfo = (await invoke('parse_chart', { path })) as ChartInfo;

    // 更新占位符为实际数据
    const aspectWidth = ref('0');
    const aspectHeight = ref('0');
    aspectWidth.value = String(chartInfo.aspectRatio);
    aspectHeight.value = '1.0';
    for (let asp of [
      [16, 9],
      [4, 3],
      [8, 5],
      [3, 2],
    ]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.aspectRatio) < 1e-4) {
        aspectWidth.value = String(asp[0]);
        aspectHeight.value = String(asp[1]);
        break;
      }
    }

    charts.value[placeholderIndex] = {
      path,
      name: chartInfo.name,
      level: chartInfo.level,
      charter: chartInfo.charter,
      status: 'pending',
      selected: true,
      chartInfo,
      aspectWidth: aspectWidth.value,
      aspectHeight: aspectHeight.value
    };
  } catch (error) {
    console.error(`Failed to parse chart: ${path}`, error);

    // 更新错误状态
    const index = charts.value.findIndex((c) => c.path === path);
    if (index !== -1) {
      let errorMessage = t('invalid-chart-file');

      if (error instanceof Error) {
        // 提取更友好的错误消息
        if (error.message.includes('as zip archive')) {
          errorMessage = t('file-type-error', { message: 'JSON file is not a ZIP archive' });
        } else if (error.message.includes('Could not find central directory')) {
          errorMessage = t('file-type-error', { message: 'Invalid ZIP file format' });
        } else {
          errorMessage = error.message;
        }
      }

      charts.value[index] = {
        path,
        name: t('parse-failed'),
        level: 'N/A',
        charter: 'N/A',
        status: 'failed',
        selected: false,
        error: errorMessage,
      };
    }
  }
}

// 清空列表
function clearList() {
  charts.value = [];
}

// 全选/取消全选
function toggleSelectAll() {
  allSelected.value = !allSelected.value;
  charts.value.forEach((chart) => {
    chart.selected = allSelected.value;
  });
}

// 构建渲染参数
async function buildRenderParams() {
  // 检查 ffmpeg
  if (!(await invoke('test_ffmpeg'))) {
    throw new Error(t('ffmpeg-not-found'));
  }

  // 获取配置
  let config: RenderConfig;

  if (selectedPreset.value === 'default') {
    // 使用保存的默认配置
    config = defaultConfig.value;
  } else {
    // 获取预设配置
    const presetsMap = (await invoke('get_presets')) as Record<string, RenderConfig>;
    config = presetsMap[selectedPreset.value];

    // 如果预设不存在，使用默认配置
    if (!config) {
      config = defaultConfig.value;
    }
  }

  // 确保关键字段存在
  if (!config.resolution) {
    throw new Error('Resolution is missing in config');
  }

  return config;
}

async function saveConfig() {
  if (!configViewRef.value) return;

  const config = await configViewRef.value.buildConfig();
  if (config) {
    saveDefaultConfig(config);
    configDialog.value = false;
  }
}

// 渲染单个谱面
async function renderSingleChart(chart: BatchChart, index: number, config: RenderConfig) {
  try {
    // 检查谱面信息是否存在
    if (!chart.chartInfo) {
      throw new Error(t('chart-info-missing'));
    }

    chart.status = 'rendering';
    currentRenderingIndex.value = index;

    await invoke('post_render', {
      params: {
        path: chart.path,
        info: chart.chartInfo,
        config,
      },
    });

    chart.status = 'done';
  } catch (error) {
    console.error(`Failed to render: ${chart.path}`, error);
    chart.status = 'failed';
    chart.error = error instanceof Error ? error.message : String(error);
    toastError(error);
  }
}

// 开始批量渲染
async function startRender() {
  const selectedCharts = charts.value.filter((chart) => chart.selected && chart.status !== 'done');

  if (selectedCharts.length === 0) {
    toast(t('no-charts-selected'), 'warning');
    return;
  }

  try {
    const config = await buildRenderParams();

    for (let i = 0; i < selectedCharts.length; i++) {
      const chart = selectedCharts[i];
      const originalIndex = charts.value.findIndex((c) => c === chart);

      if (chart.status !== 'pending') continue;

      await renderSingleChart(chart, originalIndex, config);
    }

    toast(t('batch-completed', { count: selectedCharts.length }), 'success');
  } catch (error) {
    toastError(error);
  } finally {
    currentRenderingIndex.value = -1;
    renderMsg.value = '';
    renderProgress.value = undefined;
  }
}

// 状态颜色
function statusColor(status: string) {
  switch (status) {
    case 'rendering':
      return 'blue';
    case 'done':
      return 'green';
    case 'failed':
      return 'red';
    default:
      return 'gray';
  }
}

// 表格头部定义
const tableHeaders = [
  { title: '', key: 'selected', width: 40, sortable: false },
  { title: t('name'), key: 'name', sortable: true },
  { title: t('level'), key: 'level', width: 100, sortable: true },
  { title: t('charter'), key: 'charter', width: 120, sortable: true },
  { title: t('status'), key: 'status', width: 120, sortable: true },
  { title: t('actions'), key: 'actions', width: 160, sortable: false },
];

// 搜索功能
const searchQuery = ref('');

// 过滤后的谱面列表
const filteredCharts = computed(() => {
  if (!searchQuery.value.trim()) {
    return charts.value;
  }
  
  const query = searchQuery.value.toLowerCase();
  return charts.value.filter(chart => 
    chart.name.toLowerCase().includes(query) ||
    chart.level.toLowerCase().includes(query) ||
    chart.charter.toLowerCase().includes(query) ||
    chart.path.toLowerCase().includes(query)
  );
});

// 计算选中数量
const selectedCount = computed(() => {
  return filteredCharts.value.filter((chart) => chart.selected).length;
});

// 计算过滤后的选中数量
const filteredSelectedCount = computed(() => {
  return filteredCharts.value.filter((chart) => chart.selected).length;
});

// 监听渲染事件
event.listen('render-msg', (msg) => {
  renderMsg.value = msg.payload as string;
});

event.listen('render-progress', (msg) => {
  let payload = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = `${t('progress')}: ${(payload.progress * 100).toFixed(2)}% | FPS: ${payload.fps} | ${t('eta')}: ${payload.estimate}s`;
  renderProgress.value = payload.progress * 100;
});

async function applyDefaultConfig() {
  await nextTick();
  configViewRef.value?.applyConfig(defaultConfig.value);
}

// 打开编辑对话框
/*
function openEditDialog(index: number) {
  editingChartIndex.value = index;
  editDialog.value = true;
}


 */
function openEditDialog(index: number) {
  editingChartIndex.value = index;
  // 确保aspectWidth和aspectHeight有值
  if (!charts.value[index].aspectWidth || !charts.value[index].aspectHeight) {
    const chart = charts.value[index];
    if (chart.chartInfo) {
      charts.value[index].aspectWidth = String(chart.chartInfo.aspectRatio);
      charts.value[index].aspectHeight = '1.0';
      for (let asp of [
        [16, 9],
        [4, 3],
        [8, 5],
        [3, 2],
      ]) {
        if (Math.abs(asp[0] / asp[1] - chart.chartInfo.aspectRatio) < 1e-4) {
          charts.value[index].aspectWidth = String(asp[0]);
          charts.value[index].aspectHeight = String(asp[1]);
          break;
        }
      }
    }
  }
  editDialog.value = true;
}

// 保存编辑的谱面信息
async function saveChartInfo() {
  if (editingChartIndex.value === -1 || !charts.value[editingChartIndex.value]?.chartInfo) return;

  // Safely check if form.value exists before calling validate
  if (!form.value) {
    toast(t('has-error'), 'error');
    return;
  }

  const { valid } = await form.value.validate();
  if (!valid) {
    toast(t('has-error'), 'error');
    return;
  }

  const chart = charts.value[editingChartIndex.value];
  const aspect = tryParseAspect(chart.aspectWidth, chart.aspectHeight);
  if (aspect !== undefined && chart.chartInfo) {
    chart.chartInfo.aspectRatio = aspect;
  }

  // Update display information in the list
  if (chart.chartInfo) {
    charts.value[editingChartIndex.value] = {
      ...chart,
      name: chart.chartInfo.name,
      level: chart.chartInfo.level,
      charter: chart.chartInfo.charter
    };
  }

  editDialog.value = false;
  toast(t('config-saved'), 'success');
}

// 尝试解析宽高比
function tryParseAspect(width: string | undefined, height: string | undefined): number | undefined {
  if (!width || !height) return undefined;

  try {
    let w = parseFloat(width);
    let h = parseFloat(height);
    if (isNaN(w) || isNaN(h)) return undefined;
    return w / h;
  } catch (e) {
    return undefined;
  }
}

// 本地存储键名
const STORAGE_KEY = 'batch_render_charts';

// 从本地存储加载任务列表
function loadChartsFromStorage() {
  const savedCharts = localStorage.getItem(STORAGE_KEY);
  if (savedCharts) {
    try {
      charts.value = JSON.parse(savedCharts);
    } catch (e) {
      console.error('Failed to parse saved charts', e);
    }
  }
}

// 保存任务列表到本地存储
function saveChartsToStorage() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(charts.value));
}

onMounted(() => {
  getPresets();
  loadChartsFromStorage(); // 加载存储的任务
});

// 监听任务列表变化并保存
watch(charts, saveChartsToStorage, { deep: true });
</script>

<template>
  <div class="pa-4 w-100 h-100 d-flex flex-column">
    <!-- 顶部导航栏 -->
    <div class="top-bar d-flex align-center justify-space-between mb-4">
      <v-btn @click="router.go(-1)" prepend-icon="mdi-arrow-left" variant="text" size="small">
        {{ t('back') }}
      </v-btn>
      <h2 class="text-h6 font-weight-medium">{{ t('title') }}</h2>
      <div style="width: 80px"></div> <!-- 占位符使标题居中 -->
    </div>

    <!-- 工具栏 -->
    <div class="toolbar mb-4">
      <div class="toolbar-section">
        <!-- 搜索框 -->
        <v-text-field
          v-model="searchQuery"
          :placeholder="t('search-placeholder')"
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          density="compact"
          hide-details
          clearable
          class="search-field"
        ></v-text-field>
      </div>
      
      <div class="toolbar-section">
        <!-- 文件操作 -->
        <v-btn :disabled="isAddingFolder" :loading="isAddingFiles" variant="text" size="small" @click="addFiles">
          <v-icon>mdi-file-plus</v-icon>
        </v-btn>
        <v-btn :disabled="isAddingFiles" :loading="isAddingFolder" variant="text" size="small" @click="addFolder">
          <v-icon>mdi-folder-plus</v-icon>
        </v-btn>
        <v-btn color="error" variant="text" size="small" @click="clearList">
          <v-icon>mdi-delete</v-icon>
        </v-btn>
        
        <v-divider vertical class="mx-2"></v-divider>
        
        <!-- 渲染控制 -->
        <v-btn color="secondary" variant="text" size="small" @click="configDialog = true">
          <v-icon>mdi-cog</v-icon>
        </v-btn>
        <v-btn
          color="primary"
          @click="startRender"
          :disabled="filteredSelectedCount === 0 || currentRenderingIndex >= 0"
          :loading="currentRenderingIndex >= 0"
          size="small">
          <v-icon>mdi-play</v-icon>
        </v-btn>
      </div>
    </div>

    <!-- 统计信息栏 -->
    <div class="stats-bar d-flex align-center justify-space-between mb-3">
      <div class="text-caption">
        {{ t('total-charts', { count: charts.length }) }}
        <span v-if="searchQuery" class="ml-2">
          ({{ t('filtered-results', { count: filteredCharts.length }) }})
        </span>
      </div>
      <div class="d-flex align-center gap-2">
        <span class="text-caption">{{ t('selected', { count: filteredSelectedCount }) }}</span>
        <v-btn @click="toggleSelectAll" variant="text" size="x-small" density="compact">
          {{ t(allSelected ? 'deselect-all' : 'select-all') }}
        </v-btn>
      </div>
    </div>

    <!-- 渲染进度显示 -->
    <v-card v-if="currentRenderingIndex >= 0" class="mb-3 progress-card">
      <v-card-text class="pa-3">
        <div class="d-flex align-center mb-2">
          <v-progress-circular indeterminate size="16" width="2" class="mr-2" />
          <span class="text-body-2">{{ t('rendering') }}: {{ charts[currentRenderingIndex]?.name }}</span>
        </div>
        <v-progress-linear :model-value="renderProgress" color="primary" height="4" rounded />
        <p class="text-caption mt-1 mb-0">{{ renderMsg }}</p>
      </v-card-text>
    </v-card>

    <!-- 配置对话框 -->
     <v-dialog v-model="configDialog" max-width="700" scrollable @after-enter="applyDefaultConfig" class="dialog-blur">
      <v-card rounded="xl" class="transparent-blur-card">
        <v-toolbar color="primary">
          <v-toolbar-title>{{ t('configure') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon @click="configDialog = false" color="blue-darken-2">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-toolbar>
        <v-card-text class="pa-0">
          <ConfigView ref="configViewRef" />
        </v-card-text>
        <v-card-actions class="justify-end pa-4">
          <v-btn
            color="primary"
            @click="saveConfig"
          >
            {{ t('save') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 谱面信息编辑对话框 -->
    <v-dialog v-model="editDialog" max-width="700" class="dialog-blur">
      <v-card rounded="xl" class="transparent-blur-card">
        <v-toolbar color="primary">
          <v-toolbar-title>{{ t('chart-info') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon @click="editDialog = false" color="blue-darken-2">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-toolbar>
        <v-card-text class="pa-4">
          <v-form ref="form" v-if="editingChartIndex >= 0 && charts[editingChartIndex]?.chartInfo">
            <v-row no-gutters class="mx-n2">
              <v-col cols="8">
                <v-text-field class="mx-2" :label="t('chart-name')" :rules="[RULES.non_empty]"
                              v-model="charts[editingChartIndex].chartInfo!.name" prepend-inner-icon="mdi-format-title"></v-text-field>
              </v-col>
              <v-col cols="4">
                <v-text-field class="mx-2" :label="t('level')" :rules="[RULES.non_empty]"
                              v-model="charts[editingChartIndex].chartInfo!.level" prepend-inner-icon="mdi-star-outline"></v-text-field>
              </v-col>
            </v-row>

            <v-row no-gutters class="mx-n2 mt-1">
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('charter')" :rules="[RULES.non_empty]"
                              v-model="charts[editingChartIndex].chartInfo!.charter" prepend-inner-icon="mdi-account-edit"></v-text-field>
              </v-col>
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('composer')"
                              v-model="charts[editingChartIndex].chartInfo!.composer" prepend-inner-icon="mdi-music-note"></v-text-field>
              </v-col>
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('illustrator')"
                              v-model="charts[editingChartIndex].chartInfo!.illustrator" prepend-inner-icon="mdi-palette"></v-text-field>
              </v-col>
            </v-row>

            <v-row no-gutters class="mx-n2 mt-1 align-center">
              <v-col cols="4">
                <div class="mx-2 d-flex flex-column">
                  <p class="text-caption" v-t="'aspect'"></p>
                  <div class="d-flex flex-row align-center justify-center">
                    <v-text-field type="number" class="mr-2" :rules="[RULES.positive]" :label="t('width')"
                                  v-model="charts[editingChartIndex].aspectWidth" prepend-inner-icon="mdi-arrow-expand-horizontal"></v-text-field>
                    <p>:</p>
                    <v-text-field type="number" class="ml-2" :rules="[RULES.positive]" :label="t('height')"
                                  v-model="charts[editingChartIndex].aspectHeight" prepend-inner-icon="mdi-arrow-expand-vertical"></v-text-field>
                  </div>
                </div>
              </v-col>
              <v-col cols="8" class="px-6">
                <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.01"
                          v-model="charts[editingChartIndex].chartInfo!.backgroundDim">
                </v-slider>
                <v-row no-gutters class="mx-n2 mt-1 align-center">
                  <v-col cols="12" class="px-6">
                    <v-switch
                      :label="t('hold_cover')"
                      v-model="charts[editingChartIndex].chartInfo!.HoldPartialCover"
                      :true-value="1"
                      :false-value="0"
                      color="primary"
                      persistent-hint
                    ></v-switch>
                  </v-col>
                </v-row>
              </v-col>
            </v-row>

            <v-row no-gutters class="mx-n2 mt-1">
              <v-col cols="12">
                <v-text-field class="mx-2" :label="t('tip')" :placeholder="t('tip-placeholder')"
                              v-model="charts[editingChartIndex].chartInfo!.tip" prepend-inner-icon="mdi-lightbulb-on-outline"></v-text-field>
              </v-col>
            </v-row>
          </v-form>
        </v-card-text>
        <v-card-actions class="justify-end pa-4">
          <v-btn color="primary" @click="saveChartInfo">{{ t('save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 渲染进度显示 -->
    <v-card v-if="currentRenderingIndex >= 0" class="mb-4 dialog-blur">
      <v-card-title class="d-flex align-center text-subtitle-1">
        <v-progress-circular indeterminate size="20" width="2" class="mr-2" />
        {{ t('rendering') }}: {{ charts[currentRenderingIndex]?.name }}
      </v-card-title>
      <v-card-text>
        <v-progress-linear :model-value="renderProgress" color="primary" height="8" rounded />
        <p class="mt-2 text-caption">{{ renderMsg }}</p>
      </v-card-text>
    </v-card>

    <!-- 谱面表格 -->
    <div class="batch-table-container flex-grow-1">
      <!-- 使用虚拟滚动表格 -->
      <v-data-table-virtual
        :headers="tableHeaders"
        :items="filteredCharts"
        :item-value="(item) => item.path"
        density="compact"
        fixed-header
        height="100%"
        class="custom-table"
        :no-data-text="searchQuery ? t('no-results') : t('no-charts')"
        :items-per-page="-1"
        hide-default-footer
      >
        <!-- 选择框列 -->
        <template v-slot:item.selected="{ item, index }">
          <v-checkbox
            v-model="item.selected"
            :disabled="item.status === 'rendering'"
            density="compact"
            hide-details
            class="custom-checkbox"
          />
        </template>

        <!-- 名称列 -->
        <template v-slot:item.name="{ item }">
          <div :title="item.name" class="text-truncate table-name-cell">{{ item.name }}</div>
        </template>

        <!-- 状态列 -->
        <template v-slot:item.status="{ item }">
          <v-chip
            :class="`status-chip status-${item.status}`"
            size="small"
            class="status-chip-custom"
          >
            {{ t(item.status) }}
          </v-chip>
          <div v-if="item.error" class="text-caption text-red mt-1 error-message">{{ item.error }}</div>
        </template>

        <!-- 操作列 -->
        <template v-slot:item.actions="{ item, index }">
          <div class="action-buttons">
            <v-btn
              @click="openEditDialog(index)"
              color="primary"
              icon="mdi-pencil"
              size="small"
              variant="flat"
              class="action-btn"
              :disabled="!item.chartInfo"
              :title="t('edit')"
            ></v-btn>
            <v-btn
              @click="previewChart(index)"
              color="primary"
              icon="mdi-play"
              size="small"
              variant="flat"
              class="action-btn"
              :disabled="!item.chartInfo"
              :title="t('preview')"
            ></v-btn>
            <v-btn
              :disabled="item.status === 'rendering'"
              color="error"
              icon="mdi-delete"
              size="small"
              variant="flat"
              class="action-btn"
              @click="charts.splice(index, 1)"
              :title="t('delete')"
            />
          </div>
        </template>
      </v-data-table-virtual>
    </div>
  </div>
</template>

<style scoped>
/* 顶部导航栏 */
.top-bar {
  background: rgba(30, 30, 30, 0.8);
  backdrop-filter: blur(8px);
  border-radius: 12px;
  padding: 8px 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  background: rgba(30, 30, 30, 0.8);
  backdrop-filter: blur(8px);
  border-radius: 12px;
  padding: 8px 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-field {
  width: 250px;
  transition: width 0.3s ease;
}

.search-field:focus-within {
  width: 300px;
}

/* 统计信息栏 */
.stats-bar {
  background: rgba(30, 30, 30, 0.6);
  border-radius: 8px;
  padding: 8px 16px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

/* 进度卡片 */
.progress-card {
  background: rgba(30, 30, 30, 0.8) !important;
  backdrop-filter: blur(8px) !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  border-radius: 12px !important;
}

/* 表格容器 */
.batch-table-container {
  background: rgba(30, 30, 30, 0.8) !important;
  backdrop-filter: blur(8px) !important;
  -webkit-backdrop-filter: blur(8px) !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  border-radius: 12px !important;
  overflow: hidden;
  flex: 1;
  min-height: 300px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2) !important;
}

/* 美化滚动条 */
.batch-table-container :deep(.v-data-table-virtual__scroll) {
  scrollbar-width: thin;
  scrollbar-color: rgba(118, 64, 193, 0.5) rgba(30, 30, 30, 0.1);
}

/* 强制显示滚动条，覆盖全局隐藏 */
.batch-table-container :deep(.v-data-table-virtual__scroll)::-webkit-scrollbar {
  width: 8px;
  display: block !important;
}

.batch-table-container :deep(.v-data-table-virtual__scroll)::-webkit-scrollbar-track {
  background: rgba(30, 30, 30, 0.1);
  border-radius: 4px;
  margin: 8px;
}

.batch-table-container :deep(.v-data-table-virtual__scroll)::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.6), rgba(156, 105, 217, 0.6));
  border-radius: 4px;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.batch-table-container :deep(.v-data-table-virtual__scroll)::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.8), rgba(156, 105, 217, 0.8));
  border-color: rgba(255, 255, 255, 0.2);
}

.batch-table-container :deep(.v-data-table-virtual__scroll)::-webkit-scrollbar-thumb:active {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.9), rgba(156, 105, 217, 0.9));
}

.text-truncate {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gap-2 {
  gap: 8px;
}

.flex-grow-1 {
  flex-grow: 1;
}

tr:hover {
  background-color: rgba(118, 64, 193, 0.1) !important;
  transition: background-color 0.3s ease;
}

tr {
  transition: background-color 0.5s ease;
}

.dialog-blur {
  backdrop-filter: blur(16px);
}

.transparent-blur-card {
  background: rgba(30, 30, 30, 0.85) !important;
  backdrop-filter: blur(12px) !important;
  -webkit-backdrop-filter: blur(12px) !important;
  border-radius: 20px !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3) !important;
}

/* 表格整体样式穿透 */
.custom-table {
  background: transparent !important;
}

/* 表头样式 */
.custom-table :deep(.v-data-table__thead) {
  background: linear-gradient(180deg, rgba(118, 64, 193, 0.3), rgba(156, 105, 217, 0.2)) !important;
}

.custom-table :deep(.v-data-table__th) {
  color: #e0e0e0 !important;
  font-weight: 500 !important;
  padding: 16px !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
}

/* 表格行样式 */
.custom-table :deep(.v-data-table__tbody tr) {
  transition: all 0.2s ease;
  background: transparent !important;
}

.custom-table :deep(.v-data-table__tbody tr:hover) {
  background: rgba(118, 64, 193, 0.08) !important;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(118, 64, 193, 0.15);
}

.custom-table :deep(.v-data-table__td) {
  color: #f0f0f0 !important;
  padding: 16px !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05) !important;
  vertical-align: middle !important;
}

/* 名称单元格样式 */
.table-name-cell {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 复选框样式优化 */
.custom-checkbox .v-icon {
  color: #8a8a94 !important;
  transition: color 0.2s ease;
}

.custom-checkbox .v-icon.checked {
  color: #7640c1 !important;
}

.custom-checkbox:disabled .v-icon {
  color: #505058 !important;
  opacity: 0.6;
}

/* 状态标签样式 */
.status-chip-custom {
  border: none !important;
  font-weight: 500;
  text-transform: capitalize;
  padding: 4px 12px;
  border-radius: 12px !important;
}

/* 不同状态的标签颜色 */
.status-chip.status-ready {
  background: rgba(76, 175, 80, 0.2) !important;
  color: #81c784 !important;
}

.status-chip.status-rendering {
  background: rgba(255, 152, 0, 0.2) !important;
  color: #ffb74d !important;
}

.status-chip.status-error {
  background: rgba(244, 67, 54, 0.2) !important;
  color: #e57373 !important;
}

.error-message {
  font-size: 12px;
  color: #ff8a80 !important;
  margin-top: 4px;
}

.action-buttons {
  display: flex;
  gap: 6px;
  justify-content: center;
}

.action-btn {
  width: 36px;
  height: 36px;
  border-radius: 8px !important;
  transition: all 0.2s ease !important;
  opacity: 0.9;
}

.action-btn:hover {
  opacity: 1;
  transform: scale(1.05);
  background: rgba(118, 64, 193, 0.2) !important;
}

.action-btn:disabled {
  opacity: 0.4 !important;
  transform: none !important;
  background: transparent !important;
}

/* 空状态样式 */
.empty-state {
  text-align: center;
  padding: 60px 20px !important;
  color: #a0a0a0;
}

.empty-state-icon {
  margin-bottom: 16px;
}

.empty-state-text {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.6);
}

/* 按钮样式优化 */
:deep(.v-btn) {
  border-radius: 8px !important;
  font-weight: 500;
  transition: all 0.3s ease;
  text-transform: none;
}

:deep(.v-btn[color="primary"]) {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.8), rgba(156, 105, 217, 0.8)) !important;
  box-shadow: 0 4px 12px rgba(118, 64, 193, 0.3);
}

:deep(.v-btn[color="primary"]:hover) {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(118, 64, 193, 0.4);
}

:deep(.v-btn[color="secondary"]) {
  background: linear-gradient(135deg, rgba(63, 81, 181, 0.8), rgba(92, 107, 192, 0.8)) !important;
  box-shadow: 0 4px 12px rgba(63, 81, 181, 0.3);
}

:deep(.v-btn[color="error"]) {
  background: linear-gradient(135deg, rgba(244, 67, 54, 0.8), rgba(229, 115, 115, 0.8)) !important;
  box-shadow: 0 4px 12px rgba(244, 67, 54, 0.3);
}

/* 组合框样式优化 */
:deep(.v-combobox .v-field) {
  background: rgba(255, 255, 255, 0.08) !important;
  border-radius: 8px !important;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
}

:deep(.v-combobox .v-field__input) {
  color: #fff !important;
}

:deep(.v-combobox .v-label) {
  color: rgba(255, 255, 255, 0.7) !important;
}

/* 进度卡片样式优化 */
:deep(.v-card) {
  background: rgba(30, 30, 30, 0.85) !important;
  backdrop-filter: blur(12px) !important;
  -webkit-backdrop-filter: blur(12px) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  border-radius: 16px !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3) !important;
}

:deep(.v-card-title) {
  color: #e0e0e0 !important;
}

:deep(.v-card-text) {
  color: rgba(255, 255, 255, 0.8) !important;
}

/* 工具栏样式优化 */
:deep(.v-toolbar) {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.8), rgba(156, 105, 217, 0.8)) !important;
  border-radius: 20px 20px 0 0 !important;
}

:deep(.v-toolbar-title) {
  color: #fff !important;
}
</style>