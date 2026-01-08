import { createMemo, type Component, type ComponentProps } from 'solid-js';

import type { Option } from '@/shared/model';

import { UpdateNotificationStyle } from '@/shared/model';
import {
  setUpdateNotificationStyle,
  updateNotificationStyle,
  useTranslation,
} from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsEntry,
} from '@/shared/ui';

export type UpdateNotificationStyleEntryProps = ComponentProps<'div'>;

type UpdateNotificationStyleName = 'toast' | 'banner';
type UpdateNotificationStyleOption = Option<
  UpdateNotificationStyle,
  UpdateNotificationStyleName
>;

const options: UpdateNotificationStyleOption[] = [
  {
    name: 'toast',
    value: UpdateNotificationStyle.Toast,
  },
  {
    name: 'banner',
    value: UpdateNotificationStyle.Banner,
  },
] as const;

export const UpdateNotificationStyleEntry: Component<
  UpdateNotificationStyleEntryProps
> = (props) => {
  const [{ t }] = useTranslation();

  const currentOption = createMemo(() =>
    options.find((option) => option.value === updateNotificationStyle()),
  );

  const handleChangeNotificationStyle = (
    style: UpdateNotificationStyleOption | null,
  ) => {
    if (style === null) {
      return;
    }

    setUpdateNotificationStyle(style.value);
  };

  return (
    <SettingsEntry title={t('settings.updateNotificationStyle')} {...props}>
      <Select
        class='w-40 min-w-40'
        multiple={false}
        options={options}
        optionValue='value'
        optionTextValue='name'
        value={currentOption()}
        onChange={handleChangeNotificationStyle}
        itemComponent={(props) => (
          <SelectItem item={props.item}>
            {t(`update.${props.item.textValue as UpdateNotificationStyle}`)}
          </SelectItem>
        )}
      >
        <SelectTrigger>
          <SelectValue<UpdateNotificationStyleOption>>
            {(state) => t(`update.${state.selectedOption().name}`)}
          </SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    </SettingsEntry>
  );
};
