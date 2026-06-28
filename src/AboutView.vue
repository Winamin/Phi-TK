<i18n>
en:
  app: Phi TK
  license: Licensed by GPLv3
  footer:
    copyright: © 2025 Phi TK. All rights reserved.

zh-CN:
  app: Phi TK
  license: 基于 GPLv3 协议授权
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
    <!-- 极简的时间线背景线 -->
    <div class="timeline-bg"></div>

    <div class="about-content">
      <!-- Logo 和版本号，带进入动画 -->
      <div class="app-header">
        <img src="/phi-tklogo.png" alt="Phi TK" class="app-logo-img" />
        <div class="version-badge">
          <v-icon size="16" icon="mdi-tag-outline" class="version-icon" />
          <span class="version-text">v{{ appVersion }}</span>
        </div>
      </div>

      <!-- 信息卡片，纵向排列，依次滑入 -->
      <div class="info-cards">
        <v-card class="info-card" :style="{ '--i': 0 }" @click="openGitHub" ripple>
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="28" icon="mdi-github" />
            </div>
            <div class="card-text">
              <h3 class="card-title">GitHub</h3>
              <p class="card-subtitle">View source code</p>
            </div>
            <div class="card-arrow">
              <v-icon size="20" icon="mdi-open-in-new" />
            </div>
          </div>
        </v-card>

        <v-card class="info-card" :style="{ '--i': 1 }">
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="28" icon="mdi-license" />
            </div>
            <div class="card-text">
              <h3 class="card-title">License</h3>
              <p class="card-subtitle">{{ t('license') }}</p>
            </div>
          </div>
        </v-card>

        <v-card class="info-card" :style="{ '--i': 2 }">
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="28" icon="mdi-information-outline" />
            </div>
            <div class="card-text">
              <h3 class="card-title">Version</h3>
              <p class="card-subtitle">v{{ appVersion }}</p>
            </div>
          </div>
        </v-card>
      </div>

      <!-- 底部版权 -->
      <div class="about-footer">
        <p class="footer-copyright">{{ t('footer.copyright') }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 容器：居中布局，暗色背景 */
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
  background-color: #121212;
}

/* 隐形的时间线背景：一条从左到右快速扫过的极细线，仅作氛围 */
.timeline-bg {
  position: absolute;
  top: 50%;
  left: -50%;
  width: 200%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.15), transparent);
  animation: scanLine 4s linear infinite;
  pointer-events: none;
}

@keyframes scanLine {
  0% {
    transform: translateX(-50%);
  }
  100% {
    transform: translateX(0%);
  }
}

/* 主要内容区域 */
.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 40px;
  z-index: 1;
  width: 100%;
  max-width: 450px;
}

/* 应用头部：Logo 和版本标签 */
.app-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  opacity: 0;
  transform: translateY(-20px);
  animation: headerAppear 0.7s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
  animation-delay: 0.2s;
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
  filter: drop-shadow(0 6px 18px rgba(0, 0, 0, 0.6));
  transition:
    transform 0.3s ease,
    filter 0.3s ease;
}

.app-logo-img:hover {
  transform: translateY(-4px) scale(1.03);
  filter: drop-shadow(0 10px 25px rgba(0, 0, 0, 0.7));
}

.version-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 14px;
  background: rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 20px;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.85);
}

/* 信息卡片容器：纵向排列 */
.info-cards {
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: 100%;
}

/* 卡片基础样式与进入动画 */
.info-card {
  background: rgba(30, 30, 30, 0.85);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  overflow: hidden;
  cursor: pointer;
  opacity: 0;
  transform: translateX(-30px);
  animation: cardSlideIn 0.55s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
  animation-delay: calc(0.15s * var(--i) + 0.6s);
  transition:
    transform 0.25s ease,
    box-shadow 0.25s ease,
    border-color 0.25s ease;
}

/* 偶数卡片从右侧滑入，增加节奏变化 */
.info-card:nth-child(even) {
  transform: translateX(30px);
}

@keyframes cardSlideIn {
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.info-card:hover {
  transform: translateY(-3px) scale(1.02);
  box-shadow: 0 14px 30px rgba(0, 0, 0, 0.5);
  border-color: rgba(255, 255, 255, 0.2);
}

/* 卡片内部布局 */
.card-content {
  display: flex;
  align-items: center;
  padding: 18px 20px;
  gap: 16px;
}

.card-icon {
  width: 50px;
  height: 50px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  flex-shrink: 0;
}

.card-text {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 1.05rem;
  font-weight: 600;
  color: white;
  margin: 0 0 4px 0;
}

.card-subtitle {
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.65);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-arrow {
  color: rgba(255, 255, 255, 0.4);
  transition:
    transform 0.2s ease,
    color 0.2s ease;
}

.info-card:hover .card-arrow {
  transform: translateX(4px);
  color: rgba(255, 255, 255, 0.85);
}

/* 底部版权 */
.about-footer {
  margin-top: 8px;
  opacity: 0;
  animation: fadeIn 0.5s ease forwards;
  animation-delay: 1.4s;
}

@keyframes fadeIn {
  to {
    opacity: 1;
  }
}

.footer-copyright {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.45);
  margin: 0;
  text-align: center;
}

/* 响应式微调 */
@media (max-width: 768px) {
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

@media (max-width: 480px) {
  .about-content {
    gap: 28px;
  }
  .info-cards {
    gap: 10px;
  }
}
</style>
