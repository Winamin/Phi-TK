import type { Directive } from 'vue';

const POLITE_ID = 'app-live-polite';
const ASSERTIVE_ID = 'app-live-assertive';

function liveRegion(id: string, assertive: boolean): HTMLElement {
  let el = document.getElementById(id);
  if (!el) {
    el = document.createElement('div');
    el.id = id;
    el.className = 'md-sr-only';
    el.setAttribute('role', assertive ? 'alert' : 'status');
    el.setAttribute('aria-live', assertive ? 'assertive' : 'polite');
    el.setAttribute('aria-atomic', 'true');
    document.body.appendChild(el);
  }
  return el;
}

export function initA11y() {
  liveRegion(POLITE_ID, false);
  liveRegion(ASSERTIVE_ID, true);
}

export function announce(message: string, assertive = false) {
  const el = liveRegion(assertive ? ASSERTIVE_ID : POLITE_ID, assertive);
  el.textContent = '';
  setTimeout(() => {
    el.textContent = message;
  }, 30);
}

const SKIP_INERT = 'script, style, link, [aria-live], [role="status"], [role="alert"]';

export function inertOutside(container: HTMLElement): HTMLElement[] {
  const touched: HTMLElement[] = [];
  let node: Element | null = container;

  while (node && node.parentElement && node !== document.body) {
    const parent: HTMLElement = node.parentElement;
    for (const sibling of Array.from(parent.children)) {
      if (sibling === node || !(sibling instanceof HTMLElement)) continue;
      if (sibling.hasAttribute('inert') || sibling.matches(SKIP_INERT)) continue;
      sibling.setAttribute('inert', '');
      touched.push(sibling);
    }
    node = parent;
  }

  return touched;
}

export interface FocusTrap {
  activate(options?: { initial?: HTMLElement | null; inert?: (HTMLElement | null)[] }): void;
  deactivate(): void;
}

export function createFocusTrap(getContainer: () => HTMLElement | null | undefined): FocusTrap {
  let active = false;
  let returnTo: HTMLElement | null = null;
  let inerted: HTMLElement[] = [];
  let borrowedTabIndex = false;

  return {
    activate({ initial, inert } = {}) {
      if (active) return;
      active = true;
      returnTo = document.activeElement instanceof HTMLElement ? document.activeElement : null;

      const container = getContainer();
      if (container) {
        if (!container.hasAttribute('tabindex')) {
          container.setAttribute('tabindex', '-1');
          borrowedTabIndex = true;
        }
        const target = initial ?? container;
        let attempts = 6;
        const placeFocus = () => {
          target.focus({ preventScroll: true });
          if (document.activeElement !== target && --attempts > 0) {
            setTimeout(placeFocus, 16);
          }
        };
        placeFocus();
        inerted.push(...inertOutside(container));
      }

      for (const el of inert ?? []) {
        if (el && !el.hasAttribute('inert')) {
          el.setAttribute('inert', '');
          inerted.push(el);
        }
      }
    },

    deactivate() {
      if (!active) return;
      active = false;

      for (const el of inerted) el.removeAttribute('inert');
      inerted = [];

      if (borrowedTabIndex) {
        getContainer()?.removeAttribute('tabindex');
        borrowedTabIndex = false;
      }

      const target = returnTo;
      returnTo = null;
      if (target && target !== document.body && target.isConnected && target.getClientRects().length) {
        target.focus({ preventScroll: true });
        return;
      }
      document.getElementById('main-content')?.focus({ preventScroll: true });
    },
  };
}

const traps = new WeakMap<HTMLElement, FocusTrap>();

export const vFocusTrap: Directive<HTMLElement, boolean> = {
  mounted(el, binding) {
    const trap = createFocusTrap(() => el);
    traps.set(el, trap);
    if (binding.value) trap.activate();
  },
  updated(el, binding) {
    if (binding.value === binding.oldValue) return;
    const trap = traps.get(el);
    if (!trap) return;
    if (binding.value) trap.activate();
    else trap.deactivate();
  },
  unmounted(el) {
    traps.get(el)?.deactivate();
    traps.delete(el);
  },
};

function resolveInShadow(host: Element | null | undefined, path: string): HTMLElement | null {
  let scope: Element | ShadowRoot | null | undefined = host?.shadowRoot;
  let found: Element | null = null;

  for (const step of path.split(' ')) {
    if (!scope) return null;
    found = scope.querySelector(step);
    if (!found) return null;
    scope = found.shadowRoot ?? found;
  }

  return found instanceof HTMLElement ? found : null;
}

export function labelShadowControl(
  host: Element | null | undefined,
  path: string | string[],
  text: string,
  attempts = 12,
) {
  const paths = Array.isArray(path) ? path : [path];

  function apply(): boolean {
    for (const candidate of paths) {
      const target = resolveInShadow(host, candidate);
      if (!target) continue;
      if (text) target.setAttribute('aria-label', text);
      else target.removeAttribute('aria-label');
      return true;
    }
    return false;
  }

  if (apply()) return;

  function retry() {
    let left = attempts;
    const tick = () => {
      if (apply() || --left <= 0) return;
      setTimeout(tick, 16);
    };
    setTimeout(tick, 16);
  }

  const complete = (host as { updateComplete?: Promise<unknown> } | null | undefined)?.updateComplete;
  if (complete && typeof complete.then === 'function') {
    void complete.then(() => {
      if (!apply()) retry();
    });
    return;
  }

  retry();
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
