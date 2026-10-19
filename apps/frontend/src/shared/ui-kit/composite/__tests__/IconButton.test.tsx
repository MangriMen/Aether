import type { ComponentProps, JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { IconButton } from '../IconButton';

const TestIcon = (props: ComponentProps<'svg'>) => (
  <svg data-testid='icon' {...props} />
);

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const button = () => document.querySelector<HTMLButtonElement>('button')!;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('IconButton', () => {
  it('renders a button with the icon', () => {
    mount(() => <IconButton icon={TestIcon} />);

    expect(button()).not.toBeNull();
    expect(document.querySelector('[data-testid="icon"]')).not.toBeNull();
  });

  it('renders children when no icon is given', () => {
    mount(() => <IconButton>child</IconButton>);

    expect(button().textContent).toBe('child');
  });

  it('prefers the icon over children', () => {
    mount(() => <IconButton icon={TestIcon}>child</IconButton>);

    expect(document.querySelector('[data-testid="icon"]')).not.toBeNull();
    expect(button().textContent).toBe('');
  });

  it('shows a spinner and disables the button while loading', () => {
    mount(() => <IconButton icon={TestIcon} loading />);

    expect(button().disabled).toBe(true);
    expect(document.querySelector('[data-testid="icon"]')).toBeNull();
    expect(document.querySelector('svg')).not.toBeNull();
  });

  it('disables the button when disabled is set', () => {
    mount(() => <IconButton disabled icon={TestIcon} />);

    expect(button().disabled).toBe(true);
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();

    mount(() => <IconButton icon={TestIcon} onClick={onClick} />);

    button().click();

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('does not call onClick while loading', () => {
    const onClick = vi.fn();

    mount(() => <IconButton icon={TestIcon} loading onClick={onClick} />);

    button().click();

    expect(onClick).not.toHaveBeenCalled();
  });

  it('merges a custom class with the variant classes', () => {
    mount(() => <IconButton class='custom' icon={TestIcon} variant='ghost' />);

    expect(button().classList.contains('custom')).toBe(true);
    expect(button().classList.contains('aspect-square')).toBe(true);
  });

  it('applies the size variant classes', () => {
    mount(() => <IconButton icon={TestIcon} size='sm' />);

    expect(button().classList.contains('size-6')).toBe(true);
  });

  it('forwards arbitrary button attributes', () => {
    mount(() => (
      <IconButton aria-label='browse' icon={TestIcon} type='button' />
    ));

    expect(button().getAttribute('aria-label')).toBe('browse');
    expect(button().getAttribute('type')).toBe('button');
  });
});
