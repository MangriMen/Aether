import type { Component, ComponentProps } from 'solid-js';
import { For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { Account } from '../model';

import { AccountCard } from './AccountCard';
import { useTranslation } from '@/shared/model';
import { TransitionGroup } from 'solid-transition-group';

export type AccountsListProps = ComponentProps<'ul'> & {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onRemove: (id: Account['id']) => void;
};

export const AccountsList: Component<AccountsListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'onActivate',
    'onRemove',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <ul
      class={cn('flex flex-col gap-2 relative overflow-x-hidden ', local.class)}
      {...others}
    >
      <TransitionGroup name='animate-list-item'>
        <For
          each={local.accounts}
          fallback={
            <span
              class={cn('inline-flex w-full justify-center items-center h-12')}
            >
              {t('account.notFound')}
            </span>
          }
        >
          {(account) => (
            <AccountCard
              class='animate-list-item w-[210.625px]'
              as='li'
              username={account.username}
              active={account.active}
              accountType={account.accountType}
              onActivate={() => local.onActivate(account.id)}
              onRemove={() => local.onRemove(account.id)}
            />
          )}
        </For>
      </TransitionGroup>
    </ul>
  );
};
