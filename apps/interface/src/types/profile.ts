/**
 * Profile metadata types for the user profile pages feature.
 */

export interface ProfileData {
  avatarUri: string;
  bio: string;
  socialLinks: string[];
}

export const DEFAULT_PROFILE: ProfileData = {
  avatarUri: "",
  bio: "",
  socialLinks: [],
};
