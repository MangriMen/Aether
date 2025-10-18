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

export type EditAllowedPathProps = ComponentProps<'div'> &
  EditAllowedItemProps<[string, string], [string, string]>;

export const EditAllowedPath: Component<EditAllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'onSave',
    'onCancel',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [src, setSrc] = createSignal('');
  const [dest, setDest] = createSignal('');

  const handleSubmit = () => {
    local.onSave?.([src(), dest()]);
  };

  createEffect(() => {
    setSrc(local.value?.[0] ?? '');
    setDest(local.value?.[1] ?? '');
  });

  return (
    <div
      class={cn('flex size-full items-start gap-2', local.class)}
      {...others}
    >
      <CombinedTextField
        class='w-full'
        inputProps={{
          type: 'text',
          class: 'h-max py-1 px-1',
          placeholder: t('pluginSettings.hostPathPlaceholder'),
        }}
        value={src()}
        onChange={setSrc}
        errorMessage={local.error?.[0]}
      />
      <CombinedTextField
        class='w-full'
        inputProps={{
          type: 'text',
          class: 'h-max py-1 px-1',
          placeholder: t('pluginSettings.pluginPathPlaceholder'),
        }}
        value={dest()}
        onChange={setDest}
        errorMessage={local.error?.[1]}
      />
      <Button size='sm' onClick={handleSubmit}>
        {t('common.ok')}
      </Button>
      <Button variant='secondary' size='sm' onClick={local.onCancel}>
        {t('common.cancel')}
      </Button>
    </div>
  );
};
