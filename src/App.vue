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
      color="surface-variant"
      :elevation="4"
      class="app-bar-shadow blur-background"
    ></v-app-bar>
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
  </v-app>
</template>

<style>
dark-theme {
  background: linear-gradient(45deg, #0f0c29, #302b63, #24243e);
}

.app-bar-shadow {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3) !important;
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

.blur-background {
  backdrop-filter: blur(10px);
  background-color: rgba(255, 182, 193, 0.5) !important;
}
</style>
