import type { Component, ComponentProps } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiCloudOffOutline from '@iconify/icons-mdi/cloud-off-outline';
import MdiSignIn from '@iconify/icons-mdi/login-variant';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip } from '@/shared/ui';

import type { AccountType } from '../model';

export type AccountLoginMethodsProps = ComponentProps<'div'> & {
  onLogin: (type: AccountType) => void;
};

export const AccountLoginMethods: Component<AccountLoginMethodsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['onLogin', 'class']);

  const [{ t }] = useTranslation();

  const onOnline = () => {
    local.onLogin('online');
  };

  const onOffline = () => {
    local.onLogin('offline');
  };

  return (
    <div class={cn('flex gap-2', local.class)} {...others}>
      <CombinedTooltip
        label={t('account.signInMinecraft')}
        as={Button}
        variant='outline'
        class='px-2'
        onClick={onOnline}
        // TODO: implement minecraft login
        disabled
      >
        <span class='flex items-center gap-2'>
          {t('account.signIn')}
          <Icon class='text-2xl' icon={MdiSignIn} />
        </span>
      </CombinedTooltip>
      <CombinedTooltip
        label={t('account.signInOffline')}
        as={Button}
        variant='outline'
        class='px-2'
        onClick={onOffline}
      >
        <span class='flex items-center gap-2'>
          {t('account.offline')}
          <Icon class='text-2xl' icon={MdiCloudOffOutline} />
        </span>
      </CombinedTooltip>
    </div>
  );
};
