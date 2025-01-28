import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';


const Label: Component<ComponentProps<'label'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <label
      class={cn(
        'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 data-[disabled]:text-muted-foreground',
        local.class,
      )}
      {...others}
    />
  );
};

export { Label };
