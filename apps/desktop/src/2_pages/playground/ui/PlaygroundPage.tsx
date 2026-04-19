import type { RouteSectionProps } from '@solidjs/router';

import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { ButtonsLab } from './ButtonsLab';
import { CheckboxLab } from './CheckboxLab';
import { SelectLab } from './SelectsLab';

export type PlaygroundPageProps = ComponentProps<'div'> & RouteSectionProps;

export const PlaygroundPage: Component<PlaygroundPageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'class',
    'params',
    'location',
    'data',
  ]);

  return (
    <div
      class={cn(
        'p-page overflow-auto flex size-full flex-col text-new-foreground',
        local.class,
      )}
      {...others}
    >
      <ButtonsLab />
      <SelectLab />
      <CheckboxLab />
    </div>
  );
};
