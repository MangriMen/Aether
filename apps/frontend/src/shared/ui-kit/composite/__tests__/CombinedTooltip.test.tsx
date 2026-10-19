import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { CombinedTooltip } from '../CombinedTooltip';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const trigger = () => document.querySelector<HTMLButtonElement>('button')!;

const hover = () => {
  trigger().dispatchEvent(
    new PointerEvent('pointerenter', { bubbles: false, pointerType: 'mouse' }),
  );
  trigger().dispatchEvent(
    new PointerEvent('pointermove', { bubbles: true, pointerType: 'mouse' }),
  );
  vi.advanceTimersByTime(1000);
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  vi.useRealTimers();
});

describe('CombinedTooltip', () => {
  it('renders the trigger with its attributes', () => {
    mount(() => (
      <CombinedTooltip aria-label='tip' label='Label'>
        trigger
      </CombinedTooltip>
    ));

    expect(trigger()).not.toBeNull();
    expect(trigger().getAttribute('aria-label')).toBe('tip');
    expect(trigger().textContent).toBe('trigger');
  });

  it('shows the label after hovering the trigger', () => {
    mount(() => (
      <CombinedTooltip label='Tooltip text'>trigger</CombinedTooltip>
    ));

    hover();

    expect(document.body.textContent).toContain('Tooltip text');
  });

  it('renders a JSX label', () => {
    mount(() => (
      <CombinedTooltip label={<strong>Rich</strong>}>trigger</CombinedTooltip>
    ));

    hover();

    expect(document.querySelector('strong')!.textContent).toBe('Rich');
  });

  it('does not show the label when disableTooltip is set', () => {
    mount(() => (
      <CombinedTooltip disableTooltip label='Tooltip text'>
        trigger
      </CombinedTooltip>
    ));

    hover();

    expect(document.body.textContent).not.toContain('Tooltip text');
  });

  it('renders a custom trigger component', () => {
    mount(() => (
      <CombinedTooltip as='a' href='/x' label='Label'>
        link
      </CombinedTooltip>
    ));

    const link = document.querySelector('a');

    expect(link).not.toBeNull();
    expect(link!.getAttribute('href')).toBe('/x');
  });
});
