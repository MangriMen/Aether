import type { Accessor, JSX } from 'solid-js';

import {
  createSignal,
  mergeProps,
  onCleanup,
  onMount,
  Show,
  splitProps,
} from 'solid-js';

type DelayedShowProps<
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
> = {
  // Time in ms to wait before showing fallback. Nothing is rendered during the delay. Default: 200.
  delay?: number;
} & ShowProps<T, TRenderFunction>;
type RequiredParameter<T> = T extends () => unknown ? never : T;

type ShowProps<
  T,
  TRenderFunction extends (item: Accessor<NonNullable<T>>) => JSX.Element,
> = {
  children: JSX.Element | RequiredParameter<TRenderFunction>;
  fallback?: JSX.Element;
  keyed?: false;
  when: false | null | T | undefined;
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
