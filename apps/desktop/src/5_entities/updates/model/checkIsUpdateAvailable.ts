import type { UpdateStatus } from './update';

export const checkIsUpdateAvailable = (
  update: UpdateStatus | null,
): update is UpdateStatus => update !== null && update.version !== '';
