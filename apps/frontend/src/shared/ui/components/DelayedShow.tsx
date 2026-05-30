import type { Accessor, JSX } from 'solid-js';

import {
  createEffect,
  createSignal,
  mergeProps,
  onCleanup,
  Show,
  splitProps,
  untrack,
} from 'solid-js';

type RequiredParameter<T> = T extends () => unknown ? never : T;
type ShowProps<
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
> = {
  when: T | undefined | null | false;
  keyed?: false;
  fallback?: JSX.Element;
  children?: JSX.Element | RequiredParameter<TRenderFunction>;
};

type DelayedShowProps<
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
> = ShowProps<T, TRenderFunction> & {
  // Time in ms to wait before showing fallback. Nothing is rendered during the delay. Default: 200.
  delay?: number;
};

export const DelayedShow = <
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
>(
  props: DelayedShowProps<T, TRenderFunction>,
) => {
  const merged = mergeProps({ delay: 200 }, props);
  const [local, others] = splitProps(merged, [
    'when',
    'fallback',
    'delay',
    'children',
  ]);

  const [delayedWhen, setDelayedWhen] = createSignal<
    T | undefined | null | false
  >(untrack(() => local.when));

  createEffect(() => {
    const currentWhen = local.when;

    if (currentWhen) {
      setDelayedWhen(() => currentWhen);
    } else {
      const timeout = setTimeout(() => {
        setDelayedWhen(() => currentWhen);
      }, merged.delay);
      onCleanup(() => clearTimeout(timeout));
    }
  });

  return (
    <Show when={delayedWhen()} fallback={local.fallback} {...others}>
      {local.children}
    </Show>
  );
};
