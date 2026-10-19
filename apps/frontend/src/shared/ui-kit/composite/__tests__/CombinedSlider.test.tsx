import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { CombinedSlider } from '../CombinedSlider';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CombinedSlider', () => {
  it('renders a slider with a thumb', () => {
    mount(() => <CombinedSlider defaultValue={[50]} maxValue={100} />);

    expect(document.querySelector('[role="slider"]')).not.toBeNull();
  });

  it('renders the track and fill', () => {
    mount(() => <CombinedSlider defaultValue={[50]} maxValue={100} />);

    expect(document.querySelectorAll('[role="slider"]')).toHaveLength(1);
    expect(document.body.querySelectorAll('div').length).toBeGreaterThan(1);
  });

  it('renders the value label when withLabel is set', () => {
    mount(() => (
      <CombinedSlider defaultValue={[42]} maxValue={100} withLabel />
    ));

    expect(document.body.textContent).toContain('42');
  });

  it('renders no value label by default', () => {
    mount(() => <CombinedSlider defaultValue={[42]} maxValue={100} />);

    expect(document.body.textContent).not.toContain('42');
  });

  it('forwards the class to the root', () => {
    mount(() => (
      <CombinedSlider class='custom' defaultValue={[10]} maxValue={100} />
    ));

    const root = document.querySelector<HTMLElement>('.custom');

    expect(root).not.toBeNull();
    expect(root!.classList.contains('flex')).toBe(true);
  });

  it('marks the slider disabled', () => {
    mount(() => <CombinedSlider defaultValue={[10]} disabled maxValue={100} />);

    expect(document.querySelector('input')!.disabled).toBe(true);
  });
});
