import { defineCollection } from 'astro:content'
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders'
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema'

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
	// Declared so Starlight stops warning that it is missing. The site is English-only, and the
	// repository is English-only by rule, so the collection is empty on purpose.
	i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
}
