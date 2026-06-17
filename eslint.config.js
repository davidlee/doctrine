module.exports = [
  {
    ignores: ['**/.doctrine/**', '**/target/**', '**/node_modules/**', '**/vendor/**'],
  },
  {
    files: ['web/map/**/*.js'],
    ignores: ['**/vendor/**'],
    languageOptions: {
      ecmaVersion: 5,
      sourceType: 'script',
      globals: {
        // Browser / runtime
        document: 'readonly',
        window: 'readonly',
        console: 'readonly',
        fetch: 'readonly',
        localStorage: 'readonly',
        // ES5 builtins
        Promise: 'readonly',
        Map: 'readonly',
        Set: 'readonly',
        // Vendor
        DOMPurify: 'readonly',
        markdownit: 'readonly'
      }
    },
    rules: {
      'no-var': 'off',
      // Project convention: IIFE wrapping, strict usage not yet uniform across files
      'strict': 'off',
      'curly': 'off',
      'semi': ['error', 'always'],
      'eqeqeq': ['error', 'always'],
      'no-undef': 'error',
      'no-unused-vars': ['error', { args: 'none', caughtErrors: 'none' }],
      'no-redeclare': 'error',
      'indent': ['error', 2, { SwitchCase: 1 }],
      'quotes': ['error', 'single', { avoidEscape: true }],
      'no-multiple-empty-lines': ['error', { max: 1 }],
      'comma-dangle': ['error', 'never'],
      'no-trailing-spaces': 'error',
      'eol-last': ['error', 'always']
    }
  }
];
