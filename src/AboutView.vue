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
import * as shell from "@tauri-apps/plugin-shell"

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

const letters = ref<string[]>([])
const containerRef = ref<HTMLElement | null>(null)

onMounted(() => {
  fetchVersion()
  letters.value = t('app').split('')

  const handleMouseMove = (e: MouseEvent) => {
    if (!containerRef.value) return

    const { left, top, width, height } = containerRef.value.getBoundingClientRect()
    const centerX = left + width / 2
    const centerY = top + height / 2
    const offsetX = (e.clientX - centerX) / 20
    const offsetY = (e.clientY - centerY) / 20

    containerRef.value.style.setProperty('--offset-x', `${offsetX}px`)
    containerRef.value.style.setProperty('--offset-y', `${offsetY}px`)
  }

  window.addEventListener('mousemove', handleMouseMove)
})

const throttle = (fn: Function, delay: number) => {
  let lastTime = 0
  return (...args: any[]) => {
    const now = Date.now()
    if (now - lastTime >= delay) {
      fn(...args)
      lastTime = now
    }
  }
}
</script>

<template>
  <div class="about-container">
    <h1 
      ref="containerRef"
      class="app-title"
      aria-label="Phi TK"
    >
      <span 
        v-for="(char, index) in letters"
        :key="index"
        class="letter"
        :style="{ '--i': index }"
      >
        {{ char === ' ' ? '&nbsp;' : char }}
      </span>
    </h1>
    
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
  display: inline-block;
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.05em;
  position: relative;
  overflow: hidden;
  height: 1.2em;
  transform: translate(var(--offset-x, 0), var(--offset-y, 0));
  transition: transform 0.2s ease-out;
}

.letter {
  display: inline-block;
  opacity: 0;
  transform: translateY(1em);
  animation: appear 0.6s forwards cubic-bezier(0.5, 1, 0.5, 1.2);
  animation-delay: calc(var(--i) * 0.1s);
  will-change: transform, opacity;
}

@keyframes appear {
  to {
    opacity: 1;
    transform: translateY(0);
  }
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
