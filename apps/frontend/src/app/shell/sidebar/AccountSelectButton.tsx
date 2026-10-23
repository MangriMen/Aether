import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import IconMdiAccount from '~icons/mdi/account';
import { createSignal, splitProps } from 'solid-js';

import type { Account, AccountType } from '@/features/account';
import type { CombinedTooltipProps, IconButtonProps } from '@/shared/ui-kit';

import { useChangeAccount, useLogout } from '@/features/account';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import {
  CombinedTooltip,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui-kit';

export type AccountSelectButtonProps = IconButtonProps & {
  accounts: Account[];
  accountsMenu: Component<
    ComponentProps<'div'> & {
      accounts: Account[];
      onActivate: (id: Account['id']) => void;
      onCreate: (type: AccountType) => void;
      onLogout: (uuid: string) => void;
    }
  >;
  createOfflineAccountDialog: Component<
    ComponentProps<'div'> & DialogRootProps
  >;
};

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

  // The mutations show their own errors; this keeps the rejection from escaping.
  const handleSelect = async (id: Account['id']) => {
    try {
      await changeAccount(id);
    } catch (error) {
      logError('Failed to change the account:', error);
    }
  };

  const { mutateAsync: logout } = useLogout();

  const handleLogout = async (uuid: string) => {
    try {
      await logout(uuid);
    } catch (error) {
      logError('Failed to log out:', error);
    }
  };

  const [accountCreationType, setAccountCreationType] =
    createSignal<AccountType | null>(null);

  const closeAccountCreationDialog = () => {
    setAccountCreationType(null);
  };

  return (
    <>
      <Popover>
        <PopoverTrigger
          as={(props: CombinedTooltipProps) => (
            <CombinedTooltip
              label={t('common.account')}
              placement='right'
              as={IconButton}
              variant='ghost'
              size='lg'
              icon={IconMdiAccount}
              {...others}
              {...props}
            />
          )}
        />
        <PopoverContent class='p-0 w-max'>
          <local.accountsMenu
            accounts={local.accounts}
            onCreate={handleCreate}
            onActivate={handleSelect}
            onLogout={handleLogout}
          />
        </PopoverContent>
      </Popover>

      <local.createOfflineAccountDialog
        open={accountCreationType() === 'offline'}
        onOpenChange={closeAccountCreationDialog}
      />
    </>
  );
};
