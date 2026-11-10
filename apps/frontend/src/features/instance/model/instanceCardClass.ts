/** The shell of an instance card, so cards of other kinds (a damaged one) keep its size and look. */
export const INSTANCE_CARD_CLASS = `
  max-w-33 min-w-33 gap-2 rounded-md bg-card/card p-2 drop-shadow-md
  hover:bg-card/hover
  focus-visible:ring-ring
  active:animate-bump-out active:bg-card/active
  relative flex h-max flex-col overflow-hidden border
  text-left outline-none
  focus-visible:ring-2 focus-visible:ring-offset-0
`;
