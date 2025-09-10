import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'json-summary'],
      reportsDirectory: 'coverage',
      all: false,
      include: ['src/lib/**/*.{ts,tsx}'],
      exclude: [
        'node_modules/**',
        'dist/**',
        'src/components/**',
        'src/pages/**',
        'src/main.tsx',
        'src/**/*.d.ts',
      ],
    },
  },
})
