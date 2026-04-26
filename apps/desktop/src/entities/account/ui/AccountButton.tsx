import type { ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { ButtonProps } from '../../../shared/ui';
import type { AccountType } from '../model';

import { cn } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import { Button, CombinedTooltip } from '../../../shared/ui';

export type AccountButtonProps = ButtonProps & {
  username: string;
  active: boolean;
  accountType: AccountType;
};

export const AccountButton = (
  props: ComponentProps<'button'> & AccountButtonProps,
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
        'bg-secondary/secondary pointer-events-none': local.active,
      })}
      variant='ghost'
      {...others}
    >
      <div class='flex flex-col items-start'>
        <span class='font-bold'>{local.username}</span>
        <span class='capitalize text-muted-foreground'>
          {t(`account.${local.accountType}`)}
        </span>
      </div>
    </CombinedTooltip>
  );
};
