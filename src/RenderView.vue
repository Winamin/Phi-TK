<i18n>
en:
  already-running: Phi TK is already running

  prev-step: Previous
  next-step: Next
  steps:
    choose: 'Choose the chart'
    config: 'Configure chart'
    options: 'Render options'
    render: 'Render'

  choose:
    archive: Archive (.zip, .pez)
    folder: Folder
    can-also-drop: You can also drag & drop the file to here
    drop: DROP CHART HERE

  chart-file: Chart file

  chart-name: Chart name
  charter: Charter
  illustrator: Illustrator
  level: Level
  aspect: Aspect ratio
  dim: Background dim

  tip: Tip
  tip-placeholder: Leave empty to choose randomly

  width: Width
  height: Height

  file:
    title: File
    chart: Chart file (empty for default)
    music: Music (empty for default)
    illustration: Illustration (empty for default)

  preview: Preview
  render: Render

  render-started: Rendering has started!
  see-tasks: See tasks

  ffmpeg-not-found: You haven't installed ffmpeg yet. Please download FFmpeg.exe and put it in the specific folder.

zh-CN:
  already-running: Phi TK 已经在运行

  prev-step: 上一步
  next-step: 下一步
  steps:
    choose: '选择谱面'
    config: '配置谱面'
    options: '渲染参数'
    render: '渲染视频'

  choose:
    archive: 压缩包 (.zip, .pez)
    folder: 文件夹
    can-also-drop: 也可以直接拖谱面到窗口哦～
    drop: 拖放谱面到这

  chart-file: 谱面文件

  chart-name: 谱面名
  charter: 谱师
  composer: 曲师
  illustrator: 画师
  level: 难度
  aspect: 宽高比
  dim: 背景昏暗程度

  tip: Tip
  tip-placeholder: 留空则随机选择

  width: 宽
  height: 高

  preview: 预览
  render: 渲染

  render-started: 视频开始渲染了！
  see-tasks: 查看任务列表

  ffmpeg-not-found: 笨蛋怎么没安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。

</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke, event, dialog, shell } from '@tauri-apps/api'
import { toastError, RULES, toast, anyFilter, isString } from './common'
import type { ChartInfo } from './model'
import { VForm } from 'vuetify/components'
import ConfigView from './components/ConfigView.vue'
import moment from 'moment'

const { t } = useI18n()
const router = useRouter()

// 检查是否唯一实例
if (!(await invoke('is_the_only_instance'))) {
  await dialog.message(t('already-running'))
  await invoke('exit_program')
}

// 步骤管理
const steps = ['choose', 'config', 'options', 'render']
const stepIndex = ref(1)
const step = computed(() => steps[stepIndex.value - 1])

// 图表信息
const chartInfo = ref<ChartInfo>()
let chartPath = ''

// 状态管理
const choosingChart = ref(false)
const parsingChart = ref(false)
const fileHovering = ref(false)

// 表单引用
const form = ref<VForm>()
const configView = ref<typeof ConfigView>()

// 宽高比
const aspectWidth = ref('0')
const aspectHeight = ref('0')

// 渲染状态
const renderMsg = ref('')
const renderProgress = ref<number>()
const renderDuration = ref<number>()

// 初始化加载
const chartInQuery = router.currentRoute.value.query.chart
if (isString(chartInQuery)) {
  onMounted(() => loadChart(chartInQuery as string))
}

// 选择图表
async function chooseChart(folder = false) {
  if (choosingChart.value) return
  choosingChart.value = true

  const file = folder
    ? await dialog.open({ directory: true })
    : await dialog.open({
        filters: [
          {
            name: t('choose.filter-name'),
            extensions: ['zip', 'pez'],
          },
          anyFilter(),
        ],
      })

  if (file) await loadChart(file as string)
  choosingChart.value = false
}

// 加载图表
async function loadChart(file: string) {
  try {
    parsingChart.value = true
    chartPath = file
    chartInfo.value = (await invoke('parse_chart', { path: file })) as ChartInfo
    stepIndex.value++

    // 设置宽高比
    aspectWidth.value = String(chartInfo.value.aspectRatio)
    aspectHeight.value = '1.0'
    for (const asp of [
      [16, 9],
      [4, 3],
      [8, 5],
      [3, 2],
    ]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.value.aspectRatio) < 1e-4) {
        aspectWidth.value = String(asp[0])
        aspectHeight.value = String(asp[1])
        break
      }
    }
  } catch (e) {
    toastError(e)
  } finally {
    parsingChart.value = false
  }
}

// 构建参数
async function buildParams() {
  const config = await configView.value!.buildConfig()
  if (!config) return null
  if (!chartInfo.value!.tip?.trim().length) chartInfo.value!.tip = null
  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  }
}

// 渲染
async function postRender() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      await dialog.message(t('ffmpeg-not-found'))
      await invoke('open_app_folder')
      await shell.open('https://mivik.moe/ffmpeg-windows/')
      return false
    }
    const params = await buildParams()
    if (!params) return false
    await invoke('post_render', { params })
    return true
  } catch (e) {
    toastError(e)
    return false
  }
}

// 预览
async function previewChart() {
  try {
    const params = await buildParams()
    if (!params) return false
    await invoke('preview_chart', { params })
    return true
  } catch (e) {
    toastError(e)
    return false
  }
}

// 下一步
async function moveNext() {
  if (step.value === 'config') {
    if ((await form.value!.validate()).valid) {
      stepIndex.value++
      configView.value!.onEnter()
    } else {
      toast(t('has-error'), 'error')
    }
    return
  }
  if (step.value === 'options' && (await postRender())) {
    stepIndex.value++
  }
}

// 事件监听
event.listen('tauri://file-drop-hover', () => (fileHovering.value = step.value === 'choose'))
event.listen('tauri://file-drop-cancelled', () => (fileHovering.value = false))
event.listen('tauri://file-drop', async (event) => {
  if (step.value === 'choose') {
    fileHovering.value = false
    await loadChart((event.payload as string[])[0])
  }
})

event.listen('render-msg', (msg) => (renderMsg.value = msg.payload as string))
event.listen('render-progress', (msg) => {
  const payload = msg.payload as { progress: number; fps: number; estimate: number }
  renderMsg.value = t('render-status', {
    progress: (payload.progress * 100).toFixed(2),
    fps: payload.fps,
    estimate: moment.duration(payload.estimate, 'seconds').humanize(true, { ss: 1 }),
  })
  renderProgress.value = payload.progress * 100
})
event.listen('render-done', (msg) => {
  stepIndex.value++
  renderDuration.value = Math.round(msg.payload as number)
})

// 解析宽高比
function tryParseAspect(): number | undefined {
  try {
    const width = parseFloat(aspectWidth.value)
    const height = parseFloat(aspectHeight.value)
    if (isNaN(width) || isNaN(height)) return undefined
    return width / height
  } catch (e) {
    return undefined
  }
}
</script>

<template>
  <div class="render-container">
    <v-stepper alt-labels v-model="stepIndex" hide-actions :items="steps.map((x) => t('steps.' + x))">
      <!-- Step 1: Choose Chart -->
      <template v-slot:item.1>
        <div class="step-choose">
          <div class="file-options">
            <v-btn class="file-option" size="large" color="primary" @click="chooseChart(false)" prepend-icon="mdi-folder-zip">
              {{ t('choose.archive') }}
            </v-btn>
            <v-divider vertical></v-divider>
            <v-btn class="file-option" size="large" color="primary" @click="chooseChart(true)" prepend-icon="mdi-folder">
              {{ t('choose.folder') }}
            </v-btn>
          </div>
          <p class="drop-hint">{{ t('choose.can-also-drop') }}</p>
          <v-overlay v-model="parsingChart" contained class="loading-overlay" persistent>
            <v-progress-circular indeterminate></v-progress-circular>
          </v-overlay>
        </div>
      </template>

      <!-- Step 2: Configure Chart -->
      <template v-slot:item.2>
        <v-form ref="form" v-if="chartInfo" class="step-config">
          <v-row no-gutters class="form-row">
            <v-col cols="8">
              <v-text-field :label="t('chart-name')" :rules="[RULES.non_empty]" v-model="chartInfo.name"></v-text-field>
            </v-col>
            <v-col cols="4">
              <v-text-field :label="t('level')" :rules="[RULES.non_empty]" v-model="chartInfo.level"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="form-row">
            <v-col cols="12" sm="4">
              <v-text-field :label="t('charter')" :rules="[RULES.non_empty]" v-model="chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field :label="t('composer')" v-model="chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field :label="t('illustrator')" v-model="chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="form-row align-center">
            <v-col cols="4">
              <div class="aspect-ratio">
                <p class="aspect-label">{{ t('aspect') }}</p>
                <div class="aspect-inputs">
                  <v-text-field type="number" :rules="[RULES.positive]" :label="t('width')" v-model="aspectWidth"></v-text-field>
                  <span>:</span>
                  <v-text-field type="number" :rules="[RULES.positive]" :label="t('height')" v-model="aspectHeight"></v-text-field>
                </div>
              </div>
            </v-col>
            <v-col cols="8">
              <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.05" v-model="chartInfo.backgroundDim"></v-slider>
            </v-col>
          </v-row>

          <v-row no-gutters class="form-row">
            <v-col cols="12">
              <v-text-field :label="t('tip')" :placeholder="t('tip-placeholder')" v-model="chartInfo.tip"></v-text-field>
            </v-col>
          </v-row>
        </v-form>
      </template>

      <!-- Step 3: Render Options -->
      <template v-slot:item.3>
        <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()"></ConfigView>
      </template>

      <!-- Step 4: Render -->
      <template v-slot:item.4>
        <div class="step-render">
          <span class="render-emoji">✨</span>
          <h2>{{ t('render-started') }}</h2>
          <v-btn @click="router.push({ name: 'tasks' })">{{ t('see-tasks') }}</v-btn>
        </div>
      </template>
    </v-stepper>

    <!-- File Drop Overlay -->
    <v-overlay v-model="fileHovering" contained class="drop-overlay" persistent>
      <h1>{{ t('choose.drop') }}</h1>
    </v-overlay>
  </div>
</template>

<style scoped>
.render-container {
  padding: 2rem;
  width: 100%;
  height: 100%;
  max-width: 1280px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.step-choose {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  padding: 2rem 0;
}

.file-options {
  display: flex;
  gap: 2rem;
  width: 100%;
  justify-content: center;
}

.file-option {
  width: 45%;
  padding: 1.5rem;
  font-size: 1.1rem;
  font-weight: 600;
}

.drop-hint {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
}

.loading-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
}

.step-config {
  padding: 2rem;
}

.form-row {
  margin-bottom: 1.5rem;
}

.aspect-ratio {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.aspect-label {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.7);
}

.aspect-inputs {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.step-render {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  padding: 2rem 0;
}

.render-emoji {
  font-size: 84px;
}

.drop-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
}

.v-progress-linear,
.v-progress-linear__determinate {
  transition: none;
}
</style>
