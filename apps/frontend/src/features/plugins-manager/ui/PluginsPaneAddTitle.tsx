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

import type { PluginProvider, PluginSourceType } from '../model/pluginProvider';

type ProviderOption = {
  value: PluginSourceType;
  label: string;
};

export type PluginsPaneAddTitleProps = ComponentProps<'div'> & {
  /** What the backend can install from. */
  providers: readonly PluginProvider[];
  /** The one shown as chosen; none while the list is not there. */
  provider: PluginSourceType | null;
  onProviderChange: (provider: PluginSourceType) => void;
  onBackClick: () => void;
};

export const PluginsPaneAddTitle: Component<PluginsPaneAddTitleProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'providers',
    'provider',
    'onProviderChange',
    'onBackClick',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const options = createMemo<ProviderOption[]>(() =>
    local.providers.map((provider) => ({
      value: provider.source_type,
      // The provider's own name: a proper name, not translated.
      label: provider.name,
    })),
  );

  const currentOption = createMemo(
    () => options().find((o) => o.value === local.provider) ?? null,
  );

  const handleChangeProvider = (option: ProviderOption | null) => {
    if (option === null) {
      return;
    }

    local.onProviderChange(option.value);
  };

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
          aria-label={t('plugins.providers.label')}
          optionValue='value'
          optionTextValue='label'
          value={currentOption()}
          options={options()}
          disabled={options().length === 0}
          onChange={handleChangeProvider}
        />
      </div>
    </div>
  );
};
