import { computed, onMounted, onUnmounted, readonly, ref, type ComputedRef, type Ref } from 'vue';

/**
 * MD3 window size classes.
 *
 * Material 3 breaks the window into five classes and prescribes a different
 * layout for each. They are exposed here as a single composable so components
 * agree on where the boundaries are instead of each hardcoding its own numbers.
 *
 * The thresholds are read from mdui's `--mdui-breakpoint-*` custom properties at
 * runtime, so the design tokens stay the single source of truth. Note that CSS
 * *cannot* do this — custom properties are not allowed in `@media` conditions —
 * which is why the stylesheets use the SCSS mixins in `styles/_breakpoints.scss`
 * carrying the same numbers.
 */
export type WindowSizeClass = 'compact' | 'medium' | 'expanded' | 'large' | 'extra-large';

type Breakpoints = Record<WindowSizeClass, number>;

/**
 * Maps a size class to the mdui custom-property suffix.
 *
 * mdui's scale is `xs 0 / sm 600 / md 840 / lg 1080 / xl 1440 / xxl 1920`, which
 * is finer-grained than MD3's five classes. The extra-large class therefore maps
 * to `xl` (1440) — deliberately *not* `xxl` (1920), which is a wider tier MD3
 * does not name.
 */
const TOKEN_NAMES: Record<Exclude<WindowSizeClass, 'compact'>, string> = {
  medium: 'sm',
  expanded: 'md',
  large: 'lg',
  'extra-large': 'xl',
};

/**
 * Fallbacks match mdui's shipped defaults. They are only used if the stylesheet
 * has not loaded yet (or a token was removed), so the app never misclassifies
 * because a variable was missing.
 */
const FALLBACK: Breakpoints = {
  compact: 0,
  medium: 600,
  expanded: 840,
  large: 1080,
  'extra-large': 1440,
};

/** Reads the px-valued mdui breakpoint tokens off the document root. */
function readBreakpoints(): Breakpoints {
  const result = { ...FALLBACK };
  try {
    const root = getComputedStyle(document.documentElement);
    for (const [cls, suffix] of Object.entries(TOKEN_NAMES)) {
      const value = parseFloat(root.getPropertyValue(`--mdui-breakpoint-${suffix}`).trim());
      if (!isNaN(value)) result[cls as WindowSizeClass] = value;
    }
  } catch (e) {
    console.error('Failed to read mdui breakpoint tokens; using fallbacks.', e);
  }
  return result;
}

function classify(px: number, bp: Breakpoints): WindowSizeClass {
  if (px >= bp['extra-large']) return 'extra-large';
  if (px >= bp.large) return 'large';
  if (px >= bp.expanded) return 'expanded';
  if (px >= bp.medium) return 'medium';
  return 'compact';
}

/**
 * Tracks the current window size class and re-evaluates on resize.
 *
 * The returned refs are readonly, and listener cleanup is handled here so no
 * caller has to remember it.
 */
export function useWindowSize() {
  const breakpoints = ref<Breakpoints>(FALLBACK);
  const width = ref(typeof window === 'undefined' ? FALLBACK.medium : window.innerWidth);
  const windowClass = ref<WindowSizeClass>(classify(width.value, FALLBACK));

  function update() {
    width.value = window.innerWidth;
    windowClass.value = classify(width.value, breakpoints.value);
  }

  onMounted(() => {
    breakpoints.value = readBreakpoints();
    update();
    window.addEventListener('resize', update);
  });

  onUnmounted(() => window.removeEventListener('resize', update));

  return {
    windowClass: readonly(windowClass) as Readonly<Ref<WindowSizeClass>>,
    width: readonly(width) as Readonly<Ref<number>>,
    /** True below the `medium` breakpoint — MD3's phone-sized layout. */
    isCompact: computed(() => windowClass.value === 'compact') as ComputedRef<boolean>,
  };
}
