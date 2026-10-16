import type { Component, ComponentProps } from 'solid-js';

import IconMdiWebPlus from '~icons/mdi/web-plus';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type AddPluginIconButtonProps = ComponentProps<'button'>;

export const AddPluginIconButton: Component<AddPluginIconButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      label={t('plugins.addPlugin')}
      as={IconButton}
      variant='secondary'
      icon={IconMdiWebPlus}
      {...props}
    />
  );
};
