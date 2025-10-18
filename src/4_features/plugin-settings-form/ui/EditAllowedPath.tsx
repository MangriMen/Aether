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

export type EditAllowedPathProps = ComponentProps<'div'> & {
  value?: [string, string];
  onOk?: (value: [string, string]) => void;
  onCancel?: () => void;
};

export const EditAllowedPath: Component<EditAllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onOk',
    'onCancel',
    'class',
  ]);

  const [{ t }] = useTranslation();

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
          type: 'text',
          class: 'h-max py-1 px-1',
          placeholder: t('pluginSettings.hostPathPlaceholder'),
        }}
        value={src()}
        onChange={setSrc}
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
  );
};
