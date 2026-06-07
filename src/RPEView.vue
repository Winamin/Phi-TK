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
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

import { toast, toastError } from './common'
import type { RPEChart } from './model'
import router from './router'

const { t } = useI18n()

const charts = ref<RPEChart[] | null>(null)

const getRPECharts = async () => {
  try {
    charts.value = (await invoke('get_rpe_charts')) as RPEChart[] | null
  } catch (e) {
    toastError(e)
    charts.value = null
  }
}

const bindRPE = async () => {
  const file = await open({ directory: true, title: t('rpe-folder') })
  if (!file) return
  try {
    await invoke('set_rpe_dir', { path: file })
    toast(t('binded'), 'success')
    await getRPECharts()
  } catch (e) {
    toastError(e)
  }
}

const unbindRPE = async () => {
  try {
    await invoke('unset_rpe_dir')
    toast(t('unbinded'), 'success')
    charts.value = null
  } catch (e) {
    toastError(e)
  }
}

onMounted(() => {
  getRPECharts()
})
</script>

<template>
  <div class="rpe-container">
    <!-- Unbinded state -->
    <template v-if="!charts">
      <div class="empty-state">
        <h2 class="empty-title">{{ t('not-binded') }}</h2>
        <button class="md3-btn md3-btn-filled" @click="bindRPE">
          <v-icon icon="mdi-link-variant" size="18" />
          <span>{{ t('bind') }}</span>
        </button>
      </div>
    </template>

    <!-- Binded state -->
    <template v-else>
      <div class="rpe-header">
        <button class="md3-btn md3-btn-text" @click="unbindRPE" style="color: #ff5252;">
          <v-icon icon="mdi-link-off" size="18" />
          <span>{{ t('unbind') }}</span>
        </button>
      </div>

      <div class="chart-list">
        <div
          v-for="chart in charts"
          :key="chart.id"
          class="chart-card"
        >
          <div class="chart-cover">
            <div
              class="cover-image"
              :style="{ backgroundImage: 'url(' + convertFileSrc(chart.illustration) + ')' }"
            ></div>
          </div>
          <div class="chart-content">
            <h3 class="chart-name">{{ chart.name }}</h3>
            <p class="chart-id">{{ chart.id }}</p>
            <div class="chart-action">
              <button class="md3-btn md3-btn-filled" @click="router.push({ name: 'render', query: { chart: chart.path } })">
                <v-icon icon="mdi-play-circle-outline" size="18" />
                <span>{{ t('render') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
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
.md3-btn-filled { background: #82b1ff; color: #002f65; font-weight: 600; }
.md3-btn-filled:hover { background: #a0c4ff; box-shadow: 0 2px 8px rgba(130, 177, 255, 0.3); }
.md3-btn-text { background: transparent; color: rgba(255, 255, 255, 0.7); }
.md3-btn-text:hover { background: rgba(255, 255, 255, 0.08); }

/* ===== Empty State ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 24px;
}

.empty-title {
  font-size: 22px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  margin: 0;
  text-align: center;
}

/* ===== Header ===== */
.rpe-header {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}

/* ===== Chart List ===== */
.chart-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  flex: 1;
}

/* ===== Chart Card (RPEView style, MD3 enhanced) ===== */
.chart-card {
  display: flex;
  flex-direction: row;
  background: rgba(25, 25, 25, 0.85);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.chart-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  border-color: rgba(255, 255, 255, 0.12);
}

.chart-cover {
  width: 35%;
  min-height: 180px;
  background: rgba(0, 0, 0, 0.15);
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
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin: 0;
}

.chart-id {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  margin: 0;
  font-family: 'Consolas', monospace;
}

.chart-action {
  margin-top: auto;
  display: flex;
  justify-content: flex-end;
}

/* ===== Responsive ===== */
@media (max-width: 600px) {
  .rpe-container { padding: 16px; }
  .chart-card { flex-direction: column; }
  .chart-cover { width: 100%; min-height: 160px; }
  .chart-content { width: 100%; }
}
</style>
