import type { PolymorphicProps } from '@kobalte/core';

import IconMdiFolder from '~icons/mdi/folder';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui-kit';

import { useRevealInExplorer } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type OpenFolderButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    instancePath?: string;
  };

export const OpenFolderButton: Component<OpenFolderButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['instancePath']);

  const [{ t }] = useTranslation();

  const { mutateAsync: revealInExplorer } = useRevealInExplorer();

  const handleClick = () => {
    if (!local.instancePath) {
      return;
    }

    revealInExplorer({ path: local.instancePath });
  };

  return (
    <CombinedTooltip
      label={t('instance.openFolder')}
      as={IconButton}
      size='lg'
      variant='secondary'
      icon={IconMdiFolder}
      onClick={handleClick}
      disabled={!local.instancePath}
      {...others}
    />
  );
};
