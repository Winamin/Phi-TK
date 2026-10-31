<i18n>
en:
  empty: Nothing here
  state-label:
    pending: Pending
    loading: Loading
    mixing: Mixing
    saving: Saving to gallery
    rendering: Rendering
    done: Done
    canceled: Canceled
    failed: Failed

  status:
    pending: Pending…
    loading: Loading…
    mixing: Mixing…
    saving: Saving to gallery…
    rendering: "Exporting { fps } FPS · estimated to end { estimate }"
    done: Done, took { duration }
    canceled: Canceled
    failed: Failed

  cancel: Cancel
  confirm: Confirm

  details: Details
  error: Error
  output: Output

  show-output: Show Log
  show-log: Show Log
  view-gallery: View in gallery
  gallery-hint: Videos are saved to Movies/Phi-TK. Open your gallery to view or share them.

  detail:
    name: Name
    path: Path
    status: Status
    progress: Progress
    fps: Frame rate
    'encoder-fps': End-to-end frame rate
    duration: Elapsed
    output: Output
    error: Error

zh-CN:
  empty: 空空如也
  state-label:
    pending: 等待中
    loading: 加载中
    mixing: 混音中
    saving: 保存到相册中
    rendering: 渲染中
    done: 已完成
    canceled: 已取消
    failed: 失败

  status:
    pending: 等待中…
    loading: 加载中…
    mixing: 混音中…
    saving: 保存到相册中…
    rendering: "导出中 { fps } FPS · 预计 { estimate } 结束"
    done: 已完成，耗时总时间 { duration }
    canceled: 已取消
    failed: 失败

  cancel: 取消
  confirm: 确定

  details: 详情
  error: 错误
  output: 输出

  show-output: 查看日志
  show-log: 查看日志
  view-gallery: 去相册查看
  gallery-hint: 视频保存到 Movies/Phi-TK，可在相册的 Phi-TK 相簿中查看或分享。

  detail:
    name: 名称
    path: 路径
    status: 状态
    progress: 进度
    fps: 帧率
    'encoder-fps': 端到端帧率
    duration: 耗时
    output: 输出
    error: 错误

</i18n>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Task, TaskStatus } from './model';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import moment from 'moment';
import { toast } from './common';
import LogWindow from './components/LogWindow.vue';

const { t } = useI18n();

const tasks = ref<Task[]>();

const ACTIVE_STATUSES = ['pending', 'loading', 'mixing', 'rendering', 'saving'];
const POLL_ACTIVE_MS = 300;
const POLL_IDLE_MS = 2000;

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
}

await updateList();
let stopped = false;
let pollTimer: ReturnType<typeof setTimeout> | undefined;

function scheduleNextPoll() {
  if (stopped) return;
  const busy = tasks.value?.some((task) => ACTIVE_STATUSES.includes(task.status.type)) ?? false;
  pollTimer = setTimeout(poll, busy ? POLL_ACTIVE_MS : POLL_IDLE_MS);
}

async function poll() {
  try {
    await updateList();
  } catch (e) {
    console.error('Failed to refresh tasks:', e);
  }
  scheduleNextPoll();
}

scheduleNextPoll();

onUnmounted(() => {
  stopped = true;
  if (pollTimer !== undefined) clearTimeout(pollTimer);
});

function formatDuration(seconds: number) {
  const duration = moment.duration(Math.ceil(seconds), 'seconds');
  const hours = Math.floor(duration.asHours());
  const minutes = duration.minutes();
  const secs = duration.seconds();
  if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
  if (minutes > 0) return `${minutes}m ${secs}s`;
  if (secs > 0) return `${secs}s`;
  return '';
}

function describeStatus(status: TaskStatus): string {
  switch (status.type) {
    case 'pending':
      return t('status.pending');
    case 'loading':
      return t('status.loading');
    case 'mixing':
      return t('status.mixing');
    case 'saving':
      return t('status.saving');
    case 'rendering': {
      const progressDisplay = status.progress;
      return t('status.rendering', {
        progress: progressDisplay,
        estimate: status.estimate ? formatDuration(status.estimate) : '',
        fps: status.encoder_fps,
      });
    }
    case 'done':
      return t('status.done', { duration: status.duration ? formatDuration(status.duration) : '' });
    case 'canceled':
      return t('status.canceled');
    case 'failed':
      return t('status.failed');
  }
}

const STATUS_ROLES: Record<string, string> = {
  pending: 'tertiary',
  loading: 'tertiary',
  mixing: 'tertiary',
  saving: 'primary',
  rendering: 'primary',
  done: 'success',
  canceled: 'warning',
  failed: 'error',
};

function statusVars(statusType: string): Record<string, string> {
  const role = STATUS_ROLES[statusType] ?? 'tertiary';
  const vars: Record<string, string> = {
    '--status-color': `var(--mdui-color-${role})`,
    '--status-container': `var(--mdui-color-${role}-container)`,
    '--status-on-container': `var(--mdui-color-on-${role}-container)`,
  };
  if (role !== 'primary') vars['--mdui-color-primary'] = `var(--mdui-color-${role})`;
  return vars;
}

const detailDialog = ref(false),
  selectedTask = ref<Task | null>(null);
const logOpen = ref(false),
  logTask = ref<Task | null>(null);

async function viewGallery(task?: Task) {
  try {
    await invoke('view_video', { uri: task?.status.type === 'done' ? task.status.video_uri : null });
  } catch {
    toast(t('gallery-hint'), 'info');
  }
}

function showOutput(task: Task) {
  logTask.value = task;
  logOpen.value = true;
}

const showDetail = (task: Task) => {
  selectedTask.value = task;
  detailDialog.value = true;
};
</script>

<template>
  <div class="tasks-container">
    <div class="tasks-header">
      <p class="md3-body">{{ t('gallery-hint') }}</p>
      <mdui-button variant="tonal" @click="viewGallery()">
        <mdui-icon-image--outlined slot="icon"></mdui-icon-image--outlined>
        {{ t('view-gallery') }}
      </mdui-button>
    </div>

    <div v-if="!tasks || !tasks.length" class="empty-state">
      <mdui-icon-inbox--outlined class="empty-icon"></mdui-icon-inbox--outlined>
      <p class="md3-body">{{ t('empty') }}</p>
    </div>

    <div v-else class="task-list">
      <mdui-card v-for="task in tasks" :key="task.id" variant="filled" class="task-card" :style="statusVars(task.status.type)">
        <div class="task-info">
          <div class="task-header">
            <div class="task-cover" aria-hidden="true">
              <div class="cover-image" :style="{ backgroundImage: 'url(' + convertFileSrc(task.cover) + ')' }"></div>
            </div>
            <div class="task-identity">
              <h3 class="task-name" :title="task.name">{{ task.name }}</h3>
              <span class="status-chip">{{ t('state-label.' + task.status.type) }}</span>
            </div>
            <mdui-button-icon :aria-label="t('details')" @click="showDetail(task)">
              <mdui-icon-info--outlined></mdui-icon-info--outlined>
            </mdui-button-icon>
          </div>


          <div class="task-status">
            <mdui-circular-progress v-if="['loading', 'mixing', 'pending', 'saving'].includes(task.status.type)" class="status-spinner"></mdui-circular-progress>
            <mdui-circular-progress v-else-if="task.status.type === 'rendering'" class="status-spinner" :value="task.status.progress"></mdui-circular-progress>
            <span class="status-text">{{ describeStatus(task.status) }}</span>
          </div>

          <mdui-linear-progress v-if="task.status.type === 'rendering'" class="status-bar" :value="task.status.progress"></mdui-linear-progress>

          <div class="task-actions">
            <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
              <mdui-button variant="text" @click="invoke('cancel_task', { id: task.id })">
                <mdui-icon-cancel slot="icon"></mdui-icon-cancel>
                {{ t('cancel') }}
              </mdui-button>
            </template>
            <template v-else-if="task.status.type === 'failed'">
              <mdui-button variant="tonal" @click="showOutput(task)">
                <mdui-icon-description--outlined slot="icon"></mdui-icon-description--outlined>
                {{ t('show-log') }}
              </mdui-button>
            </template>
            <template v-else-if="task.status.type === 'done'">
              <mdui-button variant="tonal" @click="showOutput(task)">
                <mdui-icon-description--outlined slot="icon"></mdui-icon-description--outlined>
                {{ t('show-output') }}
              </mdui-button>
              <mdui-button variant="text" @click="viewGallery(task)">
                <mdui-icon-image--outlined slot="icon"></mdui-icon-image--outlined>
                {{ t('view-gallery') }}
              </mdui-button>
            </template>
          </div>
        </div>
      </mdui-card>
    </div>

    <mdui-dialog class="log-dialog" close-on-esc close-on-overlay-click :open="detailDialog" @close="detailDialog = false">
      <span slot="headline">{{ t('details') }}</span>
      <div v-if="selectedTask" class="detail-list">
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.name') }}</span
          ><span>{{ selectedTask.name }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.path') }}</span
          ><span class="break-all">{{ selectedTask.path }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.status') }}</span
          ><span>{{ describeStatus(selectedTask.status) }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'rendering'" class="detail-row">
          <span class="detail-label">{{ t('detail.progress') }}</span
          ><span>{{ Math.round(selectedTask.status.progress * 100) }}%</span>
        </div>
        <div v-if="selectedTask.status.type === 'rendering'" class="detail-row">
          <span class="detail-label">{{ t('detail.encoder-fps') }}</span
          ><span>{{ selectedTask.status.encoder_fps }} FPS</span>
        </div>
        <div v-if="selectedTask.status.type === 'done' && selectedTask.status.duration" class="detail-row">
          <span class="detail-label">{{ t('detail.duration') }}</span
          ><span>{{ formatDuration(selectedTask.status.duration) }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'done' && selectedTask.output" class="detail-row">
          <span class="detail-label">{{ t('detail.output') }}</span
          ><span>{{ t('gallery-hint') }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'failed' && selectedTask.status.error" class="detail-row">
          <span class="detail-label">{{ t('detail.error') }}</span
          ><span class="error-text">{{ selectedTask.status.error }}</span>
        </div>
      </div>
      <mdui-button slot="action" variant="tonal" @click="detailDialog = false">{{ t('confirm') }}</mdui-button>
    </mdui-dialog>

    <LogWindow :open="logOpen" :task="logTask" @close="logOpen = false" />
  </div>
</template>

<style scoped lang="scss">
.tasks-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: var(--app-page-padding, 16px);
}
.tasks-header {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
  flex-shrink: 0;
  margin-bottom: 16px;
}
.empty-state {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
}
.empty-icon {
  font-size: 48px;
  color: rgb(var(--mdui-color-on-surface-variant));
}
.empty-state p {
  margin: 0;
}
.task-list {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  overscroll-behavior-y: contain;
}
.task-card {
  display: block;
  flex-shrink: 0;
  border-radius: var(--mdui-shape-corner-medium, 12px);
  background-color: rgb(var(--mdui-color-surface-container-low));
}
.task-info {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 12px;
  padding: var(--app-card-padding, 16px);
}
.task-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.task-cover {
  width: 56px;
  height: 56px;
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
.task-identity {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}
.task-name {
  margin: 0;
  max-width: 100%;
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-title-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.status-chip {
  padding: 2px 8px;
  border-radius: var(--mdui-shape-corner-small, 8px);
  background-color: rgb(var(--status-container));
  color: rgb(var(--status-on-container));
  font-size: var(--mdui-typescale-label-medium-size);
  font-weight: var(--mdui-typescale-label-medium-weight);
  line-height: var(--mdui-typescale-label-medium-line-height);
}
.task-path {
  margin: 0;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.task-status {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.status-spinner {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  stroke: rgb(var(--status-color));
}
.status-text {
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow-wrap: anywhere;
}
.status-bar {
  --shape-corner: var(--mdui-shape-corner-full);
  height: 4px;
}
.task-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.log-dialog::part(panel) {
  min-width: min(400px, calc(100vw - 48px));
  max-width: min(560px, calc(100vw - 48px));
}
.detail-list {
  display: flex;
  flex-direction: column;
}
.detail-row {
  display: flex;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid rgb(var(--mdui-color-outline-variant));
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
}
.detail-row:last-child {
  border-bottom: 0;
}
.detail-row > span:last-child {
  min-width: 0;
  overflow-wrap: anywhere;
}
.detail-label {
  width: 80px;
  flex-shrink: 0;
  color: rgb(var(--mdui-color-on-surface-variant));
}
.break-all {
  overflow-wrap: anywhere;
}
.error-text {
  color: rgb(var(--mdui-color-error));
  font-family: 'Roboto Mono', Consolas, monospace;
  overflow-wrap: anywhere;
}
</style>
