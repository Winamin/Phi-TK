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
import { computed, onMounted, onUnmounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useWindowSize } from './window-size';
const { t } = useI18n();
const route = useRoute();
const router = useRouter();

/**
 * MD3's adaptive navigation: `<mdui-navigation-rail>` for the `medium` class and
 * up, `<mdui-navigation-bar>` below it. The rail is a side column and the bar is
 * a bottom row, so the main content area changes shape with it — that is the
 * whole point of the size classes, and a fixed rail on a phone-width window
 * would eat most of the horizontal space.
 */
const { isCompact } = useWindowSize();

/** Icons are `@mdui/icons` custom-element tag names, rendered through `<component :is>`. */
const navItems = [
  { key: 'render', icon: 'mdui-icon-play-circle--outlined', activeIcon: 'mdui-icon-play-circle' },
  { key: 'rpe', icon: 'mdui-icon-menu-book--outlined', activeIcon: 'mdui-icon-menu-book' },
  { key: 'tasks', icon: 'mdui-icon-dns--outlined', activeIcon: 'mdui-icon-dns' },
  { key: 'batch-render', icon: 'mdui-icon-pending-actions--outlined', activeIcon: 'mdui-icon-pending-actions' },
  { key: 'setting', icon: 'mdui-icon-settings--outlined', activeIcon: 'mdui-icon-settings' },
  { key: 'about', icon: 'mdui-icon-info--outlined', activeIcon: 'mdui-icon-info' },
];

const navigateTo = (name: string) => {
  router.push({ name });
};

window.goto = navigateTo;

/**
 * `<mdui-navigation-rail>` also fires `change` when its `value` is set programmatically —
 * which happens on every route change, since `value` mirrors `route.name`. Ignoring
 * no-op transitions keeps that from bouncing back into the router.
 */
function onNavChange(e: Event) {
  const name = (e.target as HTMLElement & { value?: string }).value;
  if (name && name !== route.name) navigateTo(name);
}

const customBackground = ref<string | null>(null);

/**
 * Views that take over the whole window hide the app rail while they are mid-flow.
 *
 * `<mdui-navigation-rail>` is app chrome, so it lives here rather than in the view —
 * which means a view that wants it out of the way has to say so. RenderView raises
 * this while a step is open and drops it again when the flow finishes, so the rail
 * comes back once the user is done rather than staying hidden.
 */
const railHidden = ref(false);

function onRailHidden(e: Event) {
  railHidden.value = (e as CustomEvent<boolean>).detail === true;
}

onMounted(() => {
  window.addEventListener('rail-hidden', onRailHidden as EventListener);
});
onUnmounted(() => {
  window.removeEventListener('rail-hidden', onRailHidden as EventListener);
});

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
  <!-- The wallpaper sits outside <mdui-layout>: it is the only place the user's image is
       meant to show through, and every surface above it is an opaque MD3 container. -->
  <div v-if="customBackground" class="custom-bg-layer" :style="backgroundStyle"></div>
  <div v-if="customBackground" class="custom-bg-overlay"></div>

  <mdui-layout full-height>
    <!-- Medium and up: a side rail. -->
    <mdui-navigation-rail
      v-if="!isCompact"
      v-show="!railHidden"
      alignment="center"
      divider
      :value="(route.name as string)"
      @change="onNavChange">
      <mdui-navigation-rail-item v-for="item in navItems" :key="item.key" :value="item.key">
        {{ t(item.key) }}
        <span slot="icon" class="rail-icon"><component :is="item.icon" /></span>
        <span slot="active-icon" class="rail-icon"><component :is="item.activeIcon" /></span>
      </mdui-navigation-rail-item>
    </mdui-navigation-rail>

    <mdui-layout-main>
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" class="route-view" />
          </template>
          <template #fallback>
            <div class="loading-container">
              <mdui-circular-progress></mdui-circular-progress>
            </div>
          </template>
        </Suspense>
      </router-view>
    </mdui-layout-main>

    <!-- Compact: a bottom bar. Declared after the main area so the layout system
         reserves space at the bottom edge rather than around the content. -->
    <mdui-navigation-bar
      v-if="isCompact"
      v-show="!railHidden"
      label-visibility="selected"
      :value="(route.name as string)"
      @change="onNavChange">
      <mdui-navigation-bar-item v-for="item in navItems" :key="item.key" :value="item.key">
        {{ t(item.key) }}
        <span slot="icon" class="rail-icon"><component :is="item.icon" /></span>
        <span slot="active-icon" class="rail-icon"><component :is="item.activeIcon" /></span>
      </mdui-navigation-bar-item>
    </mdui-navigation-bar>
  </mdui-layout>
</template>

<style scoped>
.custom-bg-layer {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

/* Vignette that keeps foreground text legible over an arbitrary wallpaper. Built from the
   background token so it darkens in dark mode and lightens in light mode. */
.custom-bg-overlay {
  position: fixed;
  inset: 0;
  background: radial-gradient(
    ellipse at center,
    rgba(var(--mdui-color-background), 0) 0%,
    rgba(var(--mdui-color-background), 0.45) 40%,
    rgba(var(--mdui-color-background), 0.92) 100%
  );
  pointer-events: none;
  z-index: 0;
}

/* `<mdui-layout>` and `<mdui-layout-main>` are transparent by default, so the wallpaper
   shows through behind the view's own surfaces. The rail deliberately stays opaque
   `surface` — MD3 treats it as chrome, not as part of the backdrop.

   `v-show` (not `v-if`) hides the rail: it is a flex sibling of `mdui-layout-main`,
   so `display: none` is enough for the main area to reclaim the full width, and
   keeping the element alive preserves the ripple/scroll state on the way back. */

.rail-icon {
  display: flex;
}

/* MD3 shared-axis X entrance for top-level destinations.
   The rail sits on the leading edge, so destinations are siblings along the
   horizontal axis and sliding them in from the trailing side states that
   relationship. A vertical rise would imply a stack that does not exist here. */
.route-view {
  width: 100%;
  min-height: 100%;
  animation: route-enter var(--app-motion-duration-spatial-default)
    var(--app-motion-spring-spatial-default, var(--mdui-motion-easing-emphasized-decelerate)) both;
}

@keyframes route-enter {
  from {
    opacity: 0;
    transform: translateX(24px);
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
</style>
