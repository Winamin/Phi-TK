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
  <mdui-slider :value="modelValue" @input="onInput"></mdui-slider>
</template>
