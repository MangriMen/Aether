import type { ValidComponent } from 'solid-js';

import MdiDelete from '@iconify/icons-mdi/delete';
import { Polymorphic, type PolymorphicProps } from '@kobalte/core';
import { splitProps } from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import type { AccountType } from '../model';

import { AccountButton } from './AccountButton';

export type AccountCardProps = {
  accountButtonProps?: ButtonProps;
  accountType: AccountType;
  active: boolean;
  onActivate?: () => void;
  onRemove?: () => void;
  removeButtonProps?: ButtonProps;
  username: string;
} & { class?: string };

export const AccountCard = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, AccountCardProps>,
) => {
  const [local, others] = splitProps(props, [
    'username',
    'active',
    'accountType',
    'onActivate',
    'onRemove',
    'accountButtonProps',
    'removeButtonProps',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <Polymorphic
      class={cn('flex justify-between border rounded-md h-12', local.class)}
      {...others}
    >
      <AccountButton
        accountType={local.accountType}
        active={local.active}
        onClick={local.onActivate}
        username={local.username}
        {...local.accountButtonProps}
      />

      <div class='flex items-start justify-start'>
        <CombinedTooltip
          as={IconButton}
          class='aspect-square size-full rounded-l-none p-0 hover:bg-destructive focus:z-10'
          icon={MdiDelete}
          label={t('account.removeAccount')}
          onClick={local.onRemove}
          variant='ghost'
          {...local.removeButtonProps}
        />
      </div>
    </Polymorphic>
  );
};
