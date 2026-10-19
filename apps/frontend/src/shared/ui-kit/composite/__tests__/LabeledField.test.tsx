import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { LabeledField } from '../LabeledField';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('LabeledField', () => {
  it('wraps a string label in a FieldLabel span', () => {
    mount(() => <LabeledField label='Name'>input</LabeledField>);

    const span = document.querySelector('span');

    expect(span).not.toBeNull();
    expect(span!.textContent).toBe('Name');
    expect(span!.classList.contains('font-medium')).toBe(true);
  });

  it('renders a JSX label as is', () => {
    mount(() => (
      <LabeledField label={<strong>Custom</strong>}>input</LabeledField>
    ));

    expect(document.querySelector('strong')!.textContent).toBe('Custom');
    expect(document.querySelector('span')).toBeNull();
  });

  it('renders children after the label', () => {
    mount(() => (
      <LabeledField label='Name'>
        <input />
      </LabeledField>
    ));

    const root = document.body.firstElementChild!;

    expect(root.children[0]!.textContent).toBe('Name');
    expect(root.children[1]!.tagName).toBe('INPUT');
  });

  it('renders no label when none is given', () => {
    mount(() => <LabeledField>input</LabeledField>);

    expect(document.querySelector('span')).toBeNull();
    expect(document.body.textContent).toContain('input');
  });

  it('merges a custom class with the base classes', () => {
    mount(() => (
      <LabeledField class='custom' label='Name'>
        input
      </LabeledField>
    ));

    const root = document.querySelector<HTMLElement>('.custom');

    expect(root).not.toBeNull();
    expect(root!.classList.contains('flex')).toBe(true);
    expect(root!.classList.contains('flex-col')).toBe(true);
  });

  it('forwards arbitrary div attributes', () => {
    mount(() => (
      <LabeledField data-testid='field' label='Name'>
        input
      </LabeledField>
    ));

    expect(document.querySelector('[data-testid="field"]')).not.toBeNull();
  });
});
