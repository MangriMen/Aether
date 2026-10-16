import {
  For,
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Badge, FieldLabel } from '@/shared/ui';

export type PluginHostFunctionsSectionProps = ComponentProps<'div'> & {
  /** `null`/`undefined`: the manifest does not declare `api.hostFunctions`. */
  hostFunctions: string[] | null | undefined;
};

/** The host functions a plugin is linked with — what it may ask the launcher to do. */
export const PluginHostFunctionsSection: Component<
  PluginHostFunctionsSectionProps
> = (props) => {
  const [local, others] = splitProps(props, ['hostFunctions', 'class']);
  const [{ t }] = useTranslation();

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <FieldLabel>{t('plugin.hostFunctions')}</FieldLabel>
      <span class='text-sm text-muted-foreground'>
        {t('plugin.hostFunctionsDescription')}
      </span>
      <Switch>
        <Match when={local.hostFunctions == null}>
          <span class='text-sm text-warning'>
            {t('plugin.hostFunctionsNotDeclared')}
          </span>
        </Match>
        <Match when={local.hostFunctions?.length === 0}>
          <span class='text-sm'>{t('plugin.hostFunctionsNone')}</span>
        </Match>
        <Match when={local.hostFunctions}>
          {(hostFunctions) => (
            <div class='gap-1.5 flex flex-wrap'>
              <For each={hostFunctions()}>
                {(name) => (
                  <Badge variant='outline' class='font-mono'>
                    {name}
                  </Badge>
                )}
              </For>
            </div>
          )}
        </Match>
      </Switch>
    </div>
  );
};
