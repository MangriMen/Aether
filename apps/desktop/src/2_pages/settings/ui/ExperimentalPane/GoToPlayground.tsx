import type { Component, ComponentProps } from 'solid-js';

import { useNavigate } from '@solidjs/router';

import { ROUTES } from '@/shared/config';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry } from '@/shared/ui';

export type GoToPlaygroundProps = ComponentProps<'div'>;

export const GoToPlayground: Component<GoToPlaygroundProps> = (props) => {
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
