// Generated using https://theme-generator.vuetifyjs.com/

import { useVuetify } from 'vuetify/lib/framework';
import { ref, watch } from 'vue';

export default {
  setup() {
    const vuetify = useVuetify();
    const prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const isDarkMode = ref(prefersDarkMode);

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      isDarkMode.value = e.matches;
      vuetify.framework.theme.dark = isDarkMode.value;
    });

    vuetify.framework.theme.dark = isDarkMode.value;

    return {
      isDarkMode,
    };
  },

  theme: {
    dark: true,
    themes: {
      light: {
        primary: '#e91e63',
        secondary: '#f44336',
        accent: '#e91e63',
      },
      dark: {
        primary: '#e91e63',
        secondary: '#f44336',
        accent: '#e91e63',
      },
    },
  },
};
