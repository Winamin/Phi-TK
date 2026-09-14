<i18n>
en:
  settings:
    title: Settings
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
    appearance:
      label: "Appearance"
      mode: "Theme"
      light: "Light"
      dark: "Dark"
      auto: "System"
      wallpaperColor: "Colours from wallpaper"
      wallpaperColorTip: "Derives the whole palette from your background image, Material You style. Turn it off to go back to the default blue."
zh-CN:
  settings:
    title: 设置
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
    appearance:
      label: "外观"
      mode: "主题"
      light: "亮色"
      dark: "深色"
      auto: "跟随系统"
      wallpaperColor: "跟随壁纸取色"
      wallpaperColorTip: "从背景图片中提取主色调，生成整套 Material You 配色。关闭后回到默认的蓝色。"
</i18n>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { appConfigDir } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

import { RULES as rules, toast } from './common';
import {
  type ThemeMode,
  applyThemeMode,
  deriveColorFromWallpaper,
  getThemeMode,
  isWallpaperColorEnabled,
  resetColorScheme,
  setWallpaperColorEnabled,
} from './theme';
import TipSwitch from './components/TipSwitch.vue';
import MdTextField from './components/md/MdTextField.vue';

defineOptions({ name: 'SettingsPanel' });

const { t } = useI18n();

const outputPath = ref<string>(localStorage.getItem('outputPath') || '');
const selectedInfo = ref<string | null>(null);

const backgroundPath = ref<string>(localStorage.getItem('customBackground') || '');

const themeMode = ref<ThemeMode>(getThemeMode());
const wallpaperColor = ref(isWallpaperColorEnabled());

const backgroundPreviewUrl = computed(() => {
  if (backgroundPath.value) {
    try { return convertFileSrc(backgroundPath.value); } catch { return backgroundPath.value; }
  }
  return '';
});

function onThemeModeChange(e: Event) {
  const mode = (e.target as HTMLElement & { value?: string }).value as ThemeMode;
  if (!mode || mode === themeMode.value) return;
  themeMode.value = mode;
  applyThemeMode(mode);
}

async function onWallpaperColorChange(enabled: boolean) {
  wallpaperColor.value = enabled;
  setWallpaperColorEnabled(enabled);
  if (enabled) {
    if (backgroundPath.value) await deriveColorFromWallpaper(backgroundPath.value);
  } else {
    resetColorScheme();
  }
}

async function selectFolder() {
  selectedInfo.value = null;
  try {
    const selected = await open({ directory: true, multiple: false, defaultPath: await appConfigDir() });
    if (selected === null) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    outputPath.value = path;
    const rootName = path.split(/[\\/]/).pop() || path;
    selectedInfo.value = t('settings.selected.picked', { name: rootName });
  } catch (err: any) {
    toast(t('settings.warning.select_error', { msg: err.message || String(err) }), 'warning');
  }
}

function saveOutputPath() {
  if (rules.non_empty(outputPath.value) !== true) { toast(t('settings.warning.empty'), 'warning'); return; }
  localStorage.setItem('outputPath', outputPath.value);
  toast(t('settings.saved'), 'success');
}

async function copyPath() {
  if (!outputPath.value) return;
  try { await writeText(outputPath.value); toast(t('settings.saved'), 'success'); }
  catch { toast(t('settings.warning.copy_error'), 'warning'); }
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
    await saveBackground();
  } catch (err: any) { console.error('Failed to select background:', err); }
}

async function saveBackground() {
  if (backgroundPath.value) localStorage.setItem('customBackground', backgroundPath.value);
  else localStorage.removeItem('customBackground');
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: backgroundPath.value }));
  if (wallpaperColor.value && backgroundPath.value) await deriveColorFromWallpaper(backgroundPath.value);
  toast(t('settings.background.saved'), 'success');
}

function clearBackground() {
  backgroundPath.value = '';
  localStorage.removeItem('customBackground');
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: null }));
  resetColorScheme();
  toast(t('settings.background.saved'), 'success');
}
</script>

<template>
  <div class="settings-container">
    <div class="settings-scroll">
      <h1 class="page-title md3-headline">{{ t('settings.title') }}</h1>

      <!-- Appearance Section -->
      <div class="section">
        <div class="section-header">
          <mdui-icon-palette--outlined class="section-icon"></mdui-icon-palette--outlined>
          <span class="section-title">{{ t('settings.appearance.label') }}</span>
        </div>
        <div class="section-card">
          <div class="setting-item">
            <div class="item-text">
              <span class="item-label">{{ t('settings.appearance.mode') }}</span>
            </div>
            <mdui-segmented-button-group
              selects="single"
              :value="themeMode"
              @change="onThemeModeChange">
              <mdui-segmented-button value="light">
                <mdui-icon-light-mode slot="icon"></mdui-icon-light-mode>
                {{ t('settings.appearance.light') }}
              </mdui-segmented-button>
              <mdui-segmented-button value="dark">
                <mdui-icon-dark-mode slot="icon"></mdui-icon-dark-mode>
                {{ t('settings.appearance.dark') }}
              </mdui-segmented-button>
              <mdui-segmented-button value="auto">
                <mdui-icon-brightness-auto slot="icon"></mdui-icon-brightness-auto>
                {{ t('settings.appearance.auto') }}
              </mdui-segmented-button>
            </mdui-segmented-button-group>
          </div>
          <mdui-divider></mdui-divider>
          <div class="setting-item">
            <div class="item-text">
              <span class="item-label">{{ t('settings.appearance.wallpaperColor') }}</span>
              <span class="item-hint">{{ t('settings.appearance.wallpaperColorTip') }}</span>
            </div>
            <TipSwitch
              :model-value="wallpaperColor"
              :label="''"
              @update:model-value="onWallpaperColorChange" />
          </div>
        </div>
      </div>

      <!-- Output Path Section -->
      <div class="section">
        <div class="section-header">
          <mdui-icon-folder-open--outlined class="section-icon"></mdui-icon-folder-open--outlined>
          <span class="section-title">{{ t('settings.outputPath.label') }}</span>
        </div>
        <div class="section-card">
          <div class="setting-item item-block">
            <MdTextField
              v-model="outputPath"
              variant="outlined"
              required
              clearable
              :label="t('settings.outputPath.label')"
              :placeholder="t('settings.outputPath.placeholder')"
              :helper="t('settings.outputPath.hint')">
              <mdui-button-icon slot="end-icon" :aria-label="t('settings.selectFolder')" @click="selectFolder">
                <mdui-icon-folder-open></mdui-icon-folder-open>
              </mdui-button-icon>
            </MdTextField>
          </div>
          <div v-if="selectedInfo" class="item-hint px-4">{{ selectedInfo }}</div>
          <div class="setting-actions">
            <mdui-button variant="tonal" @click="saveOutputPath">
              <mdui-icon-save--outlined slot="icon"></mdui-icon-save--outlined>
              {{ t('settings.save') }}
            </mdui-button>
            <mdui-button variant="text" @click="selectFolder">
              <mdui-icon-folder--outlined slot="icon"></mdui-icon-folder--outlined>
              {{ t('settings.selectFolder') }}
            </mdui-button>
            <div class="action-spacer"></div>
            <mdui-tooltip :content="t('settings.copy')">
              <mdui-button-icon :disabled="!outputPath" @click="copyPath">
                <mdui-icon-content-copy></mdui-icon-content-copy>
              </mdui-button-icon>
            </mdui-tooltip>
            <mdui-tooltip :content="t('settings.clear')">
              <mdui-button-icon :disabled="!outputPath" @click="clearPath">
                <mdui-icon-close></mdui-icon-close>
              </mdui-button-icon>
            </mdui-tooltip>
          </div>
        </div>
      </div>

      <!-- Background Section -->
      <div class="section">
        <div class="section-header">
          <mdui-icon-image--outlined class="section-icon"></mdui-icon-image--outlined>
          <span class="section-title">{{ t('settings.background.label') }}</span>
        </div>
        <div class="section-card">
          <div class="setting-item item-block">
            <MdTextField
              v-model="backgroundPath"
              variant="outlined"
              readonly
              :label="t('settings.background.label')"
              :placeholder="t('settings.background.placeholder')"
              :helper="t('settings.background.hint')">
              <mdui-button-icon slot="end-icon" :aria-label="t('settings.background.selectFile')" @click="selectBackground">
                <mdui-icon-image></mdui-icon-image>
              </mdui-button-icon>
            </MdTextField>
          </div>

          <div v-if="backgroundPath" class="bg-preview">
            <img :src="backgroundPreviewUrl" alt="Background preview" class="preview-img" />
          </div>

          <div class="setting-actions">
            <mdui-button variant="tonal" @click="selectBackground">
              <mdui-icon-image--outlined slot="icon"></mdui-icon-image--outlined>
              {{ t('settings.background.selectFile') }}
            </mdui-button>
            <div class="action-spacer"></div>
            <mdui-tooltip :content="t('settings.background.clear')">
              <mdui-button-icon :disabled="!backgroundPath" @click="clearBackground">
                <mdui-icon-close></mdui-icon-close>
              </mdui-button-icon>
            </mdui-tooltip>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './styles/breakpoints' as bp;
@use './styles/motion' as mo;

.settings-container {
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.settings-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.page-title {
  color: rgb(var(--mdui-color-on-surface));
  margin: 0 0 16px 0;
}

/* ===== Section ===== */
.section {
  display: flex;
  flex-direction: column;
  gap: 0;

  /* Sections rise in sequence as the page appears. */
  @include mo.enter-rise(10px);
  @include mo.stagger(6, 45ms, 40ms);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
}

.section-icon {
  font-size: 1.25rem;
  color: rgb(var(--mdui-color-primary));
}

.section-title {
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: var(--mdui-typescale-title-small-weight);
  color: rgb(var(--mdui-color-primary));
}

.section-card {
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-extra-large);
  padding: 0;
  overflow: hidden;
}

/* ===== Setting Item ===== */
.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  min-height: 56px;
}

.setting-item.item-block {
  flex-direction: column;
  align-items: stretch;
}

.item-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.item-label {
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
  color: rgb(var(--mdui-color-on-surface));
}

.item-hint {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}

.px-4 {
  padding-left: 20px;
  padding-right: 20px;
}

/* ===== Actions ===== */
.setting-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px 16px;
  flex-wrap: wrap;
}

.action-spacer {
  flex: 1;
  min-width: 8px;
}

/* ===== Background Preview ===== */
.bg-preview {
  margin: 0 20px;
  width: calc(100% - 40px);
  max-width: 360px;
  height: 140px;
  border-radius: var(--mdui-shape-corner-medium);
  overflow: hidden;
  background-color: rgb(var(--mdui-color-surface-container-highest));
}

.preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ===== Responsive ===== */
@include bp.compact {
  .settings-scroll { padding: 16px; }
  .setting-item { padding: 12px 16px; }
  .setting-actions { padding: 8px 16px 12px; }
  .action-spacer { display: none; }
}
</style>
