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

  detail:
    name: Name
    path: Path
    status: Status
    progress: Progress
    fps: Frame rate
    duration: Elapsed
    output: Output
    error: Error

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

  detail:
    name: 名称
    path: 路径
    status: 状态
    progress: 进度
    fps: 帧率
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
import { toastError } from './common';

const { t } = useI18n();

const tasks = ref<Task[]>();

const ACTIVE_STATUSES = ['pending', 'loading', 'mixing', 'rendering'];
const POLL_ACTIVE_MS = 300;
const POLL_IDLE_MS = 2000;

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
}

await updateList();

// Adaptive polling: a fixed interval can stack calls when IPC is slower than the tick,
// so chain the next tick only after the previous one settles.
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
    case 'pending': return t('status.pending');
    case 'loading': return t('status.loading');
    case 'mixing': return t('status.mixing');
    case 'rendering': {
      const progressDisplay = status.progress;
      return t('status.rendering', {
        progress: progressDisplay,
        fps: status.fps,
        estimate: status.estimate ? formatDuration(status.estimate) : '',
      });
    }
    case 'done': return t('status.done', { duration: status.duration ? formatDuration(status.duration) : '' });
    case 'canceled': return t('status.canceled');
    case 'failed': return t('status.failed');
  }
}

/**
 * MD3 has no semantic status palette, so each task state borrows a colour role. `success`
 * and `warning` are the custom colours registered in `theme.ts`; the rest are stock roles.
 */
const STATUS_ROLES: Record<string, string> = {
  pending: 'tertiary', loading: 'tertiary', mixing: 'tertiary',
  rendering: 'primary', done: 'success', canceled: 'warning', failed: 'error',
};

/**
 * Feeds the status colour to descendants as custom properties. `--mdui-color-primary` is
 * included because mdui's progress components hardcode it — reassigning it on the host
 * recolours them without reaching into their shadow roots.
 */
function statusVars(statusType: string): Record<string, string> {
  const role = STATUS_ROLES[statusType] ?? 'tertiary';
  return {
    '--status-color': `var(--mdui-color-${role})`,
    '--status-container': `var(--mdui-color-${role}-container)`,
    '--status-on-container': `var(--mdui-color-on-${role}-container)`,
    '--mdui-color-primary': `var(--mdui-color-${role})`,
  };
}

const errorDialog = ref(false), errorDialogMessage = ref('');
const outputDialog = ref(false), outputDialogMessage = ref('');
const detailDialog = ref(false), selectedTask = ref<Task | null>(null);

async function showInFolder(path: string) {
  try { await invoke('show_in_folder', { path }); } catch (e) { toastError(e); }
}

async function showFolder() {
  try { await invoke('show_folder'); } catch (e) { toastError(e); }
}

function showOutput(task: Task) {
  if (task.status.type === 'done') {
    outputDialogMessage.value = task.status.output;
    outputDialog.value = true;
  }
}

const showDetail = (task: Task) => {
  selectedTask.value = task;
  detailDialog.value = true;
};
</script>

<template>
  <div class="tasks-container">
    <!-- Header -->
    <div class="tasks-header">
      <h2 class="tasks-title md3-headline">{{ t('output') }}</h2>
      <mdui-button variant="tonal" @click="showFolder()">
        <mdui-icon-folder-open--outlined slot="icon"></mdui-icon-folder-open--outlined>
        {{ t('show-in-folder') }}
      </mdui-button>
    </div>

    <!-- Empty state -->
    <div v-if="!tasks || !tasks.length" class="empty-state">
      <mdui-icon-inbox--outlined class="empty-icon"></mdui-icon-inbox--outlined>
      <p class="md3-body">{{ t('empty') }}</p>
    </div>

    <!-- Task list -->
    <div v-else class="task-list">
      <!-- `<mdui-dropdown>` is `display: contents`, so the card below stays a flex item of
           the list. Anchoring the menu at the pointer replaces the old hand-tracked
           `contextMenuX`/`contextMenuY` coordinates. -->
      <mdui-dropdown v-for="task in tasks" :key="task.id" trigger="contextmenu" open-on-pointer>
        <mdui-card slot="trigger" variant="filled" class="task-card" :style="statusVars(task.status.type)">
          <!-- Cover (left 35%) -->
          <div class="task-cover">
            <div
              class="cover-image"
              :style="{ backgroundImage: 'url(' + convertFileSrc(task.cover) + ')' }"></div>
          </div>

          <!-- Info (right 65%) -->
          <div class="task-info">
            <div class="task-header">
              <h3 class="task-name" :title="task.name">{{ task.name }}</h3>
              <span class="status-chip">{{ task.status.type.toUpperCase() }}</span>
            </div>

            <p class="task-path" :title="task.path">{{ task.path }}</p>

            <div class="task-status">
              <mdui-circular-progress
                v-if="['loading', 'mixing', 'pending'].includes(task.status.type)"
                class="status-spinner"></mdui-circular-progress>
              <mdui-circular-progress
                v-else-if="task.status.type === 'rendering'"
                class="status-spinner"
                :value="task.status.progress"></mdui-circular-progress>
              <span class="status-text">{{ describeStatus(task.status) }}</span>
            </div>

            <mdui-linear-progress
              v-if="task.status.type === 'rendering'"
              class="status-bar"
              :value="task.status.progress"></mdui-linear-progress>

            <div class="task-actions">
              <template v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)">
                <mdui-button variant="text" @click="invoke('cancel_task', { id: task.id })">
                  <mdui-icon-cancel slot="icon"></mdui-icon-cancel>
                  {{ t('cancel') }}
                </mdui-button>
              </template>
              <template v-else-if="task.status.type === 'failed'">
                <mdui-button variant="text" @click="errorDialogMessage = task.status.error; errorDialog = true;">
                  <mdui-icon-error--outlined slot="icon"></mdui-icon-error--outlined>
                  {{ t('details') }}
                </mdui-button>
              </template>
              <template v-else-if="task.status.type === 'done'">
                <mdui-button variant="text" @click="showOutput(task)">
                  <mdui-icon-description--outlined slot="icon"></mdui-icon-description--outlined>
                  {{ t('show-output') }}
                </mdui-button>
                <mdui-button variant="text" @click="showInFolder(task.output)">
                  <mdui-icon-folder-open--outlined slot="icon"></mdui-icon-folder-open--outlined>
                  {{ t('show-in-folder') }}
                </mdui-button>
              </template>
            </div>
          </div>
        </mdui-card>

        <mdui-menu>
          <mdui-menu-item @click="showDetail(task)">
            <mdui-icon-info--outlined slot="icon"></mdui-icon-info--outlined>
            {{ t('details') }}
          </mdui-menu-item>
        </mdui-menu>
      </mdui-dropdown>
    </div>

    <!-- Error dialog. No `close-on-overlay-click`: a stray click should not throw away a
         failure message the user has not read yet. -->
    <mdui-dialog
      class="log-dialog"
      close-on-esc
      :open="errorDialog"
      @close="errorDialog = false">
      <mdui-icon-warning slot="icon" class="dialog-error-icon"></mdui-icon-warning>
      <span slot="headline">{{ t('error') }}</span>
      <pre class="log-pre log-pre-error">{{ errorDialogMessage }}</pre>
      <mdui-button slot="action" variant="text" @click="errorDialog = false">{{ t('confirm') }}</mdui-button>
    </mdui-dialog>

    <!-- Output dialog -->
    <mdui-dialog
      class="log-dialog"
      close-on-esc
      close-on-overlay-click
      :open="outputDialog"
      @close="outputDialog = false">
      <span slot="headline">{{ t('output') }}</span>
      <pre class="log-pre">{{ outputDialogMessage }}</pre>
      <mdui-button slot="action" variant="tonal" @click="outputDialog = false">{{ t('confirm') }}</mdui-button>
    </mdui-dialog>

    <!-- Detail dialog -->
    <mdui-dialog
      class="log-dialog"
      close-on-esc
      close-on-overlay-click
      :open="detailDialog"
      @close="detailDialog = false">
      <span slot="headline">{{ t('details') }}</span>
      <div v-if="selectedTask" class="detail-list">
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.name') }}</span><span>{{ selectedTask.name }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.path') }}</span><span class="break-all">{{ selectedTask.path }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">{{ t('detail.status') }}</span><span>{{ describeStatus(selectedTask.status) }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'rendering'" class="detail-row">
          <span class="detail-label">{{ t('detail.progress') }}</span><span>{{ Math.round(selectedTask.status.progress * 100) }}%</span>
        </div>
        <div v-if="selectedTask.status.type === 'rendering'" class="detail-row">
          <span class="detail-label">{{ t('detail.fps') }}</span><span>{{ selectedTask.status.fps }} FPS</span>
        </div>
        <div v-if="selectedTask.status.type === 'done' && selectedTask.status.duration" class="detail-row">
          <span class="detail-label">{{ t('detail.duration') }}</span><span>{{ formatDuration(selectedTask.status.duration) }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'done' && selectedTask.output" class="detail-row">
          <span class="detail-label">{{ t('detail.output') }}</span><span class="break-all">{{ selectedTask.output }}</span>
        </div>
        <div v-if="selectedTask.status.type === 'failed' && selectedTask.status.error" class="detail-row">
          <span class="detail-label">{{ t('detail.error') }}</span><span class="error-text">{{ selectedTask.status.error }}</span>
        </div>
      </div>
      <mdui-button slot="action" variant="tonal" @click="detailDialog = false">{{ t('confirm') }}</mdui-button>
    </mdui-dialog>
  </div>
</template>

<style scoped lang="scss">
@use './styles/breakpoints' as bp;
@use './styles/motion' as mo;

.tasks-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  max-width: 960px;
  margin: 0 auto;
  padding: 24px;
}

/* ===== Header ===== */
.tasks-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
  flex-shrink: 0;
}

.tasks-title {
  color: rgb(var(--mdui-color-on-surface));
  margin: 0;
}

/* ===== Empty State ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 16px;
}

.empty-icon {
  font-size: 4rem;
  color: rgb(var(--mdui-color-outline));
}

/* ===== Task List ===== */
.task-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  overflow-y: auto;
  padding: 2px;
}

/* `<mdui-card>` is `display: inline-block` in its shadow root; a document-level rule
   outranks `:host`, so this makes it a row. */
.task-card {
  display: flex;
  flex-direction: row;
  min-height: 120px;
  /* The lift is spatial (it moves), the shadow is an effect. */
  @include mo.spatial((transform, box-shadow));

  @include mo.enter-rise(14px);
  @include mo.stagger(10, 45ms, 40ms);
}

.task-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--mdui-elevation-level2);
}

/* Cover (35%) */
.task-cover {
  width: 35%;
  flex-shrink: 0;
  background-color: rgb(var(--mdui-color-surface-container-low));
}

.cover-image {
  width: 100%;
  height: 100%;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  min-height: 120px;
}

/* Info (65%) */
.task-info {
  width: 65%;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.task-name {
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-title-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

/* Deliberately not an `<mdui-chip>`: this is a read-only state badge, and MD3 chips are
   interactive affordances with a 2rem tap target to match. */
.status-chip {
  flex-shrink: 0;
  padding: 3px 10px;
  border-radius: var(--mdui-shape-corner-full);
  background-color: rgb(var(--status-container));
  color: rgb(var(--status-on-container));
  font-size: var(--mdui-typescale-label-small-size);
  font-weight: var(--mdui-typescale-label-large-weight);
  line-height: var(--mdui-typescale-label-small-line-height);
  letter-spacing: var(--mdui-typescale-label-small-tracking);
}

.task-path {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-status {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

/* `<mdui-circular-progress>` sizes itself to 2.5rem via `:host`; overriding it here is
   the same document-level-beats-`:host` trick used for the icon buttons. */
.status-spinner {
  width: 1.25rem;
  height: 1.25rem;
  stroke: rgb(var(--status-color));
}

.status-bar {
  --shape-corner: var(--mdui-shape-corner-full);
  height: 3px;
}

.status-text {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-actions {
  display: flex;
  gap: 8px;
  margin-top: auto;
  flex-wrap: wrap;
}

/* ===== Dialogs ===== */
/* mdui caps the panel at 35rem; log output reads much better wider than that. */
.log-dialog::part(panel) {
  min-width: min(25rem, 100%);
  max-width: min(45rem, 100%);
}

.dialog-error-icon {
  color: rgb(var(--mdui-color-error));
}

.log-pre {
  margin: 0;
  padding: 12px;
  border-radius: var(--mdui-shape-corner-medium);
  background-color: rgb(var(--mdui-color-surface-container-highest));
  color: rgb(var(--mdui-color-on-surface));
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: 1.5;
  max-height: 60vh;
  overflow: auto;
  white-space: pre-wrap;
}

.log-pre-error {
  color: rgb(var(--mdui-color-error));
  background-color: rgb(var(--mdui-color-error-container));
}

/* ===== Detail list ===== */
.detail-list {
  display: flex;
  flex-direction: column;
}

.detail-row {
  display: flex;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(var(--mdui-color-outline-variant), 0.5);
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-label {
  width: 4.5rem;
  flex-shrink: 0;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-weight: var(--mdui-typescale-label-large-weight);
}

.break-all {
  word-break: break-all;
}

.error-text {
  color: rgb(var(--mdui-color-error));
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: var(--mdui-typescale-body-small-size);
  word-break: break-all;
}

/* ===== Responsive ===== */
@include bp.compact {
  .tasks-container { padding: 16px; }
  .task-card { flex-direction: column; }
  .task-cover { width: 100%; min-height: 140px; }
  .task-info { width: 100%; }
}
</style>
