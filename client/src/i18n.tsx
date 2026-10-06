import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';
import { createContext, useContext } from 'react';
import en from '../public/locales/en.ftl?raw';
import ko from '../public/locales/ko.ftl?raw';

export type Language = 'en' | 'ko';
export type Translate = (key: string, args?: Record<string, FluentVariable>) => string;

export function createTranslator(language: Language, catalogs = { en, ko }): Translate {
  const locales: Language[] = language === 'en' ? ['en'] : ['ko', 'en'];
  const bundles = locales.map(locale => {
    const bundle = new FluentBundle(locale, { useIsolating: false });
    const errors = bundle.addResource(new FluentResource(catalogs[locale]));
    if (errors.length) throw new Error(`Invalid Fluent catalog: ${errors.join(', ')}`);
    return bundle;
  });
  return (key, args) => {
    for (const bundle of bundles) {
      const message = bundle.getMessage(key);
      if (message?.value != null) {
        const errors: Error[] = [];
        const value = bundle.formatPattern(message.value, args, errors);
        if (!errors.length) return value;
      }
    }
    for (const bundle of bundles) {
      const message = bundle.getMessage('unknown-message');
      if (message?.value) return bundle.formatPattern(message.value);
    }
    throw new Error('Fluent catalog requires unknown-message');
  };
}

const translators = { en: createTranslator('en'), ko: createTranslator('ko') };
export const Localization = createContext<{ language: Language; t: Translate }>({ language: 'en', t: translators.en });
export function translatorFor(language: Language) { return translators[language]; }
export function useLocalization() { return useContext(Localization); }
