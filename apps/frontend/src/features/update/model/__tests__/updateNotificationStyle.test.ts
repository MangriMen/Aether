import { describe, expect, it } from 'vitest';

import {
  deserializeUpdateNotificationStyle,
  UpdateNotificationStyle,
} from '../updateNotificationStyle';

describe('deserializeUpdateNotificationStyle', () => {
  it('should keep the stored value when it is a known style', () => {
    expect(deserializeUpdateNotificationStyle('"banner"')).toBe(
      UpdateNotificationStyle.Banner,
    );
  });

  it('should fall back to the default when the stored value is not a known style', () => {
    expect(deserializeUpdateNotificationStyle('"popup"')).toBe(
      UpdateNotificationStyle.Toast,
    );
    expect(deserializeUpdateNotificationStyle('{broken')).toBe(
      UpdateNotificationStyle.Toast,
    );
  });
});
