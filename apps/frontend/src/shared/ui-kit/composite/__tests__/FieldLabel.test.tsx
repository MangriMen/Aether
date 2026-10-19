import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { FieldLabel } from '../FieldLabel';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('FieldLabel', () => {
  it('renders a span with the base class', () => {
    mount(() => <FieldLabel>Name</FieldLabel>);

    const span = document.querySelector('span');

    expect(span).not.toBeNull();
    expect(span!.textContent).toBe('Name');
    expect(span!.classList.contains('font-medium')).toBe(true);
  });

  it('merges a custom class with the base class', () => {
    mount(() => <FieldLabel class='custom'>Name</FieldLabel>);

    const span = document.querySelector('span')!;

    expect(span.classList.contains('font-medium')).toBe(true);
    expect(span.classList.contains('custom')).toBe(true);
  });

  it('forwards arbitrary span attributes', () => {
    mount(() => (
      <FieldLabel data-testid='label' title='tip'>
        Name
      </FieldLabel>
    ));

    const span = document.querySelector('span')!;

    expect(span.getAttribute('data-testid')).toBe('label');
    expect(span.getAttribute('title')).toBe('tip');
  });
});
