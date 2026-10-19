import type { ProgressEvent } from './progressEvent';

export interface ProgressBytesFormatters {
  formatSize: (bytes: number) => string;
  formatSpeed: (bytesPerSecond: number) => string;
}

/** `done / total · speed`; empty when the total size is unknown. */
export const formatProgressBytes = (
  payload: Pick<ProgressEvent, 'bytesDone' | 'bytesTotal' | 'speed'>,
  { formatSize, formatSpeed }: ProgressBytesFormatters,
): string => {
  const { bytesDone, bytesTotal, speed } = payload;

  if (bytesDone === null || bytesTotal === null) {
    return '';
  }

  const size = `${formatSize(bytesDone)} / ${formatSize(bytesTotal)}`;

  return speed === null ? size : `${size} · ${formatSpeed(speed)}`;
};
