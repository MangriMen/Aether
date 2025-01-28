import MdiFolderIcon from '@iconify/icons-mdi/folder';
import type { PolymorphicProps } from '@kobalte/core';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import type { Instance } from '@/entities/instance';
import { useInstanceActions } from '@/entities/instance';

import { useTranslate } from '@/app/model';

export type InstanceOpenFolderButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    instance: Instance;
  };

const InstanceOpenFolderButton: Component<InstanceOpenFolderButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instance']);

  const [{ t }] = useTranslate();

  const { openFolder: handleOpenFolder } = useInstanceActions();

  const handleClick = () => handleOpenFolder(local.instance);

  return (
    <CombinedTooltip
      label={t('instance.openFolder')}
      as={IconButton}
      class='aspect-square p-2'
      variant='secondary'
      icon={MdiFolderIcon}
      onClick={handleClick}
      {...others}
    />
  );
};

export default InstanceOpenFolderButton;
