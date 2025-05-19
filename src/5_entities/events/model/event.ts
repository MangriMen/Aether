import type { InstanceEvent } from './instance';
import type { LoadingPayload } from './loading';
import type { ProcessPayload } from './process';
import type { WarningPayload } from './warning';

export type LauncherEvent = 'loading' | 'process' | 'instance' | 'warning';

export type LauncherEventPayload<T = LauncherEvent> = T extends 'loading'
  ? LoadingPayload
  : T extends 'process'
    ? ProcessPayload
    : T extends 'instance'
      ? InstanceEvent
      : T extends 'warning'
        ? WarningPayload
        : never;
