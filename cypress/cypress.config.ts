import { defineConfig } from 'cypress';

export default defineConfig({
  component: {
    devServer: {
      framework: 'react',
      bundler: 'vite',
      viteConfig: {
        // reuse project's vite config
        // the plugin will pick up vite.config.ts automatically when run from project root
      }
    },
    specPattern: 'cypress/component/**/*.cy.{js,ts,tsx}'
  }
});
