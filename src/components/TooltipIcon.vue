<script setup lang="ts">
/**
 * A small "?" affordance: an info icon that reveals explanatory text on hover/focus.
 *
 * `mdui-tooltip` caps itself at 20rem, so the old `max-width="250px"` is no longer needed.
 */
defineProps<{ tooltip: string }>();
</script>

<template>
  <mdui-tooltip :content="tooltip" placement="bottom">
    <mdui-button-icon class="tip-btn" tabindex="-1">
      <mdui-icon-info--outlined class="tip-glyph"></mdui-icon-info--outlined>
    </mdui-button-icon>
  </mdui-tooltip>
</template>

<style scoped>
/* `<mdui-button-icon>` sizes itself to 2.5rem (40dp) via a `:host` rule, and its
   `overflow: hidden` rules out expanding the hit area with a pseudo-element.
   Growing the box to 48dp and pulling it back with a negative margin keeps the
   MD3 minimum touch target without letting the larger box reflow the label row
   it sits in. */
.tip-btn {
  width: 48px;
  height: 48px;
  margin: -12px;
}

/* The glyph stays small even though its hit area does not. mdui slots the icon
   rather than exposing a `part`, so the size is set on the slotted element. */
.tip-glyph {
  font-size: 1rem;
  color: rgb(var(--mdui-color-on-surface-variant));
}
</style>
