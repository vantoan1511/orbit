---
name: orbit-ui-review
description: Reviews Orbit UI changes for visual consistency, PrimeVue/Nora consistency, component reuse, styling duplication, and adherence to the Orbit technical desktop design. Use after implementing or modifying UI.
---

# Orbit UI Review

## Purpose

This is a consistency gate, not a general code review.

The primary question is:

> Does this change look and behave like it belongs to the existing Orbit UI system?

## Review Procedure

### 1. Compare With Existing UI

Find the closest existing screen or component.

Compare:

- page hierarchy
- sidebar/navigation
- toolbar
- spacing
- control dimensions
- typography
- table density
- borders
- radius
- semantic colors
- empty/loading/error states

### 2. PrimeVue Consistency

For every PrimeVue component touched:

- locate its canonical styling
- check whether the change duplicates an existing customization
- check whether a local override should be moved to the shared theme/component layer
- check whether the change accidentally creates a second variant

### 3. Search for Duplication

Search the codebase for:

- duplicated PrimeVue selectors
- repeated dimensions
- repeated component-specific CSS
- copied style blocks
- page-specific versions of common controls

Pay special attention to `.p-*` selectors and PrimeVue component names.

### 4. Evaluate Design Character

Reject changes that make Orbit more:

- card-heavy
- spacious
- rounded
- colorful
- decorative
- SaaS-like

unless there is a clear product reason.

### 5. Check Reuse

Ask:

- Was an existing component available?
- Was an existing pattern available?
- Was an existing token available?
- Was an existing PrimeVue customization available?

If yes and it was not reused, determine whether there is a real reason.

## Findings

Classify issues:

**Blocker**
- introduces conflicting global PrimeVue styling
- duplicates an existing component pattern
- breaks the established visual system

**Important**
- creates unnecessary local styling
- introduces inconsistent spacing/density
- creates a new visual variant without semantic justification

**Minor**
- small alignment/spacing inconsistency
- naming or organization issue

## Final Verdict

A UI change should be considered ready only when:

- visual hierarchy is consistent
- PrimeVue/Nora remains canonical
- shared components are reused
- no duplicate component styling exists
- no accidental visual variant exists
- the implementation is maintainable
