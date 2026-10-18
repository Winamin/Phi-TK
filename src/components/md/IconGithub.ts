/**
 * GitHub logo as a `<mdui-icon-github>` element. Material Symbols has no brand icons, so
 * this is the one hand-authored icon; the shadow styling mirrors `@mdui/icons` so it sizes
 * and inherits colour exactly like the generated ones.
 *
 * Deliberately a plain custom element rather than a Lit one: `lit` is only a transitive
 * dependency of mdui, and a static SVG needs no reactivity.
 */
const TEMPLATE = `<style>
:host {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1em;
  height: 1em;
  vertical-align: -0.125em;
  fill: currentColor;
}
svg { width: 100%; height: 100% }
</style>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .3a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2.2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.7-1.6-2.7-.3-5.5-1.3-5.5-6 0-1.2.5-2.3 1.3-3.1-.2-.4-.6-1.6.1-3.2 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.8 18.3 5 18.3 5c.7 1.6.2 2.8.1 3.2.8.8 1.3 1.9 1.3 3.2 0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 .3"/></svg>`;

export class IconGithub extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' }).innerHTML = TEMPLATE;
  }
}

if (!customElements.get('mdui-icon-github')) {
  customElements.define('mdui-icon-github', IconGithub);
}

declare global {
  interface HTMLElementTagNameMap {
    'mdui-icon-github': IconGithub;
  }
}
