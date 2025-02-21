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

const routes = [
  {
    path: '/',
    name: 'home',
    redirect: '/render'
  },
  {
    path: '/render',
    name: 'render',
    component: () => import('@/views/RenderView.vue')
    },
]
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
      :elevation="4"
      class="app-bar-shadow blur-background"
    >
      <template v-if="route.name !== 'home'">
        <v-btn
          icon
          @click="router.go(-1)"
          class="ml-2"
        >
          <v-icon>mdi-arrow-left</v-icon>
        </v-btn>
      </template>
      <v-app-bar-title class="text-center">Phi TK</v-app-bar-title>
    </v-app-bar>

    <v-main>
      <router-view v-slot="{ Component }">
        <transition name="scale" mode="out-in">
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

      <!-- 中央导航菜单 -->
      <div v-if="route.name === 'home'" class="center-menu">
        <v-list
          nav
          class="glass-container py-6"
          density="compact"
        >
          <v-list-item
            v-for="key in ['render', 'rpe', 'tasks', 'about']"
            :key="key"
            :prepend-icon="icons[key as keyof typeof icons]"
            :title="t(key)"
            @click="router.push({ name: key })"
            class="menu-item"
          ></v-list-item>
        </v-list>
      </div>
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
    transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    background 0.3s ease,
    box-shadow 0.3s ease;
  margin: 8px 12px;
  border-radius: 12px;
}

.list-item-hover:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  transform: translateX(8px);
}

.active-item {
  background: linear-gradient(45deg, rgba(33, 150, 243, 0.2), transparent) !important;
  box-shadow: 2px 0 12px rgba(33, 150, 243, 0.2);
  border-left: 4px solid #2196F3;
  box-sizing: border-box;
  margin-left: 4px;
  transform: translateX(8px);
  transition: all 0.3s ease;
}

.active-item::before {
  content: '';
  position: absolute;
  left: -4px;
  top: 0;
  height: 100%;
  width: 4px;
  background: #2196F3;
  transform: scaleY(0);
  transition: transform 0.3s ease;
}

.active-item.active-item::before {
  transform: scaleY(1);
}
  
.glow-spinner {
  filter: drop-shadow(0 0 8px #2196F3);
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

.blur-background {
  backdrop-filter: blur(50px) saturate(180%);
  background: linear-gradient(45deg, rgba(168, 98, 153, 0.403), rgba(101, 66, 182, 0.6)) !important;
  transform: translateZ(0);
  position: relative;
  z-index: 1;
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: absolute;
  width: 100%;
}

.slide-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.slide-leave-to {
  opacity: 0;
  transform: translateX(-100%);
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

.v-navigation-drawer {
  overflow: visible !important;
}

.v-navigation-drawer::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  backdrop-filter: blur(50px) saturate(180%);
  background: linear-gradient(45deg, rgba(168, 98, 153, 0.403), rgba(101, 66, 182, 0.6)) !important;
  z-index: -1;
  width: calc(100% + 12px) !important;
  margin-right: -12px !important;
  right: -6px !important;
  border-radius: 0 16px 16px 0;
}

.nav-drawer-border {
  border-right: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.center-menu {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2;
}

.glass-container {
  background: rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(20px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.3),
    inset 0 0 12px rgba(255, 255, 255, 0.05);
}

.menu-item {
  margin: 12px 24px;
  border-radius: 16px;
  transition: 
    transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    background 0.3s ease;
  
  :deep(.v-list-item__prepend) {
    margin-right: 16px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.2);
  }
  
  &:hover {
    background: rgba(255, 255, 255, 0.08) !important;
    transform: translateX(8px) scale(1.02);
  }
  
  &.v-list-item--active {
    background: linear-gradient(
      45deg,
      rgba(33, 150, 243, 0.15),
      transparent
    ) !important;
    border-left: 4px solid #2196f3;
  }
}

.scale-enter-active,
.scale-leave-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  position: absolute;
  width: 100%;
}

.scale-enter-from {
  opacity: 0;
  transform: scale(0.9);
}

.scale-leave-to {
  opacity: 0;
  transform: scale(1.1);
}
</style>
