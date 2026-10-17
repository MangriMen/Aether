import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';

import type { SettingsEntryProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import { Button } from '@/shared/ui-kit';

import { APP_ROUTES } from '../../appRoutes';

export type GoToPlaygroundEntryProps = SettingsEntryProps;

export const GoToPlaygroundEntry: Component<GoToPlaygroundEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const navigate = useNavigate();

  const handleClick = () => {
    navigate(APP_ROUTES.PLAYGROUND);
  };

  return (
    <SettingsEntry
      title={t('settings.playground')}
      description={t('settings.playgroundDescription')}
      {...props}
    >
      <Button variant='default' onClick={handleClick}>
        {t('settings.playground')}
      </Button>
    </SettingsEntry>
  );
};
