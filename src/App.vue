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

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { VSonner } from 'vuetify-sonner';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const onLoaded = ref<() => void>();
const component = ref();

export function useOnLoaded() {
  return onLoaded;
}
  
watch(component, (comp) => {
  if (comp && onLoaded.value) onLoaded.value();
});

declare global {
  interface Window {
    goto: (name: string) => void;
  }
}

window.goto = (name: string) => {
  router.push({ name });
};

const icons = {
  render: 'mdi-auto-fix',
  rpe: 'mdi-bookshelf',
  tasks: 'mdi-server',
  about: 'mdi-information-outline',
};
</script>

<template>
  <v-app id="Phi TK" class="dark-theme">
    <div class="glass-container fixed-top w-100 h-100">
      <v-app-bar 
        title="Phi TK" 
        color="surface-variant"
        :elevation="4"
        class="app-bar-shadow glass-item"
      ></v-app-bar>
    </div>

    <v-main class="d-flex justify-center animated-background">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" />
          </template>
          <template #fallback>
            <div class="flex justify-center pa-8">
              <v-progress-circular 
                indeterminate 
                size="large"
                color="accent"
                class="glow-spinner"
              />
            </div>
          </template>
        </Suspense>
      </router-view>
    </v-main>

    <v-navigation-drawer 
      expand-on-hover 
      rail 
      permanent
      color="surface"
      :elevation="8"
      class="nav-drawer-border"
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
  </v-app>
</template>

<style>
.dark-theme {
  background: linear-gradient(45deg, #0f0c29, #302b63, #24243e);
}

.glass-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  backdrop-filter: blur(8px) saturate(1.5);
  -webkit-backdrop-filter: blur(8px) saturate(1.5);
  background-color: rgba(255, 255, 255, 0.06);
  border-radius: 16px;
}

.glass-item {
  backdrop-filter: blur(4px) saturate(1.5);
  -webkit-backdrop-filter: blur(4px) saturate(1.5);
  background-color: rgba(255, 255, 255, 0.03);
}

.app-bar-shadow {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3) !important;
  backdrop-filter: blur(4px) saturate(1.5);
  -webkit-backdrop-filter: blur(4px) saturate(1.5);
}

.nav-drawer-border {
  border-right: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.list-item-hover {
  transition: all 0.3s ease;
  margin: 8px 12px;
  border-radius: 12px;
}

.list-item-hover:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  transform: translateX(8px);
}

.active-item {
  background: linear-gradient(45deg, rgba(33, 150, 243, 0.2), transparent) !important;
  border-left: 4px solid #2196F3;
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
  animation: animateFlow 40s linear infinite;
  opacity: 0.1;
}

@keyframes animateFlow {
  0% { transform: translate(-25%, -25%) rotate(0deg); }
  100% { transform: translate(-25%, -25%) rotate(360deg); }
}
</style>
