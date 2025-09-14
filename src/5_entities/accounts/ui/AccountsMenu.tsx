import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Account, AccountType } from '@/entities/accounts';

import { AccountLoginMethods, AccountsList } from '@/entities/accounts';
import { Separator } from '@/shared/ui';

import { useSortedAccounts } from '../lib';

export type AccountsMenuProps = {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onCreate: (type: AccountType) => void;
  onLogout: (id: Account['id']) => void;
} & ComponentProps<'div'>;

export const AccountsMenu: Component<AccountsMenuProps> = (props) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'onActivate',
    'onCreate',
    'onLogout',
  ]);

  const sortedAccounts = useSortedAccounts(() => local.accounts);

  return (
    <div {...others}>
      <AccountsList
        accounts={sortedAccounts()}
        class='max-h-48 overflow-y-auto p-3'
        onActivate={local.onActivate}
        onRemove={local.onLogout}
      />
      <Separator class='mb-3' />
      <AccountLoginMethods class='px-3 pb-3' onLogin={local.onCreate} />
    </div>
  );
};
