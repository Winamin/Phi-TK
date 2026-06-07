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
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

defineOptions({ name: 'SettingsPanel' });
import { RULES as rules } from './common';

const { t } = useI18n();

const outputPath = ref<string>(localStorage.getItem('outputPath') || '');
const saved = ref(false);
const warning = ref('');
const selectedInfo = ref<string | null>(null);

const backgroundPath = ref<string>(localStorage.getItem('customBackground') || '');
const backgroundSaved = ref(false);

const backgroundPreviewUrl = computed(() => {
  if (backgroundPath.value) {
    try { return convertFileSrc(backgroundPath.value); } catch { return backgroundPath.value; }
  }
  return '';
});

async function selectFolder() {
  warning.value = '';
  selectedInfo.value = null;
  try {
    const selected = await open({ directory: true, multiple: false, defaultPath: await appConfigDir() });
    if (selected === null) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    outputPath.value = path;
    const rootName = path.split(/[\\/]/).pop() || path;
    selectedInfo.value = t('settings.selected.picked', { name: rootName });
  } catch (err: any) {
    warning.value = t('settings.outputPath.warning.select_error', { msg: err.message || String(err) });
  }
}

function saveOutputPath() {
  if (!rules.non_empty(outputPath.value)) { warning.value = t('settings.outputPath.warning.empty'); return; }
  localStorage.setItem('outputPath', outputPath.value);
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

async function copyPath() {
  if (!outputPath.value) return;
  try { await writeText(outputPath.value); saved.value = true; setTimeout(() => (saved.value = false), 1500); }
  catch { warning.value = t('settings.outputPath.warning.copy_error'); }
}

function clearPath() {
  outputPath.value = '';
  selectedInfo.value = null;
  localStorage.removeItem('outputPath');
}

async function selectBackground() {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'] }], defaultPath: await appConfigDir() });
    if (selected === null) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    backgroundPath.value = path as string;
    saveBackground();
  } catch (err: any) { console.error('Failed to select background:', err); }
}

function saveBackground() {
  if (backgroundPath.value) localStorage.setItem('customBackground', backgroundPath.value);
  else localStorage.removeItem('customBackground');
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: backgroundPath.value }));
  backgroundSaved.value = true;
  setTimeout(() => (backgroundSaved.value = false), 1500);
}

function clearBackground() {
  backgroundPath.value = '';
  localStorage.removeItem('customBackground');
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: null }));
  backgroundSaved.value = true;
  setTimeout(() => (backgroundSaved.value = false), 1500);
}
</script>

<template>
  <div class="settings-container">
    <div class="settings-scroll">
      <!-- Output Path -->
      <div class="md3-card">
        <div class="card-label">{{ t('settings.outputPath.label') }}</div>
        <v-text-field
          v-model="outputPath"
          :label="t('settings.outputPath.label')"
          :placeholder="t('settings.outputPath.placeholder')"
          :rules="[rules.non_empty]"
          clearable
          density="compact"
          :hint="t('settings.outputPath.hint')"
          persistent-hint
          variant="outlined"
          append-outer-icon="mdi-folder-open"
          @click:append-outer="selectFolder"
        />
        <div v-if="selectedInfo" class="hint-text">{{ selectedInfo }}</div>
        <div class="card-actions">
          <button class="md3-btn md3-btn-tonal" @click="selectFolder">
            <v-icon icon="mdi-folder-outline" size="18" />
            <span>{{ t('settings.selectFolder') }}</span>
          </button>
          <button class="md3-btn md3-btn-filled" @click="saveOutputPath">
            <v-icon icon="mdi-content-save-outline" size="18" />
            <span>{{ t('settings.save') }}</span>
          </button>
          <button class="md3-btn md3-btn-text" @click="copyPath" :disabled="!outputPath" :title="t('settings.copy')">
            <v-icon icon="mdi-content-copy" size="18" />
          </button>
          <button class="md3-btn md3-btn-text" @click="clearPath" :disabled="!outputPath" :title="t('settings.clear')">
            <v-icon icon="mdi-close" size="18" />
          </button>
        </div>
      </div>

      <!-- Background -->
      <div class="md3-card">
        <div class="card-label">{{ t('settings.background.label') }}</div>
        <v-text-field
          v-model="backgroundPath"
          :label="t('settings.background.label')"
          :placeholder="t('settings.background.placeholder')"
          clearable
          density="compact"
          :hint="t('settings.background.hint')"
          persistent-hint
          variant="outlined"
          append-outer-icon="mdi-image"
          @click:append-outer="selectBackground"
          readonly
        />

        <div v-if="backgroundPath" class="bg-preview">
          <img :src="backgroundPreviewUrl" alt="Background preview" class="preview-img" />
        </div>

        <div class="card-actions">
          <button class="md3-btn md3-btn-tonal" @click="selectBackground">
            <v-icon icon="mdi-image-outline" size="18" />
            <span>{{ t('settings.background.selectFile') }}</span>
          </button>
          <button class="md3-btn md3-btn-filled" @click="saveBackground">
            <v-icon icon="mdi-content-save-outline" size="18" />
            <span>{{ t('settings.save') }}</span>
          </button>
          <button class="md3-btn md3-btn-text" @click="clearBackground" :disabled="!backgroundPath" :title="t('settings.background.clear')">
            <v-icon icon="mdi-close" size="18" />
          </button>
        </div>
      </div>
    </div>

    <v-alert v-if="warning" type="warning" class="mt-4" density="compact" variant="tonal">
      {{ warning }}
    </v-alert>

    <v-snackbar v-model="saved" :timeout="1500" color="success" location="top">
      {{ t('settings.saved') }}
    </v-snackbar>
    <v-snackbar v-model="backgroundSaved" :timeout="1500" color="success" location="top">
      {{ t('settings.background.saved') }}
    </v-snackbar>
  </div>
</template>

<style scoped>
.settings-container {
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  padding: 24px;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.settings-scroll {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ===== MD3 Card ===== */
.md3-card {
  background: rgba(30, 30, 30, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 20px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.card-label {
  font-size: 11px;
  font-weight: 700;
  color: rgba(130, 177, 255, 0.8);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.hint-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

/* ===== MD3 Buttons ===== */
.md3-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.2, 0, 0, 1);
  white-space: nowrap;
  font-family: inherit;
}
.md3-btn:disabled { opacity: 0.5; cursor: default; }
.md3-btn-text { background: transparent; color: rgba(255, 255, 255, 0.7); }
.md3-btn-text:hover:not(:disabled) { background: rgba(255, 255, 255, 0.08); }
.md3-btn-tonal { background: rgba(130, 177, 255, 0.12); color: #82b1ff; }
.md3-btn-tonal:hover:not(:disabled) { background: rgba(130, 177, 255, 0.2); }
.md3-btn-filled { background: #82b1ff; color: #002f65; font-weight: 600; }
.md3-btn-filled:hover:not(:disabled) { background: #a0c4ff; }

/* ===== Background Preview ===== */
.bg-preview {
  width: 100%;
  max-width: 360px;
  height: 140px;
  border-radius: 16px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.3);
}

.preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ===== Responsive ===== */
@media (max-width: 600px) {
  .settings-container { padding: 16px; }
}
</style>
