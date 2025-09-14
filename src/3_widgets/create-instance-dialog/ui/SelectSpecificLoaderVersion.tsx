import { splitProps } from 'solid-js';

import type { LoaderVersion } from '@/entities/minecraft';
import type { SelectRootProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Badge,
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

export type SelectSpecificLoaderVersionProps<
  Option extends LoaderVersion = LoaderVersion,
> = {
  errorMessage?: string;
  multiple?: false;
} & SelectRootProps<Option, never, 'div'>;

export const SelectSpecificLoaderVersion = <
  T extends LoaderVersion = LoaderVersion,
>(
  props: SelectSpecificLoaderVersionProps<T>,
) => {
  const [local, others] = splitProps(props, ['errorMessage', 'class']);

  const [{ t }] = useTranslation();

  return (
    <Select
      class={cn('flex flex-col gap-2 w-full', local.class)}
      optionTextValue={'id'}
      optionValue={'id'}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      virtualized
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
      </SelectTrigger>
      <SelectContent
        class='h-[170px]'
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
        options={others.options}
        optionValue={'id'}
        virtualized
      />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
};
