import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { TitledBlockProps } from './types';

export const TitledBlock: Component<TitledBlockProps> = (props) => {
  const [local, others] = splitProps(props, ['title', 'class', 'children']);

  return (
    <div class={cn('flex flex-col gap-4', local.class)} {...others}>
      <span class='text-xl font-bold'>{local.title}</span>
      {local.children}
    </div>
  );
};
