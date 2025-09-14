import type { PolymorphicProps } from '@kobalte/core';

import MdiFolderIcon from '@iconify/icons-mdi/folder';
import { type Component, splitProps, type ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useRevealInExplorer } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type OpenFolderButtonProps<T extends ValidComponent = 'button'> = {
  instancePath?: string;
} & PolymorphicProps<T, IconButtonProps<T>>;

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
      as={IconButton}
      class='aspect-square p-2'
      disabled={!local.instancePath}
      icon={MdiFolderIcon}
      label={t('instance.openFolder')}
      onClick={handleClick}
      variant='secondary'
      {...others}
    />
  );
};
