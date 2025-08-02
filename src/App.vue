<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About
  batch-render: Batch Render

zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于
  batch-render: 批量渲染

</i18n>

<script lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { VSonner } from 'vuetify-sonner';
import Dock from './Dock.vue'; // 导入Dock组件

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
      navItems: [] as any[] // 存储导航项数据
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
import { h, ref, computed, onMounted } from 'vue';

const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

// 定义图标映射
const icons = {
  render: 'mdi-auto-fix',
  rpe: 'mdi-bookshelf',
  tasks: 'mdi-server',
  about: 'mdi-information-outline',
  'batch-render': 'mdi-playlist-play',
};

// 创建Dock所需items
const dockItems = computed(() => {
  return [
    'render',
    'rpe',
    'tasks',
    'batch-render',
    'about'
  ].map(key => ({
    icon: () => h('v-icon', { size: '20px' }, icons[key as keyof typeof icons]),
    label: t(key),
    onClick: () => router.push({ name: key }),
    className: route.name === key ? 'active-dock-item' : '',
  }));
});

window.goto = (name: string) => {
  router.push({ name });
};

const handleResize = () => {
  // 可以在这里添加响应式处理
};

onMounted(() => {
  window.addEventListener('resize', handleResize);
  handleResize();
});
</script>

<template>
  <v-app id="phi-tk" class="dark-theme">
    <v-sonner position="top-center" />
    <v-app-bar :elevation="0" class="app-bar-shadow blur-background">
      <v-app-bar-title class="mx-5 text-gradient glow-title">Phi TK</v-app-bar-title>
    </v-app-bar>

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

    <!-- 添加横向Dock组件到页面底部 -->
    <div class="dock-container">
      <Dock
        :items="dockItems"
        :panel-height="68"
        :base-item-size="50"
        :magnification="70"
        :distance="200"
        :dock-height="80"
        :spring="{ mass: 0.1, stiffness: 150, damping: 12 }"
        class="dock-glass blur-background"
      />
    </div>
  </v-app>
</template>

<style>
.dark-theme {
  background: linear-gradient(45deg, #312c5c, #554f83, #8a8ab6);
}

.blur-background {
  backdrop-filter: blur(40px) saturate(200%);
  background: linear-gradient(
    135deg,
    rgba(88, 59, 126, 0.15) 0%,
    rgba(186, 104, 200, 0.1) 100%
  ) !important;
  border: 1px solid rgba(255,255,255,0.1) !important;
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
  background: linear-gradient(45deg, #c5c3ca, #d1cace);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;

  text-shadow:0 0 12px rgb(242 237 255 / 44%), 0 0 24px rgb(0 0 0 / 22%);
  animation: glow-pulse 2s ease-in-out infinite;
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #9552f3);
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
}

::-webkit-scrollbar {
  display: none;
  width: 0 !important;
}

html {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

/* 横向Dock样式 */
.dock-container {
  position: fixed;
  bottom: 20px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  z-index: 1000;
}

.dock-glass {
  backdrop-filter: blur(40px) saturate(200%);
  background: linear-gradient(
    135deg,
    rgba(88, 59, 126, 0.15) 0%,
    rgba(186, 104, 200, 0.1) 100%
  ) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  border-radius: 24px !important;
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3) !important;
  padding: 8px 24px !important;
  transition: all 0.3s ease;
}

/* 激活状态样式 */
.active-dock-item {
  box-shadow: 0 0 15px #8a4fff, inset 0 0 10px rgba(138, 79, 255, 0.3) !important;
  border-color: #8a4fff !important;
  transform: translateY(-10px) !important;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .dock-glass {
    padding: 6px 12px !important;
    border-radius: 20px !important;
    max-width: 95%;
  }

  .dock-container {
    bottom: 10px;
  }
}

/* 添加悬停动画效果 */
.v-enter-active {
  transition: all 0.3s ease;
}
.v-leave-active {
  transition: all 0.3s cubic-bezier(1, 0.5, 0.8, 1);
}
.v-enter-from,
.v-leave-to {
  transform: translateY(20px);
  opacity: 0;
}

/* 图标悬停放大效果 */
.dock-item {
  transition: transform 0.2s ease-in-out;
}
.dock-item:hover {
  transform: scale(1.2) translateY(-10px);
}
</style>