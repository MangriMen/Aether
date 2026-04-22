export const MIN_USERNAME_LENGTH = 3;
export const MAX_USERNAME_LENGTH = 16;

export const createOfflineAccountErrorParams: Record<
  string,
  Record<string, string | number>
> = {
  INVALID_USERNAME_LENGTH: {
    min: MIN_USERNAME_LENGTH,
    max: MAX_USERNAME_LENGTH,
  },
};
