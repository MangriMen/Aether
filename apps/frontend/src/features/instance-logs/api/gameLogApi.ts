import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import {
  gameLogCommands,
  gameLogEvents,
  useEventListener,
} from '@/platform/ipc';

import type { GameLogEvent, LogSource } from '../model/gameLog';

const GAME_LOG_QUERY_KEYS = {
  SOURCES: (instanceId: string) => ['instance', 'logs', instanceId, 'sources'],
};

/** A page of the log is read in pieces this big, so a huge log never travels at once. */
const PAGE_BYTES = 256 * 1024;
/** A log read whole comes in bigger pieces, to be done in fewer round trips. */
export const WHOLE_FILE_PAGE_BYTES = 1024 * 1024;

export const useGameLogSources = (instanceId: Accessor<string>) =>
  useQuery(() => ({
    queryKey: GAME_LOG_QUERY_KEYS.SOURCES(instanceId()),
    queryFn: (): Promise<LogSource[]> => gameLogCommands.listLogs(instanceId()),
    // Logs come and go with game runs, not with anything the UI could subscribe to.
    staleTime: 0,
    gcTime: 0,
  }));

export const readGameLogTail = (instanceId: string, sourceId: string) =>
  gameLogCommands.readLog(instanceId, sourceId, { kind: 'tail' }, PAGE_BYTES);

export const readGameLogBefore = (
  instanceId: string,
  sourceId: string,
  at: number,
  maxBytes: number = PAGE_BYTES,
) =>
  gameLogCommands.readLog(
    instanceId,
    sourceId,
    { kind: 'before', at },
    maxBytes,
  );

export const readGameLogAfter = (
  instanceId: string,
  sourceId: string,
  at: number,
) =>
  gameLogCommands.readLog(
    instanceId,
    sourceId,
    { kind: 'after', at },
    PAGE_BYTES,
  );

/** New lines of the game's console log, in batches. Listens while the owner is mounted. */
export const useGameLogEventListener = (
  callback: (event: GameLogEvent) => void,
) => {
  useEventListener(gameLogEvents.gameLogEventDto, (e) => {
    callback(e.payload);
  });
};
