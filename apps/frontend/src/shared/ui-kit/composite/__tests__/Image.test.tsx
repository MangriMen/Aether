import type { JSX } from 'solid-js';

import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';

import { Image } from '../Image';

beforeAll(() => {
  globalThis.ResizeObserver ??= class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
});

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('Image', () => {
  it('renders an img with the given src and alt', () => {
    mount(() => <Image alt='logo' src='/logo.png' />);

    const img = document.querySelector('img');

    expect(img).not.toBeNull();
    expect(img!.getAttribute('src')).toBe('/logo.png');
    expect(img!.getAttribute('alt')).toBe('logo');
  });

  it('defaults alt to an empty string', () => {
    mount(() => <Image src='/logo.png' />);

    expect(document.querySelector('img')!.getAttribute('alt')).toBe('');
  });

  it('renders the fallback icon when src is missing', () => {
    mount(() => <Image />);

    expect(document.querySelector('img')).toBeNull();
    expect(document.querySelector('svg')).not.toBeNull();
  });

  it('replaces the img with the fallback icon after an error', () => {
    mount(() => <Image src='/broken.png' />);

    document.querySelector('img')!.dispatchEvent(new Event('error'));

    expect(document.querySelector('img')).toBeNull();
    expect(document.querySelector('svg')).not.toBeNull();
  });

  it('retries when src changes after a failure', () => {
    const [src, setSrc] = createSignal('/broken.png');

    mount(() => <Image src={src()} />);

    document.querySelector('img')!.dispatchEvent(new Event('error'));

    expect(document.querySelector('img')).toBeNull();

    setSrc('/fixed.png');

    expect(document.querySelector('img')!.getAttribute('src')).toBe(
      '/fixed.png',
    );
  });

  it('merges a custom class on the wrapper and imgClass on the img', () => {
    mount(() => <Image class='wrapper' imgClass='picture' src='/a.png' />);

    const wrapper = document.querySelector<HTMLElement>('.wrapper');

    expect(wrapper).not.toBeNull();
    expect(wrapper!.classList.contains('rounded-md')).toBe(true);
    expect(document.querySelector('img')!.classList.contains('picture')).toBe(
      true,
    );
  });

  it('forwards arbitrary div attributes to the wrapper', () => {
    mount(() => <Image data-testid='image' src='/a.png' />);

    expect(document.querySelector('[data-testid="image"]')).not.toBeNull();
  });
});
