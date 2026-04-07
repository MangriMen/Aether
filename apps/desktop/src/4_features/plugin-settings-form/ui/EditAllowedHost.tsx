import {
  createEffect,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTextField } from '@/shared/ui';

import type { EditAllowedItemProps } from '../model';

export type EditAllowedHostProps = ComponentProps<'div'> &
  EditAllowedItemProps<string, string>;

export const EditAllowedHost: Component<EditAllowedHostProps> = (props) => {
  const [local, others] = splitProps(props, [
    'name',
    'value',
    'error',
    'onSave',
    'onCancel',
    'class',
  ]);

  const [{ t }] = useTranslation();

  let inputRef: HTMLInputElement | undefined;

  const [value, setValue] = createSignal('');

  const handleSubmit = () => {
    local.onSave?.(value());
  };

  createEffect(() => {
    setValue(local.value ?? '');
    inputRef?.focus();
  });

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='flex size-full items-center gap-2'>
        <CombinedTextField
          class='w-full'
          name={local.name}
          inputProps={{
            ref: (el) => {
              inputRef = el;
            },
            type: 'text',
            class: 'h-max py-1 px-1',
            onKeyDown: (e) => e.key === 'Enter' && handleSubmit(),
            placeholder: t('pluginSettings.hostUrlPlaceholder'),
          }}
          value={value()}
          onChange={setValue}
          errorMessage={local.error}
        />
        <Button class='h-full' size='sm' onClick={handleSubmit}>
          {t('common.ok')}
        </Button>
        <Button
          class='h-full'
          variant='secondary'
          size='sm'
          onClick={local.onCancel}
        >
          {t('common.cancel')}
        </Button>
      </div>
    </div>
  );
};
