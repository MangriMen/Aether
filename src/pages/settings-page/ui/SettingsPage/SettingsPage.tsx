import type { RouteSectionProps } from '@solidjs/router';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { AppVersion } from '@/entities/settings';

import { SelectTheme } from '@/features/select-color-mode';

// eslint-disable-next-line boundaries/element-types
import { useI18nContext } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';
import { SettingsPane } from '../SettingsPane';

import ChangeLanguageEntry from './ChangeLanguageEntry';
import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';
import UpdateAppEntry from './UpdateAppEntry';

export type SettingsPageProps = ComponentProps<'div'> & RouteSectionProps;

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);
  const [{ t }] = useI18nContext();

  return (
    <div class='flex size-full flex-col gap-4 p-4' {...others}>
      <SettingsPane class='container max-w-screen-lg' title={t('launcher')}>
        <SettingsEntry
          title={t('colorScheme')}
          description={t('colorSchemeDescription')}
        >
          <SelectTheme />
        </SettingsEntry>
        <SelectThemeForColorModeEntry />
        <ChangeLanguageEntry />
      </SettingsPane>
      <SettingsPane class='container max-w-screen-lg' title={t('update')}>
        <UpdateAppEntry />
      </SettingsPane>
      <AppVersion class='mt-auto' />
    </div>
  );
};
