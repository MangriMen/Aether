import type { Component, ComponentProps } from 'solid-js';

import { Match, splitProps, Switch } from 'solid-js';

import type { PluginSourceTypeDto } from '@/shared/api';

import { cn } from '@/shared/lib';

import { InstallFromGithubView } from './install-from-github';

export type AddPluginViewProps = ComponentProps<'div'> & {
  provider?: PluginSourceTypeDto | null;
  onInstalled?: () => void;
};

export const AddPluginView: Component<AddPluginViewProps> = (props) => {
  const [local, others] = splitProps(props, [
    'provider',
    'class',
    'onInstalled',
  ]);

  return (
    <div class={cn('p-0.5 flex flex-col', local.class)} {...others}>
      <Switch>
        <Match when={local.provider === 'git_hub'}>
          <InstallFromGithubView onInstalled={local.onInstalled} />
        </Match>
      </Switch>
    </div>
  );
};
