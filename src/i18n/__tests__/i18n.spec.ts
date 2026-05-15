import { describe, it, expect } from 'vitest'
import { createI18n } from 'vue-i18n'
import en from '../en.json'
import ptBR from '../pt-BR.json'

function makeI18n(locale: string) {
  return createI18n({
    legacy: false,
    locale,
    fallbackLocale: 'en',
    messages: {
      en,
      'pt-BR': ptBR,
    },
  })
}

describe('i18n', () => {
  it('t("app.title") returns English string when locale is "en" (R-M7.2)', () => {
    const { global } = makeI18n('en')
    const t = global.t
    expect(t('app.title')).toBe('Orcker')
  })

  it('t("app.title") returns Portuguese string when locale is "pt-BR"', () => {
    const { global } = makeI18n('pt-BR')
    const t = global.t
    // app.title is the same in both languages ("Orcker")
    expect(t('app.title')).toBe('Orcker')
  })

  it('t("settings.appearance") returns translated string in pt-BR', () => {
    const { global } = makeI18n('pt-BR')
    const t = global.t
    expect(t('settings.appearance')).toBe('Aparência')
  })

  it('t("settings.appearance") returns English string in en', () => {
    const { global } = makeI18n('en')
    const t = global.t
    expect(t('settings.appearance')).toBe('Appearance')
  })
})
