import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { SelectRootProps } from '@/shared/ui';
import {
  Badge,
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import type { LoaderVersion } from '@/5_entities/minecraft';

import { useTranslate } from '@/shared/model';

export type SelectSpecificLoaderVersionProps<
  Option extends LoaderVersion = LoaderVersion,
> = SelectRootProps<Option, never, 'div'> & {
  multiple?: false;
  errorMessage?: string;
};

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
