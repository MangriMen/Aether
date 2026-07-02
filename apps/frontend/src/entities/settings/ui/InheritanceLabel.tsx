import {
  createSignal,
  onCleanup,
  onMount,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';

export type InheritanceLabelProps = ComponentProps<'span'> & {
  label?: string;
  inheritanceLabel?: string;
  isInheritance?: boolean;
};

export const InheritanceLabel: Component<InheritanceLabelProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'inheritanceLabel',
    'isInheritance',
    'class',
  ]);

  const [isReady, setIsReady] = createSignal(false);

  onMount(() => {
    const timerId = requestAnimationFrame(() => {
      setIsReady(true);
    });

    onCleanup(() => cancelAnimationFrame(timerId));
  });

  return (
    <span
      class={cn(
        'gap-2 text-lg font-medium inline-flex items-baseline',
        local.class,
      )}
      {...others}
    >
      {local.label}
      <span
        class={cn(
          'text-sm font-normal text-muted-foreground invisible italic opacity-0',
          {
            'transition-opacity': isReady(),
            'visible opacity-100': local.isInheritance,
          },
        )}
      >
        {local.inheritanceLabel}
      </span>
    </span>
  );
};
