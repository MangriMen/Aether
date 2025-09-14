import { z } from 'zod';

export type CreateOfflineAccountFormSchemaErrors = 'usernameError';

export type CreateOfflineAccountFormValues = {
  username: string;
};

export const CreateOfflineAccountFormSchema = z.object({
  username: z.string().regex(/^\w{1,16}$/, {
    message: 'usernameError',
  }),
});
