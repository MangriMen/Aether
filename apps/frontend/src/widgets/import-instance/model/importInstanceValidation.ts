import * as v from 'valibot';

export const ImportInstanceSchema = v.object({
  path: v.pipe(v.string(), v.minLength(1)),
});

export type ImportInstanceSchemaInput = v.InferInput<
  typeof ImportInstanceSchema
>;
export type ImportInstanceSchemaOutput = v.InferOutput<
  typeof ImportInstanceSchema
>;
