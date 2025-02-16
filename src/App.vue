<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About

zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于
</i18n>

<script lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { VSonner } from 'vuetify-sonner';

const onLoaded = ref<() => void>();
const component = ref();

watch(component, (comp) => {
  if (comp && onLoaded.value) onLoaded.value();
});

export function useOnLoaded() {
  return onLoaded;
}

declare global {
  interface Window {
    goto: (name: string) => void;
  }
}

export default {};
</script>

<script setup lang="ts">
const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

const icons = {
  render: 'mdi-auto-fix',
  rpe: 'mdi-bookshelf',
  tasks: 'mdi-server',
  about: 'mdi-information-outline',
};

window.goto = (name: string) => {
  router.push({ name });
};
</script>

<template>
  <v-app id="Phi TK" class="dark-theme">
    <v-sonner position="top-center" />
    <v-app-bar 
      title="Phi TK" 
      :elevation="4"
      class="app-bar-shadow blur-background"
    ></v-app-bar>
    <v-navigation-drawer 
      expand-on-hover 
      rail 
      permanent
      :elevation="8"
      class="nav-drawer-border blur-background"
    >
      <v-list density="compact" nav class="py-4">
        <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"
          class="list-item-hover"
          active-class="active-item"
        ></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex justify-center animated-background">
      <router-view v-slot="{ Component }">
        <transition
          name="slide"
          mode="out-in"
        >
          <Suspense timeout="0">
            <template #default>
              <component :is="Component" ref="component" />
            </template>
            <template #fallback>
              <div class="loading-overlay">
                <v-progress-circular 
                  indeterminate 
                  size="64"
                  color="accent"
                  class="glow-spinner"
                />
              </div>
            </template>
          </Suspense>
        </transition>
      </router-view>
    </v-main>
  </v-app>
</template>

<style>
.dark-theme {
  background: linear-gradient(45deg, #0f0c29, #302b63, #24243e);
}

.app-bar-shadow {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3) !important;
}

.nav-drawer-border {
  border-right: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.list-item-hover {
  transition: 
    transform 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55),
    background 0.3s ease,
    box-shadow 0.4s ease,
    filter 0.3s ease;
  margin: 12px 16px;
  border-radius: 16px;
  transform-style: preserve-3d;
  filter: brightness(1);
}

.list-item-hover:hover {
  background: rgba(255, 255, 255, 0.08) !important;
  transform: 
    translateX(12px)
    scale(1.05)
    translateZ(20px);
  box-shadow: 
    0 8px 24px rgba(33, 150, 243, 0.2),
    0 4px 12px rgba(255, 255, 255, 0.1);
  filter: brightness(1.2);
}

.active-item {
  background: linear-gradient(
    135deg,
    rgba(33, 150, 243, 0.15) 0%,
    rgba(156, 39, 176, 0.15) 100%
  ) !important;
  border-left: 4px solid transparent;
  position: relative;
  overflow: hidden;
  transition: all 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.active-item::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    45deg,
    rgba(33, 150, 243, 0.1) 25%,
    rgba(156, 39, 176, 0.1) 50%,
    rgba(33, 150, 243, 0.1) 75%
  );
  animation: holographic-flow 6s linear infinite;
  z-index: -1;
}

@keyframes holographic-flow {
  0% { transform: translate(0, 0) rotate(0deg); }
  100% { transform: translate(-50%, -50%) rotate(360deg); }
}

.active-item::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border: 2px solid transparent;
  border-image: linear-gradient(135deg, #2196F3 0%, #9C27B0 100%);
  border-image-slice: 1;
  border-radius: 12px;
  animation: border-glow 2s ease-in-out infinite;
}

@keyframes border-glow {
  0%, 100% { opacity: 0.8; }
  50% { opacity: 0.2; }
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  perspective: 1200px;
  transform-style: preserve-3d;
}

.slide-enter-from {
  opacity: 0;
  transform: 
    rotateY(45deg)
    translateZ(-200px)
    scale(0.95);
  filter: blur(8px);
}

.slide-leave-to {
  opacity: 0;
  transform: 
    rotateY(-45deg)
    translateZ(-200px)
    scale(0.95);
  filter: blur(8px);
}

.glow-spinner {
  animation: 
    spin 1.5s linear infinite,
    color-pulse 2s ease-in-out infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes color-pulse {
  0%, 100% { 
    filter: drop-shadow(0 0 12px #2196F3);
  }
  50% { 
    filter: 
      drop-shadow(0 0 16px #9C27B0)
      drop-shadow(0 0 24px #2196F3);
  }
}

.animated-background {
  position: relative;
  overflow: hidden;
}

.animated-background::before {
  content: '';
  position: absolute;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    45deg,
    rgba(255, 255, 255, 0.01) 25%,
    transparent 25%,
    transparent 50%,
    rgba(255, 255, 255, 0.01) 50%,
    rgba(255, 255, 255, 0.01) 75%,
    transparent 75%,
    transparent
  );
  animation: animateFlow 0.8s linear infinite;
  opacity: 0.1;
}

@keyframes animateFlow {
  0% { transform: translate(-25%, -25%) rotate(0deg); }
  100% { transform: translate(-25%, -25%) rotate(360deg); }
}

.animated-background::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: 
    radial-gradient(circle at 20% 80%, rgba(156, 39, 176, 0.05) 0%, transparent 40%),
    radial-gradient(circle at 80% 20%, rgba(33, 150, 243, 0.05) 0%, transparent 40%);
  animation: particle-drift 20s linear infinite;
}

@keyframes particle-drift {
  0% { transform: translate(0, 0); }
  50% { transform: translate(3%, 3%); }
  100% { transform: translate(0, 0); }
}

.blur-background {
  backdrop-filter: blur(50px) saturate(180%);
  background: linear-gradient(45deg, rgba(168, 98, 153, 0.403), rgba(101, 66, 182, 0.6)) !important;
  transform: translateZ(0);
  position: relative;
  z-index: 1;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  backdrop-filter: blur(20px);
  background: rgba(16, 16, 36, 0.8);
  z-index: 999;
  display: flex;
  align-items: center;
  justify-content: center;
}

html {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

html::-webkit-scrollbar {
  display: none;
}
</style>
