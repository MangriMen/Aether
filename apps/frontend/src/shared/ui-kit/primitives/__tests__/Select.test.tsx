import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../Select';

type Opt = { id: string };

const makeOptions = (count: number): Opt[] =>
  Array.from({ length: count }, (_, i) => ({ id: `v${i}` }));

const VIEWPORT = 170;

let dispose: (() => void) | undefined;

const originals = {
  getBoundingClientRect: HTMLElement.prototype.getBoundingClientRect,
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

  vi.unstubAllGlobals();
};

const mountVirtualized = (initial: Opt[]) => {
  const [options, setOptions] = createSignal(initial);

  dispose = render(
    () => (
      <Select<Opt>
        virtualized
        open
        options={options()}
        optionValue='id'
        optionTextValue='id'
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
        )}
      >
        <SelectTrigger aria-label='Version'>x</SelectTrigger>
        <SelectContent
          class='h-42.5'
          virtualized
          options={options()}
          optionValue='id'
          itemComponent={(props) => (
            <SelectItem item={props.item} style={props.style}>
              {props.item.textValue}
            </SelectItem>
          )}
        />
      </Select>
    ),
    document.body,
  );

  return setOptions;
};

const listbox = () =>
  document.body.querySelector<HTMLElement>('[role="listbox"]');

const itemCount = () =>
  document.body.querySelectorAll('[role="option"]').length;

const totalSize = () =>
  Number.parseInt(
    listbox()!
      .firstElementChild!.getAttribute('style')
      ?.match(/height:\s*(\d+)px/)?.[1] ?? '',
    10,
  );

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

beforeEach(() => {
  fakeLayout();
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  restoreLayout();
});

describe('virtualized SelectContent', () => {
  it('renders only a small window of a large option list', async () => {
    mountVirtualized(makeOptions(1000));
    await tick();

    expect(listbox()).not.toBeNull();
    expect(totalSize()).toBe(1000 * 32);
    expect(itemCount()).toBeGreaterThan(0);
    expect(itemCount()).toBeLessThan(40);
  });

  it('updates the total size and the window when the options change', async () => {
    const setOptions = mountVirtualized(makeOptions(50));
    await tick();

    const before = listbox();
    expect(totalSize()).toBe(50 * 32);

    setOptions(makeOptions(200));
    await tick();

    expect(listbox()).toBe(before);
    expect(totalSize()).toBe(200 * 32);
    expect(itemCount()).toBeLessThan(40);

    setOptions(makeOptions(10));
    await tick();

    expect(listbox()).toBe(before);
    expect(totalSize()).toBe(10 * 32);
  });

  // The only test here that fails when the virtualizer is recreated on option changes.
  it('keeps the scroll position when the options change', async () => {
    const setOptions = mountVirtualized(makeOptions(500));
    await tick();

    const element = listbox()!;
    element.scrollTop = 3200;
    element.dispatchEvent(new Event('scroll'));
    await tick();

    const firstIndex = () =>
      Number.parseInt(
        document.body.querySelector('[role="option"]')!.textContent!.slice(1),
        10,
      );

    expect(firstIndex()).toBeGreaterThan(50);

    setOptions(makeOptions(600));
    await tick();

    expect(firstIndex()).toBeGreaterThan(50);
  });
});

describe('SelectContent', () => {
  it('marks the open list as a top layer so enclosing focus traps ignore it', () => {
    dispose = render(
      () => (
        <Select
          open
          options={['Apple', 'Banana']}
          itemComponent={(props) => (
            <SelectItem item={props.item}>{props.item.rawValue}</SelectItem>
          )}
        >
          <SelectTrigger aria-label='Fruit'>
            <SelectValue<string>>
              {(state) => state.selectedOption()}
            </SelectValue>
          </SelectTrigger>
          <SelectContent />
        </Select>
      ),
      document.body,
    );

    const listbox = document.querySelector('[role="listbox"]');

    expect(listbox).not.toBeNull();
    expect(listbox!.closest('[data-kb-top-layer]')).not.toBeNull();
  });
});
