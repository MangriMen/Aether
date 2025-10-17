import type { InstanceEvent } from './instance';
import type { LoadingPayload } from './loading';
import type { PluginPayload } from './plugin';
import type { ProcessPayload } from './process';
import type { WarningPayload } from './warning';

export type LauncherEvent =
  | 'loading'
  | 'process'
  | 'instance'
  | 'warning'
  | 'plugin';

export type LauncherEventPayload<T = LauncherEvent> = T extends 'loading'
  ? LoadingPayload
  : T extends 'process'
    ? ProcessPayload
    : T extends 'instance'
      ? InstanceEvent
      : T extends 'warning'
        ? WarningPayload
        : T extends 'plugin'
          ? PluginPayload
          : never;
