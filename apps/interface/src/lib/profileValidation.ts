/**
 * Validation helpers for user profile data.
 */

const MAX_BIO_LENGTH = 280;
const MAX_SOCIAL_LINKS = 5;

/**
 * Returns true when the bio does not exceed 280 characters.
 */
export function validateBio(bio: string): boolean {
  return bio.length <= MAX_BIO_LENGTH;
}

/**
 * Returns true when the links array contains at most 5 entries and every
 * entry is parseable by `new URL()`.
 */
export function validateSocialLinks(links: string[]): boolean {
  if (links.length > MAX_SOCIAL_LINKS) return false;
  for (const link of links) {
    try {
      new URL(link);
    } catch {
      return false;
    }
  }
  return true;
}

export { MAX_BIO_LENGTH, MAX_SOCIAL_LINKS };
