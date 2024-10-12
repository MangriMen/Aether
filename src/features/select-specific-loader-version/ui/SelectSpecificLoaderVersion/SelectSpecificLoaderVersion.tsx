import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { LoaderVersion } from '@/entities/minecraft';

import { SelectSpecificLoaderVersionProps } from '.';

export const SelectSpecificLoaderVersion = <
  T extends LoaderVersion = LoaderVersion,
>(
  props: SelectSpecificLoaderVersionProps<T>,
) => {
  return (
    <Select
      class='w-full'
      optionValue={'id'}
      optionTextValue={'id'}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.rawValue.id}</SelectItem>
      )}
      {...props}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption().id}</SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
