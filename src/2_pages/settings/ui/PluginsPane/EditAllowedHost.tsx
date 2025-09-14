import {
  type Component,
  type ComponentProps,
  createEffect,
  createSignal,
  splitProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { Button, CombinedTextField } from '@/shared/ui';

export type EditAllowedHostProps = {
  onCancel?: () => void;
  onOk?: (value: string) => void;
  value?: string;
} & ComponentProps<'div'>;

export const EditAllowedHost: Component<EditAllowedHostProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onOk',
    'onCancel',
    'class',
  ]);

  let inputRef: HTMLInputElement | undefined;

  const [value, setValue] = createSignal('');

  const handleSubmit = () => {
    local.onOk?.(value());
  };

  createEffect(() => {
    setValue(local.value ?? '');
    inputRef?.focus();
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
          onKeyDown: (e) => e.key === 'Enter' && handleSubmit(),
          ref: (el) => {
            inputRef = el;
          },
          type: 'text',
        }}
        onChange={setValue}
        value={value()}
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
