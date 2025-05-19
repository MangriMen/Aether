export interface AetherLauncherError {
  message: string;
}

export const isAetherLauncherError = (
  error: unknown,
): error is AetherLauncherError => {
  return !!error && typeof error === 'object' && 'message' in error;
};
