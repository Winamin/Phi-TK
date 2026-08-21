<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About
  batch-render: Batch Render
  setting: Setting
  more: More

zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于
  batch-render: 批量渲染
  setting: 设置
  more: 更多

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
import { computed, onMounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { VSonner } from 'vuetify-sonner';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const navItems = [
  { key: 'render', icon: 'mdi-play-circle-outline', activeIcon: 'mdi-play-circle' },
  { key: 'rpe', icon: 'mdi-book-open-page-variant-outline', activeIcon: 'mdi-book-open-page-variant' },
  { key: 'tasks', icon: 'mdi-server-network-outline', activeIcon: 'mdi-server-network' },
  { key: 'batch-render', icon: 'mdi-timeline-clock-outline', activeIcon: 'mdi-timeline-clock' },
  { key: 'setting', icon: 'mdi-cog-outline', activeIcon: 'mdi-cog' },
  { key: 'about', icon: 'mdi-information-outline', activeIcon: 'mdi-information' },
];

const navigateTo = (name: string) => {
  router.push({ name });
};

window.goto = navigateTo;

const customBackground = ref<string | null>(null);

onMounted(() => {
  customBackground.value = localStorage.getItem('customBackground');
  window.addEventListener('customBackgroundChanged', ((event: CustomEvent) => {
    customBackground.value = event.detail;
  }) as EventListener);
});

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
  <v-app id="phi-tk">
    <div v-if="customBackground" class="custom-bg-layer" :style="backgroundStyle"></div>
    <div v-if="customBackground" class="custom-bg-overlay"></div>
    <v-sonner position="top-center" />

    <nav class="md3-nav-rail">
      <div class="rail-items">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="rail-item"
          :class="{ 'is-active': route.name === item.key }"
          :aria-label="t(item.key)"
          @click="navigateTo(item.key)">
          <v-icon :icon="route.name === item.key ? item.activeIcon : item.icon" size="24" class="rail-icon" />
        </button>
      </div>
    </nav>

    <!-- Main content area -->
    <v-main class="md3-main">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" class="route-view" />
          </template>
          <template #fallback>
            <div class="loading-container">
              <v-progress-circular indeterminate size="48" color="primary" />
            </div>
          </template>
        </Suspense>
      </router-view>
    </v-main>
  </v-app>
</template>

<style>
* {
  box-sizing: border-box;
}
</style>

<style scoped>
.custom-bg-layer {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}
.custom-bg-overlay {
  position: fixed;
  inset: 0;
  background: radial-gradient(ellipse at center, transparent 0%, rgba(13, 13, 13, 0.4) 40%, rgba(13, 13, 13, 0.92) 100%);
  pointer-events: none;
  z-index: 0;
}

.md3-nav-rail {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 80px;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: rgba(20, 20, 20, 0.92);
  backdrop-filter: blur(24px) saturate(180%);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 100;
  padding: 12px 0;
}

.rail-items {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
  flex: 1;
  justify-content: center;
}

.rail-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 56px;
  padding: 4px 0;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 16px;
  transition: all 0.25s cubic-bezier(0.2, 0, 0, 1);
  -webkit-tap-highlight-color: transparent;
}

.rail-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.rail-item.is-active .rail-indicator {
  transform: translate(-50%, -50%) scaleX(1);
  background: rgba(130, 177, 255, 0.25);
}

.rail-icon {
  position: relative;
  z-index: 1;
  color: rgba(255, 255, 255, 0.6);
  transition: color 0.25s ease;
}

.rail-item.is-active .rail-icon {
  color: #82b1ff;
}

.rail-item.is-active .rail-label {
  color: #82b1ff;
  font-weight: 600;
}

/* ===== Main Content ===== */
.md3-main {
  margin-left: 80px !important;
  min-height: 100vh;
  position: relative;
  z-index: 1;
}

.route-view {
  width: 100%;
  min-height: 100vh;
  animation: routeEnter 0.35s cubic-bezier(0.2, 0, 0, 1) both;
}

@keyframes routeEnter {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
}

@media (max-width: 600px) {
  .md3-nav-rail {
    width: 64px;
  }
  .md3-main {
    margin-left: 64px !important;
  }
  .rail-item {
    width: 48px;
  }
}
</style>
