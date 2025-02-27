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
  if (window.innerWidth < 768) {
    drawer.value = false;
  }
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
</style>
