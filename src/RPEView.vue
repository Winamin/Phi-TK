<i18n>
en:
  not-binded: You have not binded RPE yet
  bind: Bind RPE
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder
  render: Render

zh-CN:
  not-binded: 你还没有绑定 RPE
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
    <!-- Unbinded state -->
    <template v-if="!charts">
      <div class="empty-state">
        <mdui-icon-link-off class="empty-icon"></mdui-icon-link-off>
        <h2 class="empty-title md3-headline">{{ t('not-binded') }}</h2>
        <mdui-button variant="filled" @click="bindRPE">
          <mdui-icon-link slot="icon"></mdui-icon-link>
          {{ t('bind') }}
        </mdui-button>
      </div>
    </template>

    <!-- Binded state -->
    <template v-else>
      <div class="rpe-header">
        <mdui-button variant="text" class="unbind-btn" @click="unbindRPE">
          <mdui-icon-link-off slot="icon"></mdui-icon-link-off>
          {{ t('unbind') }}
        </mdui-button>
      </div>

      <div class="chart-list">
        <mdui-card v-for="chart in charts" :key="chart.id" variant="filled" class="chart-card">
          <div class="chart-cover">
            <div
              class="cover-image"
              :style="{ backgroundImage: 'url(' + convertFileSrc(chart.illustration) + ')' }"></div>
          </div>
          <div class="chart-content">
            <h3 class="chart-name md3-title-large">{{ chart.name }}</h3>
            <p class="chart-id">{{ chart.id }}</p>
            <div class="chart-action">
              <mdui-button
                variant="filled"
                @click="router.push({ name: 'render', query: { chart: chart.path } })">
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
@use './styles/breakpoints' as bp;
@use './styles/motion' as mo;

.rpe-container {
  padding: 24px;
  width: 100%;
  height: 100vh;
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ===== Empty State ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 24px;
}

.empty-icon {
  font-size: 4rem;
  color: rgb(var(--mdui-color-outline));
}

.empty-title {
  color: rgb(var(--mdui-color-on-surface-variant));
  margin: 0;
  text-align: center;
}

/* ===== Header ===== */
.rpe-header {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}

/* Unbinding is destructive, so the action carries the error colour. */
.unbind-btn {
  --mdui-color-primary: var(--mdui-color-error);
}

/* ===== Chart List ===== */
.chart-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  flex: 1;
  padding: 2px;
}

/* `<mdui-card>` is `display: inline-block` in its shadow root; a document-level rule
   outranks `:host`, so this makes it a row. */
.chart-card {
  display: flex;
  flex-direction: row;
  /* Same trap as the tasks list: a column flex container squeezes its items before it
     scrolls, and `mdui-card` clips, so a compressed card cuts through its own text.
     The card keeps its natural height and `.chart-list` scrolls instead. */
  flex-shrink: 0;
  @include mo.spatial(box-shadow);

  @include mo.enter-rise(14px);
  @include mo.stagger(10, 45ms, 40ms);
}

.chart-card:hover {
  box-shadow: var(--mdui-elevation-level2);
}

.chart-cover {
  width: 35%;
  min-height: 180px;
  background-color: rgb(var(--mdui-color-surface-container-low));
}

.cover-image {
  width: 100%;
  height: 100%;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
}

.chart-content {
  width: 65%;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chart-name {
  color: rgb(var(--mdui-color-on-surface));
  margin: 0;
}

.chart-id {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  margin: 0;
  font-family: 'Roboto Mono', 'Consolas', monospace;
}

.chart-action {
  margin-top: auto;
  display: flex;
  justify-content: flex-end;
}

/* ===== Responsive ===== */
@include bp.compact {
  .rpe-container {
    padding: 16px;
  }
  .chart-card {
    flex-direction: column;
  }
  .chart-cover {
    width: 100%;
    min-height: 160px;
  }
  .chart-content {
    width: 100%;
  }
}
</style>
