import { definePreset } from '@primeuix/themes'
import type { StyleOptions } from '@primeuix/styled'
import Nora from '@primeuix/themes/nora'
import {
  OVERLAY_SURFACES,
  OVERLAY_SURFACE_TOKENS,
  SIZED_CONTROLS,
  buildControlSizingCss,
  buildOverlaySurfaceCss
} from './uiContract.ts'

declare module '@primeuix/themes/types/card' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace CardTokenSections {
    interface Root {
      borderColor?: string
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/datatable' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace DataTableTokenSections {
    interface Root {
      borderColor?: string
    }
  }
}

declare module '@primeuix/themes/types/button' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace ButtonTokenSections {
    interface Root {
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/dialog' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace DialogTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/drawer' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace DrawerTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/popover' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace PopoverTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/select' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace SelectTokenSections {
    interface Overlay {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/multiselect' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace MultiSelectTokenSections {
    interface Overlay {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/tieredmenu' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace TieredMenuTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/toast' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace ToastTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/contextmenu' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace ContextMenuTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/menu' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace MenuTokenSections {
    interface Root {
      backdropBlur?: string
      sheenHighlight?: string
    }
  }
}

declare module '@primeuix/themes/types/tag' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace TagTokenSections {
    interface Root {
      backdropBlur?: string
    }
  }
}

declare module '@primeuix/themes/types/badge' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace BadgeTokenSections {
    interface Root {
      backdropBlur?: string
    }
  }
}

const MENU_ITEM_TOKENS = {
  focusBackground: 'var(--bg-hover)',
  borderRadius: 'var(--ui-radius-control)',
  color: '{text.color}',
  focusColor: '{text.color}'
}

function buildToastSeverity(statusVar: string, softVar: string) {
  return {
    background: `linear-gradient(${softVar}, ${softVar}), var(--ui-overlay-bg)`,
    borderColor: `var(--border) var(--border) var(--border) ${statusVar}`,
    color: statusVar,
    detailColor: 'var(--text-secondary)',
    shadow: 'var(--ui-card-shadow, var(--shadow))',
    closeButton: {
      hoverBackground: 'var(--bg-hover)',
      focusRing: { color: statusVar, shadow: 'none' }
    }
  }
}

function buildToastCss({ dt }: StyleOptions): string {
  const blur = String(dt('toast.root.backdrop.blur') ?? '')
  return `
.p-toast-message,
.p-toast-message.p-toast-message-info,
.p-toast-message.p-toast-message-success,
.p-toast-message.p-toast-message-warn,
.p-toast-message.p-toast-message-error,
.p-toast-message.p-toast-message-secondary,
.p-toast-message.p-toast-message-contrast {
  -webkit-backdrop-filter: ${blur};
  backdrop-filter: ${blur};
}
`
}

const TOAST_COLOR_SCHEME = {
  info: buildToastSeverity('var(--info)', 'var(--info-soft)'),
  success: buildToastSeverity('var(--success)', 'var(--success-soft)'),
  warn: buildToastSeverity('var(--warning)', 'var(--warning-soft)'),
  error: buildToastSeverity('var(--danger)', 'var(--danger-soft)'),
  secondary: {
    background: 'var(--ui-overlay-bg)',
    borderColor: 'var(--border)',
    color: 'var(--text-primary)',
    detailColor: 'var(--text-secondary)',
    shadow: 'var(--ui-card-shadow, var(--shadow))',
    closeButton: {
      hoverBackground: 'var(--bg-hover)',
      focusRing: { color: 'var(--border-strong)', shadow: 'none' }
    }
  },
  contrast: {
    background: 'var(--ui-overlay-bg)',
    borderColor: 'var(--border-strong)',
    color: 'var(--text-primary)',
    detailColor: 'var(--text-secondary)',
    shadow: 'var(--ui-card-shadow, var(--shadow))',
    closeButton: {
      hoverBackground: 'var(--bg-hover)',
      focusRing: { color: 'var(--border-strong)', shadow: 'none' }
    }
  }
}

function buildBadgeSeverity(statusVar: string, softVar: string) {
  return {
    background: softVar,
    color: statusVar
  }
}

const BADGE_COLOR_SCHEME = {
  primary: buildBadgeSeverity('var(--accent)', 'var(--accent-soft)'),
  secondary: {
    background: 'var(--bg-hover)',
    color: 'var(--text-secondary)'
  },
  success: buildBadgeSeverity('var(--success)', 'var(--success-soft)'),
  info: buildBadgeSeverity('var(--info)', 'var(--info-soft)'),
  warn: buildBadgeSeverity('var(--warning)', 'var(--warning-soft)'),
  danger: buildBadgeSeverity('var(--danger)', 'var(--danger-soft)'),
  contrast: {
    background: 'var(--text-primary)',
    color: 'var(--bg-app)'
  }
}

function buildTagCss({ dt }: StyleOptions): string {
  const blur = String(dt('tag.root.backdrop.blur') ?? 'var(--ui-glass-blur)')
  return `
.p-tag {
  border: 1px solid color-mix(in srgb, currentColor 20%, transparent);
  -webkit-backdrop-filter: ${blur};
  backdrop-filter: ${blur};
}

.p-tag.p-tag-secondary {
  border-color: var(--border);
}

.p-tag.p-tag-contrast {
  border-color: transparent;
}
`
}

function buildBadgeCss({ dt }: StyleOptions): string {
  const blur = String(dt('badge.root.backdrop.blur') ?? 'var(--ui-glass-blur)')
  return `
.p-badge {
  border: 1px solid color-mix(in srgb, currentColor 20%, transparent);
  -webkit-backdrop-filter: ${blur};
  backdrop-filter: ${blur};
}

.p-badge-dot {
  border: none;
}

.p-badge.p-badge-secondary {
  border-color: var(--border);
}

.p-badge.p-badge-contrast {
  border-color: transparent;
}
`
}

function componentCss(component: string) {
  const overlaySpecs = OVERLAY_SURFACES.filter((s) => s.component === component)
  const sizingSpecs = SIZED_CONTROLS.filter((s) => s.component === component)
  if (overlaySpecs.length === 0 && sizingSpecs.length === 0) return undefined

  return ({ dt }: StyleOptions): string => {
    const parts: string[] = []
    for (const spec of overlaySpecs) {
      parts.push(buildOverlaySurfaceCss(spec, dt))
    }
    for (const spec of sizingSpecs) {
      parts.push(buildControlSizingCss(spec))
    }
    return parts.join('\n')
  }
}

function overlayTokens(): Record<string, Record<string, unknown>> {
  const result: Record<string, Record<string, unknown>> = {}
  for (const spec of OVERLAY_SURFACES) {
    result[spec.component] = {
      [spec.tokenGroup]: { ...OVERLAY_SURFACE_TOKENS },
      css: componentCss(spec.component)
    }
  }
  return result
}

const overlays = overlayTokens()

export const Noir = definePreset(Nora, {
  primitive: {
    primary: {
      50: '#eef4ff',
      100: '#d9e6ff',
      200: '#bcd4ff',
      300: '#8eb8ff',
      400: '#6aa8ff',
      500: '#4f8cff',
      600: '#3c78e8',
      700: '#2d62cc',
      800: '#284fa5',
      900: '#254482',
      950: '#1a2b50'
    }
  },
  semantic: {
    borderRadius: {
      none: '0',
      xs: 'var(--ui-radius-control)',
      sm: 'var(--ui-radius-control)',
      md: 'var(--ui-radius-control)',
      lg: 'var(--ui-radius-card)',
      xl: 'var(--ui-radius-card)'
    },
    primary: {
      50: '{primary.50}',
      100: '{primary.100}',
      200: '{primary.200}',
      300: '{primary.300}',
      400: '{primary.400}',
      500: '{primary.500}',
      600: '{primary.600}',
      700: '{primary.700}',
      800: '{primary.800}',
      900: '{primary.900}',
      950: '{primary.950}'
    },
    focusRing: {
      width: '2px',
      style: 'solid',
      color: '{primary.color}',
      offset: '2px'
    },
    formField: {
      paddingX: '0.625rem',
      paddingY: '0.375rem',
      background: 'var(--ui-control-bg)',
      borderRadius: 'var(--ui-radius-control)',
      borderColor: 'var(--input-border, var(--border))',
      hoverBorderColor: 'var(--border-strong)',
      focusBorderColor: '{primary.color}',
      focusRing: {
        width: '0',
        style: 'none',
        color: 'transparent',
        offset: '0'
      }
    },
    content: {
      background: 'var(--ui-card-bg)',
      hoverBackground: 'var(--bg-hover)',
      borderRadius: 'var(--ui-radius-card)',
      borderColor: 'var(--border)'
    },
    overlay: {
      select: {
        background: 'var(--ui-overlay-bg)',
        borderRadius: 'var(--ui-radius-card)',
        borderColor: 'var(--border)',
        shadow: 'var(--ui-card-shadow, var(--shadow))'
      },
      popover: {
        background: 'var(--ui-overlay-bg)',
        borderRadius: 'var(--ui-radius-card)',
        borderColor: 'var(--border)',
        shadow: 'var(--ui-card-shadow, var(--shadow))'
      },
      modal: {
        background: 'var(--ui-dialog-bg)',
        borderRadius: 'var(--ui-radius-card)',
        borderColor: 'var(--border)',
        shadow: 'var(--ui-card-shadow, var(--shadow))'
      }
    },
    colorScheme: {
      light: {
        mask: {
          background: 'var(--ui-mask-bg)'
        },
        primary: {
          color: '{primary.500}',
          contrastColor: '#ffffff',
          hoverColor: '{primary.600}',
          activeColor: '{primary.700}'
        },
        highlight: {
          background: '{primary.500}',
          focusBackground: '{primary.600}',
          color: '#ffffff',
          focusColor: '#ffffff'
        },
        surface: {
          0: '#ffffff',
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          300: '#cbd5e1',
          400: '#94a3b8',
          500: '#64748b',
          600: '#475569',
          700: '#334155',
          800: '#1e293b',
          900: '#0f172a',
          950: '#020617'
        },
        content: {
          background: 'var(--ui-card-bg)',
          hoverBackground: 'var(--bg-hover)',
          borderColor: 'var(--border)'
        },
        formField: {
          background: 'var(--ui-control-bg)',
          borderColor: 'var(--input-border, var(--border))',
          hoverBorderColor: 'var(--border-strong)',
          focusBorderColor: '{primary.color}'
        },
        overlay: {
          select: {
            background: 'var(--ui-overlay-bg)',
            borderColor: 'var(--border)'
          },
          popover: {
            background: 'var(--ui-overlay-bg)',
            borderColor: 'var(--border)'
          },
          modal: {
            background: 'var(--ui-dialog-bg)',
            borderColor: 'var(--border)'
          }
        }
      },
      dark: {
        mask: {
          background: 'var(--ui-mask-bg)'
        },
        primary: {
          color: '{primary.400}',
          contrastColor: '{surface.950}',
          hoverColor: '{primary.300}',
          activeColor: '{primary.200}'
        },
        highlight: {
          background: '{primary.400}',
          focusBackground: '{primary.300}',
          color: '{surface.950}',
          focusColor: '{surface.950}'
        },
        surface: {
          0: '#ffffff',
          50: '#f4f4f5',
          100: '#e4e4e7',
          200: '#d4d4d8',
          300: '#a1a1aa',
          400: '#71717a',
          500: '#52525b',
          600: '#3f3f46',
          700: '#27272a',
          800: '#18181b',
          900: '#0e1012',
          950: '#0b0d0f'
        },
        content: {
          background: 'var(--ui-card-bg)',
          hoverBackground: 'var(--bg-hover)',
          borderColor: 'var(--border)'
        },
        formField: {
          background: 'var(--ui-control-bg)',
          borderColor: 'var(--input-border, var(--border))',
          hoverBorderColor: 'var(--border-strong)',
          focusBorderColor: '{primary.color}'
        },
        overlay: {
          select: {
            background: 'var(--ui-overlay-bg)',
            borderColor: 'var(--border)'
          },
          popover: {
            background: 'var(--ui-overlay-bg)',
            borderColor: 'var(--border)'
          },
          modal: {
            background: 'var(--ui-dialog-bg)',
            borderColor: 'var(--border)'
          }
        }
      }
    }
  },
  components: {
    card: {
      root: {
        background: 'var(--ui-card-bg)',
        borderColor: 'var(--border)',
        borderRadius: 'var(--ui-radius-card)',
        shadow: 'var(--ui-card-shadow, none)',
        backdropBlur: 'var(--ui-glass-blur)',
        sheenHighlight: 'var(--ui-sheen-highlight)'
      },
      body: {
        padding: '1.25rem',
        gap: '0.75rem'
      },
      caption: {
        gap: '0.25rem'
      },
      title: {
        fontSize: '0.875rem',
        fontWeight: '600'
      },
      subtitle: {
        color: '{text.muted.color}'
      }
    },
    datatable: {
      root: {
        borderColor: 'var(--border)'
      },
      headerCell: {
        padding: '0.625rem 0.75rem',
        borderColor: 'var(--border)'
      },
      bodyCell: {
        padding: '0.5rem 0.75rem',
        borderColor: 'var(--border-subtle, var(--border))'
      },
      row: {
        background: 'var(--ui-card-bg)',
        hoverBackground: 'var(--bg-hover)'
      },
      colorScheme: {
        light: {
          root: {
            borderColor: 'var(--border)'
          },
          headerCell: {
            background: '{surface.100}',
            hoverBackground: '{surface.200}',
            color: '{text.muted.color}',
            borderColor: 'var(--border)'
          },
          row: {
            background: 'var(--ui-card-bg)',
            hoverBackground: 'var(--bg-hover)'
          },
          bodyCell: {
            borderColor: 'var(--border-subtle, var(--border))'
          }
        },
        dark: {
          root: {
            borderColor: 'var(--border)'
          },
          headerCell: {
            background: '{surface.900}',
            hoverBackground: '{surface.800}',
            color: '{text.muted.color}',
            borderColor: 'var(--border)'
          },
          row: {
            background: 'var(--ui-card-bg)',
            hoverBackground: 'var(--bg-hover)'
          },
          bodyCell: {
            borderColor: 'var(--border-subtle, var(--border))'
          }
        }
      }
    },
    button: {
      root: {
        paddingX: 'var(--ui-control-padding-x-md)',
        paddingY: '0.375rem',
        borderRadius: 'var(--ui-radius-control)',
        sheenHighlight: 'var(--ui-sheen-highlight)',
        iconOnlyWidth: 'var(--ui-control-height-md)',
        sm: {
          fontSize: 'var(--ui-control-font-size-sm)',
          paddingX: 'var(--ui-control-padding-x-sm)',
          paddingY: '0.25rem',
          iconOnlyWidth: 'var(--ui-control-height-sm)'
        }
      },
      colorScheme: {
        light: {
          root: {
            secondary: {
              background: 'var(--ui-control-bg)',
              hoverBackground: 'var(--bg-hover)',
              activeBackground: 'var(--bg-active)',
              borderColor: 'var(--border)',
              hoverBorderColor: 'var(--border-strong)',
              activeBorderColor: 'var(--border-strong)',
              color: '{text.color}'
            }
          }
        },
        dark: {
          root: {
            secondary: {
              background: 'var(--ui-control-bg)',
              hoverBackground: 'var(--bg-hover)',
              activeBackground: 'var(--bg-active)',
              borderColor: 'var(--border)',
              hoverBorderColor: 'var(--border-strong)',
              activeBorderColor: 'var(--border-strong)',
              color: '{text.color}'
            }
          }
        }
      },
      css: componentCss('button')
    },
    select: {
      root: {
        background: 'var(--ui-control-bg)',
        paddingX: 'var(--ui-control-padding-x-md)',
        paddingY: '0.375rem',
        borderColor: 'var(--input-border, var(--border))',
        borderRadius: 'var(--ui-radius-control)',
        sm: {
          fontSize: 'var(--ui-control-font-size-sm)',
          paddingX: 'var(--ui-control-padding-x-sm)',
          paddingY: '0.25rem'
        }
      },
      ...overlays.select,
      option: {
        padding: '0.375rem 0.625rem',
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    inputtext: {
      root: {
        background: 'var(--ui-control-bg)',
        paddingX: 'var(--ui-control-padding-x-md)',
        paddingY: '0.375rem',
        borderColor: 'var(--input-border, var(--border))',
        borderRadius: 'var(--ui-radius-control)',
        sm: {
          fontSize: 'var(--ui-control-font-size-sm)',
          paddingX: 'var(--ui-control-padding-x-sm)',
          paddingY: '0.25rem'
        }
      },
      css: componentCss('inputtext')
    },
    textarea: {
      root: {
        background: 'var(--ui-control-bg)',
        paddingX: 'var(--ui-control-padding-x-md)',
        paddingY: '0.375rem',
        borderColor: 'var(--input-border, var(--border))',
        borderRadius: 'var(--ui-radius-control)',
        sm: {
          fontSize: 'var(--ui-control-font-size-sm)',
          paddingX: 'var(--ui-control-padding-x-sm)',
          paddingY: '0.25rem'
        }
      },
      css: componentCss('textarea')
    },
    inputnumber: {
      button: {
        background: 'var(--ui-control-bg)',
        borderColor: 'var(--input-border, var(--border))',
        borderRadius: 'var(--ui-radius-control)'
      },
      css: componentCss('inputnumber')
    },
    multiselect: {
      root: {
        background: 'var(--ui-control-bg)',
        paddingX: 'var(--ui-control-padding-x-md)',
        paddingY: '0.375rem',
        borderColor: 'var(--input-border, var(--border))',
        borderRadius: 'var(--ui-radius-control)',
        sm: {
          fontSize: 'var(--ui-control-font-size-sm)',
          paddingX: 'var(--ui-control-padding-x-sm)',
          paddingY: '0.25rem'
        }
      },
      ...overlays.multiselect,
      option: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    checkbox: {
      root: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    popover: {
      ...overlays.popover
    },
    drawer: {
      root: {
        background: 'var(--ui-card-bg)',
        borderColor: 'var(--border)',
        shadow: 'var(--ui-card-shadow, var(--shadow))',
        backdropBlur: 'var(--ui-glass-blur)',
        sheenHighlight: 'var(--ui-sheen-highlight)'
      }
    },
    contextmenu: {
      ...overlays.contextmenu,
      item: MENU_ITEM_TOKENS
    },
    menu: {
      ...overlays.menu,
      item: MENU_ITEM_TOKENS
    },
    tieredmenu: {
      ...overlays.tieredmenu,
      item: MENU_ITEM_TOKENS
    },
    tabs: {
      tab: {
        padding: '0.5rem 0.875rem',
        fontWeight: '500',
        borderColor: 'var(--border)'
      },
      tablist: {
        background: 'transparent',
        borderColor: 'var(--border)'
      },
      tabpanel: {
        background: 'transparent',
        padding: '0.75rem 0'
      }
    },
    paginator: {
      root: {
        padding: '0.375rem 0.75rem',
        borderRadius: 'var(--ui-radius-card)'
      },
      navButton: {
        width: '1.75rem',
        height: '1.75rem',
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    dialog: {
      root: {
        background: 'var(--ui-dialog-bg)',
        borderColor: 'var(--border)',
        borderRadius: 'var(--ui-radius-card)',
        shadow: 'var(--ui-card-shadow, var(--shadow))',
        backdropBlur: 'var(--ui-dialog-blur)',
        sheenHighlight: 'var(--ui-sheen-highlight)'
      },
      header: {
        padding: '1rem 1.25rem'
      },
      content: {
        padding: '0 1.25rem 1.25rem 1.25rem'
      },
      footer: {
        padding: '0 1.25rem 1rem 1.25rem'
      },
      colorScheme: {
        light: {
          root: {
            background: 'var(--ui-dialog-bg)',
            borderColor: 'var(--border)',
            color: '{text.color}'
          }
        },
        dark: {
          root: {
            background: 'var(--ui-dialog-bg)',
            borderColor: 'var(--border)',
            color: '{text.color}'
          }
        }
      }
    },
    toast: {
      root: {
        width: '24rem',
        borderRadius: 'var(--ui-radius-card)',
        borderWidth: '1px 1px 1px 3px',
        transitionDuration: '{transition.duration}',
        backdropBlur: 'var(--ui-glass-blur)',
        sheenHighlight: 'var(--ui-sheen-highlight)'
      },
      icon: {
        size: '1rem'
      },
      content: {
        padding: '0.75rem 0.875rem',
        gap: '0.625rem'
      },
      text: {
        gap: '0.25rem'
      },
      summary: {
        fontWeight: '600',
        fontSize: '0.875rem'
      },
      detail: {
        fontWeight: '400',
        fontSize: '0.75rem'
      },
      closeButton: {
        width: '1.5rem',
        height: '1.5rem',
        borderRadius: 'var(--ui-radius-control)',
        focusRing: {
          width: '1px',
          style: 'solid',
          offset: '1px'
        }
      },
      closeIcon: {
        size: '0.875rem'
      },
      colorScheme: {
        light: TOAST_COLOR_SCHEME,
        dark: TOAST_COLOR_SCHEME
      },
      css: buildToastCss
    },
    tooltip: {
      root: {
        padding: '0.25rem 0.5rem',
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    tag: {
      root: {
        fontSize: '0.75rem',
        fontWeight: '500',
        padding: '0.125rem 0.5rem',
        borderRadius: 'var(--ui-radius-control)',
        roundedBorderRadius: '9999px',
        gap: '0.25rem',
        backdropBlur: 'var(--ui-glass-blur)'
      },
      icon: {
        size: '0.75rem'
      },
      colorScheme: {
        light: BADGE_COLOR_SCHEME,
        dark: BADGE_COLOR_SCHEME
      },
      css: buildTagCss
    },
    badge: {
      root: {
        borderRadius: 'var(--ui-radius-control)',
        padding: '0 0.375rem',
        fontSize: '0.6875rem',
        fontWeight: '600',
        minWidth: '1.25rem',
        height: '1.25rem',
        backdropBlur: 'var(--ui-glass-blur)'
      },
      dot: {
        size: '0.5rem'
      },
      sm: {
        fontSize: '0.625rem',
        minWidth: '1rem',
        height: '1rem'
      },
      lg: {
        fontSize: '0.875rem',
        minWidth: '1.5rem',
        height: '1.5rem'
      },
      xl: {
        fontSize: '1rem',
        minWidth: '1.75rem',
        height: '1.75rem'
      },
      colorScheme: {
        light: BADGE_COLOR_SCHEME,
        dark: BADGE_COLOR_SCHEME
      },
      css: buildBadgeCss
    }
  }
})

export const NoirPreset = Noir
