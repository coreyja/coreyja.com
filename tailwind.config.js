const defaultColors = require('tailwindcss/colors')

const colors = {
  grey: {
    100: '#EFEDF2',
    200: '#E3DFEB',
    300: '#D9D3E3',
    400: '#C8C0D7',
    500: '#B0A9BD',
    600: '#9D97A6',
    700: '#908C96',
    800: '#6B6870',
    900: '#4C494F',
    999: '#202024',
  },
  primary: {
    100: '#F9F7FF',
    200: '#F2EDFF',
    300: '#EAE0FF',
    400: '#E1D4FF',
    500: '#AE93ED',
  },
  secondary: {
    100: '#EFE5FF',
    200: '#D9C2FF',
    300: '#C9A8FF',
    400: '#B284FF',
    500: '#874EE5',
  },
  success: {
    100: '#B7F2B6',
    200: '#82E581',
    300: '#52CC50',
    400: '#37A835',
    500: '#217D1F',
  },
  warning: {
    100: '#FFF3C9',
    200: '#FAE496',
    300: '#F0D678',
    400: '#F0CA44',
    500: '#E1B000',
  },
  error: {
    100: '#FFD5CC',
    200: '#FFAC99',
    300: '#FA8970',
    400: '#F06040',
    500: '#B23317',
  }
}

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./server/**/*.{rs,html}"],
  theme: {
    fontSize: {
      sm: '0.8rem',
      base: '1rem',
      lg: '1.25rem',
      xl: '1.563rem',
      '2xl': '1.953rem',
      '3xl': '2.441rem',
      '4xl': '3.052rem',
    },
    colors: {
      ...defaultColors,
      "background": "rgba(51, 41, 67, 1)",
      "coding_background": "#231c2e",
      "text": colors.primary[400],
      "subtitle": colors.grey[600],
      ...colors
    },
    extend: {
      fontFamily: {
        'sans': ['Quicksand', 'sans-serif'],
      },
      backgroundImage: {
        'header-background': 'url("/static/header_background.png")',
      }
    },
  },
  plugins: [],
}
