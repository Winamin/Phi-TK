<i18n>
en:
  not-binded: RPE is not connected
  empty: No charts found in this RPE folder
  bind: Bind RPE
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder
  render: Render

zh-CN:
  not-binded: 你还没有绑定 RPE
  empty: 此 RPE 文件夹中还没有谱面
  bind: 绑定 RPE
  binded: 绑定成功
  unbind: 解绑 RPE
  unbinded: 解绑成功
  rpe-folder: 请选择 RPE 所在文件夹
  render: 渲染
</i18n>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import { toast, toastError } from './common';
import type { RPEChart } from './model';
import router from './router';

const { t } = useI18n();

const charts = ref<RPEChart[] | null>(null);

const getRPECharts = async () => {
  try {
    charts.value = (await invoke('get_rpe_charts')) as RPEChart[] | null;
  } catch (e) {
    toastError(e);
    charts.value = null;
  }
};

const bindRPE = async () => {
  const file = await open({ directory: true, title: t('rpe-folder') });
  if (!file) return;
  try {
    await invoke('set_rpe_dir', { path: file });
    toast(t('binded'), 'success');
    await getRPECharts();
  } catch (e) {
    toastError(e);
  }
};

const unbindRPE = async () => {
  try {
    await invoke('unset_rpe_dir');
    toast(t('unbinded'), 'success');
    charts.value = null;
  } catch (e) {
    toastError(e);
  }
};

onMounted(() => {
  getRPECharts();
});
</script>

<template>
  <div class="rpe-container">
    <template v-if="!charts">
      <div class="empty-state">
        <mdui-icon-link-off class="empty-icon"></mdui-icon-link-off>
        <h2 class="empty-title md3-title-large">{{ t('not-binded') }}</h2>
        <mdui-button variant="filled" @click="bindRPE">
          <mdui-icon-link slot="icon"></mdui-icon-link>
          {{ t('bind') }}
        </mdui-button>
      </div>
    </template>

    <template v-else>
      <div class="rpe-header">
        <mdui-button variant="text" class="unbind-btn" @click="unbindRPE">
          <mdui-icon-link-off slot="icon"></mdui-icon-link-off>
          {{ t('unbind') }}
        </mdui-button>
      </div>

      <div class="chart-list">
        <p v-if="!charts.length" class="md3-body">{{ t('empty') }}</p>
        <mdui-card v-for="chart in charts" :key="chart.id" variant="filled" class="chart-card">
          <div class="chart-cover">
            <div class="cover-image" :style="{ backgroundImage: 'url(' + convertFileSrc(chart.illustration) + ')' }"></div>
          </div>
          <div class="chart-content">
            <h3 class="chart-name md3-title-medium">{{ chart.name }}</h3>
            <p class="chart-id">{{ chart.id }}</p>
            <div class="chart-action">
              <mdui-button variant="tonal" @click="router.push({ name: 'render', query: { chart: chart.path } })">
                <mdui-icon-play-circle--outlined slot="icon"></mdui-icon-play-circle--outlined>
                {{ t('render') }}
              </mdui-button>
            </div>
          </div>
        </mdui-card>
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
.rpe-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: var(--app-page-padding, 16px);
}
.empty-state {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
}
.empty-icon {
  font-size: 48px;
  color: rgb(var(--mdui-color-on-surface-variant));
}
.empty-title {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface));
  text-align: center;
}
.rpe-header {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}
.unbind-btn {
  --mdui-color-primary: var(--mdui-color-error);
}
.chart-list {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  overscroll-behavior-y: contain;
}
.chart-card {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  flex-shrink: 0;
  gap: 16px;
  padding: var(--app-card-padding, 16px);
  border-radius: var(--mdui-shape-corner-medium, 12px);
  background-color: rgb(var(--mdui-color-surface-container-low));
}
.chart-cover {
  width: 72px;
  height: 72px;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: var(--mdui-shape-corner-small, 8px);
  background-color: rgb(var(--mdui-color-surface-container-highest));
}
.cover-image {
  width: 100%;
  height: 100%;
  background: center / cover no-repeat;
}
.chart-content {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 8px;
}
.chart-name {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface));
  overflow-wrap: anywhere;
}
.chart-id {
  margin: 0;
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow-wrap: anywhere;
}
.chart-action {
  display: flex;
  justify-content: flex-end;
  margin-top: 4px;
}
</style>
