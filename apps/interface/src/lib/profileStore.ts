/**
 * profileStore — client-side persistence layer for profile metadata.
 *
 * Uses localStorage key: fmc:profile:<address>
 * Consistent with the existing fmc: namespace convention.
 */

import type { ProfileData } from "@/types/profile";
import { DEFAULT_PROFILE } from "@/types/profile";

function profileKey(address: string): string {
  return `fmc:profile:${address}`;
}

function isProfileData(value: unknown): value is ProfileData {
  if (!value || typeof value !== "object") return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.avatarUri === "string" &&
    typeof obj.bio === "string" &&
    Array.isArray(obj.socialLinks) &&
    (obj.socialLinks as unknown[]).every((l) => typeof l === "string")
  );
}

/**
 * Reads profile metadata for the given Stellar address from localStorage.
 * Returns DEFAULT_PROFILE when the entry is missing, invalid, or localStorage
 * is unavailable.
 */
export function readProfile(address: string): ProfileData {
  try {
    const raw = localStorage.getItem(profileKey(address));
    if (!raw) return { ...DEFAULT_PROFILE };
    const parsed: unknown = JSON.parse(raw);
    if (!isProfileData(parsed)) return { ...DEFAULT_PROFILE };
    return parsed;
  } catch {
    return { ...DEFAULT_PROFILE };
  }
}

/**
 * Serialises profile data to JSON and writes it to localStorage.
 * Silently no-ops if localStorage is unavailable.
 */
export function writeProfile(address: string, data: ProfileData): void {
  try {
    localStorage.setItem(profileKey(address), JSON.stringify(data));
  } catch {
    // localStorage unavailable — silently no-op
  }
}
