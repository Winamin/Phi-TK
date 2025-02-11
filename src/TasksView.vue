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
    rendering: 渲染中（{ progress }%），{ fps } FPS，预计 { estimate } 结束
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
import { ref, onUnmounted } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import type { Task, TaskStatus } from './model';

import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';

import moment from 'moment';
import { toastError } from './common';

const tasks = ref<Task[]>();

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
  console.log(tasks.value[0]);
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

function describeStatus(status: TaskStatus): string {
  switch (status.type) {
    case 'pending':
      return t('status.pending');
    case 'loading':
      return t('status.loading');
    case 'mixing':
      return t('status.mixing');
    case 'rendering':
      return t('status.rendering', {
        progress: (status.progress * 100).toFixed(2),
        fps: status.fps,
        estimate: status.estimate ? moment.duration(Math.ceil(status.estimate), 'seconds').humanize(true, { ss: 0, s: 60, m: 60 }) : '',
      });
    case 'done':
      return t('status.done', {
        duration: moment.duration(Math.ceil(status.duration), 'seconds').humanize(false, { ss: 0, s: 60, m: 60 }),
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

const cardRefs = ref<HTMLElement[]>([]);

function handleMouseMove(e: MouseEvent, index: number) {
  const card = cardRefs.value[index];
  if (!card) return;

  const rect = card.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const centerX = rect.width / 2;
  const centerY = rect.height / 2;
  
  const rotateY = ((x - centerX) / centerX) * 180;
  const rotateX = ((centerY - y) / centerY) * 180;
  
  card.style.transform = `
    perspective(1000px)
    rotateX(${rotateX}deg)
    rotateY(${rotateY}deg)
    translateZ(20px)
  `;
  
  const shadowX = (centerX - x) / 10;
  const shadowY = (centerY - y) / 10;
  card.style.boxShadow = `
    ${shadowX}px ${shadowY}px 30px rgba(0, 0, 0, 0.3),
    0 0 20px rgba(33, 150, 243, 0.2)
  `;
}

function handleMouseLeave(index: number) {
  const card = cardRefs.value[index];
  if (!card) return;

  card.style.transform = `
    perspective(1000px)
    rotateX(0deg)
    rotateY(0deg)
    translateZ(0)
  `;
  card.style.boxShadow = `
    0 12px 24px rgba(0, 0, 0, 0.3),
    0 0 20px rgba(33, 150, 243, 0.1)
  `;
}
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled" v-t="'empty'"></h1>
    <v-card 
      v-for="(task, index) in tasks" 
      :key="task.id" 
      class="task-card"
      :ref="el => cardRefs[index] = el as HTMLElement"
      @mousemove="handleMouseMove($event, index)"
      @mouseleave="handleMouseLeave(index)"
    >
      <div class="d-flex flex-row align-stretch">
        <div class="d-flex flex-row align-center" style="width: 35%">
          <div
            style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-repeat: no-repeat; background-size: cover"
            :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"
            class="task-cover"
          ></div>
        </div>
        <div class="d-flex flex-column w-100">
          <v-card-title>{{ task.name }}</v-card-title>
          <v-card-subtitle class="mt-n2">{{ task.path }}</v-card-subtitle>
          <div class="w-100 pa-4 pb-2 pr-2 mt-2">
            <p class="mb-2 text-medium-emphasis">{{ describeStatus(task.status) }}</p>
            <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
              <v-progress-circular
                v-if="task.status.type !== 'rendering'"
                :indeterminate="true"
                color="accent"
                class="glow-spinner"
              ></v-progress-circular>
              <v-progress-linear
                v-else
                :model-value="task.status.progress * 100"
                color="accent"
                height="12"
                rounded
              ></v-progress-linear>
              <div class="pt-4 d-flex justify-end">
                <v-btn 
                  variant="flat" 
                  color="error" 
                  prepend-icon="mdi-cancel" 
                  @click="invoke('cancel_task', { id: task.id })" 
                  v-t="'cancel'"
                  class="hover-scale"
                ></v-btn>
              </div>
            </template>
            <div v-if="task.status.type === 'failed'" class="pt-4 d-flex justify-end">
              <v-btn
                variant="flat"
                color="error"
                prepend-icon="mdi-alert-circle-outline"
                @click="errorDialogMessage = task.status.error; errorDialog = true;"
                v-t="'details'"
                class="hover-scale"
              ></v-btn>
            </div>
            <div v-if="task.status.type === 'done'" class="pt-4 d-flex justify-end gap-2">
              <v-btn
                variant="flat"
                color="secondary"
                prepend-icon="mdi-text-box-outline"
                @click="showOutput(task)"
                v-t="'show-output'"
                class="hover-scale"
              ></v-btn>
              <v-btn
                variant="flat"
                color="accent"
                prepend-icon="mdi-folder-open-outline"
                @click="showInFolder(task.output)"
                v-t="'show-in-folder'"
                class="hover-scale"
              ></v-btn>
            </div>
          </div>
        </div>
      </div>
    </v-card>


    <v-dialog v-model="errorDialog" width="auto" min-width="400px">
      <v-card class="glass-background">
        <v-card-title class="text-gradient" v-t="'error'"></v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ errorDialogMessage }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn 
            color="primary" 
            variant="flat" 
            @click="errorDialog = false" 
            v-t="'confirm'"
            class="hover-scale"
          ></v-btn>
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
          <v-btn 
            color="primary" 
            variant="flat" 
            @click="outputDialog = false" 
            v-t="'confirm'"
            class="hover-scale"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
.task-card {
  will-change: transform, box-shadow;
  transition: 
    transform 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    box-shadow 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    background 0.3s ease;
}

.task-cover {
  transition: 
    transform 0.6s cubic-bezier(0.23, 1, 0.32, 1),
    filter 0.4s ease;
  transform-origin: left center;
  filter: saturate(0.9) brightness(0.98);
}

.task-card:hover .task-cover {
  filter: saturate(1.2) brightness(1.05);
  transform: scale(1.08) translateZ(20px);
}

@keyframes enhanced-shimmer {
  0% { transform: translateX(-150%) skewX(-20deg); }
  100% { transform: translateX(250%) skewX(-20deg); }
}

.v-progress-linear::after {
  animation: enhanced-shimmer 2s infinite;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.4) 50%,
    rgba(255, 255, 255, 0) 100%
  );
}

.hover-scale {
  transition:
    transform 0.3s cubic-bezier(0.68, -0.55, 0.27, 1.55),
    box-shadow 0.3s ease,
    filter 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.08) translateZ(10px);
  filter: 
    drop-shadow(0 4px 12px rgba(33, 150, 243, 0.4))
    brightness(1.1);
}

.glow-spinner {
  filter: 
    drop-shadow(0 0 12px rgba(33, 150, 243, 0.6))
    contrast(1.2);
  animation: spinner-pulse 1.5s ease-in-out infinite;
}

@keyframes spinner-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.dialog-enter-active,
.dialog-leave-active {
  transition: 
    opacity 0.4s cubic-bezier(0.23, 1, 0.32, 1),
    transform 0.4s cubic-bezier(0.23, 1, 0.32, 1);
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.97);
}

.text-gradient {
  position: relative;
  background-clip: text;
  -webkit-background-clip: text;
  color: transparent;
  animation: gradient-shift 8s ease infinite;
  background-image: linear-gradient(
    45deg,
    #2196F3 0%,
    #E91E63 50%,
    #2196F3 100%
  );
  background-size: 300% 300%;
}

@keyframes gradient-shift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.task-card::after {
  content: '';
  position: absolute;
  inset: -1px;
  border-radius: inherit;
  z-index: -1;
  background: linear-gradient(
    45deg,
    rgba(33, 150, 243, 0.3),
    rgba(233, 30, 99, 0.3),
    rgba(33, 150, 243, 0.3)
  );
  opacity: 0;
  transition: opacity 0.6s ease;
  animation: border-glow 4s linear infinite;
}

@keyframes border-glow {
  to { background-position: 200% 0; }
}

.task-card:hover::after {
  opacity: 0.6;
}
</style>

<v-dialog 
  v-model="errorDialog" 
  transition="dialog"
  width="auto" 
  min-width="400px"
>
