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
    }
  }
}

export const Noir = definePreset(Nora, NoirPreset)
