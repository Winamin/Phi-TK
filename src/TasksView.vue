<i18n>
en:
  empty: Nothing here

  status:
    pending: Pending…
    loading: Loading…
    mixing: Mixing…
    rendering: Rendering ({ progress }%), { fps } FPS, estimated to end { estimate }
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
    rendering: 渲染中  { progress }%  { fps } FPS  预计 { estimate } 结束
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
import { ref, onUnmounted, reactive } from 'vue';

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
  //console.log(tasks.value[0]);
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
    return `${hours} ${t('H')} ${minutes} ${t('m')} ${secs} ${t('s')}`;
  } else if (minutes > 0) {
    return `${minutes} ${t('m')} ${secs} ${t('s')}`;
  } else if (secs > 0) {
    return `${secs} ${t('s')}`;
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

const errorDialog = ref(false),
  errorDialogMessage = ref('');

const outputDialog = ref(false),
  outputDialogMessage = ref('');

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
  rotateYSensitivity: 15,
  rotateXSensitivity: 10,
  translateSensitivity: 15,
  scaleFactor: 1.02,
  translateZ: 30,
  enterDuration: 0.3,
  leaveDuration: 0.6,
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
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled" v-t="'empty'"></h1>
    <v-slide-y-transition>
      <v-form ref="form" style="max-height: 48vh" class="animated-form">
        <v-row class="text-center">
          <v-col cols="12" class="mt-n4">
            <v-btn @click="showFolder()" v-t="'show-in-folder'" class="hover-scale" color="primary"></v-btn>
          </v-col>
        </v-row>
      </v-form>
    </v-slide-y-transition>
    <div
      class="task-card-container"
      v-for="task in tasks"
      :key="task.id"
      @mouseenter="handleMouseEnter(task.id.toString())"
      @mousemove="handleMouseMove($event, task.id.toString())"
      @mouseleave="handleMouseLeave(task.id.toString())">
      <v-card
        class="task-card"
        :id="'card-' + task.id.toString()"
        :style="{
          transform: cardTransforms[task.id.toString()]?.transform,
          transition: cardTransforms[task.id.toString()]?.transition,
        }">
        <div class="d-flex flex-row align-stretch">
          <div class="d-flex flex-row align-center" style="width: 35%">
            <div
              style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-size: cover; background-color: #f0f0f0;"
              :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"
              class="task-cover"></div>
          </div>
          <div class="d-flex flex-column w-100">
            <v-card-title>{{ task.name }}</v-card-title>
            <v-card-subtitle class="mt-n2">{{ task.path }}</v-card-subtitle>
            <div class="w-100 pa-4 pb-2 pr-2 mt-2">
              <p class="mb-2 text-medium-emphasis">{{ describeStatus(task.status) }}</p>
              <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
                <v-progress-circular v-if="task.status.type !== 'rendering'" :indeterminate="true" color="accent" class="glow-spinner"></v-progress-circular>
                <v-progress-linear v-else :model-value="task.status.progress * 100" color="accent" height="12" rounded></v-progress-linear>
                <div class="pt-4 d-flex justify-end">
                  <v-btn variant="flat" color="error" prepend-icon="mdi-cancel" @click="invoke('cancel_task', { id: task.id })" v-t="'cancel'" class="hover-scale"></v-btn>
                </div>
              </template>
              <div v-if="task.status.type === 'failed'" class="pt-4 d-flex justify-end">
                <v-btn
                  variant="flat"
                  color="error"
                  prepend-icon="mdi-alert-circle-outline"
                  @click="
                    errorDialogMessage = task.status.error;
                    errorDialog = true;
                  "
                  v-t="'details'"
                  class="hover-scale"></v-btn>
              </div>
              <div v-if="task.status.type === 'done'" class="pt-4 d-flex justify-end gap-2">
                <v-btn variant="flat" color="secondary" prepend-icon="mdi-text-box-outline" @click="showOutput(task)" v-t="'show-output'" class="hover-scale"></v-btn>
                <v-btn variant="flat" color="accent" prepend-icon="mdi-folder-open-outline" @click="showInFolder(task.output)" v-t="'show-in-folder'" class="hover-scale"></v-btn>
              </div>
            </div>
          </div>
        </div>
      </v-card>
    </div>

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
  </div>
</template>

<style scoped>
.task-card-container {
  perspective: 916px;
  transform-style: preserve-3d;
  margin: 1rem 0;
}

.task-card {
  will-change: transform;
  backface-visibility: hidden;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.task-card:hover {
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}

.task-card-container {
  perspective: 1000px;
  transform-style: preserve-3d;
}
/*
.task-card:hover .task-cover {
  transform: rotateY(15deg) translateZ(30px);
}
*/

.v-btn {
  margin-left: 8px;
}

.task-cover {
  border-radius: 6px 0 0 16px;
  transform-origin: center;
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

.glow-spinner {
  filter: drop-shadow(0 0 8px #2196f3);
}

pre {
  background: rgba(0, 0, 0, 0.3) !important;
  padding: 8px !important;
  border-radius: 8px;
  font-family: 'Fira Code', monospace;
}

.animated-form {
  transition:
    opacity 0.1s ease,
    transform 0.1s ease;
}

.v-slide-y-transition-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.v-overlay--active {
  opacity: 0.8 !important;
  background-color: #000000 !important;
  backdrop-filter: blur(10px);
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
  padding: 8px;
  border-radius: 4px;
  max-height: 60vh;
  overflow: auto;
}

.v-card-actions.justify-end {
  justify-content: flex-start !important;
  padding-left: 16px !important;
}

.v-btn {
  min-width: 100px !important;
  margin: 8px !important;
}

.v-btn--primary {
  background: #2196F3 !important;
  border-color: #5799d1 !important;
}

.v-btn--primary:hover {
  background: #4a96e1 !important;
}
</style>