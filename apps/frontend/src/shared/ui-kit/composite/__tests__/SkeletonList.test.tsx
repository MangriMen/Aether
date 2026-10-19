import type { JSX } from 'solid-js';

import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { SkeletonList } from '../SkeletonList';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('SkeletonList', () => {
  it('renders one skeleton per item', () => {
    mount(() => <SkeletonList itemsCount={3} />);

    expect(document.body.children).toHaveLength(3);
  });

  it('renders nothing for a zero count', () => {
    mount(() => <SkeletonList itemsCount={0} />);

    expect(document.body.children).toHaveLength(0);
  });

  it('follows a changing itemsCount', () => {
    const [count, setCount] = createSignal(2);

    mount(() => <SkeletonList itemsCount={count()} />);

    expect(document.body.children).toHaveLength(2);

    setCount(4);

    expect(document.body.children).toHaveLength(4);
  });

  it('forwards the class to every skeleton', () => {
    mount(() => <SkeletonList class='my-skeleton' itemsCount={2} />);

    expect(document.querySelectorAll('.my-skeleton')).toHaveLength(2);
  });

  it('forwards arbitrary attributes to every skeleton', () => {
    mount(() => <SkeletonList data-testid='skeleton' itemsCount={2} />);

    expect(document.querySelectorAll('[data-testid="skeleton"]')).toHaveLength(
      2,
    );
  });
});
