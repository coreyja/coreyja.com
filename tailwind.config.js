/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./server/**/*.{rs,html}"],
  theme: {
    extend: {
      colors: {
        "background": "rgba(51, 41, 67, 1)",
      },
    },
  },
  plugins: [],
}
