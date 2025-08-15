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
    <TextType
      :text="[t('app')]"
      :typingSpeed="200"
      :pauseDuration="1500"
      :showCursor="true"
      cursorCharacter=""
      class="app-title"
      aria-label="Phi TK"
    />

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
  width: 100%;
  height: 100%;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
}

.app-title {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.05em;
  height: 1.2em;
  display: flex;
  justify-content: center;
}

.version-label {
  font-size: 1.25rem;
  font-weight: 500;
  opacity: 0.8;
}

.github-btn {
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.license-text {
  font-size: 0.9rem;
  opacity: 0.7;
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