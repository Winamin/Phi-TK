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
const { t } = useI18n();
const route = useRoute();
const router = useRouter();

// 导航项配置
const navItems = [
  { key: 'render', icon: 'mdi-play-circle' },
  { key: 'rpe', icon: 'mdi-book-open-page-variant' },
  { key: 'tasks', icon: 'mdi-server' },
  { key: 'setting', icon: 'mdi-cog-outline' },
];

// 下拉菜单项配置
const dropdownItems = [
  { key: 'batch-render', icon: 'mdi-timeline' },
  { key: 'about', icon: 'mdi-information-outline' },
];

// 导航处理函数
const navigateTo = (name: string) => {
  router.push({ name });
};

window.goto = navigateTo;

// 下拉菜单状态
const showDropdown = ref(false);
</script>

<template>
  <v-app id="phi-tk" class="dark-theme">
    <v-sonner position="top-center" />
    
    <!-- 顶部导航栏 -->
    <v-app-bar :elevation="0" class="app-bar-glass blur-background">
      <!-- 左侧标题 -->
      <v-app-bar-title class="text-gradient glow-title">Phi TK</v-app-bar-title>
      
      <v-spacer></v-spacer>
      
      <!-- 右侧导航图标 -->
      <div class="nav-icons-container">
        <v-btn
          v-for="item in navItems"
          :key="item.key"
          :icon="item.icon"
          :active="route.name === item.key"
          variant="text"
          size="large"
          class="nav-icon-btn"
          @click="navigateTo(item.key)"
          v-tooltip:bottom="t(item.key)"
        />
        
        <!-- 更多选项下拉菜单 -->
        <v-menu
          v-model="showDropdown"
          :close-on-content-click="false"
          location="bottom end"
          transition="slide-y-transition"
        >
          <template v-slot:activator="{ props }">
            <v-btn
              icon="mdi-chevron-down"
              variant="text"
              size="large"
              class="nav-icon-btn"
              v-bind="props"
              v-tooltip:bottom="t('more')"
            />
          </template>
          
          <v-list class="dropdown-menu-glass">
            <v-list-item
              v-for="item in dropdownItems"
              :key="item.key"
              :active="route.name === item.key"
              :prepend-icon="item.icon"
              :title="t(item.key)"
              @click="navigateTo(item.key)"
              class="dropdown-item"
            />
          </v-list>
        </v-menu>
      </div>
    </v-app-bar>

    <!-- 主内容区域 -->
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
  background: linear-gradient(135deg, #1f1b3d, #3c2d6d, #5b4a9a);
  min-height: 100vh;
}

.app-bar-glass {
  backdrop-filter: blur(40px) saturate(200%);
  background: rgba(40, 32, 72, 0.7) !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.2);
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

.nav-icons-container {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-right: 16px;
}

.nav-icon-btn {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 12px;
  position: relative;
  overflow: hidden;
  background: rgba(50, 42, 90, 0.3);
  color: rgba(255, 255, 255, 0.8);
}

.nav-icon-btn::before {
  content: '';
  position: absolute;
  left: -100%;
  top: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255,255,255,0.15),
    transparent
  );
  transition: 0.5s;
}

.nav-icon-btn:hover {
  transform: translateY(-2px) scale(1.05);
  background: linear-gradient(
    135deg,
    rgba(96, 67, 140, 0.4) 0%,
    rgba(118, 64, 193, 0.4) 100%
  ) !important;
  box-shadow: 0 4px 20px rgba(118, 64, 193, 0.4);
  color: #fff;
}

.nav-icon-btn:hover::before {
  left: 100%;
}

.nav-icon-btn.v-btn--active {
  background: linear-gradient(
    135deg,
    rgba(118, 64, 193, 0.6) 0%,
    rgba(156, 105, 217, 0.6) 100%
  ) !important;
  color: #fff;
  box-shadow: 0 0 20px rgba(118, 64, 193, 0.6);
}

.dropdown-menu-glass {
  backdrop-filter: blur(40px) saturate(200%);
  background: rgba(40, 32, 72, 0.9) !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.dropdown-item {
  transition: all 0.2s ease;
  margin: 4px 8px;
  border-radius: 8px;
}

.dropdown-item:hover {
  background: rgba(118, 64, 193, 0.3) !important;
  transform: translateX(4px);
}

.dropdown-item.v-list-item--active {
  background: rgba(118, 64, 193, 0.5) !important;
  color: #fff;
}

.route-transition {
  transition: all 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55);
}

.text-gradient {
  background: linear-gradient(45deg, #c5b8ff, #d1c4e9);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  font-weight: 600;
  letter-spacing: 1px;
  text-shadow: 0 0 8px rgba(193, 176, 255, 0.3);
  animation: glow-pulse 3s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% { text-shadow: 0 0 8px rgba(193, 176, 255, 0.3); }
  50% { text-shadow: 0 0 20px rgba(193, 176, 255, 0.6); }
}

.glow-spinner {
  filter: drop-shadow(0 0 12px #b19dff);
}

.animated-background::after {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 1000 1000' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  opacity: 0.05;
  pointer-events: none;
  animation: particleFlow 20s linear infinite;
}

@keyframes particleFlow {
  0% { transform: translate(0, 0); }
  100% { transform: translate(-50%, -50%); }
}

@media (max-width: 768px) {
  .nav-icon-btn {
    size: medium;
  }
  
  .text-gradient {
    font-size: 1.2rem;
  }
  
  .blur-background {
    backdrop-filter: blur(20px);
  }
}

/* 全局隐藏滚动条 */
::-webkit-scrollbar {
  display: none;
  width: 0 !important;
}

html {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>