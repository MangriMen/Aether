import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type TitledBlockProps = ComponentProps<'div'> & {
  title?: string;
};

export const TitledBlock: Component<TitledBlockProps> = (props) => {
  const [local, others] = splitProps(props, ['title', 'class', 'children']);

  return (
    <div class={cn('flex flex-col gap-4', local.class)} {...others}>
      <Show when={local.title}>
        <h2 class='text-2xl font-bold'>{local.title}</h2>
      </Show>
      {local.children}
    </div>
  );
};
