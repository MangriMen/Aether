import MdiSettingsIcon from '@iconify/icons-mdi/settings';
import type { PolymorphicProps } from '@kobalte/core';
import { useNavigate } from '@solidjs/router';
import type { Component, ValidComponent } from 'solid-js';

import { CombinedTooltip, IconButton, type IconButtonProps } from '@/shared/ui';

import { useTranslate } from '@/shared/model';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslate();

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
