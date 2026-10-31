<script setup lang="ts">
import MdSwitch from './md/MdSwitch.vue';

/**
 * A switch row with supporting text that stays readable on touch devices.
 *
 * `modelValue` is declared explicitly: it used to reach the inner Vuetify `<v-switch>`
 * only through Vue's attribute fallthrough, which no longer works now that the root is a
 * wrapper component — every `v-model` on a TipSwitch would have silently stopped updating.
 */
defineProps<{
  modelValue?: boolean;
  label: string;
  tooltip?: string;
}>();

defineEmits<{
  'update:modelValue': [value: boolean];
}>();
</script>

<template>
  <button type="button" class="tip-switch" role="switch" :aria-checked="!!modelValue" @click="$emit('update:modelValue', !modelValue)">
    <span v-if="label || tooltip" class="tip-switch-text">
      <span class="tip-switch-label">{{ label }}</span>
      <span v-if="tooltip" class="tip-switch-helper">{{ tooltip }}</span>
    </span>
    <MdSwitch :model-value="modelValue" class="tip-switch-control" aria-hidden="true" tabindex="-1" />
  </button>
</template>

<style scoped>
.tip-switch {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: 100%;
  min-height: var(--app-touch-target, 48px);
  padding: 4px 0;
  border: 0;
  border-radius: var(--mdui-shape-corner-extra-small);
  background: transparent;
  font: inherit;
  text-align: start;
  cursor: pointer;
  min-width: 0;
  -webkit-tap-highlight-color: transparent;
  touch-action: manipulation;
}

.tip-switch:focus-visible {
  outline: 2px solid rgb(var(--mdui-color-primary));
  outline-offset: 4px;
}

.tip-switch:disabled {
  cursor: default;
  opacity: 0.38;
}

.tip-switch-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.tip-switch-label {
  color: rgb(var(--mdui-color-on-surface));
  font-size: var(--mdui-typescale-body-large-size);
  line-height: var(--mdui-typescale-body-large-line-height);
}

.tip-switch-helper {
  color: rgb(var(--mdui-color-on-surface-variant));
  font-size: var(--mdui-typescale-body-small-size);
  line-height: var(--mdui-typescale-body-small-line-height);
}

.tip-switch-control {
  pointer-events: none;
}
</style>
