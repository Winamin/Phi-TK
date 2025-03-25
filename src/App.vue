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
      expand-on-hover
      rail
      permanent
      class="nav-drawer-glass blur-background"
    >
      <v-list density="compact" nav>
        <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"
          class="list-item-hover glow-item"
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
  background: linear-gradient(135deg, #539ed3 0%, #2494d5 50%, #0b90d3 100%);
  min-height: 100vh;
}

.blur-background {
  backdrop-filter: blur(40px) saturate(180%);
  background: linear-gradient(
    135deg, 
    rgba(191, 219, 254, 0.3) 0%,
    rgba(125, 211, 252, 0.2) 100%
  ) !important;
  border: 1px solid rgba(255, 255, 255, 0.3) !important;
}

.nav-drawer-glass {
  border-right: 1px solid rgba(255, 255, 255, 0.4) !important;
  box-shadow: 4px 0 20px rgba(147, 197, 253, 0.2);
  perspective: 1000px;
}

.list-item-hover {
  transition: transform 0.3s ease;
  margin: 8px 0;
  background: transparent !important;
  box-shadow: none !important;
  border-radius: 0;
  transform-style: preserve-3d;

  &:hover {
    transform: translateX(12px) scale(1.02) rotateY(3deg) translateZ(10px);
    background: rgba(255, 255, 255, 0.05) !important;

    .v-list-item__prepend {
      color: #38bdf8;
      transform: scale(1.2) rotate(15deg);
    }
  }

  &::after {
    content: '';
    position: absolute;
    inset: 0;
    background: radial-gradient(
      600px circle at var(--x) var(--y),
      rgba(56, 189, 248, 0.2),
      transparent 40%
    );
    opacity: 0;
    transition: opacity 0.3s;
    pointer-events: none;
  }

  &:hover::after {
    opacity: 1;
  }
}

.route-transition {
  transition: all 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55);
  animation: gentlePulse 8s ease-in-out infinite alternate;
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #7dd3fc);
}

.text-gradient {
  background: linear-gradient(45deg, #ec0505, #d80bcd);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.app-bar-shadow {
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.4s ease;

  &:hover {
    box-shadow: 
      0 8px 40px rgba(125, 211, 252, 0.3),
      0 0 30px rgba(125, 211, 252, 0.2);
  }
}

@keyframes gentlePulse {
  0%, 100% { background-color: rgba(224, 242, 254, 0.7); }
  50% { background-color: rgba(186, 230, 253, 0.9); }
}

@keyframes particleFlow {
  0% { transform: translate(0, 0); }
  100% { transform: translate(-50%, -50%); }
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

@media (max-width: 768px) {
  .blur-background {
    backdrop-filter: blur(20px);
  }

  .nav-drawer-glass {
    border-right-width: 0.5px;
  }
}

.fade-blur-enter-from,
.fade-blur-leave-to {
  opacity: 0;
  transform: rotateY(10deg) translateZ(50px);
  filter: blur(5px);
}

.v-list-item__prepend {
  transition: 
    transform 0.4s cubic-bezier(0.68, -0.55, 0.27, 1.55),
    filter 0.3s ease;
}

.v-list-item:focus::before,
.v-list-item:focus-visible::before {
  opacity: 0 !important;
}

.v-list-item:focus {
  outline: none;
  box-shadow: none;
}
<style>
