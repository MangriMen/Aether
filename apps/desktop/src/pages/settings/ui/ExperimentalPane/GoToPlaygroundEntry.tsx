import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';

import type { SettingsEntryProps } from '@/shared/ui';

import { ROUTES } from '@/shared/config';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry } from '@/shared/ui';

export type GoToPlaygroundEntryProps = SettingsEntryProps;

export const GoToPlaygroundEntry: Component<GoToPlaygroundEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const navigate = useNavigate();

  const handleClick = () => {
    navigate(ROUTES.PLAYGROUND);
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
