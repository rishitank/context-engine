/**
 * Jest setup file
 * 
 * This file runs before all tests to set up the testing environment.
 */

import { jest } from '@jest/globals';

// Make jest available globally
(globalThis as any).jest = jest;

// Set longer timeout for async operations
jest.setTimeout(30000);

// Suppress console output during tests (optional - comment out for debugging)
// global.console = {
//   ...console,
//   log: jest.fn(),
//   debug: jest.fn(),
//   info: jest.fn(),
//   warn: jest.fn(),
//   error: jest.fn(),
// };

