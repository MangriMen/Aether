import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import MdiPlayIcon from '@iconify/icons-mdi/play';

import type { IconButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type InstancePlayButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const InstancePlayButton: Component<InstancePlayButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      as={IconButton}
      icon={MdiPlayIcon}
      label={t('instance.launch')}
      variant='success'
      {...props}
    />
  );
};
