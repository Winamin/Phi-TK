import { setTheme } from 'mdui/functions/setTheme.js';
import { setColorScheme } from 'mdui/functions/setColorScheme.js';
import { removeColorScheme } from 'mdui/functions/removeColorScheme.js';
import { getColorFromImage } from 'mdui/functions/getColorFromImage.js';
import { convertFileSrc } from '@tauri-apps/api/core';

export type ThemeMode = 'light' | 'dark' | 'auto';

export const SEED_COLOR = '#82b1ff';

const THEME_MODE_KEY = 'themeMode';
const DERIVED_COLOR_KEY = 'derivedColor';
const WALLPAPER_COLOR_KEY = 'wallpaperColor';

export const BACKGROUND_KEY = 'customBackground';

const CUSTOM_COLORS = [
  { name: 'success', value: '#7dd87d' },
  { name: 'warning', value: '#ffb86b' },
];

export function getThemeMode(): ThemeMode {
  const stored = localStorage.getItem(THEME_MODE_KEY);
  return stored === 'light' || stored === 'dark' || stored === 'auto' ? stored : 'auto';
}

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

function startMediaListener() {
  if (mediaQuery) return;
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaHandler = (e: MediaQueryListEvent) => {
    if (getThemeMode() === 'auto') {
      setTheme(e.matches ? 'dark' : 'light');
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
  reapplyCurrentScheme();
}

export function isWallpaperColorEnabled(): boolean {
  return localStorage.getItem(WALLPAPER_COLOR_KEY) === '1';
}

export function setWallpaperColorEnabled(enabled: boolean) {
  localStorage.setItem(WALLPAPER_COLOR_KEY, enabled ? '1' : '0');
}

let currentSeed = SEED_COLOR;

export function applyColorScheme(hex: string) {
  currentSeed = hex;
  setColorScheme(hex, { customColors: CUSTOM_COLORS });
  promoteSchemeToInline();
  console.info('[theme] applied', hex, '-> primary =', readAppliedPrimary());
}

function prefersDark(): boolean {
  const mode = getThemeMode();
  if (mode === 'dark') return true;
  if (mode === 'light') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function promoteSchemeToInline() {
  const el = document.querySelector<HTMLStyleElement>('style[id^="mdui-custom-color-scheme"]');
  const root = document.documentElement;
  if (!el?.textContent) {
    console.warn('[theme] mdui injected no colour scheme style');
    return;
  }
  const suffix = prefersDark() ? '-dark' : '-light';
  const tokenRe = new RegExp('--mdui-color-([a-z0-9-]+)' + suffix + '\\s*:\\s*([0-9]+\\s*,\\s*[0-9]+\\s*,\\s*[0-9]+)', 'g');
  let n = 0;
  for (const m of el.textContent.matchAll(tokenRe)) {
    root.style.setProperty('--mdui-color-' + m[1], m[2]);
    n++;
  }
  // MDUI's legacy Scheme.surface shares tone 10 with low containers in dark mode.
  // MD3's current surface uses tone 6 (dark) / 98 (light), so grouped cards remain visible.
  const surface = root.style.getPropertyValue(prefersDark() ? '--mdui-color-surface-dim' : '--mdui-color-surface-bright');
  if (surface) {
    root.style.setProperty('--mdui-color-surface', surface);
    root.style.setProperty('--mdui-color-background', surface);
  }
  if (n === 0) console.warn('[theme] no MD3 tokens promoted for seed', currentSeed);
  const bg = root.style.getPropertyValue('--mdui-color-background');
  const fg = root.style.getPropertyValue('--mdui-color-on-background');
  if (bg) root.style.backgroundColor = 'rgb(' + bg + ')';
  if (fg) root.style.color = 'rgb(' + fg + ')';
}

export function reapplyCurrentScheme() {
  const derived = localStorage.getItem(DERIVED_COLOR_KEY);
  applyColorScheme(isWallpaperColorEnabled() && derived ? derived : SEED_COLOR);
}

export function applyStoredColorScheme() {
  reapplyCurrentScheme();
}

export function resetColorScheme() {
  localStorage.removeItem(DERIVED_COLOR_KEY);
  removeColorScheme();
  applyColorScheme(SEED_COLOR);
}

export function getDerivedColor(): string | null {
  return localStorage.getItem(DERIVED_COLOR_KEY);
}

export function readAppliedPrimary(): string {
  return getComputedStyle(document.documentElement).getPropertyValue('--mdui-color-primary').trim();
}

export function colorSchemeDiagnostics(): string {
  const cls = document.documentElement.className;
  const schemeClass = (cls.match(/mdui-custom-color-scheme-[\w-]+/) || ['(none)'])[0];
  const styles = document.querySelectorAll('style[id^="mdui-custom-color-scheme"]').length;
  const inline = document.documentElement.style.getPropertyValue('--mdui-color-primary').trim() || '(none)';
  return (
    'class=' +
    schemeClass +
    ' | injectedStyle=' +
    styles +
    ' | inlinePrimary=' +
    inline +
    ' | primary=' +
    readAppliedPrimary() +
    ' | derived=' +
    (getDerivedColor() ?? '(none)') +
    ' | html=' +
    cls
  );
}

export async function deriveColorFromWallpaper(path: string): Promise<string | null> {
  try {
    const img = new Image();
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

export async function ensureDerivedColor(): Promise<void> {
  if (!isWallpaperColorEnabled() || localStorage.getItem(DERIVED_COLOR_KEY)) return;
  const path = localStorage.getItem(BACKGROUND_KEY);
  if (path) await deriveColorFromWallpaper(path);
}

export function initTheme() {
  setTheme(getThemeMode());
  applyStoredColorScheme();
  if (getThemeMode() === 'auto') startMediaListener();
  void ensureDerivedColor();
}
