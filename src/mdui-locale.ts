import { loadLocale } from 'mdui/functions/loadLocale.js';
import type { LoadFunc } from 'mdui/internal/localize.js';

import * as zhCn from 'mdui/locales/zh-cn.js';
import * as zhTw from 'mdui/locales/zh-tw.js';

type LocalePack = Awaited<ReturnType<LoadFunc>>;

/**
 * Locale packs for mdui's own built-in strings (the text-field pattern error, the
 * alert/confirm/prompt button labels). `en-us` is compiled into mdui itself, so only the
 * Chinese packs need loading. They are statically imported rather than lazily fetched —
 * the Tauri CSP restricts `connect-src` to `ipc:`, so a dynamic chunk fetch would fail.
 */
const templates = new Map<string, LocalePack>([
  ['zh-cn', zhCn],
  ['zh-tw', zhTw],
]);

loadLocale(async (locale) => templates.get(locale)!);

/** Maps an app locale (`en` / `zh-CN` / `zh-TW`) onto the matching mdui locale code. */
export function mduiLocaleFor(locale: string): 'en-us' | 'zh-cn' | 'zh-tw' {
  if (locale === 'zh-CN') return 'zh-cn';
  if (locale === 'zh-TW') return 'zh-tw';
  return 'en-us';
}
