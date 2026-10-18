<script setup lang="ts">
import TooltipIcon from './TooltipIcon.vue';
import MdSwitch from './md/MdSwitch.vue';

/**
 * A labelled switch with an optional tooltip.
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
  <label class="tip-switch">
    <MdSwitch :model-value="modelValue" @update:model-value="$emit('update:modelValue', $event)" />
    <span class="tip-switch-label">
      {{ label }}
      <TooltipIcon v-if="tooltip" :tooltip="tooltip" />
    </span>
  </label>
</template>

<style scoped>
.tip-switch {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  min-width: 0;
}

.tip-switch-label {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  color: rgb(var(--mdui-color-on-surface));
  font-size: var(--mdui-typescale-body-medium-size);
  line-height: var(--mdui-typescale-body-medium-line-height);
}
</style>
