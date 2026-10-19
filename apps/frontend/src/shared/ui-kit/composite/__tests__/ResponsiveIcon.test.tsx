import type { ComponentProps, JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';

import { ResponsiveIcon } from '../ResponsiveIcon';

beforeAll(() => {
  globalThis.ResizeObserver ??= class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
});

const TestIcon = (props: ComponentProps<'svg'>) => (
  <svg data-testid='icon' {...props} />
);

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ResponsiveIcon', () => {
  it('renders the given icon component', () => {
    mount(() => <ResponsiveIcon icon={TestIcon} />);

    expect(document.querySelector('[data-testid="icon"]')).not.toBeNull();
  });

  it('applies the base classes', () => {
    mount(() => <ResponsiveIcon icon={TestIcon} />);

    const icon = document.querySelector('[data-testid="icon"]')!;

    expect(icon.classList.contains('inline-flex')).toBe(true);
    expect(icon.classList.contains('size-full')).toBe(true);
  });

  it('merges a custom class with the base classes', () => {
    mount(() => <ResponsiveIcon class='custom' icon={TestIcon} />);

    const icon = document.querySelector('[data-testid="icon"]')!;

    expect(icon.classList.contains('custom')).toBe(true);
    expect(icon.classList.contains('inline-flex')).toBe(true);
  });

  it('sets no font-size before the element is measured', () => {
    mount(() => <ResponsiveIcon icon={TestIcon} />);

    const icon = document.querySelector<HTMLElement>('[data-testid="icon"]')!;

    expect(icon.style.fontSize).toBe('');
  });
});
