<script setup lang="ts">
import { computed, ref, useAttrs } from 'vue';

defineOptions({ inheritAttrs: false });

/**
 * Stand-in for Vuetify's `<v-combobox>`: a free-text field with a dropdown of presets.
 * mdui has no combobox component — `<mdui-select>` does not accept arbitrary input — so
 * this composes `<mdui-text-field>` with `<mdui-dropdown>` + `<mdui-menu>`.
 *
 * Used for the resolution field, where any `WxH` string is valid but the common presets
 * should still be one click away.
 */
withDefaults(
  defineProps<{
    modelValue?: string;
    items: readonly string[];
    label?: string;
    /** Declared explicitly rather than left to attribute fallthrough, which would land it
        on the `<mdui-dropdown>` root instead of the field inside. */
    variant?: 'filled' | 'outlined';
  }>(),
  { variant: 'outlined' },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const open = ref(false);
const attrs = useAttrs();
const fieldAttrs = computed(() => Object.fromEntries(Object.entries(attrs).filter(([key]) => key !== 'class' && key !== 'style')));

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).value);
}

function pick(item: string) {
  emit('update:modelValue', item);
  open.value = false;
}
</script>

<template>
  <mdui-dropdown :open="open" placement="bottom-end" trigger="manual" class="md-combobox" :class="attrs.class" :style="attrs.style" @closed="open = false">
    <mdui-text-field v-bind="fieldAttrs" slot="trigger" :variant="variant" :value="modelValue ?? ''" :label="label" @input="onInput">
      <mdui-button-icon slot="end-icon" :aria-label="label" :aria-expanded="open" aria-haspopup="menu" @click="open = !open">
        <mdui-icon-arrow-drop-down></mdui-icon-arrow-drop-down>
      </mdui-button-icon>
    </mdui-text-field>
    <mdui-menu>
      <mdui-menu-item v-for="item in items" :key="item" @click="pick(item)">{{ item }}</mdui-menu-item>
    </mdui-menu>
  </mdui-dropdown>
</template>

<style scoped>
.md-combobox {
  display: block;
  width: 100%;
  min-width: 0;
}

mdui-text-field {
  display: block;
  width: 100%;
  min-width: 0;
}

mdui-text-field::part(container) {
  min-height: 56px;
}

mdui-button-icon,
mdui-menu-item {
  min-height: var(--app-touch-target, 48px);
}

mdui-button-icon {
  width: var(--app-touch-target, 48px);
}
</style>
