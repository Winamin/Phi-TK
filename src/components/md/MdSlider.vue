<script setup lang="ts">
/**
 * `v-model`-capable wrapper around `<mdui-slider>`. The element's `value` is already a
 * number, but the DOM `input` event carries a string, so it is coerced back here.
 */
defineProps<{
  modelValue?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

function onInput(e: Event) {
  emit('update:modelValue', Number((e.target as HTMLInputElement).value));
}
</script>

<template>
  <mdui-slider class="md-slider" :value="modelValue" @input="onInput"></mdui-slider>
</template>

<style scoped>
.md-slider {
  display: block;
  width: 100%;
  min-width: 0;
  height: var(--app-touch-target, 48px);
  touch-action: pan-y;
}
</style>
