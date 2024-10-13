import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Label } from '@/shared/ui';

import { SettingsEntryProps } from './types';

export const SettingsEntry: Component<SettingsEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'description',
    'inputId',
    'class',
    'children',
  ]);

  return (
    <div
      class={cn('flex items-center justify-between', local.class)}
      {...others}
    >
      <Label class='flex flex-col' for={local.inputId}>
        <h3 class='text-lg font-bold'>{local.title}</h3>
        <span>{local.description}</span>
      </Label>
      {local.children}
    </div>
  );
};
