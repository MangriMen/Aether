import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import MdiAccount from '@iconify/icons-mdi/account';
import { createSignal, splitProps } from 'solid-js';

import type { Account, AccountType } from '@/entities/accounts';
import type { IconButtonProps } from '@/shared/ui';

import { useChangeAccount, useLogout } from '@/entities/accounts';
import { useTranslation } from '@/shared/model';
import {
  CombinedTooltip,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

export type AccountSelectButtonProps = {
  accounts: Account[];
  accountsMenu: Component<
    {
      accounts: Account[];
      onActivate: (id: Account['id']) => void;
      onCreate: (type: AccountType) => void;
      onLogout: (uuid: string) => void;
    } & ComponentProps<'div'>
  >;
  createOfflineAccountDialog: Component<
    ComponentProps<'div'> & DialogRootProps
  >;
} & IconButtonProps;

export const AccountSelectButton: Component<AccountSelectButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'accountsMenu',
    'createOfflineAccountDialog',
  ]);

  const [{ t }] = useTranslation();

  const handleCreate = (type: AccountType) => {
    setAccountCreationType(type);
  };

  const { mutateAsync: changeAccount } = useChangeAccount();

  const handleSelect = async (id: Account['id']) => {
    await changeAccount(id);
  };

  const { mutateAsync: logout } = useLogout();

  const handleLogout = async (uuid: string) => {
    await logout(uuid);
  };

  const [accountCreationType, setAccountCreationType] =
    createSignal<AccountType | null>(null);

  const closeAccountCreationDialog = () => {
    setAccountCreationType(null);
  };

  return (
    <>
      <Popover>
        <PopoverTrigger>
          <CombinedTooltip
            as={IconButton}
            icon={MdiAccount}
            label={t('common.account')}
            placement='right'
            variant='ghost'
            {...others}
          />
        </PopoverTrigger>
        <PopoverContent class='w-max p-0'>
          <local.accountsMenu
            accounts={local.accounts}
            onActivate={handleSelect}
            onCreate={handleCreate}
            onLogout={handleLogout}
          />
        </PopoverContent>
      </Popover>

      <local.createOfflineAccountDialog
        onOpenChange={closeAccountCreationDialog}
        open={accountCreationType() === 'offline'}
      />
    </>
  );
};
