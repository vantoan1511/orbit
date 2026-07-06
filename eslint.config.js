import pluginVue from 'eslint-plugin-vue'
import vueTsConfigs from '@vue/eslint-config-typescript'
import skipFormatting from '@vue/eslint-config-prettier'

export default [
  {
    name: 'app/files-to-lint',
    files: ['**/*.{ts,mts,tsx,vue}']
  },

  {
    name: 'app/files-to-ignore',
    ignores: [
      '**/dist/**',
      '**/dist-ssr/**',
      '**/coverage/**',
      'bin/**',
      'build/**',
      'components.d.ts'
    ]
  },

  ...pluginVue.configs['flat/essential'],
  ...vueTsConfigs(),
  skipFormatting
]
