import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { CombinedTextField } from '../CombinedTextField';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const input = () => document.querySelector<HTMLInputElement>('input')!;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CombinedTextField', () => {
  it('renders a text input', () => {
    mount(() => <CombinedTextField />);

    expect(input()).not.toBeNull();
    expect(input().getAttribute('type')).toBe('text');
  });

  it('renders the label and links it to the input', () => {
    mount(() => <CombinedTextField label='Name' />);

    const label = document.querySelector('label');

    expect(label).not.toBeNull();
    expect(label!.textContent).toBe('Name');
    expect(label!.getAttribute('for')).toBe(input().id);
  });

  it('renders no label when none is given', () => {
    mount(() => <CombinedTextField />);

    expect(document.querySelector('label')).toBeNull();
  });

  it('forwards inputProps to the input', () => {
    mount(() => (
      <CombinedTextField
        inputProps={{ placeholder: 'Type here', type: 'email' }}
      />
    ));

    expect(input().getAttribute('placeholder')).toBe('Type here');
    expect(input().getAttribute('type')).toBe('email');
  });

  it('renders leading and trailing icons', () => {
    mount(() => (
      <CombinedTextField
        leadingIcons={<span data-testid='leading' />}
        trailingIcons={<span data-testid='trailing' />}
      />
    ));

    expect(document.querySelector('[data-testid="leading"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="trailing"]')).not.toBeNull();
  });

  it('shows the error message and marks the input invalid', () => {
    mount(() => <CombinedTextField errorMessage='Required' />);

    expect(document.body.textContent).toContain('Required');
    expect(input().getAttribute('aria-invalid')).toBe('true');
  });

  it('does not mark the input invalid without an error message', () => {
    mount(() => <CombinedTextField />);

    expect(input().getAttribute('aria-invalid')).not.toBe('true');
  });

  it('merges a custom class with the base classes', () => {
    mount(() => <CombinedTextField class='custom' />);

    const root = document.querySelector<HTMLElement>('.custom');

    expect(root).not.toBeNull();
    expect(root!.classList.contains('flex')).toBe(true);
    expect(root!.classList.contains('flex-col')).toBe(true);
  });
});
