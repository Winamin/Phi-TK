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
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled" v-t="'empty'"></h1>
    <v-slide-y-transition>
      <v-form 
        ref="form" 
        style="max-height: 48vh;"
        class="animated-form"
      >
        <v-row class="text-center">
          <v-col cols="12" class="mt-n4">
            <v-btn 
              @click="showFolder()" 
              v-t="'show-in-folder'"
              class="hover-scale"
            ></v-btn>
          </v-col>
        </v-row>
      </v-form>
    </v-slide-y-transition>
   <div class="task-card-container" v-for="task in tasks" :key="task.id">
    <v-card class="task-card">
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
  </div>

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
.task-card-container {
  perspective: 1000px;
  transform-style: preserve-3d;
}

.task-card {
  transform: rotateY(0.01deg);
  border-radius: 16px !important;
  background: rgba(255, 255, 255, 0.03) !important;
  backdrop-filter: blur(8px);
  transition: 
    transform 0.68s ease,
    box-shadow 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transform-style: preserve-3d;
}

/*
.task-card:hover {
  transform: rotateY(15deg) translateZ(30px) scale(1.02);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}
*/
  
.task-card-container {
  perspective: 1000px;
  transform-style: preserve-3d;
}
  
.task-card:hover {
  transform: rotateY(360deg);
  transition-duration: 0.2s;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}
  
.task-card:hover .task-cover {
  transform: rotateY(15deg) translateZ(30px);
}

.task-card:not(:hover) {
  transition-duration: 0.37s;
}

@keyframes rotate360 {
  from {
    transform: rotateY(0deg);
  }
  to {
    transform: rotateY(360deg);
  }
}
  
.v-btn {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  border-radius: 50px;
  box-shadow: 0 8px 15px rgba(0, 0, 0, 0.2);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
}

.v-btn:hover {
  transform: translateY(-4px);
  box-shadow: 0 15px 30px rgba(0, 0, 0, 0.3);
}

.task-cover {
  border-radius: 16px 0 0 16px;
  transform-origin: center;
}

.glass-background {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.text-gradient {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #2196F3);
}

pre {
  background: rgba(0, 0, 0, 0.3) !important;
  padding: 16px !important;
  border-radius: 8px;
  font-family: 'Fira Code', monospace;
}

.animated-form {
  transition: opacity 0.1s ease, transform 0.1s ease;
}

.v-slide-y-transition-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
