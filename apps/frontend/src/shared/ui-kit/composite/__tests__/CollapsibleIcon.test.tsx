import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { CollapsibleIcon } from '../CollapsibleIcon';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const button = () => document.querySelector<HTMLButtonElement>('button')!;

const icon = () => button().querySelector('svg')!;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CollapsibleIcon', () => {
  it('renders a ghost icon button with a chevron', () => {
    mount(() => <CollapsibleIcon />);

    expect(button()).not.toBeNull();
    expect(icon()).not.toBeNull();
    expect(button().classList.contains('aspect-square')).toBe(true);
  });

  it('does not rotate the chevron when closed', () => {
    mount(() => <CollapsibleIcon open={false} />);

    expect(icon().classList.contains('-rotate-180')).toBe(false);
  });

  it('rotates the chevron when open', () => {
    mount(() => <CollapsibleIcon open />);

    expect(icon().classList.contains('-rotate-180')).toBe(true);
  });

  it('merges a custom class with the button classes', () => {
    mount(() => <CollapsibleIcon class='custom' />);

    expect(button().classList.contains('custom')).toBe(true);
    expect(button().classList.contains('aspect-square')).toBe(true);
  });

  it('forwards attributes to the button', () => {
    mount(() => <CollapsibleIcon aria-label='toggle' type='button' />);

    expect(button().getAttribute('aria-label')).toBe('toggle');
    expect(button().getAttribute('type')).toBe('button');
  });
});
