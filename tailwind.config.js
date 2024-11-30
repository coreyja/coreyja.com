const defaultColors = require("tailwindcss/colors");

const colors = {
  grey: {
    100: "#EFEDF2",
    200: "#E3DFEB",
    300: "#D9D3E3",
    400: "#C8C0D7",
    500: "#B0A9BD",
    600: "#9D97A6",
    700: "#908C96",
    800: "#6B6870",
    900: "#4C494F",
    999: "#202024",
  },
  success: {
    100: "#B7F2B6",
    200: "#82E581",
    300: "#52CC50",
    400: "#37A835",
    500: "#217D1F",
  },
  warning: {
    100: "#FFF3C9",
    200: "#FAE496",
    300: "#F0D678",
    400: "#F0CA44",
    500: "#E1B000",
  },
  error: {
    100: "#FFD5CC",
    200: "#FFAC99",
    300: "#FA8970",
    400: "#F06040",
    500: "#B23317",
  },
};

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./**/*.{rs,html,md}",
    "!./target/**/*",
  ],
  theme: {
    fontSize: {
      sm: "0.8rem",
      base: "1rem",
      lg: "1.25rem",
      xl: "1.563rem",
      "2xl": "1.953rem",
      "3xl": "2.441rem",
      "4xl": "3.052rem",
    },
    colors: {
      ...defaultColors,
      background: "#F0F0FF",
      coding_background: "#231c2e",
      text: "#121131",
      codeText: "#F0F0FF",
      subtitle: "#121131",
      berryBlue: "#A1A8FF",
      almostBackground: "#040249",
      footer: "#2f2f34",
      ...colors,
    },
    extend: {
      fontFamily: {
        sans: ["Quicksand", "sans-serif"],
        mono: ["ComicCode", "monospace"],
      },
    },
  },
  plugins: [],
};
