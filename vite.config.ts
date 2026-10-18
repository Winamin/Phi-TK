import { fileURLToPath, URL } from 'node:url';

import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';

import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load environment variables
  const env = loadEnv(mode, process.cwd());
  process.env = { ...process.env, ...env };

  return {
    // Surfaced in the UI (e.g. the Render start screen) via `__APP_VERSION__`.
    define: {
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? '0.0.0'),
    },
    plugins: [
      vue({
        template: {
          compilerOptions: {
            // mdui ships Web Components; Vue must not try to resolve <mdui-*> as Vue components.
            isCustomElement: (tag) => tag.startsWith('mdui-'),
          },
        },
      }),
      // All translations live in SFC <i18n> blocks; there is no src/locales directory.
      VueI18nPlugin({
        defaultSFCLang: 'yml',
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
