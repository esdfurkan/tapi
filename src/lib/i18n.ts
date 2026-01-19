import { writable, derived, get } from 'svelte/store';

// Load all json files from ./translations/
const translationFiles = import.meta.glob('./translations/*.json', { eager: true });

const translations: Record<string, any> = {};

Object.entries(translationFiles).forEach(([path, module]: [string, any]) => {
    const code = path.match(/\.\/translations\/(.+)\.json$/)?.[1];
    if (code) {
        translations[code] = module.default || module;
    }
});

export const availableLocales = Object.keys(translations);

// Current locale store
export const locale = writable('en');

// Fallback locale
const fallbackLocale = 'en';

// Helper to get nested keys (e.g. "settings.title")
function getNested(obj: any, path: string): any {
    return path.split('.').reduce((prev, curr) => prev ? prev[curr] : null, obj);
}

// Derived store for the translation function
export const t = derived(locale, ($locale) => (key: string, vars: Record<string, any> = {}) => {
    // Try current locale
    let text = getNested(translations[$locale], key);

    // Fallback if missing
    if (!text) {
        text = getNested(translations[fallbackLocale], key);
    }

    // Return key if still missing
    if (!text) return key;

    // Replace variables like {name}
    Object.keys(vars).forEach((k) => {
        const regex = new RegExp(`{${k}}`, 'g');
        text = text.replace(regex, vars[k]);
    });

    return text;
});

// Function to load a custom JSON language file (for future expansion)
// usage: loadCustomLanguage('my-lang', { ...jsonContent })
export function loadCustomLanguage(code: string, data: any) {
    translations[code] = data;
}

// Function to set language
export function setLocale(code: string) {
    locale.set(code);
    if (!translations[code]) {
        console.warn(`Language '${code}' not found in translations, using fallback for missing keys.`);
    }
}
