<script setup lang="ts">
import { nextTick, onMounted, ref, useAttrs, watch } from 'vue';
import { labelShadowControl } from '../../a11y';

/**
 * `v-model`-capable wrapper around `<mdui-text-field>`.
 *
 * mdui components are Web Components, on which Vue's `v-model` does not work — the
 * documented pattern is `:value` + `@input`. This wrapper re-exposes `modelValue` so the
 * existing call sites keep using `v-model`, and centralises the string↔number coercion.
 *
 * Every other attribute (label, placeholder, helper, suffix, clearable, required, min,
 * max, step, variant, …) falls through to the underlying element unchanged, and children
 * pass straight into the element's named slots (`icon`, `end-icon`, `helper`, …).
 */
const props = defineProps<{
  modelValue?: string | number | null;
  /** Emit a `number` instead of the raw string. Implied by `type="number"`. */
  number?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string | number];
}>();

const attrs = useAttrs();
const el = ref<HTMLElement>();

function syncLabel() {
  const source = [attrs.label, attrs.placeholder].find(
    (v): v is string => typeof v === 'string' && v.length > 0,
  );
  labelShadowControl(el.value, ['input', 'textarea'], source ?? '');
}

onMounted(syncLabel);

watch(
  () => [attrs.label, attrs.placeholder],
  async () => {
    await nextTick();
    syncLabel();
  },
);

function onInput(e: Event) {
  const raw = (e.target as HTMLInputElement).value;
  if (props.number || attrs.type === 'number') {
    // Keep an empty field distinguishable from 0 rather than emitting NaN.
    emit('update:modelValue', raw === '' ? '' : Number(raw));
  } else {
    emit('update:modelValue', raw);
  }
}
</script>

<template>
  <mdui-text-field ref="el" :value="modelValue ?? ''" @input="onInput">
    <!-- `<mdui-text-field>` has no unnamed slot, so only `slot="…"` children render. -->
    <slot></slot>
  </mdui-text-field>
</template>
