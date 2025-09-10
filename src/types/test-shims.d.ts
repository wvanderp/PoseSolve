// Minimal shims for test libraries used only in dev/tests.
declare module 'vitest' {
  export const describe: any;
  export const it: any;
  export const expect: any;
  export const beforeEach: any;
  export const afterEach: any;
  export const vi: any;
}

declare module '@testing-library/react' {
  export const render: any;
  export const fireEvent: any;
  export const screen: any;
  export default any;
}

declare module 'cypress/react' {
  const mount: any;
  export { mount };
}
