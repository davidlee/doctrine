/**
 * @vitest-environment jsdom
 *
 * DOM tests for app.ts exported helpers (SL-110 Item 1).
 *
 * Importing app.ts runs its boot block. With no `.entity-list` in the document
 * at import time, `bootstrap()` bails early (before any fetch), so the import is
 * side-effect-light and `highlightViewButtons` is available to test directly.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { highlightViewButtons } from './app';

function makeViewBtn(view: string): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.className = 'view-btn';
  btn.setAttribute('data-view', view);
  return btn;
}

function buildViewToggle(): { semantic: HTMLElement; actionability: HTMLElement } {
  const toggle = document.createElement('div');
  toggle.className = 'view-toggle';
  const semantic = makeViewBtn('semantic');
  const actionability = makeViewBtn('actionability');
  toggle.append(semantic, actionability);
  document.body.append(toggle);
  return { semantic, actionability };
}

describe('highlightViewButtons', () => {
  beforeEach(() => {
    document.body.replaceChildren();
  });

  it('toggles view-btn--active onto the matching button', () => {
    const { semantic, actionability } = buildViewToggle();

    highlightViewButtons('actionability');

    expect(actionability.classList.contains('view-btn--active')).toBe(true);
    expect(semantic.classList.contains('view-btn--active')).toBe(false);
  });

  it('moves the active class when the mode changes', () => {
    const { semantic, actionability } = buildViewToggle();

    highlightViewButtons('semantic');
    expect(semantic.classList.contains('view-btn--active')).toBe(true);
    expect(actionability.classList.contains('view-btn--active')).toBe(false);

    highlightViewButtons('actionability');
    expect(semantic.classList.contains('view-btn--active')).toBe(false);
    expect(actionability.classList.contains('view-btn--active')).toBe(true);
  });

  it('clears a stale seed class off the non-matching button', () => {
    const { semantic, actionability } = buildViewToggle();
    // Simulate index.html's default seed on the semantic button.
    semantic.classList.add('view-btn--active');

    highlightViewButtons('actionability');

    expect(semantic.classList.contains('view-btn--active')).toBe(false);
    expect(actionability.classList.contains('view-btn--active')).toBe(true);
  });
});
