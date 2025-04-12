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
import { invoke } from '@tauri-apps/api'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'

import { toast, toastError } from './common'
import type { RPEChart } from './model'
import router from './router'

const { t } = useI18n()

// 响应式数据
const charts = ref<RPEChart[] | null>(null)

// 获取 RPE 图表
const getRPECharts = async () => {
  try {
    charts.value = (await invoke('get_rpe_charts')) as RPEChart[] | null
  } catch (e) {
    toastError(e)
    charts.value = null
  }
}

// 绑定 RPE
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

// 解绑 RPE
const unbindRPE = async () => {
  try {
    await invoke('unset_rpe_dir')
    toast(t('unbinded'), 'success')
    charts.value = null
  } catch (e) {
    toastError(e)
  }
}

// 初始化加载
onMounted(() => {
  getRPECharts()
})
</script>

<template>
  <div class="rpe-container">
    <!-- 未绑定状态 -->
    <template v-if="!charts">
      <h1 class="unbinded-title text-gradient">{{ t('not-binded') }}</h1>
      <div class="d-flex justify-center">
        <v-btn
          size="large"
          color="primary"
          variant="flat"
          class="bind-btn hover-scale"
          @click="bindRPE"
        >
          {{ t('bind') }}
        </v-btn>
      </div>
    </template>

    <!-- 已绑定状态 -->
    <template v-else>
      <div class="d-flex justify-center mb-4">
        <v-btn
          size="large"
          color="error"
          variant="flat"
          class="unbind-btn hover-scale"
          @click="unbindRPE"
        >
          {{ t('unbind') }}
        </v-btn>
      </div>

      <!-- 图表列表 -->
      <v-card
        v-for="chart in charts"
        :key="chart.id"
        class="chart-card"
      >
        <div class="d-flex flex-row align-stretch">
          <!-- 封面 -->
          <div class="chart-cover">
            <div
              class="cover-image"
              :style="{ backgroundImage: 'url(' + convertFileSrc(chart.illustration) + ')' }"
            ></div>
          </div>

          <!-- 内容 -->
          <div class="chart-content">
            <v-card-title class="chart-name">{{ chart.name }}</v-card-title>
            <v-card-subtitle class="chart-id">{{ chart.id }}</v-card-subtitle>
            <div class="d-flex justify-end pa-4">
              <v-btn
                color="accent"
                variant="flat"
                class="render-btn hover-scale"
                @click="router.push({ name: 'render', query: { chart: chart.path } })"
              >
                {{ t('render') }}
              </v-btn>
            </div>
          </div>
        </div>
      </v-card>
    </template>
  </div>
</template>

<style scoped>
.rpe-container {
  padding: 2rem;
  width: 100%;
  height: 100%;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.unbinded-title {
  font-size: 2rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 1.5rem;
}

.bind-btn,
.unbind-btn {
  font-weight: 600;
  padding: 12px 24px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.chart-card {
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.chart-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.3);
}

.chart-cover {
  width: 35%;
  min-height: 240px;
  background: rgba(0, 0, 0, 0.1);
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
  padding: 1rem;
}

.chart-name {
  font-size: 1.5rem;
  font-weight: 600;
}

.chart-id {
  font-size: 0.9rem;
  opacity: 0.7;
}

.render-btn {
  font-weight: 600;
  padding: 8px 16px;
  transition: all 0.3s ease;
}

.hover-scale {
  transition: transform 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #2196f3);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>
