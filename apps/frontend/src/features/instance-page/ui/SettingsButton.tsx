import type { PolymorphicProps } from '@kobalte/core';

import { useNavigate } from '@solidjs/router';
import IconMdiSettings from '~icons/mdi/cog';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { Instance } from '@/features/instance';

import { INSTANCE_ROUTES } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import {
  CombinedTooltip,
  IconButton,
  type IconButtonProps,
} from '@/shared/ui-kit';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    instanceId: Instance['id'];
  };

export const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['instanceId']);

  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () =>
    navigate(INSTANCE_ROUTES.INSTANCE_SETTINGS(local.instanceId));

  return (
    <CombinedTooltip
      label={t('instance.settings')}
      as={IconButton}
      size='lg'
      variant='secondary'
      icon={IconMdiSettings}
      onClick={handleClick}
      {...others}
    />
  );
};
