import { fileURLToPath, URL } from 'node:url';

import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';

import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite';

import { resolve, dirname } from 'node:path';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load environment variables
  const env = loadEnv(mode, process.cwd());
  process.env = { ...process.env, ...env };

  return {
    plugins: [
      vue(),
      VueI18nPlugin({
        defaultSFCLang: 'yml',
        include: resolve(dirname(fileURLToPath(import.meta.url)), './src/locales/**'),
      }),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    build: {
      chunkSizeWarningLimit: 1000,
      rollupOptions: {
        output: {
          manualChunks(id) {
            if (id.includes('node_modules')) {
              return id.toString().split('node_modules/')[1].split('/')[0].toString();
            }
          },
        },
      },
    },
  };
});
