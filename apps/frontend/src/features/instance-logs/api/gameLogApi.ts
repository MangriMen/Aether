import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import {
  instanceCommands,
  instanceEvents,
  useEventListener,
} from '@/platform/ipc';

import type { GameLogEvent, LogSource } from '../model/gameLog';

const GAME_LOG_QUERY_KEYS = {
  SOURCES: (instanceId: string) => ['instance', 'logs', instanceId, 'sources'],
};

/** A page of the log is read in pieces this big, so a huge log never travels at once. */
const PAGE_BYTES = 256 * 1024;

export const useGameLogSources = (instanceId: Accessor<string>) =>
  useQuery(() => ({
    queryKey: GAME_LOG_QUERY_KEYS.SOURCES(instanceId()),
    queryFn: (): Promise<LogSource[]> =>
      instanceCommands.listLogs(instanceId()),
    // Logs come and go with game runs, not with anything the UI could subscribe to.
    staleTime: 0,
    gcTime: 0,
  }));

export const readGameLogTail = (instanceId: string, sourceId: string) =>
  instanceCommands.readLog(instanceId, sourceId, { kind: 'tail' }, PAGE_BYTES);

export const readGameLogBefore = (
  instanceId: string,
  sourceId: string,
  at: number,
) =>
  instanceCommands.readLog(
    instanceId,
    sourceId,
    { kind: 'before', at },
    PAGE_BYTES,
  );

export const readGameLogAfter = (
  instanceId: string,
  sourceId: string,
  at: number,
) =>
  instanceCommands.readLog(
    instanceId,
    sourceId,
    { kind: 'after', at },
    PAGE_BYTES,
  );

/** New lines of the game's console log, in batches. Listens while the owner is mounted. */
export const useGameLogEventListener = (
  callback: (event: GameLogEvent) => void,
) => {
  useEventListener(instanceEvents.gameLogEventDto, (e) => {
    callback(e.payload);
  });
};
