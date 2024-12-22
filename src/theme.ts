// Generated using https://theme-generator.vuetifyjs.com/

import { createVuetify } from 'vuetify';
import 'vuetify/styles';
import { aliases, mdi } from 'vuetify/iconsets/mdi';

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
