<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import { labelShadowControl } from '../../a11y';

/**
 * `v-model`-capable wrapper around `<mdui-checkbox>`. Like the switch, its state lives in
 * `checked` rather than `value`.
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
  labelShadowControl(el.value, 'input', props.label ?? '');
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
  <mdui-checkbox ref="el" :checked="modelValue" @change="onChange">
    <slot></slot>
  </mdui-checkbox>
</template>
