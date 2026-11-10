import { Show, type Component } from 'solid-js';

import type { Option } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import {
  CombinedTooltip,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui-kit';

export type LogSourceSelectProps = {
  options: Option<string>[];
  value: Option<string> | undefined;
  onChange: (value: string) => void;
  class?: string;
};

/** Picks one log; a long name is cut short in the box and shown whole in a tooltip. */
export const LogSourceSelect: Component<LogSourceSelectProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    // The select does not follow a value that changes under it, so it is built anew for each.
    <Show when={props.value?.value ?? 'none'} keyed>
      {(_key) => (
        <Select
          class={props.class}
          multiple={false}
          options={props.options}
          optionValue='value'
          optionTextValue='name'
          value={props.value}
          onChange={(option: Option<string> | null) => {
            if (option) {
              props.onChange(option.value);
            }
          }}
          disabled={props.options.length === 0}
          placeholder={t('logs.noLogs')}
          itemComponent={(itemProps) => (
            <SelectItem item={itemProps.item}>
              {itemProps.item.textValue}
            </SelectItem>
          )}
        >
          <CombinedTooltip
            as={SelectTrigger}
            label={props.value?.name}
            disableTooltip={!props.value}
            class='gap-1.5'
          >
            <SelectValue<Option<string>> class='min-w-0 truncate'>
              {(state) => state.selectedOption().name}
            </SelectValue>
          </CombinedTooltip>
          <SelectContent />
        </Select>
      )}
    </Show>
  );
};
