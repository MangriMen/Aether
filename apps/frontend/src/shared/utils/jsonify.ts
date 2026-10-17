/** @public Debug helper: pretty JSON for logging while developing. */
export const jsonify = (value: unknown) => JSON.stringify(value, null, 2);
