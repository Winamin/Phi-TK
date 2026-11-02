<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import { labelShadowControl } from '../../a11y';

/**
 * `v-model`-capable wrapper around `<mdui-slider>`. The element's `value` is already a
 * number, but the DOM `input` event carries a string, so it is coerced back here.
 */
const props = defineProps<{
  modelValue?: number;
  label?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: number];
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

function onInput(e: Event) {
  emit('update:modelValue', Number((e.target as HTMLInputElement).value));
}
</script>

<template>
  <mdui-slider ref="el" :value="modelValue" @input="onInput"></mdui-slider>
</template>
