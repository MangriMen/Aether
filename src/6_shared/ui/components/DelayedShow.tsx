import type { Accessor, JSX } from 'solid-js';

import {
  createSignal,
  mergeProps,
  onCleanup,
  onMount,
  Show,
  splitProps,
} from 'solid-js';

type RequiredParameter<T> = T extends () => unknown ? never : T;
type ShowProps<
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
> = {
  when: T | undefined | null | false;
  keyed?: false;
  fallback?: JSX.Element;
  children: JSX.Element | RequiredParameter<TRenderFunction>;
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
  const [local, others] = splitProps(props, ['fallback', 'delay']);
  const mergedLocal = mergeProps({ delay: 200 }, local);

  const [delayPassed, setDelayPassed] = createSignal(false);

  let timeout: ReturnType<typeof setTimeout>;
  onMount(() => {
    timeout = setTimeout(() => setDelayPassed(true), mergedLocal.delay);
  });
  onCleanup(() => {
    clearTimeout(timeout);
  });

  return (
    <Show fallback={delayPassed() ? mergedLocal.fallback : null} {...others} />
  );
};
