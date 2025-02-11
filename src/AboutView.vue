<i18n>
en:
  app: Phi TK
  license: Licensed by GPLv3

zh-CN:
  app: Phi TK
  license: 基于 GPLv3 协议授权
</i18n>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getVersion } from '@tauri-apps/api/app'
import { open } from '@tauri-apps/api/shell'

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
    <h1 class="app-title gradient-text">{{ t('app') }}</h1>
    
    <v-scale-transition>
      <h4 class="version-label text-glow">v{{ appVersion }}</h4>
    </v-scale-transition>

    <v-btn
      color="accent"
      variant="flat"
      prepend-icon="mdi-github"
      class="github-btn hover-scale"
      @click="openGitHub"
    >
      GitHub
    </v-btn>

    <p class="license-text text-gradient">{{ t('license') }}</p>
  </div>
</template>

<style scoped>
.about-container {
  padding: 2rem;
  min-width: 600px;
  min-height: 300px;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  animation: fadeIn 0.5s cubic-bezier(0, 0, 0, 1) forwards;
  opacity: 0;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    scale: 0.8;
    transform: translateY(0px);
  }
  to {
    opacity: 1;
    scale: 1;
    transform: translateY(0px);
  }
}

.app-title {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.version-label {
  font-size: 1.25rem;
  font-weight: 500;
  opacity: 0.8;
}

.github-btn {
  background: rgba(147, 147, 147, 0.2);
  padding: 0px 24px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 4px 4px 6px rgba(0, 0, 0, 0.1);
}

.license-text {
  font-size: 0.9rem;
  opacity: 0.7;
}

.gradient-text {
  background: linear-gradient(45deg, #2196f3, #e91e63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.text-glow {
  text-shadow: 0 0 12px rgba(33, 150, 243, 0.4);
}

.hover-scale {
  transition: transform 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #4caf50, #ffeb3b);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>
