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
  <mdui-checkbox class="md-checkbox" :checked="modelValue" @change="onChange">
    <slot></slot>
  </mdui-checkbox>
</template>

<style scoped>
.md-checkbox {
  min-width: var(--app-touch-target, 48px);
  min-height: var(--app-touch-target, 48px);
}

.md-checkbox::part(control) {
  width: var(--app-touch-target, 48px);
  height: var(--app-touch-target, 48px);
}

.md-checkbox::part(label) {
  white-space: normal;
  overflow-wrap: anywhere;
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
}
</style>
