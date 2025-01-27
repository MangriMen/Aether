import MdiCloudOffOutline from '@iconify/icons-mdi/cloud-off-outline';
import MdiSignIn from '@iconify/icons-mdi/login-variant';
import { Icon } from '@iconify-icon/solid';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Button } from '@/shared/ui';

import type { AccountType } from '../model';

export type AccountLoginMethodsProps = ComponentProps<'div'> & {
  onLogin: (type: AccountType) => void;
};

export const AccountLoginMethods: Component<AccountLoginMethodsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['onLogin', 'class']);

  const onOnline = () => {
    local.onLogin('online');
  };

  const onOffline = () => {
    local.onLogin('offline');
  };

  return (
    <div class={cn('flex gap-2', local.class)} {...others}>
      <Button
        variant='outline'
        class='px-2'
        onClick={onOnline}
        // TODO: implement minecraft login
        disabled
        title='Sign in minecraft account'
      >
        <span class='flex items-center gap-2'>
          Sign in
          <Icon class='text-2xl' icon={MdiSignIn} />
        </span>
      </Button>

      <Button
        variant='outline'
        class='px-2'
        title='Sign in offline'
        onClick={onOffline}
      >
        <span class='flex items-center gap-2'>
          Offline
          <Icon class='text-2xl' icon={MdiCloudOffOutline} />
        </span>
      </Button>
    </div>
  );
};
