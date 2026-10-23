/**
 * The tag to install: the user's pick if the current repository still has it, else the
 * newest release. A pick left over from a previously entered repository must not stick.
 */
export const resolveReleaseTag = (
  releases: readonly { tag_name: string }[],
  picked: string,
): string => {
  if (releases.some((release) => release.tag_name === picked)) {
    return picked;
  }

  return releases[0]?.tag_name ?? '';
};
