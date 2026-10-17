import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/utils';

import { AllowedTruncatedText } from './AllowedTruncatedText';

export type AllowedPathComponentProps = ComponentProps<'div'> & {
  value?: string;
  icon?: Component;
  label?: string;
};

export const AllowedPathComponent: Component<AllowedPathComponentProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'icon',
    'label',
    'class',
  ]);

  return (
    <div class={cn('gap-2 flex min-w-0 items-center', local.class)} {...others}>
      <span class='shrink-0'>{local.icon?.({})}</span>
      <AllowedTruncatedText
        class='grow'
        value={local.value}
        tooltipPrefix={local.label}
      />
    </div>
  );
};
