<template>
  <i18n>
en:
  app: Phi TK

zh-CN:
  app: Phi TK
</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';

const appVersion = ref<string>('');
getVersion().then((version) => {
  appVersion.value = version;
});

const openGitHub = () => {
  open('https://github.com/Winamin/Phi-TK.git');
};
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <h1 v-t="'app'"></h1>
    <h4 class="mt-n2">v{{ appVersion }}</h4>
    <v-btn prepend-icon="mdi-github" @click="openGitHub">GitHub</v-btn>
    <p>Licensed by GPLv3</p>
  </div>
</template>

<style scoped>
.glass-background {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
}

.gradient-text {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.text-glow {
  text-shadow: 0 0 8px rgba(33, 150, 243, 0.5);
}

.hover-scale {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #4CAF50, #FFEB3B);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>
