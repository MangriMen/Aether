import type { PolymorphicProps } from '@kobalte/core';

import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import { createMemo, Show, splitProps, type Component } from 'solid-js';

import type { ButtonProps } from '../../../shared/ui';

import { useTranslation } from '../../../shared/model';
import { Button } from '../../../shared/ui';

export type ContentInstallButtonProps = {
  isInstalling: boolean;
  isInstalled: boolean;
  isLoading?: boolean;
};

export const ContentInstallButton: Component<
  PolymorphicProps<'button', ButtonProps<'button'>> & ContentInstallButtonProps
> = (props) => {
  const [{ t }] = useTranslation();

  const [local, others] = splitProps(props, [
    'isInstalling',
    'isInstalled',
    'isLoading',
    'class',
  ]);

  const installButtonText = createMemo(() => {
    if (local.isInstalling) return t('common.installing');
    if (local.isInstalled) return t('common.installed');
    return t('common.install');
  });

  const isLoading = createMemo(() => local.isInstalling || local.isLoading);
  const isDisabled = createMemo(() => local.isInstalled || local.isLoading);

  return (
    <Button
      class='px-3'
      leadingIcon={() => (
        <Show when={local.isInstalled} fallback={<IconMdiDownload />}>
          <IconMdiCheck />
        </Show>
      )}
      loading={isLoading()}
      disabled={isDisabled()}
      {...others}
    >
      {installButtonText()}
    </Button>
  );
};
