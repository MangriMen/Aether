import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import {
  createEffect,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedTextField,
  CombinedTooltip,
  IconButton,
} from '@/shared/ui';

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

  const handleBrowse = async () => {
    const file = await open({
      multiple: false,
      directory: true,
    });

    if (file) {
      setSrc(file);
      setDest(file);
    }
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
        leadingIcons={
          <div class='mr-1 flex h-full items-center justify-center'>
            <CombinedTooltip
              label={t('common.browse')}
              as={IconButton}
              variant='ghost'
              class='size-7 bg-secondary-dark/50 enabled:hover:bg-secondary-dark'
              type='button'
              size='sm'
              onClick={handleBrowse}
              icon={IconMdiFileFindOutline}
            />
          </div>
        }
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
