<i18n>
en:
  settings:
    outputPath:
      label: "Custom output path"
      placeholder: "Click to choose folder or type manually"
      hint: "Prefer Browser FS API; use Electron for absolute path"
      aria: "Custom output path"
    selectFolder: "Select folder"
    save: "Save"
    copy: "Copy path"
    clear: "Clear"
    saved: "Saved"
    selected:
      picked: "Selected folder: {name} (browser does not expose absolute path)"
      fallback: "Fallback selection: {count} files (root: {root})"
    warning:
      empty: "Path must not be empty"
      select_error: "Failed to select folder: {msg}"
      copy_error: "Copy failed: please copy manually"
zh-CN:
  settings:
    outputPath:
      label: "自定义输出路径"
      placeholder: "点击选择文件夹或手动输入"
      hint: "优先使用浏览器文件系统访问 API; 若需绝对路径请在 Electron 环境下运行"
      aria: "自定义输出路径"
    selectFolder: "选择文件夹"
    save: "保存路径"
    copy: "复制路径"
    clear: "清除"
    saved: "保存成功！"
    selected:
      picked: "已选择文件夹：{name}（浏览器不暴露绝对路径）"
      fallback: "回退选择：{count} 个文件（根：{root})"
    warning:
      empty: "路径不能为空"
      select_error: "选择文件夹时出错：{msg}"
      copy_error: "复制失败：请手动复制"
</i18n>

<script setup lang="ts">
defineOptions({ name: 'SettingsPanel' });

import { ref } from 'vue';
import { RULES as rules } from './common';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const outputPath = ref<string>(localStorage.getItem('outputPath') || '');
const saved = ref(false);
const warning = ref('');
const selectedInfo = ref<string | null>(null);

const dirInput = ref<HTMLInputElement | null>(null);
const pathField = ref<HTMLElement | null>(null);

let dirHandle: any = null;
const pickerAvailable = 'showDirectoryPicker' in window;

function selectFolder() {
  warning.value = '';
  selectedInfo.value = null;

  if (pickerAvailable) {
    (window as any).showDirectoryPicker()
      .then((handle: any) => {
        if (!handle) return;
        dirHandle = handle;
        outputPath.value = handle.name || '已选择文件夹';
        selectedInfo.value = `已选择文件夹：${handle.name}（浏览器不暴露绝对路径）`;
      })
      .catch((err: any) => {
        if (err && err.name !== 'AbortError') {
          warning.value = '选择文件夹时出错：' + (err.message || err);
        }
      });
  } else {
    // 回退 -> 触发隐藏的 input[file webkitdirectory]
    dirInput.value?.click();
  }
}

function onDirInputChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const files = input.files;
  if (!files || files.length === 0) return;

  const first = files[0] as any;
  const rel = first?.webkitRelativePath as string | undefined;

  let root = '';
  if (rel) {
    root = rel.split('/')[0] || first.name;
  } else {
    root = files[0].name;
  }

  outputPath.value = root;
  selectedInfo.value = `回退选择：${files.length} 个文件（根：${root}）`;
  input.value = '';
}

function saveOutputPath() {
  if (!rules.non_empty(outputPath.value)) {
    warning.value = '路径不能为空';
    return;
  }
  localStorage.setItem('outputPath', outputPath.value);
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

function clearPath() {
  outputPath.value = '';
  selectedInfo.value = null;
  dirHandle = null;
  localStorage.removeItem('outputPath');
}

async function copyPath() {
  if (!outputPath.value) return;
  try {
    await navigator.clipboard.writeText(outputPath.value);
    saved.value = true;
    setTimeout(() => (saved.value = false), 1500);
  } catch (err) {
    warning.value = '复制失败：请手动复制';
  }
}
</script>

<template>
  <v-card class="pa-6 app-card">
    <div class="controls-grid">
      <div class="path-area">
        <v-text-field
          v-model="outputPath"
          :label="t('settings.outputPath.label')"
          :placeholder="t('settings.outputPath.placeholder')"
          :rules="[rules.non_empty]"
          clearable
          ref="pathField"
          dense
          :hint="t('settings.outputPath.hint')"
          persistent-hint
          class="path-field"
          append-outer-icon="mdi-folder-open"
          @click:append-outer="selectFolder"
          :aria-label="t('settings.outputPath.aria')"
        />

        <div v-if="selectedInfo" class="mt-2 caption" role="status" aria-live="polite">
          {{ selectedInfo }}
        </div>

        <input
          ref="dirInput"
          type="file"
          webkitdirectory
          directory
          multiple
          style="display: none"
          @change="onDirInputChange"
          aria-hidden="true"
        />
      </div>

      <div class="button-area" role="group" :aria-label="t('settings.selectFolder')">
        <div class="button-wrapper">
          <v-btn small color="secondary" class="mr-2" @click="selectFolder">
            {{ t('settings.selectFolder') }}
          </v-btn>

          <v-btn small color="primary" class="mr-2" @click="saveOutputPath">
            {{ t('settings.save') }}
          </v-btn>

          <v-btn small icon class="mr-2" @click="copyPath" :disabled="!outputPath" :title="t('settings.copy')">
            <v-icon>mdi-content-copy</v-icon>
          </v-btn>

          <v-btn small icon @click="clearPath" :disabled="!outputPath" :title="t('settings.clear')">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </div>
      </div>
    </div>

    <v-snackbar v-model="saved" :timeout="1500" color="success" top right>
      {{ t('settings.saved') }}
    </v-snackbar>

    <v-alert v-if="warning" type="warning" class="mt-4">
      {{ warning }}
    </v-alert>
  </v-card>
</template>

<style scoped>
.app-card {
  max-width: 980px;
  margin: 18px auto;
  width: 95%;
  background-color: #003366;
  color: #ffffff;
  box-shadow: 0 6px 18px rgba(0,0,0,0.25);
  border-radius: 12px;
}

.controls-grid {
  display: grid;
  grid-template-columns: minmax(260px, 1fr) auto;
  gap: 12px;
  align-items: center;
}

.path-area .path-field {
  width: 100%;
  box-sizing: border-box;
}

.button-area {
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.button-wrapper {
  display: flex;
  gap: 8px;
  flex-wrap: nowrap;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  padding-bottom: 2px;
}

@media (max-width: 720px) {
  .controls-grid {
    grid-template-columns: 1fr;
  }

  .button-area {
    margin-top: 8px;
    justify-content: flex-start;
  }

  .button-wrapper {
    flex-wrap: wrap;
    overflow-x: visible;
  }
}

.v-btn.primary {
  background-color: #1e88e5 !important;
}
.v-btn.secondary {
  background-color: #00509e !important;
  color: #fff !important;
}

.caption {
  color: rgba(255,255,255,0.85);
}
</style>
