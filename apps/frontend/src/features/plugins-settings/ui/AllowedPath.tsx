import IconMdiArrowRightBold from '~icons/mdi/arrow-right-bold';
import IconMdiMonitor from '~icons/mdi/monitor';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Badge } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { AllowedItemProps } from '../model/allowedItem';
import type {
  AllowedPathErrors,
  AllowedPathValue,
} from '../model/pluginSettingsValidation';

import { AllowedItemLeadingItems } from './AllowedItemLeadingItems';
import { AllowedPathComponent } from './AllowedPathComponent';

export type AllowedPathProps = ComponentProps<'div'> &
  AllowedItemProps<AllowedPathValue, AllowedPathErrors>;

export const AllowedPath: Component<AllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'leadingItems',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const accessLabel = () =>
    local.value?.readOnly === false
      ? t('pluginSettings.readWrite')
      : t('pluginSettings.readOnly');

  return (
    <div
      class={cn(
        `
          group min-h-7.5 gap-2 rounded-sm px-2 py-1
          hover:bg-secondary/hover
          flex items-start justify-between select-none
        `,
        local.class,
      )}
      {...others}
    >
      <div class='grid w-full min-w-0 grid-cols-[1fr_auto_1fr] items-start'>
        <div class='flex h-full min-w-0 flex-col'>
          <AllowedPathComponent
            class='grow'
            label={t('pluginSettings.host')}
            icon={IconMdiMonitor}
            value={local.value?.hostPath}
          />
          <span class='text-destructive'>{local.error?.hostPath}</span>
        </div>
        <IconMdiArrowRightBold class='mx-2 my-auto shrink-0' />
        <div class='gap-2 flex h-full min-w-0 grow items-center'>
          <AllowedPathComponent
            class='grow'
            label={t('pluginSettings.plugin')}
            icon={IconMdiPuzzle}
            value={local.value?.pluginPath}
          />
          <Badge variant='outline' class='shrink-0'>
            {accessLabel()}
          </Badge>
          <span class='text-destructive'>{local.error?.pluginPath}</span>
        </div>
      </div>
      <AllowedItemLeadingItems items={local.leadingItems} />
    </div>
  );
};
