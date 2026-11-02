<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import { labelShadowControl } from '../../a11y';

/**
 * `v-model`-capable wrapper around `<mdui-switch>`.
 *
 * `<mdui-switch>` carries its state in `checked`, not `value` — binding `value` would
 * silently do nothing. All other attributes fall through.
 */
const props = defineProps<{
  modelValue?: boolean;
  label?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const el = ref<HTMLElement>();

function syncLabel() {
  labelShadowControl(el.value, 'input[type="checkbox"]', props.label ?? '');
}

onMounted(syncLabel);

watch(
  () => props.label,
  async () => {
    await nextTick();
    syncLabel();
  },
);

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).checked);
}
</script>

<template>
  <mdui-switch ref="el" :checked="modelValue" @change="onChange"></mdui-switch>
</template>
