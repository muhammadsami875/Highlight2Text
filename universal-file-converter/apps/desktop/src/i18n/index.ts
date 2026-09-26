import en from "./en.json";

type Translations = typeof en;

const translations: Record<string, Translations> = { en };

let currentLocale = "en";

export function setLocale(locale: string) {
  if (translations[locale]) {
    currentLocale = locale;
  }
}

export function t(key: string, params?: Record<string, string>): string {
  const keys = key.split(".");
  let value: unknown = translations[currentLocale];
  for (const k of keys) {
    if (value && typeof value === "object" && k in value) {
      value = (value as Record<string, unknown>)[k];
    } else {
      return key;
    }
  }
  if (typeof value !== "string") return key;
  if (!params) return value;
  return value.replace(/\{(\w+)\}/g, (_, name) => params[name] ?? `{${name}}`);
}
