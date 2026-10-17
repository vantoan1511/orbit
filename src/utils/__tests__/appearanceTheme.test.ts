import assert from 'node:assert/strict'
import test from 'node:test'
import { syncAppearanceToDom, type AppearanceDomSyncOptions } from '../appearanceTheme.ts'

test('syncAppearanceToDom sets root data attributes and CSS variables on target element', () => {
  const attributes: Record<string, string> = {}
  const styleProperties: Record<string, string> = {}
  const classListSet = new Set<string>()

  const mockTarget = {
    classList: {
      toggle(cls: string, force?: boolean) {
        if (force === undefined) {
          if (classListSet.has(cls)) classListSet.delete(cls)
          else classListSet.add(cls)
        } else if (force) {
          classListSet.add(cls)
        } else {
          classListSet.delete(cls)
        }
      },
      contains(cls: string) {
        return classListSet.has(cls)
      }
    },
    setAttribute(name: string, value: string) {
      attributes[name] = value
    },
    getAttribute(name: string) {
      return attributes[name]
    },
    style: {
      setProperty(name: string, value: string) {
        styleProperties[name] = value
      },
      getPropertyValue(name: string) {
        return styleProperties[name] || ''
      }
    }
  }

  const options: AppearanceDomSyncOptions = {
    themeMode: 'dark',
    resolvedTheme: 'dark',
    accentColor: 'emerald',
    uiStyle: 'glass'
  }

  syncAppearanceToDom(options, mockTarget as unknown as HTMLElement)

  // Verify attributes
  assert.equal(attributes['data-theme'], 'dark')
  assert.equal(attributes['data-theme-mode'], 'dark')
  assert.equal(attributes['data-accent'], 'emerald')
  assert.equal(attributes['data-ui-style'], 'glass')
  assert.equal(mockTarget.classList.contains('my-app-dark'), true)

  // Verify core accent CSS variables
  assert.ok(styleProperties['--accent'], 'should set --accent')
  assert.ok(styleProperties['--accent-hover'], 'should set --accent-hover')
  assert.ok(styleProperties['--accent-active'], 'should set --accent-active')
  assert.ok(styleProperties['--accent-soft'], 'should set --accent-soft')
  assert.ok(styleProperties['--selection'], 'should set --selection')
  assert.equal(styleProperties['--input-focus'], styleProperties['--accent'])

  // Verify PrimeVue primary variables synchronization
  assert.ok(styleProperties['--p-primary-color'], 'should set --p-primary-color')
  assert.ok(styleProperties['--p-primary-hover-color'], 'should set --p-primary-hover-color')
  assert.ok(styleProperties['--p-primary-active-color'], 'should set --p-primary-active-color')
  assert.ok(styleProperties['--p-primary-contrast-color'], 'should set --p-primary-contrast-color')
  assert.equal(styleProperties['--p-focus-ring-color'], styleProperties['--accent'])
})

test('syncAppearanceToDom handles light mode accurately', () => {
  const attributes: Record<string, string> = {}
  const styleProperties: Record<string, string> = {}
  const classListSet = new Set<string>()

  const mockTarget = {
    classList: {
      toggle(cls: string, force?: boolean) {
        if (force) classListSet.add(cls)
        else classListSet.delete(cls)
      },
      contains(cls: string) {
        return classListSet.has(cls)
      }
    },
    setAttribute(name: string, value: string) {
      attributes[name] = value
    },
    style: {
      setProperty(name: string, value: string) {
        styleProperties[name] = value
      }
    }
  }

  const options: AppearanceDomSyncOptions = {
    themeMode: 'light',
    resolvedTheme: 'light',
    accentColor: 'violet',
    uiStyle: 'flat'
  }

  syncAppearanceToDom(options, mockTarget as unknown as HTMLElement)

  assert.equal(attributes['data-theme'], 'light')
  assert.equal(attributes['data-accent'], 'violet')
  assert.equal(mockTarget.classList.contains('my-app-dark'), false)
  assert.equal(styleProperties['--p-primary-contrast-color'], '#ffffff')
})

test('syncAppearanceToDom handles missing target element gracefully without error', () => {
  const options: AppearanceDomSyncOptions = {
    themeMode: 'system',
    resolvedTheme: 'dark',
    accentColor: 'cyan',
    uiStyle: 'aero'
  }

  assert.doesNotThrow(() => {
    syncAppearanceToDom(options, null)
  })
})
