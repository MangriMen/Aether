import { splitProps } from 'solid-js';

import type { ModdedLoaderVersion } from '@/entities/minecraft';
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
  Option extends ModdedLoaderVersion = ModdedLoaderVersion,
> = SelectRootProps<Option, never, 'div'> & {
  multiple?: false;
  errorMessage?: string;
};

export const SelectSpecificLoaderVersion = <
  T extends ModdedLoaderVersion = ModdedLoaderVersion,
>(
  props: SelectSpecificLoaderVersionProps<T>,
) => {
  const [local, others] = splitProps(props, ['errorMessage', 'class']);

  const [{ t }] = useTranslation();

  return (
    <Select
      class={cn('gap-2 flex w-full flex-col', local.class)}
      virtualized
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      optionValue='id'
      optionTextValue='id'
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
      </SelectTrigger>
      <SelectContent
        class='h-42.5'
        virtualized
        options={others.options}
        optionValue='id'
        itemComponent={(props) => (
          <SelectItem item={props.item} style={props.style}>
            <div class='gap-2 inline-flex'>
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
