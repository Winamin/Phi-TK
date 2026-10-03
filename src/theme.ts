import { setTheme } from 'mdui/functions/setTheme.js';
import { setColorScheme } from 'mdui/functions/setColorScheme.js';
import { removeColorScheme } from 'mdui/functions/removeColorScheme.js';
import { getColorFromImage } from 'mdui/functions/getColorFromImage.js';
import { convertFileSrc } from '@tauri-apps/api/core';

export type ThemeMode = 'light' | 'dark' | 'auto';

/**
 * Brand seed colour, carried over from the previous Vuetify palette. mdui derives the
 * full MD3 tonal palette (primary/secondary/tertiary/surface/…) from this single hex.
 */
export const SEED_COLOR = '#82b1ff';

const THEME_MODE_KEY = 'themeMode';
const DERIVED_COLOR_KEY = 'derivedColor';
const WALLPAPER_COLOR_KEY = 'wallpaperColor';

/** 自定义背景图的存储键。设置页写入、取色时读取，只此一处定义。 */
export const BACKGROUND_KEY = 'customBackground';

/** `success` / `warning` have no MD3 equivalent, so they are registered as custom colours. */
const CUSTOM_COLORS = [
  { name: 'success', value: '#7dd87d' },
  { name: 'warning', value: '#ffb86b' },
];

export function getThemeMode(): ThemeMode {
  const stored = localStorage.getItem(THEME_MODE_KEY);
  return stored === 'light' || stored === 'dark' || stored === 'auto' ? stored : 'dark';
}

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

function startMediaListener() {
  if (mediaQuery) return;
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaHandler = (e: MediaQueryListEvent) => {
    if (getThemeMode() === 'auto') {
      setTheme(e.matches ? 'dark' : 'light');
      // 明暗切换后 MD3 token 要换成另一套，否则内联变量还是上一次的。
      reapplyCurrentScheme();
    }
  };
  mediaQuery.addEventListener('change', mediaHandler);
}

function stopMediaListener() {
  if (mediaQuery && mediaHandler) {
    mediaQuery.removeEventListener('change', mediaHandler);
    mediaQuery = null;
    mediaHandler = null;
  }
}

export function applyThemeMode(mode: ThemeMode) {
  setTheme(mode);
  localStorage.setItem(THEME_MODE_KEY, mode);
  if (mode === 'auto') {
    startMediaListener();
  } else {
    stopMediaListener();
  }
  // light <-> dark 用的是两套 token，必须重新应用一次。
  reapplyCurrentScheme();
}

/* ------------------------------------------------------------------ *
 * 配色
 * ------------------------------------------------------------------ */

/** Whether the accent colour should be derived from the wallpaper (Material You). */
export function isWallpaperColorEnabled(): boolean {
  return localStorage.getItem(WALLPAPER_COLOR_KEY) === '1';
}

export function setWallpaperColorEnabled(enabled: boolean) {
  localStorage.setItem(WALLPAPER_COLOR_KEY, enabled ? '1' : '0');
}

let currentSeed = SEED_COLOR;

/**
 * 应用 Material You 配色。
 *
 * mdui 的 `setColorScheme` 会往 <head> 末尾插一个 <style> —— 那只是普通样式表：一旦应用
 * 自己的 CSS 在它之后加载（开发模式 HMR 重新注入、路由级样式按需注入等），同特异性的
 * 规则就会被后者覆盖，表现为「取到色了、界面却不变」。
 *
 * 所以这里在调用 mdui 之后，把 MD3 token 提升为 <html> 上的**内联**自定义属性：内联样式
 * 优先级最高，之后再加载多少样式表都覆盖不掉。
 */
export function applyColorScheme(hex: string) {
  currentSeed = hex;
  setColorScheme(hex, { customColors: CUSTOM_COLORS });
  promoteSchemeToInline();
  console.info('[theme] applied', hex, '-> primary =', readAppliedPrimary());
}

/** 当前主题模式实际对应的明暗。 */
function prefersDark(): boolean {
  const mode = getThemeMode();
  if (mode === 'dark') return true;
  if (mode === 'light') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

/** 把 mdui 生成的 light/dark token 中「当前生效」的那一套写成 <html> 的内联变量。 */
function promoteSchemeToInline() {
  const el = document.querySelector<HTMLStyleElement>('style[id^="mdui-custom-color-scheme"]');
  const root = document.documentElement;
  if (!el?.textContent) {
    console.warn('[theme] mdui injected no colour scheme style');
    return;
  }
  const suffix = prefersDark() ? '-dark' : '-light';
  const tokenRe = new RegExp(
    '--mdui-color-([a-z0-9-]+)' + suffix + '\\s*:\\s*([0-9]+\\s*,\\s*[0-9]+\\s*,\\s*[0-9]+)',
    'g',
  );
  let n = 0;
  for (const m of el.textContent.matchAll(tokenRe)) {
    root.style.setProperty('--mdui-color-' + m[1], m[2]);
    n++;
  }
  if (n === 0) console.warn('[theme] no MD3 tokens promoted for seed', currentSeed);
  const bg = root.style.getPropertyValue('--mdui-color-background');
  const fg = root.style.getPropertyValue('--mdui-color-on-background');
  if (bg) root.style.backgroundColor = 'rgb(' + bg + ')';
  if (fg) root.style.color = 'rgb(' + fg + ')';
}

/** 按当前设置重新应用配色（切换主题模式、系统明暗变化时用）。 */
export function reapplyCurrentScheme() {
  const derived = localStorage.getItem(DERIVED_COLOR_KEY);
  applyColorScheme(isWallpaperColorEnabled() && derived ? derived : SEED_COLOR);
}

/**
 * Re-applies whichever accent colour is currently in effect: the wallpaper-derived one
 * when that feature is on and a colour has been cached, otherwise the brand seed.
 */
export function applyStoredColorScheme() {
  reapplyCurrentScheme();
}

export function resetColorScheme() {
  localStorage.removeItem(DERIVED_COLOR_KEY);
  removeColorScheme();
  applyColorScheme(SEED_COLOR);
}

/** 当前缓存下来的壁纸取色结果（没有则为 null）。 */
export function getDerivedColor(): string | null {
  return localStorage.getItem(DERIVED_COLOR_KEY);
}

/** 当前【实际生效】的主色，形如 `0, 109, 57`。用来和取到的种子色对照。 */
export function readAppliedPrimary(): string {
  return getComputedStyle(document.documentElement).getPropertyValue('--mdui-color-primary').trim();
}

/** 一行诊断：配色 class、注入的 <style>、当前生效主色、缓存种子。排查「没生效」时看这个。 */
export function colorSchemeDiagnostics(): string {
  const cls = document.documentElement.className;
  const schemeClass = (cls.match(/mdui-custom-color-scheme-[\w-]+/) || ['(none)'])[0];
  const styles = document.querySelectorAll('style[id^="mdui-custom-color-scheme"]').length;
  const inline = document.documentElement.style.getPropertyValue('--mdui-color-primary').trim() || '(none)';
  return (
    'class=' + schemeClass + ' | injectedStyle=' + styles + ' | inlinePrimary=' + inline +
    ' | primary=' + readAppliedPrimary() + ' | derived=' + (getDerivedColor() ?? '(none)') + ' | html=' + cls
  );
}

/* ------------------------------------------------------------------ *
 * 壁纸取色
 * ------------------------------------------------------------------ */

/**
 * Extracts the dominant colour from a wallpaper file and applies it as the accent.
 * `path` is a filesystem path, converted to an `asset:` URL the webview can load.
 * Returns the derived hex, or null when the image could not be read.
 */
export async function deriveColorFromWallpaper(path: string): Promise<string | null> {
  try {
    const img = new Image();
    // 必须设 crossOrigin：asset 协议属于另一个 origin，不设的话 canvas 会被污染，
    // getImageData 会抛 SecurityError —— 这个开关以前「看起来是摆设」就是因为这个
    // 异常被下面的 catch 吞掉了。Tauri 的 asset 协议会返回 Access-Control-Allow-Origin，
    // 所以带上这个属性后 CORS 校验能通过。
    img.crossOrigin = 'anonymous';
    img.src = convertFileSrc(path);
    await img.decode();
    const hex = await getColorFromImage(img);
    localStorage.setItem(DERIVED_COLOR_KEY, hex);
    applyColorScheme(hex);
    return hex;
  } catch (e) {
    console.error('failed to derive colour from wallpaper', e);
    return null;
  }
}

/**
 * 开关开着、但还没有缓存颜色时补算一次（升级前开过开关、或缓存被清掉的情况）。
 */
export async function ensureDerivedColor(): Promise<void> {
  if (!isWallpaperColorEnabled() || localStorage.getItem(DERIVED_COLOR_KEY)) return;
  const path = localStorage.getItem(BACKGROUND_KEY);
  if (path) await deriveColorFromWallpaper(path);
}

/** Called once at startup, before the app mounts. */
export function initTheme() {
  setTheme(getThemeMode());
  applyStoredColorScheme();
  if (getThemeMode() === 'auto') startMediaListener();
  // 别阻塞启动：取色是异步的，算完自动应用。
  void ensureDerivedColor();
}
