import MdiAccount from '@iconify/icons-mdi/account';
import type { Component } from 'solid-js';
import { createSignal, splitProps } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import {
  CombinedTooltip,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import type { Account, AccountType } from '@/entities/accounts';
import {
  changeAccount,
  logout,
  refetchAccountStateResource,
} from '@/entities/accounts';

// eslint-disable-next-line boundaries/element-types
import type { AccountSelectCardProps } from '@/features/account-select-card';

// eslint-disable-next-line boundaries/element-types
import { CreateOfflineAccountDialog } from '@/widgets/create-offline-account-dialog';

// eslint-disable-next-line boundaries/element-types
import { useTranslate } from '@/app/model';

export type AccountSelectProps = IconButtonProps & {
  accountSelectCard: Component<AccountSelectCardProps>;
};

export const AccountSelectButton: Component<AccountSelectProps> = (props) => {
  const [local, others] = splitProps(props, ['accountSelectCard']);

  const [{ t }] = useTranslate();

  const handleCreate = (type: AccountType) => {
    setAccountCreationType(type);
  };

  const handleSelect = async (id: Account['id']) => {
    await changeAccount(id);
    refetchAccountStateResource();
  };

  const handleLogout = async (uuid: string) => {
    await logout(uuid);
    refetchAccountStateResource();
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
            label={t('common.account')}
            placement='right'
            as={IconButton}
            variant='ghost'
            icon={MdiAccount}
            {...others}
          />
        </PopoverTrigger>
        <PopoverContent class='w-max p-0'>
          <local.accountSelectCard
            onCreate={handleCreate}
            onActivate={handleSelect}
            onLogout={handleLogout}
          />
        </PopoverContent>
      </Popover>

      <CreateOfflineAccountDialog
        open={accountCreationType() === 'offline'}
        onOpenChange={closeAccountCreationDialog}
      />
    </>
  );
};
