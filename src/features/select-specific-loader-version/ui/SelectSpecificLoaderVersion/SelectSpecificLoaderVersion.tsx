import {
  Badge,
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
        <SelectItem item={props.item}>
          <div class='inline-flex gap-2'>
            {props.item.rawValue.id}
            {props.item.rawValue.stable ? (
              <Badge variant='default'>stable</Badge>
            ) : (
              ''
            )}
          </div>
        </SelectItem>
      )}
      {...props}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
      </SelectTrigger>
      <SelectContent class='max-h-[148px] overflow-y-auto' />
    </Select>
  );
};
