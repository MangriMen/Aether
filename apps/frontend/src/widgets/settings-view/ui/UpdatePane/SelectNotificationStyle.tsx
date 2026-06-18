import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { Option } from '@/shared/model';

import { cn } from '@/shared/lib';
import {
  setUpdateNotificationStyle,
  updateNotificationStyle,
  UpdateNotificationStyle,
  useTranslation,
} from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

export type SelectNotificationStyleProps = Omit<
  ComponentProps<'div'>,
  'onChange'
>;

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

export const SelectNotificationStyle: Component<
  SelectNotificationStyleProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

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
    <Select
      class={cn('w-40 min-w-40', local.class)}
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
      {...others}
    >
      <SelectTrigger>
        <SelectValue<UpdateNotificationStyleOption>>
          {(state) => t(`update.${state.selectedOption().name}`)}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
