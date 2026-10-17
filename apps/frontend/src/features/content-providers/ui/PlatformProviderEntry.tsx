import { createMemo, type Component } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/settings-layout';
import type { Option } from '@/shared/ui-kit';

import { useSetPlatformProvider } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui-kit';

import type { PlatformProvider } from '../model/groupProvidersByPlatform';

export type PlatformProviderEntryProps = SettingsEntryProps & {
  platform: string;
  providers: PlatformProvider[];
  /** The provider id stored in the settings, if the user chose one. */
  selected?: string;
  /** Replaces saving the choice. */
  onChoose?: (platform: string, providerId: string | null) => void;
};

// Select values cannot be null, and an empty id never names a provider.
const AUTOMATIC = '';

export const PlatformProviderEntry: Component<PlatformProviderEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();
  const setPlatformProvider = useSetPlatformProvider();

  const options = createMemo<Option<string>[]>(() => {
    const available = props.providers.map((provider) => ({
      name: provider.name,
      value: provider.id,
    }));

    // A stored choice whose plugin is gone is shown, not dropped: resolving
    // stays an error until the user picks another provider.
    const selected = props.selected;
    const isUnavailable =
      selected && !available.some((o) => o.value === selected);

    return [
      {
        name: t('settings.contentProviders.automatic') ?? '',
        value: AUTOMATIC,
      },
      ...available,
      ...(isUnavailable
        ? [
            {
              name:
                t('settings.contentProviders.unavailable', { id: selected }) ??
                '',
              value: selected,
            },
          ]
        : []),
    ];
  });

  const current = createMemo(() =>
    options().find((o) => o.value === (props.selected ?? AUTOMATIC)),
  );

  const handleChange = (option: Option<string> | null) => {
    if (!option) {
      return;
    }

    const providerId = option.value === AUTOMATIC ? null : option.value;

    if (props.onChoose) {
      props.onChoose(props.platform, providerId);
      return;
    }

    setPlatformProvider.mutate({ platform: props.platform, providerId });
  };

  return (
    <SettingsEntry
      title={props.platform}
      description={t('settings.contentProviders.entryDescription', {
        platform: props.platform,
      })}
      variant={props.variant}
    >
      <Select
        class='w-64'
        options={options()}
        value={current()}
        onChange={handleChange}
        optionValue='value'
        optionTextValue='name'
        disallowEmptySelection
        disabled={setPlatformProvider.isPending}
        itemComponent={(itemProps) => (
          <SelectItem item={itemProps.item}>
            {itemProps.item.textValue}
          </SelectItem>
        )}
      >
        <SelectTrigger>
          <SelectValue<Option<string>>>
            {(state) => state.selectedOption()?.name}
          </SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    </SettingsEntry>
  );
};
