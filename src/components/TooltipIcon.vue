<script setup lang="ts">
/**
 * Supplementary help that can be opened by touch, mouse, or keyboard.
 *
 * `mdui-tooltip` caps itself at 20rem, so the old `max-width="250px"` is no longer needed.
 */
defineProps<{ tooltip: string }>();
</script>

<template>
  <mdui-tooltip :content="tooltip" placement="bottom" trigger="click hover focus">
    <mdui-button-icon class="tip-btn" :aria-label="tooltip">
      <mdui-icon-info--outlined class="tip-glyph"></mdui-icon-info--outlined>
    </mdui-button-icon>
  </mdui-tooltip>
</template>

<style scoped>
.tip-btn {
  width: var(--app-touch-target, 48px);
  height: var(--app-touch-target, 48px);
  flex-shrink: 0;
}

/* The glyph stays small even though its hit area does not. mdui slots the icon
   rather than exposing a `part`, so the size is set on the slotted element. */
.tip-glyph {
  font-size: 1rem;
  color: rgb(var(--mdui-color-on-surface-variant));
}
</style>
