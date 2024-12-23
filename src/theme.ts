// Generated using https://theme-generator.vuetifyjs.com/

import { createVuetify } from 'vuetify';
import 'vuetify/styles';
import { aliases, mdi } from 'vuetify/iconsets/mdi';
import { watchEffect } from 'vue';

watchEffect(() => {
  const prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
  vuetify.framework.theme.global.name.value = prefersDarkMode ? 'myCustomDarkTheme' : 'myCustomLightTheme';
});

const myCustomLightTheme = {
  dark: false,
  colors: {
    primary: '#e91e63',
    secondary: '#f44336',
    accent: '#e91e63',
  },
};

const myCustomDarkTheme = {
  dark: true,
  colors: {
    primary: '#e91e63',
    secondary: '#f44336',
    accent: '#e91e63',
  },
};

const vuetify = createVuetify({
  theme: {
    defaultTheme: 'myCustomLightTheme',
    themes: {
      myCustomLightTheme,
      myCustomDarkTheme,
    },
  },
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: { mdi },
  },
});

export default vuetify;
