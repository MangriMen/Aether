import type { Component, ComponentProps } from 'solid-js';

import { Match, splitProps, Switch } from 'solid-js';

import type { PluginSourceTypeDto } from '@/shared/api/bindings/plugin';

import { InstallFromGithubView } from '@/features/install-plugin-from-github';
import { cn } from '@/shared/lib';

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
