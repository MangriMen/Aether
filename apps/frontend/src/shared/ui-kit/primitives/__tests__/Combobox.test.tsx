import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  Combobox,
  ComboboxContent,
  ComboboxControl,
  ComboboxInput,
  ComboboxItem,
  ComboboxTrigger,
} from '../Combobox';

const makeOptions = (count: number): string[] =>
  Array.from({ length: count }, (_, i) => `v${i}`);

const VIEWPORT = 170;

let dispose: (() => void) | undefined;

const originals = {
  getBoundingClientRect: HTMLElement.prototype.getBoundingClientRect,
  scrollTo: Element.prototype.scrollTo,
  offsetHeight: Object.getOwnPropertyDescriptor(
    HTMLElement.prototype,
    'offsetHeight',
  ),
  offsetWidth: Object.getOwnPropertyDescriptor(
    HTMLElement.prototype,
    'offsetWidth',
  ),
  clientHeight: Object.getOwnPropertyDescriptor(
    Element.prototype,
    'clientHeight',
  ),
};

const hadResizeObserver = 'ResizeObserver' in globalThis;

// jsdom has no layout: the list gets a fixed size, so the virtualizer has a viewport to fill.
const fakeLayout = () => {
  HTMLElement.prototype.getBoundingClientRect = () =>
    ({
      width: 200,
      height: VIEWPORT,
      top: 0,
      left: 0,
      right: 200,
      bottom: VIEWPORT,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    }) as DOMRect;
  Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
    configurable: true,
    get: () => VIEWPORT,
  });
  Object.defineProperty(HTMLElement.prototype, 'offsetWidth', {
    configurable: true,
    get: () => 200,
  });
  Object.defineProperty(Element.prototype, 'clientHeight', {
    configurable: true,
    get: () => VIEWPORT,
  });

  // The virtualizer scrolls the list with `scrollTo`, which jsdom does not have.
  Element.prototype.scrollTo = function scrollTo(
    this: Element,
    options?: ScrollToOptions | number,
  ) {
    this.scrollTop = typeof options === 'object' ? (options.top ?? 0) : 0;
    this.dispatchEvent(new Event('scroll'));
  } as Element['scrollTo'];

  if (!hadResizeObserver) {
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe() {}
        unobserve() {}
        disconnect() {}
      },
    );
  }
};

const restoreLayout = () => {
  HTMLElement.prototype.getBoundingClientRect = originals.getBoundingClientRect;

  for (const [proto, name, desc] of [
    [HTMLElement.prototype, 'offsetHeight', originals.offsetHeight],
    [HTMLElement.prototype, 'offsetWidth', originals.offsetWidth],
    [Element.prototype, 'clientHeight', originals.clientHeight],
  ] as const) {
    if (desc) {
      Object.defineProperty(proto, name, desc);
    } else {
      delete (proto as unknown as Record<string, unknown>)[name];
    }
  }

  Element.prototype.scrollTo = originals.scrollTo;
  vi.unstubAllGlobals();
};

const mount = (initial: string[], virtualized: boolean) => {
  const [options, setOptions] = createSignal(initial);

  dispose = render(
    () => (
      <Combobox<string>
        virtualized={virtualized}
        open
        options={options()}
        itemComponent={(props) => (
          <ComboboxItem item={props.item}>{props.item.rawValue}</ComboboxItem>
        )}
      >
        <ComboboxControl aria-label='Version'>
          <ComboboxInput />
          <ComboboxTrigger />
        </ComboboxControl>
        <ComboboxContent<string>
          class='h-42.5'
          virtualized={virtualized}
          itemComponent={(props) => (
            <ComboboxItem item={props.item} style={props.style}>
              {props.item.rawValue}
            </ComboboxItem>
          )}
        />
      </Combobox>
    ),
    document.body,
  );

  return setOptions;
};

const input = () => document.body.querySelector<HTMLInputElement>('input')!;

const listbox = () =>
  document.body.querySelector<HTMLElement>('[role="listbox"]');

const itemTexts = () =>
  [...document.body.querySelectorAll('[role="option"]')].map(
    (item) => item.textContent,
  );

const totalSize = () =>
  Number.parseInt(
    listbox()!
      .firstElementChild!.getAttribute('style')
      ?.match(/height:\s*(\d+)px/)?.[1] ?? '',
    10,
  );

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

const type = async (text: string) => {
  input().value = text;
  input().dispatchEvent(new InputEvent('input', { bubbles: true }));
  await tick();
};

beforeEach(() => {
  fakeLayout();
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  restoreLayout();
});

describe('Combobox', () => {
  it('draws every option when it is not virtualized', async () => {
    mount(makeOptions(60), false);
    await tick();

    expect(itemTexts()).toHaveLength(60);
  });

  it('marks the open list as a top layer so enclosing focus traps ignore it', async () => {
    mount(makeOptions(3), false);
    await tick();

    expect(listbox()!.closest('[data-kb-top-layer]')).not.toBeNull();
  });
});

describe('virtualized ComboboxContent', () => {
  it('draws only a small window of a large option list', async () => {
    mount(makeOptions(1000), true);
    await tick();

    expect(totalSize()).toBe(1000 * 32);
    expect(itemTexts().length).toBeGreaterThan(0);
    expect(itemTexts().length).toBeLessThan(40);
    expect(itemTexts()[0]).toBe('v0');
  });

  it('follows the options when they change', async () => {
    const setOptions = mount(makeOptions(50), true);
    await tick();

    const before = listbox();
    expect(totalSize()).toBe(50 * 32);

    setOptions(makeOptions(200));
    await tick();

    expect(listbox()).toBe(before);
    expect(totalSize()).toBe(200 * 32);
    expect(itemTexts().length).toBeLessThan(40);

    setOptions(makeOptions(10));
    await tick();

    expect(totalSize()).toBe(10 * 32);
  });

  it('draws only the options the typed text keeps', async () => {
    mount(makeOptions(1000), true);
    await tick();

    await type('v99');

    // v99, v990..v999
    expect(totalSize()).toBe(11 * 32);
    expect(itemTexts().every((text) => text!.startsWith('v99'))).toBe(true);

    await type('');

    expect(totalSize()).toBe(1000 * 32);
  });
});
