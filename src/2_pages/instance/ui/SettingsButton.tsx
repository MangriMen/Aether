import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import MdiSettingsIcon from '@iconify/icons-mdi/settings';
import { useNavigate } from '@solidjs/router';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton, type IconButtonProps } from '@/shared/ui';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate('settings');

  return (
    <CombinedTooltip
      label={t('instance.settings')}
      as={IconButton}
      class='aspect-square p-2'
      variant='secondary'
      icon={MdiSettingsIcon}
      onClick={handleClick}
      {...props}
    />
  );
};
