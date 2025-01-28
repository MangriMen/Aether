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

import type { LoaderVersion } from '@/entities/minecraft';


import { useTranslate } from '@/app/model';

import type { SelectSpecificLoaderVersionProps } from '.';

export const SelectSpecificLoaderVersion = <
  T extends LoaderVersion = LoaderVersion,
>(
  props: SelectSpecificLoaderVersionProps<T>,
) => {
  const [local, others] = splitProps(props, ['errorMessage', 'class']);

  const [{ t }] = useTranslate();

  return (
    <Select
      class={cn('flex flex-col gap-2 w-full', local.class)}
      virtualized
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      optionValue={'id'}
      optionTextValue={'id'}
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
      </SelectTrigger>
      <SelectContent
        class='h-[170px]'
        virtualized
        options={others.options}
        optionValue={'id'}
        itemComponent={(props) => (
          <SelectItem item={props.item} style={props.style}>
            <div class='inline-flex gap-2'>
              {props.item.rawValue.id}
              {props.item.rawValue.stable ? (
                <Badge variant='default'>
                  {t('createInstance.loaderVersionStable')}
                </Badge>
              ) : (
                ''
              )}
            </div>
          </SelectItem>
        )}
      />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
};
