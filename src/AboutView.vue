<i18n>
en:
  app: Phi TK
  license: Licensed by GPLv3
  license-title: License
  version-title: Version
  source: View source code
  footer:
    copyright: © 2025 Phi TK. All rights reserved.

zh-CN:
  app: Phi TK
  license: 基于 GPLv3 协议授权
  license-title: 许可证
  version-title: 版本
  source: 查看源代码
  footer:
    copyright: © 2025 Phi TK. 保留所有权利。
</i18n>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/plugin-shell';

const { t } = useI18n();

const appVersion = ref('0.1.63');
const fetchVersion = async () => {
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    console.error('Failed to get version:', e);
  }
};

const openGitHub = () => {
  open('https://github.com/Winamin/Phi-TK.git').catch((e) => {
    console.error('Failed to open GitHub:', e);
  });
};

onMounted(() => {
  fetchVersion();
});
</script>

<template>
  <div class="about-container">
    <div class="about-content">
      <div class="app-header">
        <img src="/phi-tklogo.png" alt="Phi TK" class="app-logo-img" />
        <span class="app-name md3-title-large">{{ t('app') }}</span>
      </div>

      <div class="info-cards">
        <mdui-card variant="filled" clickable class="info-card" :aria-label="t('source')" @click="openGitHub">
          <div class="card-content">
            <div class="card-icon">
              <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor" aria-hidden="true">
                <path
                  d="M12 .5C5.73.5.5 5.73.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56 0-.28-.01-1.02-.02-2-3.2.7-3.88-1.54-3.88-1.54-.53-1.34-1.29-1.7-1.29-1.7-1.05-.72.08-.71.08-.71 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.7 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.18a11.1 11.1 0 0 1 5.8 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.84 1.19 3.1 0 4.43-2.69 5.41-5.25 5.69.41.36.78 1.06.78 2.14 0 1.55-.01 2.8-.01 3.18 0 .31.21.68.8.56A11.51 11.51 0 0 0 23.5 12C23.5 5.73 18.27.5 12 .5z" />
              </svg>
            </div>
            <div class="card-text">
              <h3 class="card-title">GitHub</h3>
              <p class="card-subtitle">{{ t('source') }}</p>
            </div>
            <div class="card-arrow">
              <mdui-icon-open-in-new></mdui-icon-open-in-new>
            </div>
          </div>
        </mdui-card>

        <mdui-card variant="filled" class="info-card">
          <div class="card-content">
            <div class="card-icon">
              <mdui-icon-workspace-premium></mdui-icon-workspace-premium>
            </div>
            <div class="card-text">
              <h3 class="card-title">{{ t('license-title') }}</h3>
              <p class="card-subtitle">{{ t('license') }}</p>
            </div>
          </div>
        </mdui-card>

        <mdui-card variant="filled" class="info-card">
          <div class="card-content">
            <div class="card-icon">
              <mdui-icon-info--outlined></mdui-icon-info--outlined>
            </div>
            <div class="card-text">
              <h3 class="card-title">{{ t('version-title') }}</h3>
              <p class="card-subtitle">v{{ appVersion }}</p>
            </div>
          </div>
        </mdui-card>
      </div>

      <div class="about-footer">
        <p class="footer-copyright">{{ t('footer.copyright') }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.about-container {
  width: 100%;
  max-width: var(--app-content-max-width, 840px);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  overflow-y: auto;
  padding: var(--app-page-padding, 16px);
  overscroll-behavior-y: contain;
}
.about-content {
  display: flex;
  flex-direction: column;
  gap: var(--app-section-gap, 24px);
  width: 100%;
}
.app-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}
.app-logo-img {
  width: 64px;
  height: 64px;
  object-fit: contain;
}
.app-name {
  color: rgb(var(--mdui-color-on-surface));
}
.info-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}
.info-card {
  display: block;
  border-radius: var(--mdui-shape-corner-medium, 12px);
  background-color: rgb(var(--mdui-color-surface-container-low));
}
.card-content {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 80px;
  padding: var(--app-card-padding, 16px);
}
.card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  color: rgb(var(--mdui-color-on-surface-variant));
  font-size: 24px;
}
.card-icon svg {
  width: 24px;
  height: 24px;
}
.card-text {
  flex: 1;
  min-width: 0;
}
.card-title {
  margin: 0 0 4px;
  font-size: var(--mdui-typescale-body-large-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-body-large-line-height);
  color: rgb(var(--mdui-color-on-surface));
}
.card-subtitle {
  margin: 0;
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  overflow-wrap: anywhere;
}
.card-arrow {
  display: flex;
  color: rgb(var(--mdui-color-on-surface-variant));
}
.footer-copyright {
  margin: 0;
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
}
</style>
