<i18n>
en:
  app: Phi TK
  license: Licensed by GPLv3
  footer:
    made: Made with ❤️ by the Phi TK team
    copyright: © 2025 Phi TK. All rights reserved.

zh-CN:
  app: Phi TK
  license: 基于 GPLv3 协议授权
  footer:
    made: 用 ❤️ 制作的 Phi TK
    copyright: © 2025 Phi TK. 保留所有权利。
</i18n>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getVersion } from '@tauri-apps/api/app'
import { open } from '@tauri-apps/plugin-shell'
import TextType from "./TextType.vue";

const { t } = useI18n()

const appVersion = ref('0.1.63')
const fetchVersion = async () => {
  try {
    appVersion.value = await getVersion()
  } catch (e) {
    console.error('Failed to get version:', e)
  }
}

const openGitHub = () => {
  open('https://github.com/Winamin/Phi-TK.git').catch((e) => {
    console.error('Failed to open GitHub:', e)
  })
}

onMounted(() => {
  fetchVersion()
})
</script>

<template>
  <div class="about-container">
    <!-- 顶部装饰 -->
    <div class="about-decoration">
      <div class="decoration-circle circle-1"></div>
      <div class="decoration-circle circle-2"></div>
      <div class="decoration-circle circle-3"></div>
    </div>

    <!-- 主要内容 -->
    <div class="about-content">
      <!-- Logo 和标题 -->
      <div class="app-header">
        <div class="logo-container">
          <div class="app-logo">
            <v-icon size="64" icon="mdi-alpha-p-circle-outline" class="logo-icon" />
          </div>
        </div>
        
        <div class="title-container">
          <TextType
            :text="[t('app')]"
            :typingSpeed="150"
            :pauseDuration="2000"
            :showCursor="true"
            cursorCharacter="|"
            class="app-title"
            aria-label="Phi TK"
          />
          <div class="version-badge">
            <v-icon size="16" icon="mdi-tag-outline" class="version-icon" />
            <span class="version-text">v{{ appVersion }}</span>
          </div>
        </div>
      </div>

      <!-- 信息卡片 -->
      <div class="info-cards">
        <!-- GitHub 卡片 -->
        <v-card 
          class="info-card github-card"
          @click="openGitHub"
          ripple
        >
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="32" icon="mdi-github" class="github-icon" />
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

        <!-- 许可证卡片 -->
        <v-card class="info-card license-card">
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="32" icon="mdi-license" class="license-icon" />
            </div>
            <div class="card-text">
              <h3 class="card-title">License</h3>
              <p class="card-subtitle">{{ t('license') }}</p>
            </div>
          </div>
        </v-card>

        <!-- 版本信息卡片 -->
        <v-card class="info-card version-card">
          <div class="card-content">
            <div class="card-icon">
              <v-icon size="32" icon="mdi-information-outline" class="info-icon" />
            </div>
            <div class="card-text">
              <h3 class="card-title">Version</h3>
              <p class="card-subtitle">v{{ appVersion }}</p>
            </div>
          </div>
        </v-card>
      </div>

      <!-- 底部信息 -->
      <div class="about-footer">
        <p class="footer-text">{{ t('footer.made') || 'Made with ❤️ by the Phi TK team' }}</p>
        <p class="footer-copyright">{{ t('footer.copyright') || '© 2023 Phi TK. All rights reserved.' }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
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

/* 装饰元素 */
.about-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
}

.decoration-circle {
  position: absolute;
  border-radius: 50%;
  filter: blur(60px);
  opacity: 0.4;
}

.circle-1 {
  width: 300px;
  height: 300px;
  background: linear-gradient(135deg, #2196F3, #64B5F6);
  top: -100px;
  left: -100px;
}

.circle-2 {
  width: 400px;
  height: 400px;
  background: linear-gradient(135deg, #9C27B0, #BA68C8);
  bottom: -150px;
  right: -150px;
}

.circle-3 {
  width: 250px;
  height: 250px;
  background: linear-gradient(135deg, #4CAF50, #81C784);
  top: 50%;
  right: 10%;
  transform: translateY(-50%);
}

/* 主要内容 */
.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 48px;
  z-index: 1;
  width: 100%;
  max-width: 800px;
}

/* 应用头部 */
.app-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  text-align: center;
}

.logo-container {
  position: relative;
}

.app-logo {
  width: 120px;
  height: 120px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
}

.app-logo:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3);
}

.logo-icon {
  background: linear-gradient(135deg, #2196F3, #9C27B0);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.title-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.app-title {
  font-size: 4rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  background: linear-gradient(135deg, #fff, #a8d8ff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  text-align: center;
  line-height: 1.2;
}

.version-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 20px;
}

.version-icon {
  color: rgba(255, 255, 255, 0.7);
}

.version-text {
  font-size: 0.875rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
}

/* 信息卡片 */
.info-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  width: 100%;
}

.info-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  overflow: hidden;
  transition: transform 0.3s ease, box-shadow 0.3s ease, border-color 0.3s ease;
  cursor: pointer;
}

.info-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.2);
  border-color: rgba(255, 255, 255, 0.2);
}

.github-card:hover {
  border-color: rgba(33, 150, 243, 0.5);
  box-shadow: 0 12px 32px rgba(33, 150, 243, 0.2);
}

.card-content {
  display: flex;
  align-items: center;
  padding: 20px;
  gap: 16px;
}

.card-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.github-card .card-icon {
  background: rgba(33, 150, 243, 0.1);
}

.license-card .card-icon {
  background: rgba(76, 175, 80, 0.1);
}

.version-card .card-icon {
  background: rgba(156, 39, 176, 0.1);
}

.github-icon {
  color: #2196F3;
}

.license-icon {
  color: #4CAF50;
}

.info-icon {
  color: #9C27B0;
}

.card-text {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: white;
  margin: 0 0 4px 0;
}

.card-subtitle {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.7);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-arrow {
  color: rgba(255, 255, 255, 0.5);
  transition: transform 0.2s ease;
}

.info-card:hover .card-arrow {
  transform: translateX(4px);
  color: rgba(255, 255, 255, 0.8);
}

/* 底部信息 */
.about-footer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
}

.footer-text,
.footer-copyright {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.6);
  margin: 0;
  text-align: center;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .about-container {
    padding: 16px;
  }
  
  .app-title {
    font-size: 3rem;
  }
  
  .app-logo {
    width: 100px;
    height: 100px;
  }
  
  .logo-icon {
    size: 48px;
  }
  
  .info-cards {
    grid-template-columns: 1fr;
  }
  
  .about-content {
    gap: 32px;
  }
}

@media (max-width: 480px) {
  .app-title {
    font-size: 2.5rem;
  }
  
  .app-logo {
    width: 80px;
    height: 80px;
  }
  
  .logo-icon {
    size: 40px;
  }
  
  .card-content {
    padding: 16px;
  }
  
  .card-icon {
    width: 48px;
    height: 48px;
  }
}
</style>