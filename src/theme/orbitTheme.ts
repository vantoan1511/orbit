import { definePreset } from '@primeuix/themes'

import Nora from '@primeuix/themes/nora'
import type { Preset } from '@primeuix/themes/types'

const NoirPreset: Preset = {
  semantic: {
    focusRing: {
      width: '2px',
      style: 'dashed',
      color: '{primary.color}',
      offset: '2px'
    },
    primary: {
      50: '{zinc.50}',
      100: '{zinc.100}',
      200: '{zinc.200}',
      300: '{zinc.300}',
      400: '{zinc.400}',
      500: '{zinc.500}',
      600: '{zinc.600}',
      700: '{zinc.700}',
      800: '{zinc.800}',
      900: '{zinc.900}',
      950: '{zinc.950}'
    },
    colorScheme: {
      light: {
        primary: {
          color: '{zinc.950}',
          inverseColor: '#ffffff',
          hoverColor: '{zinc.900}',
          activeColor: '{zinc.800}'
        },
        highlight: {
          background: '{zinc.950}',
          focusBackground: '{zinc.700}',
          color: '#ffffff',
          focusColor: '#ffffff'
        }
      },
      dark: {
        primary: {
          color: '{slate.50}',
          inverseColor: '{slate.950}',
          hoverColor: '{slate.100}',
          activeColor: '{slate.200}'
        },
        highlight: {
          background: 'rgba(250, 250, 250, .16)',
          focusBackground: 'rgba(250, 250, 250, .24)',
          color: 'rgba(255,255,255,.87)',
          focusColor: 'rgba(255,255,255,.87)'
        }
      }
    }
  },
  components: {
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
        borderColor: '{surface.200}',
        color: '{text.muted.color}'
      },
      bodyCell: {
        borderColor: '{surface.100}'
      },
      row: {
        hoverBackground: '{surface.100}'
      },
      paginatorBottom: {
        borderColor: '{surface.200}',
        borderWidth: '1px 0 0 0'
      },
      colorScheme: {
        light: {
          root: {
            borderColor: '{surface.200}'
          },
          headerCell: {
            background: '{surface.100}',
            hoverBackground: '{surface.200}',
            borderColor: '{surface.200}',
            color: '{surface.600}'
          },
          bodyCell: {
            borderColor: '{surface.100}'
          },
          row: {
            hoverBackground: '{surface.100}'
          },
          paginatorBottom: {
            borderColor: '{surface.200}',
            borderWidth: '1px 0 0 0'
          }
        },
        dark: {
          root: {
            borderColor: '{surface.800}'
          },
          headerCell: {
            background: '{surface.900}',
            hoverBackground: '{surface.800}',
            borderColor: '{surface.800}',
            color: '{surface.400}'
          },
          bodyCell: {
            borderColor: '{surface.900}'
          },
          row: {
            hoverBackground: '{surface.800}'
          },
          paginatorBottom: {
            borderColor: '{surface.800}',
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
        borderRadius: '{content.border.radius}'
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
            borderColor: '{surface.200}',
            color: '{surface.900}',
            shadow: '{overlay.popover.shadow}'
          }
        },
        dark: {
          root: {
            background: '{surface.900}',
            borderColor: '{surface.800}',
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
        borderRadius: '{content.border.radius}',
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
