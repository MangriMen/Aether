import type { PolymorphicProps } from '@kobalte/core';

import { useNavigate } from '@solidjs/router';
import IconMdiSettings from '~icons/mdi/cog';
import { splitProps, type Component, type ValidComponent } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { ROUTES } from '@/shared/config';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton, type IconButtonProps } from '@/shared/ui';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    instanceId: Instance['id'];
  };

export const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['instanceId']);

  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () =>
    navigate(ROUTES.INSTANCE_SETTINGS(local.instanceId));

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
