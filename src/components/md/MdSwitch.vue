<script setup lang="ts">
/**
 * `v-model`-capable wrapper around `<mdui-switch>`.
 *
 * `<mdui-switch>` carries its state in `checked`, not `value` — binding `value` would
 * silently do nothing. All other attributes fall through.
 */
defineProps<{
  modelValue?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).checked);
}
</script>

<template>
  <mdui-switch :checked="modelValue" @change="onChange"></mdui-switch>
</template>
