<script setup lang="ts">
/**
 * `v-model`-capable wrapper around `<mdui-checkbox>`. Like the switch, its state lives in
 * `checked` rather than `value`.
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
  <mdui-checkbox :checked="modelValue" @change="onChange">
    <slot></slot>
  </mdui-checkbox>
</template>
