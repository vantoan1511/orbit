import { definePreset } from '@primeuix/themes'

import Nora from '@primeuix/themes/nora'
import type { Preset } from '@primeuix/themes/types'

declare module '@primeuix/themes/types/card' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace CardTokenSections {
    interface Root {
      borderColor?: string
    }
  }
}

const NoirPreset: Preset = {
  semantic: {
    borderRadius: {
      none: '0',
      xs: 'var(--ui-radius-control)',
      sm: 'var(--ui-radius-control)',
      md: 'var(--ui-radius-control)',
      lg: 'var(--ui-radius-card)',
      xl: 'var(--ui-radius-card)'
    },
    formField: {
      borderRadius: 'var(--ui-radius-control)',
      borderColor: 'var(--input-border, var(--border))',
      hoverBorderColor: 'var(--border-strong)',
      focusBorderColor: '{primary.color}'
    },
    content: {
      borderRadius: 'var(--ui-radius-card)',
      borderColor: 'var(--border)'
    },
    overlay: {
      select: {
        borderRadius: 'var(--ui-radius-control)',
        borderColor: 'var(--border)'
      },
      popover: {
        borderRadius: 'var(--ui-radius-card)',
        borderColor: 'var(--border)'
      },
      modal: {
        borderRadius: 'var(--ui-radius-card)',
        borderColor: 'var(--border)'
      }
    },
    focusRing: {
      width: '2px',
      style: 'dashed',
      color: '{primary.color}',
      offset: '2px'
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
    colorScheme: {
      light: {
        content: {
          borderColor: 'var(--border)'
        },
        formField: {
          borderColor: 'var(--input-border, var(--border))',
          hoverBorderColor: 'var(--border-strong)',
          focusBorderColor: '{primary.color}'
        },
        overlay: {
          select: {
            borderColor: 'var(--border)'
          },
          popover: {
            borderColor: 'var(--border)'
          },
          modal: {
            borderColor: 'var(--border)'
          }
        },
        primary: {
          color: '{primary.500}',
          contrastColor: '#ffffff',
          inverseColor: '#ffffff',
          hoverColor: '{primary.600}',
          activeColor: '{primary.700}'
        },
        highlight: {
          background: 'color-mix(in srgb, {primary.color}, transparent 88%)',
          focusBackground: 'color-mix(in srgb, {primary.color}, transparent 80%)',
          color: '{primary.color}',
          focusColor: '{primary.color}'
        }
      },
      dark: {
        content: {
          borderColor: 'var(--border)'
        },
        formField: {
          borderColor: 'var(--input-border, var(--border))',
          hoverBorderColor: 'var(--border-strong)',
          focusBorderColor: '{primary.color}'
        },
        overlay: {
          select: {
            borderColor: 'var(--border)'
          },
          popover: {
            borderColor: 'var(--border)'
          },
          modal: {
            borderColor: 'var(--border)'
          }
        },
        primary: {
          color: '{primary.400}',
          contrastColor: '{slate.950}',
          inverseColor: '{slate.950}',
          hoverColor: '{primary.300}',
          activeColor: '{primary.200}'
        },
        highlight: {
          background: 'color-mix(in srgb, {primary.color}, transparent 86%)',
          focusBackground: 'color-mix(in srgb, {primary.color}, transparent 78%)',
          color: '{primary.color}',
          focusColor: '{primary.color}'
        }
      }
    }
  },
  components: {
    card: {
      root: {
        background: 'var(--bg-card)',
        borderColor: 'var(--border)',
        borderRadius: 'var(--ui-radius-card)',
        shadow: 'var(--ui-card-shadow, none)'
      }
    },
    button: {
      root: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    select: {
      root: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    inputtext: {
      root: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    textarea: {
      root: {
        borderRadius: 'var(--ui-radius-control)'
      }
    },
    popover: {
      root: {
        borderColor: 'var(--border)',
        borderRadius: 'var(--ui-radius-card)'
      }
    },
    drawer: {
      root: {
        borderColor: 'var(--border)'
      }
    },
    contextmenu: {
      item: {
        focusBackground: 'transparent',
        activeBackground: 'transparent'
      }
    },
    menu: {
      item: {
        focusBackground: 'transparent'
      }
    },
    datatable: {
      header: {
        background: 'transparent',
        borderColor: 'transparent'
      },
      headerCell: {
        borderColor: 'var(--border)',
        color: '{text.muted.color}'
      },
      bodyCell: {
        borderColor: 'var(--border)'
      },
      row: {
        hoverBackground: '{surface.100}'
      },
      paginatorBottom: {
        borderColor: 'var(--border)',
        borderWidth: '1px 0 0 0'
      },
      colorScheme: {
        light: {
          root: {
            borderColor: 'var(--border)'
          },
          headerCell: {
            background: '{surface.100}',
            hoverBackground: '{surface.200}',
            borderColor: 'var(--border)',
            color: '{surface.600}'
          },
          bodyCell: {
            borderColor: 'var(--border)'
          },
          row: {
            hoverBackground: '{surface.100}'
          },
          paginatorBottom: {
            borderColor: 'var(--border)',
            borderWidth: '1px 0 0 0'
          }
        },
        dark: {
          root: {
            borderColor: 'var(--border)'
          },
          headerCell: {
            background: '{surface.900}',
            hoverBackground: '{surface.800}',
            borderColor: 'var(--border)',
            color: '{surface.400}'
          },
          bodyCell: {
            borderColor: 'var(--border)'
          },
          row: {
            hoverBackground: '{surface.800}'
          },
          paginatorBottom: {
            borderColor: 'var(--border)',
            borderWidth: '1px 0 0 0'
          }
        }
      }
    },
    toast: {
      root: {
        borderWidth: '1px',
        borderRadius: '{content.border.radius}'
      },
      icon: {
        size: '1rem'
      },
      content: {
        padding: '0.75rem 1rem',
        gap: '0.75rem'
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
        borderRadius: '{content.border.radius}'
      },
      closeIcon: {
        size: '0.875rem'
      },
      colorScheme: {
        light: {
          info: {
            background: '{surface.0}',
            borderColor: '{surface.200}',
            color: '{blue.600}',
            detailColor: '{surface.600}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.600}', shadow: 'none' }
            }
          },
          success: {
            background: '{surface.0}',
            borderColor: '{surface.200}',
            color: '{green.600}',
            detailColor: '{surface.600}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.600}', shadow: 'none' }
            }
          },
          warn: {
            background: '{surface.0}',
            borderColor: '{surface.200}',
            color: '{yellow.600}',
            detailColor: '{surface.600}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.600}', shadow: 'none' }
            }
          },
          error: {
            background: '{surface.0}',
            borderColor: '{surface.200}',
            color: '{red.600}',
            detailColor: '{surface.600}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.600}', shadow: 'none' }
            }
          },
          secondary: {
            background: '{surface.0}',
            borderColor: '{surface.200}',
            color: '{surface.600}',
            detailColor: '{surface.500}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.600}', shadow: 'none' }
            }
          },
          contrast: {
            background: '{surface.900}',
            borderColor: '{surface.900}',
            color: '{surface.50}',
            detailColor: '{surface.300}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.50}', shadow: 'none' }
            }
          }
        },
        dark: {
          info: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
            color: '{blue.400}',
            detailColor: '{surface.400}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.400}', shadow: 'none' }
            }
          },
          success: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
            color: '{green.400}',
            detailColor: '{surface.400}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.400}', shadow: 'none' }
            }
          },
          warn: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
            color: '{yellow.400}',
            detailColor: '{surface.400}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.400}', shadow: 'none' }
            }
          },
          error: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
            color: '{red.400}',
            detailColor: '{surface.400}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.400}', shadow: 'none' }
            }
          },
          secondary: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
            color: '{surface.400}',
            detailColor: '{surface.500}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.800}',
              focusRing: { color: '{surface.400}', shadow: 'none' }
            }
          },
          contrast: {
            background: '{surface.0}',
            borderColor: '{surface.0}',
            color: '{surface.950}',
            detailColor: '{surface.600}',
            shadow: '{overlay.popover.shadow}',
            closeButton: {
              hoverBackground: '{surface.100}',
              focusRing: { color: '{surface.950}', shadow: 'none' }
            }
          }
        }
      }
    },
    dialog: {
      root: {
        borderColor: 'var(--border)',
        borderRadius: 'var(--ui-radius-card)'
      },
      header: {
        padding: '1rem 1.25rem 0.75rem 1.25rem',
        gap: '0.5rem'
      },
      title: {
        fontSize: '1rem',
        fontWeight: '700'
      },
      content: {
        padding: '0 1.25rem 1rem 1.25rem'
      },
      footer: {
        padding: '0.75rem 1.25rem 1.25rem 1.25rem',
        gap: '0.5rem'
      },
      colorScheme: {
        light: {
          root: {
            background: '{surface.0}',
            borderColor: 'var(--border)',
            color: '{surface.900}',
            shadow: '{overlay.popover.shadow}'
          }
        },
        dark: {
          root: {
            background: '{surface.900}',
            borderColor: 'var(--border)',
            color: '{surface.50}',
            shadow: '{overlay.popover.shadow}'
          }
        }
      }
    },
    confirmdialog: {
      icon: {
        size: '1.25rem'
      },
      content: {
        gap: '0.75rem'
      }
    },
    tooltip: {
      root: {
        padding: '0.25rem 0.5rem'
      }
    },
    tag: {
      root: {
        fontSize: '0.625rem',
        fontWeight: '600',
        padding: '0.125rem 0.375rem',
        borderRadius: 'var(--ui-radius-control)',
        gap: '0.25rem'
      },
      colorScheme: {
        light: {
          secondary: {
            background: '{surface.100}',
            color: '{surface.700}'
          },
          info: {
            background: 'rgba(79, 140, 255, 0.12)',
            color: '#2563eb'
          },
          success: {
            background: 'rgba(40, 167, 69, 0.12)',
            color: '#16a34a'
          },
          warn: {
            background: 'rgba(244, 161, 0, 0.12)',
            color: '#d97706'
          },
          danger: {
            background: 'rgba(214, 69, 69, 0.12)',
            color: '#dc2626'
          },
          contrast: {
            background: '{surface.950}',
            color: '{surface.0}'
          }
        },
        dark: {
          secondary: {
            background: '{surface.800}',
            color: '{surface.300}'
          },
          info: {
            background: 'rgba(106, 168, 255, 0.14)',
            color: '#60a5fa'
          },
          success: {
            background: 'rgba(16, 185, 129, 0.14)',
            color: '#34d399'
          },
          warn: {
            background: 'rgba(245, 158, 11, 0.14)',
            color: '#fbbf24'
          },
          danger: {
            background: 'rgba(244, 63, 94, 0.14)',
            color: '#f87171'
          },
          contrast: {
            background: '{surface.0}',
            color: '{surface.950}'
          }
        }
      }
    }
  }
}

export const Noir = definePreset(Nora, NoirPreset)
