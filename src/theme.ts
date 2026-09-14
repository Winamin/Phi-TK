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
}

/** Whether the accent colour should be derived from the wallpaper (Material You). */
export function isWallpaperColorEnabled(): boolean {
  return localStorage.getItem(WALLPAPER_COLOR_KEY) === '1';
}

export function setWallpaperColorEnabled(enabled: boolean) {
  localStorage.setItem(WALLPAPER_COLOR_KEY, enabled ? '1' : '0');
}

export function applyColorScheme(hex: string) {
  setColorScheme(hex, { customColors: CUSTOM_COLORS });
}

/**
 * Re-applies whichever accent colour is currently in effect: the wallpaper-derived one
 * when that feature is on and a colour has been cached, otherwise the brand seed.
 */
export function applyStoredColorScheme() {
  const derived = localStorage.getItem(DERIVED_COLOR_KEY);
  applyColorScheme(isWallpaperColorEnabled() && derived ? derived : SEED_COLOR);
}

export function resetColorScheme() {
  localStorage.removeItem(DERIVED_COLOR_KEY);
  removeColorScheme();
  applyColorScheme(SEED_COLOR);
}

/**
 * Extracts the dominant colour from a wallpaper file and applies it as the accent.
 * `path` is a filesystem path, converted to an `asset:` URL the webview can load.
 * Returns the derived hex, or null when the image could not be read.
 */
export async function deriveColorFromWallpaper(path: string): Promise<string | null> {
  try {
    const img = new Image();
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

/** Called once at startup, before the app mounts. */
export function initTheme() {
  setTheme(getThemeMode());
  applyStoredColorScheme();
  if (getThemeMode() === 'auto') startMediaListener();
}
