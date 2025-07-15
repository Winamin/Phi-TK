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
  pending: Pending
  rendering: Rendering
  done: Done
  failed: Failed
  total-selected: "Total selected: {count}"
  all: All
  none: None
  configure: Configure
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
  batch-completed: "Batch completed: {count} charts rendered"
  ffmpeg-not-found: FFmpeg not found
  chart-info-missing: Chart info missing

zh-CN:
  title: 批量渲染
  select-charts: 选择谱面
  select-preset: 选择预设
  add-files: 添加文件
  add-folder: 添加文件夹
  clear-list: 清空列表
  start-render: 开始渲染
  back: 返回
  name: 名称
  level: 难度
  charter: 谱师
  status: 状态
  pending: 等待中
  rendering: 渲染中
  done: 已完成
  failed: 失败
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
  add-files-failed: 添加文件失败
  no-charts-found: 文件夹中未找到谱面
  charts-added: "已添加 {count} 个谱面"
  add-folder-failed: 添加文件夹失败
  no-charts-selected: 未选择谱面
  batch-completed: "批量渲染完成: {count} 个谱面已渲染"
  ffmpeg-not-found: 未找到 FFmpeg
  chart-info-missing: 谱面信息缺失
</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { useRouter } from 'vue-router';
const router = useRouter();

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { event } from '@tauri-apps/api';

import { toastError, toast } from './common';
import type { ChartInfo, RenderConfig } from './model';
import ConfigView from '@/components/ConfigView.vue';

interface BatchChart {
  path: string;
  name: string;
  level: string;
  charter: string;
  status: 'pending' | 'rendering' | 'done' | 'failed';
  selected: boolean;
  error?: string;
  chartInfo?: ChartInfo;
}

const charts = ref<BatchChart[]>([]);
const selectedPreset = ref<string>('default');
const presets = ref<{name: string}[]>([]);
const allSelected = ref(false);
const configViewRef = ref<InstanceType<typeof ConfigView> | null>(null);
const configDialog = ref(false);

// 当前渲染状态
const currentRenderingIndex = ref<number>(-1);
const renderMsg = ref('');
const renderProgress = ref<number>();

// 获取预设列表
async function getPresets() {
  try {
    const presetsMap = await invoke('get_presets') as Record<string, any>;
    presets.value = Object.keys(presetsMap).map(name => ({ name }));
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
  try {
    const files = await open({
      multiple: true,
      filters: [{ name: t('chart-file'), extensions: ['zip', 'json', 'pek'] }]
    });

    if (!files) return;

    const fileArray = Array.isArray(files) ? files : [files];

    for (const file of fileArray) {
      const filePath = typeof file === 'string' ? file : (file as any).path;
      await addChart(filePath);
    }
  } catch (error) {
    console.error('Failed to add files', error);
    toast(t('add-files-failed'), 'error');
  }
}

// 添加文件夹并解析所有谱面
async function addFolder() {
  try {
    const folder = await open({ directory: true });
    if (!folder) return;

    const folderPath = typeof folder === 'string' ? folder : (folder as any).path;
    const files = await invoke('list_chart_files', { path: folderPath }) as string[];

    if (!files || files.length === 0) {
      toast(t('no-charts-found'), 'warning');
      return;
    }

    for (const file of files) {
      await addChart(file);
    }

    toast(t('charts-added', { count: files.length }), 'success');
  } catch (error) {
    console.error('Failed to add folder', error);
    toast(t('add-folder-failed'), 'error');
  }
}

// 添加单个谱面并解析完整信息
async function addChart(path: string) {
  try {
    // 检查是否已添加
    if (charts.value.some(c => c.path === path)) {
      return;
    }

    const chartInfo = await invoke('parse_chart', { path }) as ChartInfo;

    charts.value.push({
      path,
      name: chartInfo.name,
      level: chartInfo.level,
      charter: chartInfo.charter,
      status: 'pending',
      selected: true,
      chartInfo
    });
  } catch (error) {
    console.error(`Failed to parse chart: ${path}`, error);
    charts.value.push({
      path,
      name: t('parse-failed'),
      level: 'N/A',
      charter: 'N/A',
      status: 'failed',
      selected: false,
      error: error instanceof Error ? error.message : String(error)
    });
  }
}

// 清空列表
function clearList() {
  charts.value = [];
}

// 全选/取消全选
function toggleSelectAll() {
  allSelected.value = !allSelected.value;
  charts.value.forEach(chart => {
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
    // 使用ConfigView的配置
    if (!configViewRef.value) {
      // 如果ConfigView未初始化，则创建一个临时配置
      console.warn('ConfigView not initialized, using default config');
      config = {
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
        uiPause: true
      };
    } else {
      const builtConfig = await configViewRef.value.buildConfig();
      if (!builtConfig) {
        throw new Error('Failed to build config: validation failed');
      }
      config = builtConfig;
    }
  } else {
    // 获取预设配置
    const presetsMap = await invoke('get_presets') as Record<string, RenderConfig>;
    config = presetsMap[selectedPreset.value];

    // 如果预设不存在，使用默认配置
    if (!config) {
      console.warn(`Preset '${selectedPreset.value}' not found, using default config`);
      config = {
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
        uiPause: true
      };
    }
  }

  // 确保关键字段存在
  if (!config.resolution) {
    throw new Error('Resolution is missing in config');
  }

  return config;
}

// 渲染单个谱面
async function renderSingleChart(chart: BatchChart, index: number, config: RenderConfig) {
  try {
    chart.status = 'rendering';
    currentRenderingIndex.value = index;

    await invoke('post_render', {
      params: {
        path: chart.path,
        info: chart.chartInfo,
        config
      }
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
  const selectedCharts = charts.value.filter(chart => chart.selected && chart.status !== 'done');

  if (selectedCharts.length === 0) {
    toast(t('no-charts-selected'), 'warning');
    return;
  }

  try {
    const config = await buildRenderParams();

    for (let i = 0; i < selectedCharts.length; i++) {
      const chart = selectedCharts[i];
      const originalIndex = charts.value.findIndex(c => c === chart);

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
    case 'rendering': return 'blue';
    case 'done': return 'green';
    case 'failed': return 'red';
    default: return 'gray';
  }
}

// 计算选中数量
const selectedCount = computed(() => {
  return charts.value.filter(chart => chart.selected).length;
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

onMounted(() => {
  getPresets();
});
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
    <v-dialog v-model="configDialog" max-width="800" scrollable>
      <v-card>
        <v-toolbar color="primary">
          <v-toolbar-title>{{ t('configure') }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon @click="configDialog = false">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-toolbar>
        <v-card-text class="pa-0">
          <!-- 确保ConfigView组件在对话框打开时渲染 -->
          <ConfigView ref="configViewRef" />
        </v-card-text>
        <v-card-actions class="justify-end pa-4">
          <v-btn color="primary" @click="configDialog = false">{{ t('close') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <div class="batch-controls">
      <!-- 顶部控制栏 -->
      <div class="d-flex align-center gap-2 mb-4 flex-wrap">
        <v-btn color="primary" @click="addFiles" prepend-icon="mdi-file-plus" size="small">
          {{ t('add-files') }}
        </v-btn>

        <v-btn color="primary" @click="addFolder" prepend-icon="mdi-folder-plus" size="small">
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
          style="min-width: 180px;"
          class="mr-2"
        />

        <v-btn
          color="primary"
          @click="startRender"
          prepend-icon="mdi-play"
          :disabled="selectedCount === 0 || currentRenderingIndex >= 0"
          :loading="currentRenderingIndex >= 0"
          size="small"
        >
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
        <v-progress-linear
          :model-value="renderProgress"
          color="primary"
          height="8"
          rounded
        />
        <p class="mt-2 text-caption">{{ renderMsg }}</p>
      </v-card-text>
    </v-card>

    <!-- 谱面表格 -->
    <div class="batch-table-container flex-grow-1">
      <v-table density="comfortable" fixed-header height="100%">
        <thead>
        <tr>
          <th width="40"></th>
          <th>{{ t('name') }}</th>
          <th width="100">{{ t('level') }}</th>
          <th width="120">{{ t('charter') }}</th>
          <th width="120">{{ t('status') }}</th>
          <th width="80">{{ t('actions') }}</th>
        </tr>
        </thead>
        <tbody>
        <tr v-for="(chart, index) in charts" :key="index">
          <td>
            <v-checkbox
              v-model="chart.selected"
              density="compact"
              hide-details
              :disabled="chart.status === 'rendering'"
            />
          </td>
          <td class="text-truncate" style="max-width: 200px;" :title="chart.name">{{ chart.name }}</td>
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
              size="small"
              icon="mdi-delete"
              @click="charts.splice(index, 1)"
              variant="tonal"
              color="error"
              :disabled="chart.status === 'rendering'"
            />
          </td>
        </tr>
        <tr v-if="charts.length === 0">
          <td colspan="6" class="text-center py-8 text-disabled">
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
</style>