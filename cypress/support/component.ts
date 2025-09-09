// Cypress component test support
import React from 'react';
import { mount } from 'cypress/react';

// Import commands
import './commands';

// Extend Cypress interface to include mount
declare global {
  namespace Cypress {
    interface Chainable {
      mount: typeof mount;
    }
  }
}

// Add mount command to Cypress
Cypress.Commands.add('mount', mount);
