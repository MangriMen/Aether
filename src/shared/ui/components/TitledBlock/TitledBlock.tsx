import { Component, splitProps } from 'solid-js';

import { TitledBlockProps } from './types';

export const TitledBlock: Component<TitledBlockProps> = (props) => {
  const [local, others] = splitProps(props, ['title', 'class', 'children']);

  return (
    <div class='flex flex-col gap-4' {...others}>
      <span class='text-xl font-bold'>{local.title}</span>
      {local.children}
    </div>
  );
};
