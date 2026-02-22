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
    background:
      label: "Custom background"
      placeholder: "Click to select image file"
      hint: "Supports JPG, PNG, WEBP format"
      selectFile: "Select image"
      clear: "Clear background"
      saved: "Background saved!"
zh-CN:
  settings:
    outputPath:
      label: "自定义输出路径"
      placeholder: "点击选择文件夹或手动输入"
      hint: "请输入正确路径"
      aria: "自定义输出路径"
    selectFolder: "选择文件夹"
    save: "保存路径"
    copy: "复制路径"
    clear: "清除"
    saved: "保存成功！"
    selected:
      picked: "已选择文件夹：{name}"
      fallback: "回退选择：{count} 个文件（根：{root})"
    warning:
      empty: "路径不能为空"
      select_error: "选择文件夹时出错：{msg}"
      copy_error: "复制失败：请手动复制"
    background:
      label: "自定义背景"
      placeholder: "点击选择图片文件"
      hint: "支持 JPG、PNG、WEBP 格式"
      selectFile: "选择图片"
      clear: "清除背景"
      saved: "背景已保存！"
</i18n>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { appConfigDir } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/core';
//import type { BaseDirectory } from '@tauri-apps/plugin-fs';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

defineOptions({ name: 'SettingsPanel' });
import { RULES as rules } from './common';

const { t } = useI18n();

const outputPath = ref<string>(localStorage.getItem('outputPath') || '');
const saved = ref(false);
const warning = ref('');
const selectedInfo = ref<string | null>(null);

// 自定义背景
const backgroundPath = ref<string>(localStorage.getItem('customBackground') || '');
const backgroundSaved = ref(false);

// 背景预览URL（用于Tauri文件路径转换）
const backgroundPreviewUrl = computed(() => {
  if (backgroundPath.value) {
    try {
      return convertFileSrc(backgroundPath.value);
    } catch {
      return backgroundPath.value;
    }
  }
  return '';
});

async function selectFolder() {
  warning.value = '';
  selectedInfo.value = null;

  try {
    const selected = await open({
      directory: true,
      multiple: false,
      // 使用 appConfigDir 作为默认起始目录
      defaultPath: await appConfigDir(),
    });

    if (selected === null) return; // 用户取消

    const path = Array.isArray(selected) ? selected[0] : selected;
    outputPath.value = path;
    const rootName = path.split(/[\\/]/).pop() || path;
    selectedInfo.value = t('settings.selected.picked', { name: rootName });
  } catch (err: any) {
    warning.value = t('settings.outputPath.warning.select_error', { msg: err.message || String(err) });
  }
}

function saveOutputPath() {
  if (!rules.non_empty(outputPath.value)) {
    warning.value = t('settings.outputPath.warning.empty');
    return;
  }
  localStorage.setItem('outputPath', outputPath.value);
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

async function copyPath() {
  if (!outputPath.value) return;
  try {
    await writeText(outputPath.value);
    saved.value = true;
    setTimeout(() => (saved.value = false), 1500);
  } catch {
    warning.value = t('settings.outputPath.warning.copy_error');
  }
}

function clearPath() {
  outputPath.value = '';
  selectedInfo.value = null;
  localStorage.removeItem('outputPath');
}

// 自定义背景相关函数
async function selectBackground() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'] }],
      defaultPath: await appConfigDir(),
    });

    if (selected === null) return;

    const path = Array.isArray(selected) ? selected[0] : selected;
    backgroundPath.value = path as string;
    // 选择后立即保存并应用
    saveBackground();
  } catch (err: any) {
    console.error('Failed to select background:', err);
  }
}

function saveBackground() {
  if (backgroundPath.value) {
    localStorage.setItem('customBackground', backgroundPath.value);
  } else {
    localStorage.removeItem('customBackground');
  }
  // 触发自定义事件通知 App.vue 更新背景
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { 
    detail: backgroundPath.value 
  }));
  backgroundSaved.value = true;
  setTimeout(() => (backgroundSaved.value = false), 1500);
}

function clearBackground() {
  backgroundPath.value = '';
  localStorage.removeItem('customBackground');
  // 触发自定义事件通知 App.vue 更新背景
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { 
    detail: null 
  }));
  backgroundSaved.value = true;
  setTimeout(() => (backgroundSaved.value = false), 1500);
}
</script>

<template>
  <v-card class="pa-6 app-card">
    <!-- 输出路径设置 -->
    <div class="section-title">输出设置</div>
    <div class="controls-grid">
      <v-text-field
        v-model="outputPath"
        :label="t('settings.outputPath.label')"
        :placeholder="t('settings.outputPath.placeholder')"
        :rules="[rules.non_empty]"
        clearable
        dense
        :hint="t('settings.outputPath.hint')"
        persistent-hint
        append-outer-icon="mdi-folder-open"
        @click:append-outer="selectFolder"
        :aria-label="t('settings.outputPath.aria')"
      />

      <div class="select-button">
        <v-btn small @click="selectFolder" :title="t('settings.selectFolder')" class="select-btn">
          <v-icon left>mdi-folder</v-icon>
          {{ t('settings.selectFolder') }}
        </v-btn>
      </div>

      <div v-if="selectedInfo" class="mt-2 caption" role="status" aria-live="polite">
        {{ selectedInfo }}
      </div>
    </div>

    <div class="button-area" role="group" :aria-label="t('settings.selectFolder')">
      <v-btn small color="primary" @click="saveOutputPath">
        {{ t('settings.save') }}
      </v-btn>
      <v-btn small icon @click="copyPath" :disabled="!outputPath" :title="t('settings.copy')">
        <v-icon>mdi-content-copy</v-icon>
      </v-btn>
      <v-btn small icon @click="clearPath" :disabled="!outputPath" :title="t('settings.clear')">
        <v-icon>mdi-close</v-icon>
      </v-btn>
    </div>

    <v-divider class="my-6"></v-divider>

    <!-- 自定义背景设置 -->
    <div class="section-title">{{ t('settings.background.label') }}</div>
    <div class="controls-grid">
      <v-text-field
        v-model="backgroundPath"
        :label="t('settings.background.label')"
        :placeholder="t('settings.background.placeholder')"
        clearable
        dense
        :hint="t('settings.background.hint')"
        persistent-hint
        append-outer-icon="mdi-image"
        @click:append-outer="selectBackground"
        readonly
      />

      <div class="select-button">
        <v-btn small @click="selectBackground" :title="t('settings.background.selectFile')" class="select-btn">
          <v-icon left>mdi-image</v-icon>
          {{ t('settings.background.selectFile') }}
        </v-btn>
      </div>
    </div>

    <!-- 背景预览 -->
    <div v-if="backgroundPath" class="background-preview mt-4">
      <div class="preview-label">预览</div>
      <div class="preview-container">
        <img :src="backgroundPreviewUrl" alt="Background preview" class="preview-image" />
      </div>
    </div>

    <div class="button-area mt-4">
      <v-btn small color="primary" @click="saveBackground">
        {{ t('settings.save') }}
      </v-btn>
      <v-btn small icon @click="clearBackground" :disabled="!backgroundPath" :title="t('settings.background.clear')">
        <v-icon>mdi-close</v-icon>
      </v-btn>
    </div>

    <v-snackbar v-model="saved" :timeout="1500" color="success" top right>
      {{ t('settings.saved') }}
    </v-snackbar>

    <v-snackbar v-model="backgroundSaved" :timeout="1500" color="success" top right>
      {{ t('settings.background.saved') }}
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
  background: rgba(18, 18, 18, 0.95) !important;
  backdrop-filter: blur(12px) !important;
  -webkit-backdrop-filter: blur(12px) !important;
  color: #ffffff;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5) !important;
  border-radius: 16px !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  max-height: calc(100vh - 150px);
  overflow-y: auto;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.controls-grid {
  display: grid;
  grid-template-columns: minmax(260px, 1fr) auto;
  gap: 16px;
  align-items: center;
  margin-bottom: 20px;
}

.select-button {
  display: flex;
  align-items: center;
}

.select-btn {
  white-space: nowrap;
  background: rgba(50, 50, 50, 0.9) !important;
  color: #fff !important;
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.select-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
  background: rgba(70, 70, 70, 0.9) !important;
}

.path-area .path-field {
  width: 100%;
  box-sizing: border-box;
}

.button-area {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
}

.button-wrapper {
  display: flex;
  gap: 8px;
  flex-wrap: nowrap;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  padding-bottom: 2px;
}

/* 自定义按钮样式 */
.button-area .v-btn {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.3s ease;
  text-transform: none;
}

.button-area .v-btn[color="primary"] {
  background: rgba(60, 60, 60, 0.9) !important;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.button-area .v-btn[color="primary"]:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
  background: rgba(80, 80, 80, 0.9) !important;
}

.button-area .v-btn.v-btn--icon {
  background: rgba(255, 255, 255, 0.1) !important;
  color: rgba(255, 255, 255, 0.8) !important;
}

.button-area .v-btn.v-btn--icon:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: #fff !important;
}

/* 文本框样式 */
:deep(.v-text-field) {
  margin-bottom: 8px;
}

:deep(.v-text-field .v-field) {
  background: rgba(255, 255, 255, 0.08) !important;
  border-radius: 8px !important;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  color: #fff !important;
}

:deep(.v-text-field .v-field__input) {
  color: #fff !important;
}

:deep(.v-text-field .v-label) {
  color: rgba(255, 255, 255, 0.7) !important;
}

:deep(.v-text-field .v-field__outline) {
  color: rgba(255, 255, 255, 0.2) !important;
}

/* 提示文本样式 */
:deep(.v-text-field .v-messages__message) {
  color: rgba(255, 255, 255, 0.6) !important;
}

/* 警告提示样式 */
:deep(.v-alert) {
  background: rgba(255, 82, 82, 0.15) !important;
  color: #ff8a80 !important;
  border: 1px solid rgba(255, 82, 82, 0.3) !important;
  border-radius: 8px !important;
}

/* 成功提示样式 */
:deep(.v-snackbar .v-snackbar__wrapper) {
  background: rgba(76, 175, 80, 0.9) !important;
  backdrop-filter: blur(8px) !important;
  border-radius: 8px !important;
}

/* 背景预览样式 */
.background-preview {
  margin-top: 16px;
}

.preview-label {
  font-size: 12px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.6);
  margin-bottom: 8px;
}

.preview-container {
  width: 100%;
  max-width: 400px;
  height: 150px;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.3);
}

.preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* 分隔线样式 */
:deep(.v-divider) {
  border-color: rgba(255, 255, 255, 0.1) !important;
}

@media (max-width: 720px) {
  .controls-grid {
    grid-template-columns: 1fr;
  }

  .button-area {
    margin-top: 16px;
    justify-content: flex-start;
  }

  .button-wrapper {
    flex-wrap: wrap;
    overflow-x: visible;
  }

  .preview-container {
    height: 120px;
  }
}

.caption {
  color: rgba(255,255,255,0.7);
  font-size: 0.875rem;
  margin-top: 4px;
  display: block;
}
</style>
