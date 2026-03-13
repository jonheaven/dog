import type { Config } from 'tailwindcss';

export default {
  darkMode: ['class'],
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        background: '#04050b',
        card: '#0d111f',
        accent: '#f59e0b'
      }
    }
  }
} satisfies Config;
