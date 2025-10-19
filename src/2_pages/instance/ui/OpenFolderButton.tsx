import type { PolymorphicProps } from '@kobalte/core';

import IconMdiFolder from '~icons/mdi/folder';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useRevealInExplorer } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

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
      class='aspect-square p-2'
      variant='secondary'
      icon={IconMdiFolder}
      onClick={handleClick}
      disabled={!local.instancePath}
      {...others}
    />
  );
};
