import 'mdui/mdui.css';
import './assets/main.css';
import './mdui-components';
import './icons';

import { createApp } from 'vue';

import { createI18n } from 'vue-i18n';

import { changeLocale } from './common';

import App from './App.vue';
import router from './router';

import { initTheme } from './theme';

export const SUPPORTED_LOCALES = ['en', 'zh-CN', 'zh-TW'];

let locale = localStorage.getItem('locale');
if (!locale) {
  locale = 'en';
  for (const alt of navigator.languages) {
    if (SUPPORTED_LOCALES.includes(alt)) {
      locale = alt;
      break;
    }
  }
}

const i18n = createI18n({
  locale: 'en',
  fallbackLocale: 'en',
  messages: {
    en: {
      rules: {
        'non-empty': 'Must not be empty',
        positive: 'Must be a positive number',
        'positive-int': 'Must be a positive integer',
      },
      'has-error': 'There are errors in the configuration',
      'any-filter': 'All files',
      // Router titles are resolved through `i18n.global.t`, so they must live in the
      // global message set — SFC <i18n> blocks are locally scoped and invisible here.
      // `title-default` is intentionally absent: `missing()` maps it to '' → plain "Phi-TK".
      'title-render': 'Render',
      'title-rpe': 'RPE',
      'title-tasks': 'Tasks',
      'title-batch-render': 'Batch Render',
      'title-setting': 'Settings',
      'title-about': 'About',
    },
    'zh-CN': {
      rules: {
        'non-empty': '不能为空',
        positive: '必须是正数',
        'positive-int': '必须是正整数',
      },
      'has-error': '配置中有错误',
      'any-filter': '所有文件',
      'title-render': '渲染',
      'title-rpe': 'RPE',
      'title-tasks': '任务',
      'title-batch-render': '批量渲染',
      'title-setting': '设置',
      'title-about': '关于',
    },
  },
  legacy: false,
  missing(_locale, key) {
    if (key.startsWith('title-')) return '';
    return key;
  },
});
changeLocale(locale);

initTheme();

const app = createApp(App);
app.use(i18n).use(router);

app.mount('#app');

export { i18n };
