import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { createSignal, splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
  SettingsEntry,
} from '@/shared/ui';

import { BackgroundOpacityEntry } from './BackgroundOpacityEntry';
import { WindowEffectEntry } from './WindowEffectEntry';
import { WindowTransparencySwitch } from './WindowTransparencyToggle';

export type WindowTransparencyEntryProps = {
  class?: string;
};

export const WindowTransparencyEntry: Component<
  WindowTransparencyEntryProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [isOpened, setIsOpened] = createSignal(false);

  const [{ t }] = useTranslation();

  return (
    <Collapsible
      class={cn(
        'flex flex-col rounded-lg bg-card/card hover:bg-card/hover border p-2',
        local.class,
      )}
      open={isOpened()}
      onOpenChange={setIsOpened}
      {...others}
    >
      <CollapsibleTrigger
        as={SettingsEntry}
        title={t('settings.toggleWindowTransparency')}
        description={t('settings.toggleWindowTransparencyDescription')}
        tabIndex={-1}
      >
        <div class='flex items-center gap-1'>
          <WindowTransparencySwitch />
          <IconButton
            variant={null}
            icon={() => (
              <IconMdiChevronDown
                class={cn('rotate-0 transition-transform', {
                  '-rotate-180': isOpened(),
                })}
              />
            )}
          />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent class='ml-2 mt-2 flex flex-col gap-4 p-0.5'>
        <WindowEffectEntry />
        <BackgroundOpacityEntry />
      </CollapsibleContent>
    </Collapsible>
  );
};
