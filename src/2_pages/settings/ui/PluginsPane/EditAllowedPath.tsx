import {
  type Component,
  type ComponentProps,
  createEffect,
  createSignal,
  splitProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { Button, CombinedTextField } from '@/shared/ui';

export type EditAllowedPathProps = {
  onCancel?: () => void;
  onOk?: (value: [string, string]) => void;
  value?: [string, string];
} & ComponentProps<'div'>;

export const EditAllowedPath: Component<EditAllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onOk',
    'onCancel',
    'class',
  ]);

  const [src, setSrc] = createSignal('');
  const [dest, setDest] = createSignal('');

  const handleSubmit = () => {
    local.onOk?.([src(), dest()]);
  };

  createEffect(() => {
    setSrc(local.value?.[0] ?? '');
    setDest(local.value?.[1] ?? '');
  });

  return (
    <div
      class={cn('flex gap-2 h-full w-full items-center', local.class)}
      {...others}
    >
      <CombinedTextField
        class='w-full'
        inputProps={{
          class: 'h-max py-1 px-1',
          type: 'text',
        }}
        onChange={setSrc}
        value={src()}
      />
      <CombinedTextField
        class='w-full'
        inputProps={{
          class: 'h-max py-1 px-1',
          type: 'text',
        }}
        onChange={setDest}
        value={dest()}
      />
      <Button class='h-full' onClick={handleSubmit} size='sm'>
        Ok
      </Button>
      <Button
        class='h-full'
        onClick={local.onCancel}
        size='sm'
        variant='secondary'
      >
        Cancel
      </Button>
    </div>
  );
};
