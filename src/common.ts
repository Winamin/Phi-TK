import { SUPPORTED_LOCALES, i18n } from './main';

import { setLocale } from 'mdui/functions/setLocale.js';
import { snackbar } from 'mdui/functions/snackbar.js';
import { mduiLocaleFor } from './mdui-locale';

import moment from 'moment';

import 'moment/dist/locale/zh-cn';
import 'moment/dist/locale/zh-hk';

export function anyFilter() {
  return {
    name: i18n.global.t('any-filter'),
    extensions: ['*'],
  };
}

export function isString(s: unknown): s is string {
  return typeof s === 'string';
}

export const RULES = {
  non_empty: (value: string | null | undefined) => (value ?? '').trim().length > 0 || i18n.global.t('rules.non-empty'),
  positive: (value: string) => (isNumeric(value) && Number(value) >= 0) || i18n.global.t('rules.positive'),
};

/**
 * Runs native constraint validation over every mdui form control inside `root`, focusing
 * and annotating the first invalid one. mdui's fields are form-associated custom elements,
 * so `required` / `min` / `max` / `type="number"` are enforced by the browser — this
 * replaces Vuetify's `<v-form>.validate()`.
 */
export function validateFields(root: HTMLElement | null | undefined): boolean {
  if (!root) return true;
  const fields = root.querySelectorAll<HTMLElement & { reportValidity?: () => boolean }>('mdui-text-field, mdui-select, mdui-checkbox, mdui-radio-group');
  for (const field of fields) {
    if (field.reportValidity && !field.reportValidity()) return false;
  }
  return true;
}

export function isNumeric(num: any) {
  return (typeof num === 'number' || (typeof num === 'string' && num.trim() !== '')) && !isNaN(num as number);
}

export function setTitle(title: string) {
  document.title = title.length ? title + ' - Phi-TK' : 'Phi-TK';
}

export function changeLocale(locale: string) {
  if (locale.startsWith('en')) locale = 'en';
  if (!SUPPORTED_LOCALES.includes(locale)) locale = 'en';
  i18n.global.locale.value = (locale === 'zh-TW' ? 'zh-CN' : locale) as typeof i18n.global.locale.value;
  localStorage.setItem('locale', locale);
  // Keep mdui's own built-in strings in step with the UI language.
  setLocale(mduiLocaleFor(locale)).catch((e) => console.error('failed to set mdui locale', e));
  const momentLocale =
    {
      'zh-CN': 'zh-cn',
      'zh-TW': 'zh-hk',
      en: 'en-us',
    }[locale] ?? 'en-us';
  moment.locale(momentLocale);
}

export function toast(message: string, kind?: 'success' | 'info' | 'warning' | 'error') {
  const bar = snackbar({
    message,
    placement: 'top',
    autoCloseDelay: 2000,
    closeOnOutsideClick: true,
  });
  if (kind) bar.classList.add(`toast-${kind}`);
}

export function toastError(error: any) {
  console.error(error);
  const msg = error instanceof Error ? error.message : String(error);
  if (msg.length) toast(msg, 'error');
}
