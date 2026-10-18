<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';

/**
 * `v-model`-capable wrapper around `<mdui-select>`.
 *
 * Two mdui quirks are handled here so no call site has to:
 *  - `<mdui-select>` fires **only** `change` — it has no `input` event.
 *  - its `value` is always a string, so numeric models would come back as strings.
 *    The original, correctly-typed item value is emitted instead.
 *
 * `items` accepts the same shapes the previous `<v-select>` call sites used: a plain
 * array of strings/numbers, or an array of objects addressed by `itemTitle`/`itemValue`.
 */
type Primitive = string | number;
type Item = Primitive | Record<string, unknown>;

const props = withDefaults(
  defineProps<{
    modelValue?: unknown;
    items: readonly Item[];
    itemTitle?: string;
    itemValue?: string;
  }>(),
  { itemTitle: 'title', itemValue: 'value' },
);

const emit = defineEmits<{
  'update:modelValue': [value: unknown];
}>();

const el = ref<HTMLElement & { value?: string }>();

/** Flattens both accepted `items` shapes into `{ key, label, value }`. */
const options = computed(() =>
  props.items.map((item) => {
    const value = typeof item === 'object' && item !== null ? (item as Record<string, unknown>)[props.itemValue] : item;
    const label = typeof item === 'object' && item !== null ? String((item as Record<string, unknown>)[props.itemTitle]) : String(item);
    return { key: String(value), label, value };
  }),
);

const selectedKey = computed(() => (props.modelValue === null || props.modelValue === undefined ? '' : String(props.modelValue)));

function onChange(e: Event) {
  const key = (e.target as HTMLElement & { value: string }).value;
  const match = options.value.find((o) => o.key === key);
  // Fall back to the raw string when the key is unknown (e.g. after `clearable`).
  emit('update:modelValue', match ? match.value : key);
}

// `<mdui-select>` resolves its display label from its `<mdui-menu-item>` children, so a
// value set while the option list is still changing can be dropped. Re-assert it once the
// new children are in the DOM.
watch(
  () => props.items,
  async () => {
    await nextTick();
    if (el.value) el.value.value = selectedKey.value;
  },
);
</script>

<template>
  <mdui-select ref="el" :value="selectedKey" @change="onChange">
    <mdui-menu-item v-for="opt in options" :key="opt.key" :value="opt.key">{{ opt.label }}</mdui-menu-item>
  </mdui-select>
</template>
