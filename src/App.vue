<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About
  batch-render: Batch Render
  setting: Setting
  more: More
  back: Back
  navigation: Main navigation

zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务
  about: 关于
  batch-render: 批量渲染
  setting: 设置
  more: 更多
  back: 返回
  navigation: 主导航

</i18n>

<script lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

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
</script>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const navItems = [
  { key: 'render', icon: 'mdui-icon-play-circle--outlined', activeIcon: 'mdui-icon-play-circle' },
  { key: 'tasks', icon: 'mdui-icon-dns--outlined', activeIcon: 'mdui-icon-dns' },
  { key: 'batch-render', icon: 'mdui-icon-pending-actions--outlined', activeIcon: 'mdui-icon-pending-actions' },
  { key: 'setting', icon: 'mdui-icon-settings--outlined', activeIcon: 'mdui-icon-settings' },
];

const moreItems = [
  { key: 'rpe', icon: 'mdui-icon-menu-book--outlined' },
  { key: 'about', icon: 'mdui-icon-info--outlined', activeIcon: 'mdui-icon-info' },
];

const moreOpen = ref(false);
const navValue = computed(() => (navItems.some((item) => item.key === route.name) ? String(route.name) : ''));
const pageTitle = computed(() => t(String(route.name ?? 'render')));
const secondaryPage = computed(() => moreItems.some((item) => item.key === route.name));
const lastPrimaryPage = ref('render');

watch(
  () => route.name,
  (name) => {
    if (navItems.some((item) => item.key === name)) lastPrimaryPage.value = String(name);
  },
);

const navigateTo = (name: string) => {
  moreOpen.value = false;
  router.push({ name });
};

window.goto = navigateTo;

function onNavChange(e: Event) {
  const name = (e.target as HTMLElement & { value?: string }).value;
  if (name && name !== route.name) {
    navigateTo(name);
  }
}

const customBackground = ref<string | null>(null);

const railHidden = ref(false);

watch(
  [railHidden, secondaryPage],
  ([focused, secondary]) => {
    document.documentElement.classList.toggle('app-focus-mode', focused || secondary);
  },
  { immediate: true },
);

function onRailHidden(e: Event) {
  railHidden.value = (e as CustomEvent<boolean>).detail === true;
}

onMounted(() => {
  window.addEventListener('rail-hidden', onRailHidden as EventListener);
});
onUnmounted(() => {
  window.removeEventListener('rail-hidden', onRailHidden as EventListener);
  document.documentElement.classList.remove('app-focus-mode');
});

function onBackgroundChanged(event: Event) {
  customBackground.value = (event as CustomEvent<string | null>).detail;
}

onMounted(() => {
  customBackground.value = localStorage.getItem('customBackground');
  window.addEventListener('customBackgroundChanged', onBackgroundChanged);
});
onUnmounted(() => window.removeEventListener('customBackgroundChanged', onBackgroundChanged));

const backgroundStyle = computed(() => {
  if (customBackground.value) {
    try {
      const imageUrl = convertFileSrc(customBackground.value);
      return {
        backgroundImage: `url('${imageUrl}')`,
        backgroundSize: 'cover',
        backgroundPosition: 'center',
        backgroundRepeat: 'no-repeat',
        backgroundAttachment: 'fixed',
      };
    } catch {
      return {};
    }
  }
  return {};
});
</script>

<template>
  <div v-if="customBackground" class="custom-bg-layer" :style="backgroundStyle" aria-hidden="true"></div>
  <div v-if="customBackground" class="custom-bg-overlay" aria-hidden="true"></div>

  <div class="mobile-layout" :class="{ 'focus-mode': railHidden || secondaryPage }">
    <mdui-top-app-bar v-if="!railHidden" variant="small" class="app-bar">
      <mdui-button-icon v-if="secondaryPage" :aria-label="t('back')" @click="navigateTo(lastPrimaryPage)">
        <mdui-icon-arrow-back></mdui-icon-arrow-back>
      </mdui-button-icon>
      <mdui-top-app-bar-title
        ><h1 class="app-title">{{ pageTitle }}</h1></mdui-top-app-bar-title
      >
      <mdui-dropdown :open="moreOpen" placement="bottom-end" @opened="moreOpen = true" @closed="moreOpen = false">
        <mdui-button-icon slot="trigger" :aria-label="t('more')">
          <mdui-icon-more-vert></mdui-icon-more-vert>
        </mdui-button-icon>
        <mdui-menu>
          <mdui-menu-item v-for="item in moreItems" :key="item.key" @click="navigateTo(item.key)">
            <component :is="item.icon" slot="icon" />
            {{ t(item.key) }}
          </mdui-menu-item>
        </mdui-menu>
      </mdui-dropdown>
    </mdui-top-app-bar>
    <main class="app-main">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" class="route-view" />
          </template>
          <template #fallback>
            <div class="loading-container" role="status">
              <mdui-circular-progress :aria-label="pageTitle"></mdui-circular-progress>
            </div>
          </template>
        </Suspense>
      </router-view>
    </main>

    <mdui-navigation-bar v-if="!railHidden && !secondaryPage" label-visibility="labeled" :aria-label="t('navigation')" :value="navValue" @change="onNavChange">
      <mdui-navigation-bar-item v-for="item in navItems" :key="item.key" :value="item.key">
        {{ t(item.key) }}
        <span slot="icon" class="rail-icon"><component :is="item.icon" /></span>
        <span slot="active-icon" class="rail-icon"><component :is="item.activeIcon" /></span>
      </mdui-navigation-bar-item>
    </mdui-navigation-bar>
  </div>
</template>

<style scoped>
.mobile-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  min-width: 0;
  padding-top: env(safe-area-inset-top);
  padding-left: env(safe-area-inset-left);
  padding-right: env(safe-area-inset-right);
}

.mobile-layout.focus-mode {
  padding-bottom: env(safe-area-inset-bottom);
}

.app-bar {
  position: relative;
  flex-shrink: 0;
  height: var(--app-top-bar-height);
  padding: 8px 4px 8px 16px;
  align-items: center;
  background-color: rgb(var(--mdui-color-surface));
  color: rgb(var(--mdui-color-on-surface));
  transition: none;
}

.app-title {
  margin: 0;
  font: inherit;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-bar mdui-top-app-bar-title {
  min-width: 0;
}

.app-bar mdui-dropdown {
  flex-shrink: 0;
}

mdui-navigation-bar {
  position: relative;
  flex-shrink: 0;
  height: calc(var(--app-navigation-height) + env(safe-area-inset-bottom));
  padding-bottom: env(safe-area-inset-bottom);
  background-color: rgb(var(--mdui-color-surface-container));
  box-shadow: none;
  transition: none;
}

.custom-bg-layer {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.custom-bg-overlay {
  position: fixed;
  inset: 0;
  background: rgba(var(--mdui-color-surface), 0.94);
  pointer-events: none;
  z-index: 0;
}

.rail-icon {
  display: flex;
}

.route-view {
  width: 100%;
  height: 100%;
  min-height: 0;
  animation: route-enter 200ms var(--mdui-motion-easing-standard) backwards;
}

@keyframes route-enter {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100%;
}

.app-main {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
</style>
