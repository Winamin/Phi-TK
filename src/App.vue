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

const onLoaded = ref<() => void>();
const component = ref();
const drawer = ref(false);

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
  <v-app id="Phigros TK">
    <v-navigation-drawer v-model="drawer" absolute temporary>
      <v-list density="compact" nav>
        <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key }); drawer = false"></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-app-bar title="phigros TK">
      <v-app-bar-nav-icon @click="drawer = true"></v-app-bar-nav-icon>
      <v-toolbar-title>phigros TK</v-toolbar-title>
    </v-app-bar>

    <v-main class="d-flex justify-center">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" />
          </template>
          <template #fallback>
            <div class="flex justify-center pa-8">
              <v-progress-circular indeterminate size="large" />
            </div>
          </template>
        </Suspense>
      </router-view>
    </v-main>
  </v-app>
</template>
