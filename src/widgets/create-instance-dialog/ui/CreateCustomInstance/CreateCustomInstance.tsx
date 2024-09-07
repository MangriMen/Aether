import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { TextField, TextFieldInput, TextFieldLabel } from '@/shared/ui';

import { SelectGameVersion } from '@/features/select-game-version';
import { SelectLoaderChips } from '@/features/select-loader-chips';

import { CreateCustomInstanceProps } from '.';

const loaders = [
  { name: 'Vanilla', value: 'vanilla' },
  { name: 'Forge', value: 'forge' },
  { name: 'Fabric', value: 'fabric' },
  { name: 'Quilt', value: 'quilt' },
];

export const CreateCustomInstance: Component<CreateCustomInstanceProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <div class={cn('flex flex-col gap-4', local.class)} {...others}>
      <TextField class='flex flex-col gap-2'>
        <TextFieldLabel for='name'>Name</TextFieldLabel>
        <TextFieldInput type='text' id='name' />
      </TextField>
      <div class='flex flex-col gap-2'>
        <span class='text-sm font-medium'>Loader</span>
        <SelectLoaderChips loaders={loaders} defaultValue={loaders[0].value} />
      </div>
      <div class='flex flex-col gap-2'>
        <span class='text-sm font-medium'>Game version</span>
        <SelectGameVersion />
      </div>
    </div>
  );
};
