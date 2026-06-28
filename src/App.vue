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
</script>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
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

    <!-- MD3 NavigationRail (left side) -->
    <nav class="md3-nav-rail">
      <div class="rail-brand">
        <span class="brand-text">Phi TK</span>
      </div>

      <div class="rail-items">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="rail-item"
          :class="{ 'is-active': route.name === item.key }"
          @click="navigateTo(item.key)"
        >
          <v-icon :icon="route.name === item.key ? item.activeIcon : item.icon" size="24" class="rail-icon" />
          <span class="rail-label">{{ t(item.key) }}</span>
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
/* ===== MD3 Global Resets ===== */
* { box-sizing: border-box; }

.v-main { background: transparent !important; }
.v-main__wrap { background: transparent !important; }
</style>

<style scoped>
/* ===== Custom Background ===== */
.custom-bg-layer {
  position: fixed; inset: 0;
  pointer-events: none; z-index: 0;
}
.custom-bg-overlay {
  position: fixed; inset: 0;
  background: radial-gradient(ellipse at center, transparent 0%, rgba(13,13,13,0.4) 40%, rgba(13,13,13,0.92) 100%);
  pointer-events: none; z-index: 0;
}

/* ===== MD3 NavigationRail ===== */
.md3-nav-rail {
  position: fixed;
  left: 0; top: 0; bottom: 0;
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

.rail-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 48px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.brand-text {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, #82b1ff, #d1e4ff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.rail-items {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
  flex: 1;
}

/* MD3 NavRail Destination */
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

/* MD3 pill indicator */
.rail-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%) scaleX(0);
  width: 32px;
  height: 32px;
  border-radius: 16px;
  background: rgba(130, 177, 255, 0.15);
  transition: transform 0.35s cubic-bezier(0.2, 0, 0, 1);
  z-index: 0;
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

.rail-label {
  position: relative;
  z-index: 1;
  font-size: 11px;
  font-weight: 500;
  margin-top: 2px;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 0.2px;
  transition: color 0.25s ease;
  white-space: nowrap;
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

/* ===== Responsive: collapse rail on narrow ===== */
@media (max-width: 600px) {
  .md3-nav-rail { width: 64px; }
  .md3-main { margin-left: 64px !important; }
  .rail-label { font-size: 9px; }
  .rail-item { width: 48px; }
  .brand-text { font-size: 11px; }
}
</style>
