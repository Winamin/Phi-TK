declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

/** Injected by Vite's `define` — see `vite.config.ts`. */
declare const __APP_VERSION__: string;
