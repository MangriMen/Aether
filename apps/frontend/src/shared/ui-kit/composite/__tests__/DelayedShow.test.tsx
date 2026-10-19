import type { JSX } from 'solid-js';

import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { DelayedShow } from '../DelayedShow';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
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

describe('DelayedShow', () => {
  it('renders children immediately when when is truthy', () => {
    mount(() => (
      <DelayedShow when>
        <span>content</span>
      </DelayedShow>
    ));

    expect(document.querySelector('span')!.textContent).toBe('content');
  });

  it('renders the fallback immediately when when is falsy', () => {
    mount(() => (
      <DelayedShow fallback={<span>fallback</span>} when={false}>
        <span>content</span>
      </DelayedShow>
    ));

    expect(document.querySelector('span')!.textContent).toBe('fallback');
  });

  it('keeps children during the delay after when turns falsy', () => {
    const [when, setWhen] = createSignal(true);

    mount(() => (
      <DelayedShow delay={100} fallback={<span>fallback</span>} when={when()}>
        <span>content</span>
      </DelayedShow>
    ));

    setWhen(false);

    expect(document.querySelector('span')!.textContent).toBe('content');

    vi.advanceTimersByTime(99);

    expect(document.querySelector('span')!.textContent).toBe('content');
  });

  it('shows the fallback after the delay elapses', () => {
    const [when, setWhen] = createSignal(true);

    mount(() => (
      <DelayedShow delay={100} fallback={<span>fallback</span>} when={when()}>
        <span>content</span>
      </DelayedShow>
    ));

    setWhen(false);
    vi.advanceTimersByTime(100);

    expect(document.querySelector('span')!.textContent).toBe('fallback');
  });

  it('cancels the pending fallback when when turns truthy again', () => {
    const [when, setWhen] = createSignal(true);

    mount(() => (
      <DelayedShow delay={100} fallback={<span>fallback</span>} when={when()}>
        <span>content</span>
      </DelayedShow>
    ));

    setWhen(false);
    vi.advanceTimersByTime(50);
    setWhen(true);
    vi.advanceTimersByTime(100);

    expect(document.querySelector('span')!.textContent).toBe('content');
  });

  it('uses a 200ms default delay', () => {
    const [when, setWhen] = createSignal(true);

    mount(() => (
      <DelayedShow fallback={<span>fallback</span>} when={when()}>
        <span>content</span>
      </DelayedShow>
    ));

    setWhen(false);
    vi.advanceTimersByTime(199);

    expect(document.querySelector('span')!.textContent).toBe('content');

    vi.advanceTimersByTime(1);

    expect(document.querySelector('span')!.textContent).toBe('fallback');
  });

  it('passes the value to a render function child', () => {
    mount(() => (
      <DelayedShow when='value'>{(item) => <span>{item()}</span>}</DelayedShow>
    ));

    expect(document.querySelector('span')!.textContent).toBe('value');
  });
});
