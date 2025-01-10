import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Badge,
  Select,
  SelectContent,
  SelectErrorMessage,
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
  const [local, others] = splitProps(props, ['errorMessage', 'class']);

  return (
    <Select
      class={cn('flex flex-col gap-2 w-full', local.class)}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
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
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
      </SelectTrigger>
      <SelectContent class='max-h-[148px] overflow-y-auto' />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
};
