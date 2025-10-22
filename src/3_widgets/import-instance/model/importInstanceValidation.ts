import z from 'zod';

export const ImportInstanceSchema = z.object({
  path: z.string().min(1),
});

export type ImportInstanceSchemaInput = z.input<typeof ImportInstanceSchema>;
export type ImportInstanceSchemaOutput = z.output<typeof ImportInstanceSchema>;
