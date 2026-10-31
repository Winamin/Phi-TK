<i18n>

en:
  settings:
    title: Settings
    warning:
      noBackground: "Set a custom background image first — there is nothing to derive colours from"
      derive_failed: "Could not derive colours from this image"
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
      wallpaperColorDone: "Accent colour derived from the wallpaper: {color}"
zh-CN:
  settings:
    title: 设置
    warning:
      noBackground: "请先设置自定义背景图，否则没有可取色的来源"
      derive_failed: "无法从这张图片取色（格式或读取失败）"
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
      wallpaperColorDone: "已从壁纸取色：{color}"
</i18n>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { appConfigDir } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/core';

import { toast } from './common';
import {
  type ThemeMode,
  applyThemeMode,
  deriveColorFromWallpaper,
  getThemeMode,
  isWallpaperColorEnabled,
  resetColorScheme,
  setWallpaperColorEnabled,
  BACKGROUND_KEY,
  getDerivedColor,
} from './theme';
import TipSwitch from './components/TipSwitch.vue';
import MdTextField from './components/md/MdTextField.vue';

defineOptions({ name: 'SettingsPanel' });

const { t } = useI18n();

const backgroundPath = ref<string>(localStorage.getItem(BACKGROUND_KEY) || '');

const themeMode = ref<ThemeMode>(getThemeMode());
const wallpaperColor = ref(isWallpaperColorEnabled());
const derivedColor = ref<string | null>(getDerivedColor());

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

function onThemeModeChange(e: Event) {
  const mode = (e.target as HTMLElement & { value?: string }).value as ThemeMode;
  if (!mode || mode === themeMode.value) return;
  themeMode.value = mode;
  applyThemeMode(mode);
}

async function onWallpaperColorChange(enabled: boolean) {
  wallpaperColor.value = enabled;
  setWallpaperColorEnabled(enabled);
  if (!enabled) {
    resetColorScheme();
    derivedColor.value = null;
    return;
  }
  if (!backgroundPath.value) {
    toast(t('settings.warning.noBackground'), 'warning');
    return;
  }
  const hex = await deriveColorFromWallpaper(backgroundPath.value);
  derivedColor.value = hex;
  if (hex) toast(t('settings.appearance.wallpaperColorDone', { color: hex }), 'success');
  else toast(t('settings.warning.derive_failed'), 'warning');
}

async function selectBackground() {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'] }], defaultPath: await appConfigDir() });
    if (selected === null) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    backgroundPath.value = path as string;
    await saveBackground();
  } catch (err: any) {
    console.error('Failed to select background:', err);
  }
}

async function saveBackground() {
  if (backgroundPath.value) localStorage.setItem(BACKGROUND_KEY, backgroundPath.value);
  else localStorage.removeItem(BACKGROUND_KEY);
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: backgroundPath.value }));
  if (wallpaperColor.value && backgroundPath.value) {
    const hex = await deriveColorFromWallpaper(backgroundPath.value);
    derivedColor.value = hex;
    if (!hex) toast(t('settings.warning.derive_failed'), 'warning');
  }
  toast(t('settings.background.saved'), 'success');
}

function clearBackground() {
  backgroundPath.value = '';
  localStorage.removeItem(BACKGROUND_KEY);
  window.dispatchEvent(new CustomEvent('customBackgroundChanged', { detail: null }));
  resetColorScheme();
  toast(t('settings.background.saved'), 'success');
}
</script>

<template>
  <div class="settings-container">
    <div class="settings-scroll">
      <div class="section">
        <div class="section-header">
          <mdui-icon-palette--outlined class="section-icon"></mdui-icon-palette--outlined>
          <span class="section-title">{{ t('settings.appearance.label') }}</span>
        </div>
        <div class="section-card">
          <div class="setting-item theme-setting">
            <div class="item-text">
              <span class="item-label">{{ t('settings.appearance.mode') }}</span>
            </div>
            <mdui-segmented-button-group class="theme-options" :aria-label="t('settings.appearance.mode')" selects="single" :value="themeMode" @change="onThemeModeChange">
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
              <span v-if="wallpaperColor && derivedColor" class="item-hint derived-color">
                <i class="derived-dot" :style="{ background: derivedColor }"></i>{{ derivedColor }}
              </span>
            </div>
            <TipSwitch
              class="wallpaper-switch"
              :model-value="wallpaperColor"
              :label="''"
              :aria-label="t('settings.appearance.wallpaperColor')"
              @update:model-value="onWallpaperColorChange" />
          </div>
        </div>
      </div>

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
              <mdui-button-icon :aria-label="t('settings.background.clear')" :disabled="!backgroundPath" @click="clearBackground">
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
.settings-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
}
.settings-scroll {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  gap: var(--app-section-gap, 24px);
  overflow-y: auto;
  padding: var(--app-page-padding, 16px);
  overscroll-behavior-y: contain;
}
.section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px;
}
.section-icon {
  font-size: 20px;
  color: rgb(var(--mdui-color-primary));
}
.section-title {
  font-size: var(--mdui-typescale-title-small-size);
  font-weight: var(--mdui-typescale-title-small-weight);
  line-height: var(--mdui-typescale-title-small-line-height);
  color: rgb(var(--mdui-color-primary));
}
.section-card {
  overflow: hidden;
  background-color: rgb(var(--mdui-color-surface-container-low));
  border-radius: var(--mdui-shape-corner-medium, 12px);
}
.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 72px;
  padding: var(--app-card-padding, 16px);
}
.setting-item.item-block,
.theme-setting {
  flex-direction: column;
  align-items: stretch;
}
.theme-options {
  width: 100%;
  min-width: 0;
}
.theme-options mdui-segmented-button {
  flex: 1;
  min-width: 0;
  min-height: var(--app-touch-target, 48px);
}
.item-text {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}
.wallpaper-switch {
  width: auto;
  flex: 0 0 auto;
}
.item-label {
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
  color: rgb(var(--mdui-color-on-surface));
}
.item-hint {
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow-wrap: anywhere;
}
.px-4 {
  padding: 0 16px;
}
.setting-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 16px 16px;
}
.action-spacer {
  flex: 1;
}
.bg-preview {
  margin: 0 16px 16px;
  max-width: 360px;
  height: 140px;
  overflow: hidden;
  border-radius: var(--mdui-shape-corner-medium, 12px);
  background-color: rgb(var(--mdui-color-surface-container-highest));
}
.preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.derived-color {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: monospace;
}
.derived-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgb(var(--mdui-color-outline));
}
@media (max-width: 399px) {
  .setting-actions > mdui-button {
    flex: 1 1 auto;
  }
  .action-spacer {
    display: none;
  }
}
</style>
