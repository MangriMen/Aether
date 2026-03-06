import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import { createMemo, Show, type Component } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type ContentInstallButtonProps = {
  isInstalling: boolean;
  isInstalled: boolean;
  onClick: () => void;
};

export const ContentInstallButton: Component<ContentInstallButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const installButtonText = createMemo(() => {
    if (props.isInstalling) return t('common.installing');
    if (props.isInstalled) return t('common.installed');
    return t('common.install');
  });

  return (
    <Button
      class='px-3'
      leadingIcon={() => (
        <Show when={props.isInstalled} fallback={<IconMdiDownload />}>
          <IconMdiCheck />
        </Show>
      )}
      onClick={props.onClick}
      loading={props.isInstalling}
      disabled={props.isInstalled}
    >
      {installButtonText()}
    </Button>
  );
};
