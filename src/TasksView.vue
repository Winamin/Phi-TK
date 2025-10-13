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
  <div class="pa-6 w-100 h-100 d-flex flex-column">
    <h1 v-if="!tasks || !tasks.length" class="text-center text-disabled empty-state" v-t="'empty'"></h1>

    <div class="task-actions mb-4">
      <v-btn
        @click="showFolder()"
        v-t="'show-in-folder'"
        variant="elevated"
        color="primary"
        prepend-icon="mdi-folder-open-outline"
      ></v-btn>
    </div>

    <div class="task-list">
      <div
        class="task-item"
        v-for="task in tasks"
        :key="task.id"
        @contextmenu="showContextMenu($event, task)"
      >
        <div class="task-cover">
          <div
            class="cover-image"
            :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"
          ></div>
        </div>

        <div class="task-info">
          <div class="task-header">
            <h3 class="task-name" :title="task.name">{{ task.name }}</h3>
            <v-chip
              :color="statusColor(task.status.type)"
              size="small"
              variant="elevated"
              class="flex-shrink-0"
            >
              {{ task.status.type.toUpperCase() }}
            </v-chip>
          </div>

          <p class="task-path" :title="task.path">{{ task.path }}</p>

          <div class="task-status">
            <div class="status-indicator">
              <template v-if="['loading', 'mixing', 'pending'].includes(task.status.type)">
                <v-progress-circular
                  indeterminate
                  :color="statusColor(task.status.type)"
                  size="24"
                  width="2"
                ></v-progress-circular>
              </template>
              <template v-else-if="task.status.type === 'rendering'">
                <v-progress-circular
                  :model-value="task.status.progress * 100"
                  :color="statusColor(task.status.type)"
                  size="24"
                  width="2"
                ></v-progress-circular>
              </template>
            </div>
            <p class="status-text">{{ describeStatus(task.status) }}</p>
          </div>

          <template v-if="task.status.type === 'rendering'">
            <v-progress-linear
              :model-value="task.status.progress * 100"
              :color="statusColor(task.status.type)"
              height="4"
              rounded
            ></v-progress-linear>
          </template>

          <div class="task-actions">
            <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
              <v-btn
                size="x-small"
                color="error"
                variant="text"
                prepend-icon="mdi-cancel"
                @click="invoke('cancel_task', { id: task.id })"
                v-t="'cancel'"
              ></v-btn>
            </template>

            <template v-else-if="task.status.type === 'failed'">
              <v-btn
                size="x-small"
                color="error"
                variant="text"
                prepend-icon="mdi-alert-circle-outline"
                @click="
                  errorDialogMessage = task.status.error;
                  errorDialog = true;
                "
                v-t="'details'"
              ></v-btn>
            </template>

            <template v-else-if="task.status.type === 'done'">
              <v-btn
                size="x-small"
                color="primary"
                variant="text"
                prepend-icon="mdi-text-box-outline"
                @click="showOutput(task)"
                v-t="'show-output'"
              ></v-btn>
              <v-btn
                size="x-small"
                color="primary"
                variant="text"
                prepend-icon="mdi-folder-open-outline"
                @click="showInFolder(task.output)"
                v-t="'show-in-folder'"
              ></v-btn>
            </template>
          </div>
        </div>
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
.task-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
  overflow-y: auto;
  padding-right: 8px;
}

/* 美化滚动条 */
/* 强制显示滚动条，覆盖全局隐藏 */
.task-list::-webkit-scrollbar {
  width: 8px;
  display: block !important;
}

.task-list::-webkit-scrollbar-track {
  background: rgba(30, 30, 30, 0.1);
  border-radius: 4px;
  margin: 8px 0;
}

.task-list::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.6), rgba(156, 105, 217, 0.6));
  border-radius: 4px;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.8), rgba(156, 105, 217, 0.8));
  border-color: rgba(255, 255, 255, 0.2);
}

.task-list::-webkit-scrollbar-thumb:active {
  background: linear-gradient(135deg, rgba(118, 64, 193, 0.9), rgba(156, 105, 217, 0.9));
}

.task-item {
  display: flex;
  background: rgba(30, 30, 30, 0.8);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.2s ease;
  border: 1px solid rgba(255, 255, 255, 0.08);
  min-height: 90px;
}

.task-item:hover {
  background: rgba(40, 40, 50, 0.9);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.task-cover {
  width: 120px;
  flex-shrink: 0;
}

.cover-image {
  width: 100%;
  height: 100%;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  min-height: 90px;
}

.task-info {
  flex: 1;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0; /* 确保flex子元素可以收缩 */
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.task-name {
  font-size: 1.1rem;
  font-weight: 500;
  margin: 0;
  color: #e0e0e0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.task-path {
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.6);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.task-status {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.status-text {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.8);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.task-actions {
  display: flex;
  gap: 8px;
  margin-top: auto;
  justify-content: flex-end;
  flex-shrink: 0;
}

.empty-state {
  margin: 60px 0;
  font-size: 1.2rem;
  color: rgba(255, 255, 255, 0.5);
}

.error-dialog {
  background: #1E1E1E;
  border-radius: 12px !important;
  border: 1px solid #FF5252;
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
  background: rgba(30, 30, 30, 0.9);
  backdrop-filter: blur(12px);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.text-gradient {
  background: linear-gradient(45deg, #3f51b5, #5c6bc0);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

pre {
  background: rgba(0, 0, 0, 0.3) !important;
  padding: 12px !important;
  border-radius: 8px;
  font-family: monospace;
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