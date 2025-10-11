<i18n>
en:
  empty: Nothing here

  status:
    pending: Pending…
    loading: Loading…
    mixing: Mixing…
    rendering: Rendering { fps } FPS, estimated to end { estimate }
    done: Done, took { duration }
    canceled: Canceled
    failed: Failed

  cancel: Cancel
  confirm: Confirm

  details: Details
  error: Error
  output: Output

  show-output: Show Output
  show-in-folder: Open Folder

zh-CN:
  empty: 空空如也

  status:
    pending: 等待中…
    loading: 加载中…
    mixing: 混音中…
    rendering: 渲染中 { fps } FPS  预计 { estimate } 结束
    done: 已完成，耗时 { duration }
    canceled: 已取消
    failed: 失败

  cancel: 取消
  confirm: 确定

  details: 详情
  error: 错误
  output: 输出

  show-output: 查看输出
  show-in-folder: 打开文件夹

</i18n>

<script setup lang="ts">
import { ref, onUnmounted, reactive, computed } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import type { Task, TaskStatus } from './model';

import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

import moment from 'moment';
import { toastError } from './common';

const tasks = ref<Task[]>();

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

function formatDuration(seconds: number) {
  const duration = moment.duration(Math.ceil(seconds), 'seconds');
  const hours = Math.floor(duration.asHours());
  const minutes = duration.minutes();
  const secs = duration.seconds();

  if (hours > 0) {
    return `${hours}${t('H')} ${minutes}${t('m')} ${secs}${t('s')}`;
  } else if (minutes > 0) {
    return `${minutes}${t('m')} ${secs}${t('s')}`;
  } else if (secs > 0) {
    return `${secs}${t('s')}`;
  } else {
    return '';
  }
}

function describeStatus(status: TaskStatus): string {
  switch (status.type) {
    case 'pending':
      return t('status.pending');
    case 'loading':
      return t('status.loading');
    case 'mixing':
      return t('status.mixing');
    case 'rendering': {
      const progressDisplay =
        status.progress >= 0.999 ? '100.00' : (status.progress * 100).toFixed(2);
      return t('status.rendering', {
        progress: progressDisplay,
        fps: status.fps,
        estimate: status.estimate ? formatDuration(status.estimate) : '',
      });
    }
    case 'done':
      return t('status.done', {
        duration: status.duration ? formatDuration(status.duration) : '',
      });
    case 'canceled':
      return t('status.canceled');
    case 'failed':
      return t('status.failed');
  }
}

function statusColor(statusType: string): string {
  const colors: Record<string, string> = {
    pending: 'info',
    loading: 'info',
    mixing: 'info',
    rendering: 'primary',
    done: 'success',
    canceled: 'warning',
    failed: 'error'
  };
  return colors[statusType] || 'info';
}

const errorDialog = ref(false),
  errorDialogMessage = ref('');

const outputDialog = ref(false),
  outputDialogMessage = ref('');

// 详情对话框
const detailDialog = ref(false),
  selectedTask = ref<Task | null>(null);

// 右键菜单
const contextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuTask = ref<Task | null>(null);

async function showInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path });
  } catch (e) {
    toastError(e);
  }
}

async function showFolder() {
  try {
    await invoke('show_folder');
  } catch (e) {
    toastError(e);
  }
}

function showOutput(task: Task) {
  if (task.status.type === 'done') {
    outputDialogMessage.value = task.status.output;
    outputDialog.value = true;
  }
}

interface CardTransform {
  transform: string;
  transition: string;
}

const cardTransforms = reactive<{ [key: string]: CardTransform }>({});
const defaultTransform = ref('rotateX(0) rotateY(0) scale(1) translateZ(0)');

const effectConfig = {
  rotateYSensitivity: 8,
  rotateXSensitivity: 5,
  translateSensitivity: 10,
  scaleFactor: 1.02,
  translateZ: 20,
  enterDuration: 0.25,
  leaveDuration: 0.5,
  enterEasing: 'cubic-bezier(0.16, 1, 0.3, 1)',
  leaveEasing: 'cubic-bezier(0.23, 1, 0.32, 1)',
};

function handleMouseEnter(taskId: string) {
  const card = document.getElementById(`card-${taskId}`);
  if (!card) return;

  cardTransforms[taskId] = {
    transform: cardTransforms[taskId]?.transform || defaultTransform.value,
    transition: `transform ${effectConfig.enterDuration}s ${effectConfig.enterEasing}`,
  };
}

function handleMouseMove(event: MouseEvent, taskId: string) {
  const card = document.getElementById(`card-${taskId}`);
  if (!card) return;

  const rect = card.getBoundingClientRect();
  const { rotateYSensitivity, rotateXSensitivity, translateSensitivity, scaleFactor, translateZ } = effectConfig;

  const mouseX = event.clientX - rect.left;
  const mouseY = event.clientY - rect.top;

  const rotateY = (mouseX / rect.width - 0.5) * rotateYSensitivity;
  const rotateX = (mouseY / rect.height - 0.5) * -rotateXSensitivity;

  const translateX = (mouseX / rect.width - 0.5) * translateSensitivity;
  const translateY = (mouseY / rect.height - 0.5) * translateSensitivity;

  cardTransforms[taskId] = {
    transform: `
        rotateY(${rotateY}deg)
        rotateX(${rotateX}deg)
        translateZ(${translateZ}px)
        scale(${scaleFactor})
        translate(${translateX}px, ${translateY}px)
      `,
    transition: cardTransforms[taskId].transition,
  };
}

function handleMouseLeave(taskId: string) {
  const card = document.getElementById(`card-${taskId}`);
  if (!card) return;

  cardTransforms[taskId] = {
    transform: cardTransforms[taskId].transform,
    transition: `transform ${effectConfig.leaveDuration}s ${effectConfig.leaveEasing}`,
  };

  requestAnimationFrame(() => {
    cardTransforms[taskId] = {
      transform: defaultTransform.value,
      transition: `transform ${effectConfig.leaveDuration}s ${effectConfig.leaveEasing}`,
    };

    setTimeout(() => {
      if (cardTransforms[taskId]?.transform === defaultTransform.value) {
        delete cardTransforms[taskId];
      }
    }, effectConfig.leaveDuration * 1000);
  });
}

// 进度环动画
const progressRing = (progress: number) => {
  const normalizedProgress = Math.min(100, Math.max(0, progress * 100));
  const circumference = 2 * Math.PI * 36;
  const offset = circumference - (normalizedProgress / 100) * circumference;
  return {
    strokeDasharray: `${circumference} ${circumference}`,
    strokeDashoffset: offset,
  };
};

// 右键菜单处理
const showContextMenu = (event: MouseEvent, task: Task) => {
  event.preventDefault();
  contextMenu.value = true;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuTask.value = task;
};

const closeContextMenu = () => {
  contextMenu.value = false;
  contextMenuTask.value = null;
};

// 显示详情
const showDetail = (task: Task) => {
  selectedTask.value = task;
  detailDialog.value = true;
  closeContextMenu();
};

// 点击其他地方关闭右键菜单
document.addEventListener('click', closeContextMenu);
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1.5rem">
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled" v-t="'empty'"></h1>

    <v-slide-y-transition>
      <v-form ref="form" class="animated-form">
        <v-row class="text-center">
          <v-col cols="12">
            <v-btn
              @click="showFolder()"
              v-t="'show-in-folder'"
              class="hover-scale"
              color="primary"
              prepend-icon="mdi-folder-open-outline"
            ></v-btn>
          </v-col>
        </v-row>
      </v-form>
    </v-slide-y-transition>

    <div class="task-grid">
      <div
        class="task-card-container"
        v-for="task in tasks"
        :key="task.id"
        @mouseenter="handleMouseEnter(task.id.toString())"
        @mousemove="handleMouseMove($event, task.id.toString())"
        @mouseleave="handleMouseLeave(task.id.toString())"
      >
        <v-card
          class="task-card"
          :id="'card-' + task.id.toString()"
          :style="{
            transform: cardTransforms[task.id.toString()]?.transform,
            transition: cardTransforms[task.id.toString()]?.transition,
          }"
          @contextmenu="showContextMenu($event, task)"
        >
          <div class="d-flex flex-row align-stretch">
            <!-- 封面 -->
            <div class="task-cover-section">
              <div
                class="task-cover"
                :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"
              ></div>
            </div>

            <!-- 内容 -->
            <div class="task-content">
              <div class="d-flex justify-space-between align-start pa-4 pb-2">
                <div>
                  <v-card-title class="task-name pa-0">{{ task.name }}</v-card-title>
                  <v-card-subtitle class="task-id pa-0">{{ task.path }}</v-card-subtitle>
                </div>
                <v-chip
                  class="status-badge"
                  :color="statusColor(task.status.type)"
                  label
                  size="small"
                >
                  {{ task.status.type.toUpperCase() }}
                </v-chip>
              </div>

              <!-- 状态区域 -->
              <div class="status-section">
                <div class="d-flex align-center">
                  <!-- 动态进度指示器 -->
                  <template v-if="['loading', 'mixing', 'pending'].includes(task.status.type)">
                    <div class="progress-indicator mr-4">
                      <v-progress-circular
                        indeterminate
                        :color="statusColor(task.status.type)"
                        size="40"
                        width="3"
                      ></v-progress-circular>
                    </div>
                  </template>

                  <template v-else-if="task.status.type === 'rendering'">
                    <div class="progress-indicator mr-4">
                      <v-progress-circular
                        :model-value="task.status.progress * 100"
                        :color="statusColor(task.status.type)"
                        size="40"
                        width="3"
                      >{{ Math.round(task.status.progress * 100) }}%</v-progress-circular>
                    </div>
                  </template>

                  <div class="flex-grow-1">
                    <p class="status-text mb-1">{{ describeStatus(task.status) }}</p>

                    <template v-if="task.status.type === 'rendering'">
                      <v-progress-linear
                        :model-value="task.status.progress * 100"
                        :color="statusColor(task.status.type)"
                        height="6"
                        rounded
                      ></v-progress-linear>
                    </template>
                  </div>
                </div>
              </div>

              <!-- 操作按钮 -->
              <div class="d-flex justify-end pa-4 pt-2">
                <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
                  <v-btn
                    color="error"
                    variant="flat"
                    prepend-icon="mdi-cancel"
                    @click="invoke('cancel_task', { id: task.id })"
                    v-t="'cancel'"
                    class="action-btn"
                  ></v-btn>
                </template>

                <template v-else-if="task.status.type === 'failed'">
                  <v-btn
                    color="error"
                    variant="flat"
                    prepend-icon="mdi-alert-circle-outline"
                    @click="
                      errorDialogMessage = task.status.error;
                      errorDialog = true;
                    "
                    v-t="'details'"
                    class="action-btn"
                  ></v-btn>
                </template>

                <template v-else-if="task.status.type === 'done'">
                  <v-btn
                    color="secondary"
                    variant="flat"
                    prepend-icon="mdi-text-box-outline"
                    @click="showOutput(task)"
                    v-t="'show-output'"
                    class="action-btn mr-2"
                  ></v-btn>
                  <v-btn
                    color="accent"
                    variant="flat"
                    prepend-icon="mdi-folder-open-outline"
                    @click="showInFolder(task.output)"
                    v-t="'show-in-folder'"
                    class="action-btn"
                  ></v-btn>
                </template>
              </div>
            </div>
          </div>
        </v-card>
      </div>
    </div>

    <!-- 错误对话框 -->
    <v-dialog
      v-model="errorDialog"
      width="auto"
      min-width="400px"
      scrim="#000000CC"
      persistent
      transition="dialog-bottom-transition"
    >
      <v-card class="error-dialog">
        <v-card-title class="d-flex align-center">
          <v-icon color="error" class="mr-2">mdi-alert</v-icon>
          <span class="text-error">Error</span>
        </v-card-title>
        <v-divider></v-divider>
        <v-card-text>
          <pre class="error-message">{{ errorDialogMessage }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="error" variant="text" @click="errorDialog = false">
            Close
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 输出对话框 -->
    <v-dialog v-model="outputDialog" width="auto" min-width="400px">
      <v-card class="glass-background">
        <v-card-title class="text-gradient" v-t="'output'"></v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ outputDialogMessage }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="flat" @click="outputDialog = false" v-t="'confirm'" class="hover-scale"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- 右键菜单 -->
    <v-menu
      v-model="contextMenu"
      :position-x="contextMenuX"
      :position-y="contextMenuY"
      absolute
      offset-y
    >
      <v-list class="context-menu">
        <v-list-item @click="showDetail(contextMenuTask!)">
          <template v-slot:prepend>
            <v-icon>mdi-information</v-icon>
          </template>
          <v-list-item-title>详细信息</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>

    <!-- 详情对话框 -->
    <v-dialog v-model="detailDialog" width="auto" min-width="500px">
      <v-card class="glass-background" v-if="selectedTask">
        <v-card-title class="text-gradient">任务详情</v-card-title>
        <v-divider></v-divider>
        <v-card-text>
          <div class="detail-section">
            <div class="detail-item">
              <span class="detail-label">名称:</span>
              <span class="detail-value">{{ selectedTask.name }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">路径:</span>
              <span class="detail-value">{{ selectedTask.path }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">状态:</span>
              <span class="detail-value">{{ describeStatus(selectedTask.status) }}</span>
            </div>
            <div class="detail-item" v-if="selectedTask.status.type === 'rendering'">
              <span class="detail-label">进度:</span>
              <span class="detail-value">{{ Math.round(selectedTask.status.progress * 100) }}%</span>
            </div>
            <div class="detail-item" v-if="selectedTask.status.type === 'rendering'">
              <span class="detail-label">帧率:</span>
              <span class="detail-value">{{ selectedTask.status.fps }} FPS</span>
            </div>
            <div class="detail-item" v-if="selectedTask.status.type === 'done' && selectedTask.status.duration">
              <span class="detail-label">耗时:</span>
              <span class="detail-value">{{ formatDuration(selectedTask.status.duration) }}</span>
            </div>
            <div class="detail-item" v-if="selectedTask.status.type === 'done' && selectedTask.output">
              <span class="detail-label">输出:</span>
              <span class="detail-value">{{ selectedTask.output }}</span>
            </div>
            <div class="detail-item" v-if="selectedTask.status.type === 'failed' && selectedTask.status.error">
              <span class="detail-label">错误:</span>
              <span class="detail-value error-text">{{ selectedTask.status.error }}</span>
            </div>
          </div>
        </v-card-text>
        <v-divider></v-divider>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="flat" @click="detailDialog = false" v-t="'confirm'" class="hover-scale"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
.task-grid {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  max-width: 800px;
  margin: 0 auto;
}

.task-card-container {
  perspective: 916px;
  transform-style: preserve-3d;
}

.task-card {
  will-change: transform;
  backface-visibility: hidden;
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.task-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.3);
}

.task-cover-section {
  width: 35%;
  min-height: 160px;
  background: rgba(0, 0, 0, 0.1);
}

.task-cover {
  width: 100%;
  height: 100%;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
}

.task-content {
  width: 65%;
  padding: 1rem;
  display: flex;
  flex-direction: column;
}

.task-name {
  font-size: 1.2rem;
  font-weight: 600;
}

.task-id {
  font-size: 0.9rem;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}

.status-section {
  margin-top: auto;
  padding: 0.5rem 0;
}

.progress-indicator {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-badge {
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  font-size: 0.7rem;
}

.status-text {
  font-weight: 500;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.85);
}

.progress-ring {
  position: relative;
  width: 80px;
  height: 80px;
}

.progress-ring__circle {
  transform: rotate(-90deg);
}

.progress-ring__circle-bg {
  stroke: rgba(255, 255, 255, 0.1);
}

.progress-ring__circle-fg {
  stroke: currentColor;
  transition: stroke-dashoffset 0.5s ease;
}

.progress-ring__text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 1rem;
  font-weight: bold;
  color: white;
}

.progress-ring__indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.glow-bar {
  box-shadow: 0 0 8px currentColor;
}

.glow-primary {
  filter: drop-shadow(0 0 6px #2196F3);
}

.glow-info {
  filter: drop-shadow(0 0 6px #21CBF3);
}

.glow-success {
  filter: drop-shadow(0 0 6px #4CAF50);
}

.glow-warning {
  filter: drop-shadow(0 0 6px #FFC107);
}

.glow-error {
  filter: drop-shadow(0 0 6px #F44336);
}

.hover-scale {
  transition:
    transform 0.3s ease,
    box-shadow 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.action-btn {
  font-weight: 600;
  padding: 8px 16px;
  transition: all 0.3s ease;
}

.v-btn:active {
  transform: scale(0.95);
}

.error-dialog {
  background: #1E1E1E;
  border-radius: 16px !important;
  border: 2px solid #FF5252;
  box-shadow: 0 0 20px rgba(255, 82, 82, 0.3);
}

.error-message {
  color: #FF5252;
  font-family: monospace;
  background: rgba(0, 0, 0, 0.5);
  padding: 12px;
  border-radius: 8px;
  max-height: 60vh;
  overflow: auto;
  line-height: 1.5;
}

.glass-background {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.text-gradient {
  background: linear-gradient(45deg, #2196f3, #f32196);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

pre {
  background: rgba(0, 0, 0, 0.3) !important;
  padding: 12px !important;
  border-radius: 8px;
  font-family: 'Fira Code', monospace;
  font-size: 0.9rem;
}

/* 右键菜单样式 */
.context-menu {
  background: rgba(30, 30, 30, 0.9) !important;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  border-radius: 8px !important;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3) !important;
}

/* 详情对话框样式 */
.detail-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-item {
  display: flex;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.detail-item:last-child {
  border-bottom: none;
}

.detail-label {
  width: 80px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
  flex-shrink: 0;
}

.detail-value {
  flex-grow: 1;
  word-break: break-all;
  color: rgba(255, 255, 255, 0.9);
}

.error-text {
  color: #ff5252;
  font-family: monospace;
  font-size: 0.9rem;
  background: rgba(255, 82, 82, 0.1);
  padding: 8px;
  border-radius: 4px;
  margin-top: 4px;
}
</style>