import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { Separator } from '@/shared/ui';

import type { Account, AccountType } from '@/entities/accounts';
import { AccountLoginMethods, AccountsList } from '@/entities/accounts';

export type AccountsMenuProps = ComponentProps<'div'> & {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onCreate: (type: AccountType) => void;
  onLogout: (uuid: string) => void;
};

export const AccountsMenu: Component<AccountsMenuProps> = (props) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'onActivate',
    'onCreate',
    'onLogout',
  ]);

  return (
    <div {...others}>
      <AccountsList
        class='max-h-48 overflow-y-auto px-3 pb-1 pt-3'
        accounts={local.accounts}
        onActivate={local.onActivate}
        onRemove={local.onLogout}
      />
      <Separator class='mb-2 mt-1' />
      <AccountLoginMethods class='px-3 pb-3' onLogin={local.onCreate} />
    </div>
  );
};
