import type { Component, ComponentProps } from 'solid-js';

import { Match, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/utils';

import type { PluginSourceType } from '../model/pluginProvider';

import { InstallFromGithubView } from './InstallFromGithubView';

export type AddPluginViewProps = ComponentProps<'div'> & {
  provider?: PluginSourceType | null;
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
