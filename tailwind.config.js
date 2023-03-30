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
      "background": "rgba(51, 41, 67, 1)",
      "text": "rgba(225, 212, 255, 1)",
      "subtitle": "rgba(200, 192, 215, 1)",
      "accent": "rgba(178, 132, 255, 1)",
    },
    extend: {
      fontFamily: {
        'sans': ['Quicksand', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
