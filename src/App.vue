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
import { onMounted, onUnmounted } from 'vue'
  
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

export default {
  data() {
    return {
      drawer: true,
    };
  },
  methods: {
    toggleNav() {
      this.drawer = !this.drawer;
    },
  },

};
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

const drawer = ref(true);
const handleResize = () => {
   drawer.value = window.innerWidth >= 768;
};

onMounted(() => {
  window.addEventListener('resize', handleResize);
  handleResize();
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
});
  
</script>

<template>
  <v-app id="phi-tk" class="dark-theme">
    <v-sonner position="top-center" />
    <v-app-bar :elevation="0" class="app-bar-shadow blur-background">
      <v-app-bar-title class="mx-5 text-gradient">Phi TK</v-app-bar-title>
    </v-app-bar>

    <v-navigation-drawer 
      v-model="drawer" 
      :width="240"
      floating
      class="nav-drawer-glass blur-background"
    >
      <v-list density="comfortable" nav class="py-4">
        <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"
          class="list-item-hover mx-3 glow-item"
          variant="flat"
        ></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex justify-center animated-background">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component 
              :is="Component" 
              ref="component"
              class="route-transition"
            />
          </template>
          <template #fallback>
            <div class="flex justify-center pa-8">
              <v-progress-circular 
                indeterminate 
                size="large"
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
.dark-theme {
  background: linear-gradient(45deg, #292364, #302b63, #24243e);
}

.blur-background {
  backdrop-filter: blur(40px) saturate(200%);
  background: linear-gradient(
    135deg, 
    rgba(98, 0, 234, 0.15) 0%, 
    rgba(186, 104, 200, 0.1) 100%
  ) !important;
  border: 1px solid rgba(255,255,255,0.1) !important;
}

.nav-drawer-glass {
  border-right: 1px solid rgba(255,255,255,0.15) !important;
  box-shadow: 4px 0 20px rgba(0,0,0,0.3);
}

.list-item-hover {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  margin: 8px 0;
  border-radius: 12px;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    left: -100%;
    top: 0;
    width: 60%;
    height: 100%;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(255,255,255,0.1),
      transparent
    );
    transition: 0.5s;
  }

  &:hover {
    transform: translateX(12px) scale(1.02);
    background: linear-gradient(
      to right, 
      rgba(98, 0, 234, 0.2) 30%, 
      transparent
    ) !important;
    box-shadow: 2px 0 12px rgba(98, 0, 234, 0.3);

    &::before {
      left: 140%;
    }
  }
}

.route-transition {
  transition: all 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55);
}

.fade-blur-enter-from,
.fade-blur-leave-to {
  opacity: 0;
  transform: rotateY(10deg) translateZ(50px);
  filter: blur(5px);
}

.text-gradient {
  background: linear-gradient(45deg, #7c4dff, #ff6ec4);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #6200ea);
}

.animated-background::after {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 1000 1000' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  opacity: 0.03;
  pointer-events: none;
  animation: particleFlow 20s linear infinite;
}

@keyframes particleFlow {
  0% { transform: translate(0, 0); }
  100% { transform: translate(-50%, -50%); }
}

@media (max-width: 768px) {
  .blur-background {
    backdrop-filter: blur(20px);
  }
  
  .nav-drawer-glass {
    border-right-width: 0.5px;
  }
}

.list-item-hover:hover .v-icon {
  transform: rotate(15deg) scale(1.2);
  filter: drop-shadow(0 0 8px rgba(156, 39, 176, 0.6));
  transition: all 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55);
}

.nav-drawer-glass {
  border-right: 1px solid rgba(255,255,255,0.2) !important;
  box-shadow: 
    8px 0 24px rgba(0,0,0,0.3),
    inset 2px 0 12px rgba(255,255,255,0.05);
  border-radius: 0 24px 24px 0;
}

.app-bar-shadow {
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.2) !important;
}

.v-main {
  padding-left: 260px !important;
  transition: padding 0.3s ease;
}

.route-transition {
  max-width: 1200px;
  width: 100%;
}

.route-transition > div {
  background: linear-gradient(
    135deg,
    rgba(45, 35, 66, 0.6) 0%,
    rgba(34, 34, 62, 0.8) 100%
  );
  backdrop-filter: blur(20px);
  border-radius: 24px;
  padding: 32px;
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.3),
    inset 0 2px 4px rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
}

@media (max-width: 960px) {
  .v-main {
    padding-left: 0 !important;
  }
  
  .nav-drawer-glass {
    border-radius: 0;
  }
  
  .route-transition > div {
    border-radius: 16px;
    padding: 24px;
  }
}

.v-list-item-title {
  font-weight: 600;
  letter-spacing: 0.5px;
  background: linear-gradient(45deg, #fff, #e0e0e0);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.glow-spinner .v-progress-circular__overlay {
  stroke: url(#gradient) !important;
  filter: drop-shadow(0 0 12px #7c4dff);
}

svg {
  position: absolute;
}

svg > defs {
  <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">
    <stop offset="0%" style="stop-color:#7C4DFF;stop-opacity:1" />
    <stop offset="100%" style="stop-color:#FF6EC4;stop-opacity:1" />
  </linearGradient>
}

.list-item-hover:hover .v-icon {
  filter: drop-shadow(0 0 12px rgba(156, 39, 176, 0.8));
  transform: scale(1.15);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 文字发光效果 */
.list-item-hover:hover .v-list-item-title {
  text-shadow: 
    0 0 10px rgba(156, 39, 176, 0.4),
    0 0 20px rgba(156, 39, 176, 0.3),
    0 0 30px rgba(156, 39, 176, 0.2);
  background: linear-gradient(45deg, #fff 20%, #e0b0ff 80%);
  -webkit-background-clip: text;
  background-clip: text;
  transition: all 0.3s ease;
}

/* 增强悬停效果容器 */
.list-item-hover {
  position: relative;
  overflow: visible;
}

/* 添加光晕扩散效果 */
.list-item-hover:hover::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 120%;
  height: 160%;
  background: radial-gradient(
    circle at 50% 50%,
    rgba(156, 39, 176, 0.3) 0%,
    rgba(156, 39, 176, 0.15) 50%,
    transparent 100%
  );
  filter: blur(16px);
  z-index: -1;
  animation: pulseGlow 1.5s infinite;
}

@keyframes pulseGlow {
  0% { opacity: 0.8; }
  50% { opacity: 1; transform: translate(-50%, -50%) scale(1.05); }
  100% { opacity: 0.8; }
}

</style>
