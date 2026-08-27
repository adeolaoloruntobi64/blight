export const BLIGHT_SETTINGS_URL = "blight://settings";

export function isInternalUrl(url: string): boolean {
  return url.startsWith("blight://");
}