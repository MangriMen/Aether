import MdiFolderIcon from '@iconify/icons-mdi/folder';
import type { PolymorphicProps } from '@kobalte/core';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import { useTranslate } from '@/shared/model';
import { useRevealInExplorer } from '@/entities/instances';

export type OpenFolderButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    instancePath?: string;
  };

export const OpenFolderButton: Component<OpenFolderButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['instancePath']);

  const [{ t }] = useTranslate();

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
      icon={MdiFolderIcon}
      onClick={handleClick}
      disabled={!local.instancePath}
      {...others}
    />
  );
};
