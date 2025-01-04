import MdiCloudOffOutline from '@iconify/icons-mdi/cloud-off-outline';
import MdiLocationEnter from '@iconify/icons-mdi/location-enter';
import MdiSignIn from '@iconify/icons-mdi/login-variant';
import MdiLogOut from '@iconify/icons-mdi/logout-variant';
import { Icon } from '@iconify-icon/solid';
import { Component, ComponentProps, For, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Button, IconButton } from '@/shared/ui';

import {
  Account,
  AccountType,
  getAccountStateResource,
} from '@/entities/accounts';

export type AccountSelectFormProps = ComponentProps<'div'> & {
  onSelectAccount: (id: Account['id']) => void;
  onCreateAccount: (type: AccountType) => void;
  onLogout: (uuid: string) => void;
};

export const AccountSelectForm: Component<AccountSelectFormProps> = (props) => {
  const [local, others] = splitProps(props, [
    'onSelectAccount',
    'onCreateAccount',
    'onLogout',
  ]);

  const accountState = getAccountStateResource();

  const createOnlineAccount = () => {
    local.onCreateAccount('online');
  };

  const createOfflineAccount = () => {
    local.onCreateAccount('offline');
  };

  return (
    <div {...others}>
      <div class='flex gap-2'>
        <Button
          variant='outline'
          class='flex items-center gap-2 px-2'
          onClick={createOnlineAccount}
        >
          Sign in
          <Icon class='text-2xl' icon={MdiSignIn} />
        </Button>

        <Button
          variant='outline'
          class='flex items-center gap-2 px-2'
          onClick={createOfflineAccount}
        >
          Offline
          <Icon class='text-2xl' icon={MdiCloudOffOutline} />
        </Button>
      </div>
      <hr class='my-2' />
      <Show
        when={accountState()?.accounts.length}
        fallback={
          <span class='inline-flex w-full justify-center'>No accounts</span>
        }
      >
        <div class='flex flex-col gap-2'>
          <For each={accountState()?.accounts}>
            {(account) => (
              <div
                class={cn(
                  'flex justify-between w-full rounded-sm h-12 px-2 py-1',
                  {
                    'bg-neutral-900': account.id === accountState()?.default,
                  },
                )}
              >
                <div>
                  <span class='font-bold'>{account.username}</span>
                </div>
                <Show
                  when={account.id === accountState()?.default}
                  fallback={
                    <IconButton
                      variant='ghost'
                      onClick={() => local.onSelectAccount(account.id)}
                    >
                      <Icon class='text-2xl' icon={MdiLocationEnter} />
                    </IconButton>
                  }
                >
                  <IconButton
                    variant='ghost'
                    onClick={() => local.onLogout(account.id)}
                  >
                    <Icon class='text-2xl' icon={MdiLogOut} />
                  </IconButton>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
