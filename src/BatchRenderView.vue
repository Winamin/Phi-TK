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
  preset: Preset
  status: Status
  pending: Pending
  rendering: Rendering
  done: Done
  failed: Failed
  total-selected: "Total selected: {count}"
  all: All
  none: None

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
  preset: 预设
  status: 状态
  pending: 等待中
  rendering: 渲染中
  done: 已完成
  failed: 失败
  total-selected: "已选择: {count}"
  all: 全选
  none: 取消全选
</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { useRouter } from 'vue-router';
const router = useRouter();

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface BatchChart {
  path: string;
  name: string;
  level: string;
  charter: string;
  preset: string;
  status: 'pending' | 'rendering' | 'done' | 'failed';
  selected: boolean;
  error?: string;
}

const charts = ref<BatchChart[]>([]);
const selectedPreset = ref<string>('default');
const presets = ref<{name: string}[]>([]);
const allSelected = ref(false);

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
      filters: [{ name: 'Chart Files', extensions: ['zip', 'json', 'pek'] }]
    });

    if (!files) return;

    // 处理文件数组，确保每个文件都有 path 属性
    const fileArray = Array.isArray(files) ? files : [files];

    for (const file of fileArray) {
      const filePath = typeof file === 'string' ? file : (file as any).path;
      await addChart(filePath);
    }
  } catch (error) {
    console.error('Failed to add files', error);
  }
}

// 添加文件夹
async function addFolder() {
  try {
    const folder = await open({ directory: true });
    if (!folder) return;

    // 这里需要实现获取文件夹内所有谱面的逻辑
    // 简化处理：只添加文件夹路径
    await addChart(folder as string);
  } catch (error) {
    console.error('Failed to add folder', error);
  }
}

// 添加单个谱面
async function addChart(path: string) {
  try {
    const info = await invoke('parse_chart', { path }) as {
      name: string;
      level: string;
      charter: string;
    };

    charts.value.push({
      path,
      name: info.name,
      level: info.level,
      charter: info.charter,
      preset: selectedPreset.value,
      status: 'pending',
      selected: true
    });
  } catch (error) {
    console.error(`Failed to parse chart: ${path}`, error);
    charts.value.push({
      path,
      name: 'Failed to parse',
      level: 'N/A',
      charter: 'N/A',
      preset: selectedPreset.value,
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

// 开始渲染
async function startRender() {
  const selectedCharts = charts.value.filter(chart => chart.selected);

  for (const chart of selectedCharts) {
    if (chart.status !== 'pending') continue;

    try {
      chart.status = 'rendering';
      await invoke('post_render', {
        params: {
          path: chart.path,
          config: { preset: chart.preset }
        }
      });
      chart.status = 'done';
    } catch (error) {
      console.error(`Failed to render: ${chart.path}`, error);
      chart.status = 'failed';
      chart.error = error instanceof Error ? error.message : String(error);
    }
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

onMounted(() => {
  getPresets();
});
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1.5rem">
    <div class="d-flex align-center justify-space-between mb-6">
      <h1>{{ t('title') }}</h1>
      <v-btn @click="router.go(-1)" prepend-icon="mdi-arrow-left">
        {{ t('back') }}
      </v-btn>
    </div>

    <div class="batch-controls mb-6">
      <div class="d-flex align-center gap-4 mb-4">
        <v-btn color="primary" @click="addFiles" prepend-icon="mdi-file-plus">
          {{ t('add-files') }}
        </v-btn>

        <v-btn color="primary" @click="addFolder" prepend-icon="mdi-folder-plus">
          {{ t('add-folder') }}
        </v-btn>

        <v-btn color="error" @click="clearList" prepend-icon="mdi-delete">
          {{ t('clear-list') }}
        </v-btn>

        <v-combobox
          v-model="selectedPreset"
          :label="t('select-preset')"
          :items="presets"
          item-title="name"
          density="comfortable"
          class="ml-auto"
          style="min-width: 200px;"
        />
      </div>

      <div class="d-flex align-center justify-space-between">
        <span>{{ t('total-selected', { count: selectedCount }) }}</span>
        <v-btn @click="toggleSelectAll" variant="text">
          {{ t(allSelected ? 'none' : 'all') }}
        </v-btn>
      </div>
    </div>

    <div class="batch-table-container">
      <v-table density="comfortable">
        <thead>
        <tr>
          <th></th>
          <th>{{ t('name') }}</th>
          <th>{{ t('level') }}</th>
          <th>{{ t('charter') }}</th>
          <th>{{ t('preset') }}</th>
          <th>{{ t('status') }}</th>
        </tr>
        </thead>
        <tbody>
        <tr v-for="(chart, index) in charts" :key="index">
          <td>
            <v-checkbox v-model="chart.selected" density="compact" hide-details />
          </td>
          <td class="text-truncate" style="max-width: 200px;">{{ chart.name }}</td>
          <td>{{ chart.level }}</td>
          <td>{{ chart.charter }}</td>
          <td>
            <v-select
              v-model="chart.preset"
              :items="presets"
              item-title="name"
              density="compact"
              variant="outlined"
              hide-details
            />
          </td>
          <td>
            <v-chip :color="statusColor(chart.status)" size="small">
              {{ t(chart.status) }}
            </v-chip>
          </td>
        </tr>
        </tbody>
      </v-table>
    </div>

    <div class="mt-auto pt-4 d-flex justify-end">
      <v-btn
        color="primary"
        size="large"
        @click="startRender"
        prepend-icon="mdi-play"
        :disabled="selectedCount === 0"
      >
        {{ t('start-render') }}
      </v-btn>
    </div>
  </div>
</template>

<style scoped>
.batch-table-container {
  max-height: 60vh;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
}

.gap-4 {
  gap: 16px;
}

.text-truncate {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>