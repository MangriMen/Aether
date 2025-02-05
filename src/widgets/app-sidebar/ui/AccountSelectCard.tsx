import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { Separator } from '@/shared/ui';

import type { Account, AccountType } from '@/entities/accounts';
import {
  AccountLoginMethods,
  AccountsList,
  getAccountStateResource,
} from '@/entities/accounts';

export type AccountSelectCardProps = ComponentProps<'div'> & {
  onActivate: (id: Account['id']) => void;
  onCreate: (type: AccountType) => void;
  onLogout: (uuid: string) => void;
};

export const AccountSelectCard: Component<AccountSelectCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'onActivate',
    'onCreate',
    'onLogout',
  ]);

  const accountState = getAccountStateResource();

  return (
    <div {...others}>
      <AccountsList
        class='max-h-48 overflow-y-auto px-3 pb-1 pt-3'
        accounts={accountState()}
        onActivate={local.onActivate}
        onRemove={local.onLogout}
      />
      <Separator class='mb-2 mt-1' />
      <AccountLoginMethods class='px-3 pb-3' onLogin={local.onCreate} />
    </div>
  );
};
