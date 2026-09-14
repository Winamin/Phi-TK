<i18n>
en:
  app: Phi TK
  license: Licensed by GPLv3
  source: View source code
  footer:
    copyright: © 2025 Phi TK. All rights reserved.

zh-CN:
  app: Phi TK
  license: 基于 GPLv3 协议授权
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
        <mdui-chip class="version-badge" elevated>
          <mdui-icon-label--outlined slot="icon"></mdui-icon-label--outlined>
          v{{ appVersion }}
        </mdui-chip>
      </div>

      <div class="info-cards">
        <mdui-card variant="filled" clickable class="info-card" :style="{ '--i': 0 }" @click="openGitHub">
          <div class="card-content">
            <div class="card-icon">
              <mdui-icon-github></mdui-icon-github>
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

        <mdui-card variant="filled" class="info-card" :style="{ '--i': 1 }">
          <div class="card-content">
            <div class="card-icon">
              <mdui-icon-workspace-premium></mdui-icon-workspace-premium>
            </div>
            <div class="card-text">
              <h3 class="card-title">License</h3>
              <p class="card-subtitle">{{ t('license') }}</p>
            </div>
          </div>
        </mdui-card>

        <mdui-card variant="filled" class="info-card" :style="{ '--i': 2 }">
          <div class="card-content">
            <div class="card-icon">
              <mdui-icon-info--outlined></mdui-icon-info--outlined>
            </div>
            <div class="card-text">
              <h3 class="card-title">Version</h3>
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
@use './styles/breakpoints' as bp;
@use './styles/motion' as mo;

.about-container {
  width: 100%;
  height: 100vh;
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  padding: 24px;
}

.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 40px;
  z-index: 1;
  width: 100%;
  max-width: 450px;
}

/* ===== Logo + version chip ===== */
.app-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  opacity: 0;
  transform: translateY(-20px);
  /* Spatial: the logo slides into place, so it settles on the spring. */
  animation: headerAppear var(--app-motion-duration-spatial-slow)
    var(--app-motion-spring-spatial-slow, var(--mdui-motion-easing-emphasized-decelerate)) 0.2s forwards;
}

@keyframes headerAppear {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.app-logo-img {
  width: 120px;
  max-width: 70vw;
  height: auto;
  object-fit: contain;
  /* Not an elevation token: `drop-shadow()` takes no spread radius and no shadow list. */
  filter: drop-shadow(0 6px 18px rgba(var(--mdui-color-shadow), 0.45));
  @include mo.spatial((transform, filter), $speed: slow);
}

.app-logo-img:hover {
  transform: translateY(-4px) scale(1.03);
}

.version-badge {
  font-family: 'Roboto Mono', 'Consolas', monospace;
}

/* ===== Info cards ===== */
.info-cards {
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: 100%;
}

/* `--i` is set per card from the template and staggers the entrance. */
.info-card {
  display: block;
  opacity: 0;
  transform: translateX(-30px);
  animation: cardSlideIn var(--app-motion-duration-spatial-default)
    var(--app-motion-spring-spatial-default, var(--mdui-motion-easing-emphasized-decelerate)) forwards;
  --i: 0;
  animation-delay: calc(0.15s * var(--i) + 0.6s);
  @include mo.spatial((transform, box-shadow));
}

.info-card:nth-child(even) {
  transform: translateX(30px);
}

@keyframes cardSlideIn {
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.info-card[clickable]:hover {
  transform: translateY(-3px);
  box-shadow: var(--mdui-elevation-level3);
}

.card-content {
  display: flex;
  align-items: center;
  padding: 18px 20px;
  gap: 16px;
}

.card-icon {
  width: 50px;
  height: 50px;
  border-radius: var(--mdui-shape-corner-medium);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgb(var(--mdui-color-secondary-container));
  color: rgb(var(--mdui-color-on-secondary-container));
  font-size: 1.75rem;
  flex-shrink: 0;
}

.card-text {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: var(--mdui-typescale-title-medium-size);
  font-weight: var(--mdui-typescale-title-medium-weight);
  line-height: var(--mdui-typescale-title-medium-line-height);
  color: rgb(var(--mdui-color-on-surface));
  margin: 0 0 4px 0;
}

.card-subtitle {
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
  color: rgb(var(--mdui-color-on-surface-variant));
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-arrow {
  display: flex;
  color: rgb(var(--mdui-color-on-surface-variant));
  transition: transform var(--mdui-motion-duration-short3) var(--mdui-motion-easing-standard);
}

.info-card:hover .card-arrow {
  transform: translateX(4px);
}

/* ===== Footer ===== */
.about-footer {
  margin-top: 8px;
  opacity: 0;
  animation: fadeIn 0.5s var(--mdui-motion-easing-standard) 1.4s forwards;
}

@keyframes fadeIn {
  to {
    opacity: 1;
  }
}

.footer-copyright {
  font-size: var(--mdui-typescale-body-small-size);
  color: rgb(var(--mdui-color-outline));
  margin: 0;
  text-align: center;
}

/* ===== Responsive ===== */
@include bp.below-expanded {
  .about-container {
    padding: 16px;
  }
  .app-logo-img {
    width: 100px;
  }
  .card-content {
    padding: 14px 16px;
  }
}

@include bp.compact {
  .about-content {
    gap: 28px;
  }
  .info-cards {
    gap: 10px;
  }
}
</style>
