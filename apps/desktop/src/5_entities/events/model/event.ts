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

export interface LauncherEventPayloadMap {
  loading: LoadingPayload;
  process: ProcessPayload;
  instance: InstanceEvent;
  warning: WarningPayload;
  plugin: PluginPayload;
}

export type LauncherEventPayload<T extends string = LauncherEvent> =
  T extends LauncherEvent ? LauncherEventPayloadMap[T] : unknown;
