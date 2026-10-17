import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import IconMdiStop from '~icons/mdi/stop';

import type { IconButtonProps } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type InstanceStopButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const InstanceStopButton: Component<InstanceStopButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      label={t('instance.stop')}
      as={IconButton}
      variant='destructive'
      icon={IconMdiStop}
      {...props}
    />
  );
};
