import { type Component } from 'solid-js';

import type { CollapsibleSettingsEntryProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CollapsibleSettingsEntry, Separator } from '@/shared/ui';

import { BackgroundOpacityEntry } from './BackgroundOpacityEntry';
import { WindowEffectEntry } from './WindowEffectEntry';
import { WindowTransparencySwitch } from './WindowTransparencyToggle';

export type WindowTransparencyEntryProps = CollapsibleSettingsEntryProps;

export const WindowTransparencyEntry: Component<
  WindowTransparencyEntryProps
> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <CollapsibleSettingsEntry
      title={t('settings.toggleWindowTransparency')}
      description={t('settings.toggleWindowTransparencyDescription')}
      tabIndex={-1}
      collapsibleContent={
        <>
          <Separator />
          <WindowEffectEntry class='px-4' />
          <Separator />
          <BackgroundOpacityEntry class='px-4' />
        </>
      }
      {...props}
    >
      <WindowTransparencySwitch
        onClick={(e) => e.stopPropagation()}
        onKeyPress={(e) => e.stopPropagation()}
      />
    </CollapsibleSettingsEntry>
  );
};
