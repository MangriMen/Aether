import type { ComponentProps, ValidComponent } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Button, CombinedTooltip, type ButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';

import type { Account, AccountType } from '../model';

export type AccountButtonProps<T extends ValidComponent = 'button'> =
  ButtonProps<T> & {
    username: Account['username'];
    type: AccountType;
    active?: boolean;
  };

const AccountButton = <T extends ValidComponent = 'button'>(
  props: ComponentProps<T> & AccountButtonProps<T>,
) => {
  const [local, others] = splitProps(props, [
    'username',
    'type',
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
          {t(`account.${local.type as AccountType}`)}
        </span>
      </div>
    </CombinedTooltip>
  );
};

export default AccountButton;
