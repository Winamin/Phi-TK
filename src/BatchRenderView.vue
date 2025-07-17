<i18n>
en:
  title: Batch Render
  select-charts: Select Charts
  select-preset: Select Preset
  add-files: Add Files
  add-folder: Add Folder
  clear-list: Clear List
  start-render: Start Render
  back: Tasks
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
  back: 任务列表
  name: 名称
  level: 难度
  charter: 谱师
  status: 状态
  pending: 已准备
  rendering: 提交中
  done: 已完成
  failed: 失败
  actions: 操作
  total-selected: "已选择: {count}"
  all: 全选
  none: 取消全选
  configure: 配置
  close: 关闭
  total-charts: "总谱面: {count}"
  select-all: 全选
  deselect-all: 取消全选
  progress: 进度
  eta: 预计
  no-charts: 未添加谱面
  save: 保存
  add-files-failed: 添加文件失败
  no-charts-found: 文件夹中未找到谱面
  charts-added: "已添加 {count} 个谱面"
  add-folder-failed: 添加文件夹失败
  no-charts-selected: 未选择谱面
  batch-completed: "提交完成: {count} 个谱面"
  ffmpeg-not-found: 未找到 FFmpeg
  chart-info-missing: 谱面信息缺失
  adding-charts: 正在添加谱面...
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

// 行颜色函数
function rowColor(status: string) {
  switch (status) {
    case 'pending': return 'bg-blue-lighten-5';
    case 'rendering': return 'bg-orange-lighten-5';
    case 'done': return 'bg-green-lighten-5';
    case 'failed': return 'bg-red-lighten-5';
    default: return '';
  }
}

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
    hevc: false,
    bitrateControl: 'CRF',
    bitrate: '28',
    targetAudio: 96000,
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
      filters: [{ name: t('chart-file'), extensions: ['zip', 'json', 'pek'] }],
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
    const validExtensions = ['.json', '.zip', '.pek'];
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

// 计算选中数量
const selectedCount = computed(() => {
  return charts.value.filter((chart) => chart.selected).length;
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
  <div class="pa-6 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1.5rem">
    <div class="d-flex align-center justify-space-between">
      <h1>{{ t('title') }}</h1>
      <v-btn @click="router.go(-1)" prepend-icon="mdi-arrow-left" variant="text" size="small">
        {{ t('back') }}
      </v-btn>
    </div>

    <!-- 配置对话框 -->
    <v-dialog v-model="configDialog" max-width="800" scrollable @after-enter="applyDefaultConfig">
      <v-card>
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
    <v-dialog v-model="editDialog" max-width="600">
      <v-card>
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
                              v-model="charts[editingChartIndex].chartInfo!.name"></v-text-field>
              </v-col>
              <v-col cols="4">
                <v-text-field class="mx-2" :label="t('level')" :rules="[RULES.non_empty]"
                              v-model="charts[editingChartIndex].chartInfo!.level"></v-text-field>
              </v-col>
            </v-row>

            <v-row no-gutters class="mx-n2 mt-1">
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('charter')" :rules="[RULES.non_empty]"
                              v-model="charts[editingChartIndex].chartInfo!.charter"></v-text-field>
              </v-col>
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('composer')"
                              v-model="charts[editingChartIndex].chartInfo!.composer"></v-text-field>
              </v-col>
              <v-col cols="12" sm="4">
                <v-text-field class="mx-2" :label="t('illustrator')"
                              v-model="charts[editingChartIndex].chartInfo!.illustrator"></v-text-field>
              </v-col>
            </v-row>

            <v-row no-gutters class="mx-n2 mt-1 align-center">
              <v-col cols="4">
                <div class="mx-2 d-flex flex-column">
                  <p class="text-caption" v-t="'aspect'"></p>
                  <div class="d-flex flex-row align-center justify-center">
                    <v-text-field type="number" class="mr-2" :rules="[RULES.positive]" :label="t('width')"
                                  v-model="charts[editingChartIndex].aspectWidth"></v-text-field>
                    <p>:</p>
                    <v-text-field type="number" class="ml-2" :rules="[RULES.positive]" :label="t('height')"
                                  v-model="charts[editingChartIndex].aspectHeight"></v-text-field>
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
                              v-model="charts[editingChartIndex].chartInfo!.tip"></v-text-field>
              </v-col>
            </v-row>
          </v-form>
        </v-card-text>
        <v-card-actions class="justify-end pa-4">
          <v-btn color="primary" @click="saveChartInfo">{{ t('save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <div class="batch-controls">
      <!-- 顶部控制栏 -->
      <div class="d-flex align-center gap-2 mb-4 flex-wrap">
        <v-btn :disabled="isAddingFolder" :loading="isAddingFiles" color="primary" prepend-icon="mdi-file-plus" size="small" @click="addFiles">
          {{ t('add-files') }}
        </v-btn>

        <v-btn :disabled="isAddingFiles" :loading="isAddingFolder" color="primary" prepend-icon="mdi-folder-plus" size="small" @click="addFolder">
          {{ t('add-folder') }}
        </v-btn>

        <v-btn color="error" @click="clearList" prepend-icon="mdi-delete" size="small">
          {{ t('clear-list') }}
        </v-btn>

        <v-btn color="secondary" @click="configDialog = true" prepend-icon="mdi-cog" size="small">
          {{ t('configure') }}
        </v-btn>

        <v-spacer></v-spacer>

        <v-combobox
          v-model="selectedPreset"
          :label="t('select-preset')"
          :items="presets"
          item-title="name"
          density="compact"
          variant="outlined"
          class="mr-2"
          style="min-width: 180px" />

        <v-btn
          color="primary"
          @click="startRender"
          prepend-icon="mdi-play"
          :disabled="selectedCount === 0 || currentRenderingIndex >= 0"
          :loading="currentRenderingIndex >= 0"
          size="small">
          {{ t('start-render') }}
        </v-btn>
      </div>

      <!-- 信息统计栏 -->
      <div class="d-flex align-center justify-space-between mt-2 text-caption">
        <span>{{ t('total-charts', { count: charts.length }) }}</span>
        <span>{{ t('total-selected', { count: selectedCount }) }}</span>
        <v-btn @click="toggleSelectAll" variant="text" size="x-small" density="compact">
          {{ t(allSelected ? 'deselect-all' : 'select-all') }}
        </v-btn>
      </div>
    </div>

    <!-- 渲染进度显示 -->
    <v-card v-if="currentRenderingIndex >= 0" class="mb-4">
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
    <div class="batch-table-container flex-grow-1" color= "blue-grey-2">
      <v-table density="comfortable" fixed-header height="100%">
        <thead>
        <tr>
          <th width="40"></th>
          <th>{{ t('name') }}</th>
          <th width="100">{{ t('level') }}</th>
          <th width="120">{{ t('charter') }}</th>
          <th width="120">{{ t('status') }}</th>
          <th width="160">{{ t('actions') }}</th>
        </tr>
        </thead>
        <tbody>
        <tr
          v-for="(chart, index) in charts"
          :key="index"
          :class="rowColor(chart.status)"
        >
          <td>
            <v-checkbox v-model="chart.selected" :disabled="chart.status === 'rendering'" density="compact" hide-details />
          </td>
          <td :title="chart.name" class="text-truncate" style="max-width: 200px">{{ chart.name }}</td>
          <td>{{ chart.level }}</td>
          <td>{{ chart.charter }}</td>
          <td>
            <v-chip :color="statusColor(chart.status)" size="small">
              {{ t(chart.status) }}
            </v-chip>
            <div v-if="chart.error" class="text-caption text-red mt-1">{{ chart.error }}</div>
          </td>
          <td>
            <v-btn
              @click="openEditDialog(index)"
              color="primary"
              icon="mdi-pencil"
              size="small"
              variant="tonal"
              class="mr-1"
              :disabled="!chart.chartInfo"
            ></v-btn>
            <v-btn
              @click="previewChart(index)"
              color="primary"
              icon="mdi-play"
              size="small"
              variant="tonal"
              class="mr-1"
              :disabled="!chart.chartInfo"
            ></v-btn>
            <v-btn
              :disabled="chart.status === 'rendering'"
              color="error"
              icon="mdi-delete"
              size="small"
              variant="tonal"
              @click="charts.splice(index, 1)"
            />
          </td>
        </tr>
        <tr v-if="charts.length === 0">
          <td class="text-center py-8 text-disabled" colspan="6">
            {{ t('no-charts') }}
          </td>
        </tr>
        </tbody>
      </v-table>
    </div>
  </div>
</template>

<style scoped>
.batch-table-container {
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  overflow: hidden;
  flex: 1;
  min-height: 300px;
}

.batch-controls {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 16px;
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
  background-color: rgba(255, 255, 255, 0.08) !important;
  transition: background-color 0.3s ease;
}

tr {
  transition: background-color 0.5s ease;
}
</style>