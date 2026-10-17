import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedSelect, CombinedTooltip, IconButton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { PluginSourceType } from '../model/pluginProvider';

type ProviderOption = {
  value: PluginSourceType;
  label: string;
};

const PROVIDER_OPTIONS: ProviderOption[] = [
  { value: 'git_hub', label: 'GitHub' },
];

export type PluginsPaneAddTitleProps = ComponentProps<'div'> & {
  provider?: PluginSourceType | null;
  onProviderChange: (provider: PluginSourceType) => void;
  onBackClick: () => void;
};

export const PluginsPaneAddTitle: Component<PluginsPaneAddTitleProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'provider',
    'onProviderChange',
    'onBackClick',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const handleChangeProvider = (option: ProviderOption | null) => {
    if (option === null) {
      return;
    }

    local.onProviderChange(option.value);
  };

  const currentOption = createMemo(
    () =>
      PROVIDER_OPTIONS.find((o) => o.value === local.provider) ??
      PROVIDER_OPTIONS[0],
  );

  return (
    <div
      class={cn('pt-0.5 flex items-center justify-between', local.class)}
      {...others}
    >
      <div class='gap-2 flex'>
        <CombinedTooltip
          label={t('plugins.back')}
          as={IconButton}
          variant='ghost'
          icon={IconMdiArrowLeft}
          onClick={local.onBackClick}
          {...props}
        />
        <h2>{t('plugins.addTitle')}</h2>
      </div>
      <div class='gap-2 flex'>
        <CombinedSelect
          optionValue='value'
          optionTextValue='label'
          value={currentOption()}
          options={PROVIDER_OPTIONS}
          onChange={handleChangeProvider}
        />
      </div>
    </div>
  );
};
