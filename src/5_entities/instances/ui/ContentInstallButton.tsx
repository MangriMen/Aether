import type { PolymorphicProps } from '@kobalte/core';

import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import {
  createMemo,
  mergeProps,
  Show,
  splitProps,
  type Component,
} from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type ContentInstallButtonProps = {
  isInstalling: boolean;
  isInstalled: boolean;
  isLoading?: boolean;
};

export const ContentInstallButton: Component<
  PolymorphicProps<'button', ButtonProps<'button'>> & ContentInstallButtonProps
> = (props) => {
  const [{ t }] = useTranslation();

  const [_local, others] = splitProps(props, [
    'isInstalling',
    'isInstalled',
    'isLoading',
    'class',
  ]);

  const local = mergeProps({ isCompatible: true }, _local);

  const installButtonText = createMemo(() => {
    if (local.isInstalling) return t('common.installing');
    if (local.isInstalled) return t('common.installed');
    return t('common.install');
  });

  return (
    <Button
      class='px-3'
      leadingIcon={() => (
        <Show when={local.isInstalled} fallback={<IconMdiDownload />}>
          <IconMdiCheck />
        </Show>
      )}
      loading={local.isInstalling || local.isLoading}
      disabled={local.isInstalled || local.isLoading}
      {...others}
    >
      {installButtonText()}
    </Button>
  );
};
