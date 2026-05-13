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
        'inline-flex items-baseline gap-2 font-medium text-lg',
        local.class,
      )}
      {...others}
    >
      {local.label}
      <span
        class={cn(
          'text-sm italic text-muted-foreground opacity-0 invisible font-normal',
          {
            'transition-opacity': isReady(),
            'opacity-100 visible': local.isInheritance,
          },
        )}
      >
        {local.inheritanceLabel}
      </span>
    </span>
  );
};
