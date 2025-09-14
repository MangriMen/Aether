import type { ValidComponent } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { type ButtonProps } from '@/shared/ui';
import { Button, CombinedTooltip } from '@/shared/ui';

import { useTranslation } from '@/shared/model';

import type { AccountType } from '../model';
import type { PolymorphicProps } from '@kobalte/core';

export type AccountButtonProps<T extends ValidComponent = 'button'> =
  ButtonProps<T> & {
    username: string;
    active: boolean;
    accountType: AccountType;
  };

export const AccountButton = <T extends ValidComponent = 'button'>(
  props: Exclude<PolymorphicProps<T, AccountButtonProps<T>>, 'label'>,
) => {
  const [local, others] = splitProps(props, [
    'username',
    'accountType',
    'active',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      label={t('account.activate')}
      disableTooltip={local.active}
      as={Button}
      class={cn('size-full rounded-r-none justify-start px-2', local.class, {
        'bg-muted pointer-events-none': local.active,
      })}
      variant='ghost'
      {...others}
    >
      <div class='flex flex-col items-start'>
        <span class='font-bold'>{local.username}</span>
        <span class='capitalize text-muted-foreground'>
          {t(`account.${local.accountType as AccountType}`)}
        </span>
      </div>
    </CombinedTooltip>
  );
};
