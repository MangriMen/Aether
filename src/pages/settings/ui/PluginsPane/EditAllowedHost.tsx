import { cn } from '@/shared/lib';
import { Button, CombinedTextField } from '@/shared/ui';
import {
  createEffect,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

export type EditAllowedHostProps = ComponentProps<'div'> & {
  value?: string;
  onOk?: (value: string) => void;
  onCancel?: () => void;
};

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
          ref: (el) => {
            inputRef = el;
          },
          type: 'text',
          class: 'h-max py-1 px-1',
          onKeyDown: (e) => e.key === 'Enter' && handleSubmit(),
        }}
        value={value()}
        onChange={setValue}
      />
      <Button class='h-full' size='sm' onClick={handleSubmit}>
        Ok
      </Button>
      <Button
        class='h-full'
        variant='secondary'
        size='sm'
        onClick={local.onCancel}
      >
        Cancel
      </Button>
    </div>
  );
};
