import { type Component } from 'solid-js';

import type { CollapsibleSettingsEntryProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CollapsibleSettingsEntry, Separator } from '@/shared/ui';

import { SelectTheme } from './SelectTheme';
import { SelectThemeForColorModeEntry } from './SelectThemeForColorModeEntry';

export type ThemeEntryProps = CollapsibleSettingsEntryProps;

export const ThemeEntry: Component<ThemeEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <CollapsibleSettingsEntry
      title={t('settings.colorScheme')}
      description={t('settings.colorSchemeDescription')}
      variant='card'
      collapsibleContent={
        <>
          <Separator />
          <SelectThemeForColorModeEntry class='px-4' />
        </>
      }
      {...props}
    >
      <SelectTheme
        onClick={(e) => e.stopPropagation()}
        onKeyPress={(e) => e.stopPropagation()}
      />
    </CollapsibleSettingsEntry>
  );
};
