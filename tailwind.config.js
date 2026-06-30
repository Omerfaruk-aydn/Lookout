/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: "#1a1b26",
          secondary: "#24283b",
          tertiary: "#2f334d",
        },
        fg: {
          primary: "#c0caf5",
          secondary: "#9aa5ce",
          muted: "#565f89",
        },
        accent: {
          blue: "#7aa2f7",
          purple: "#bb9af7",
          green: "#9ece6a",
          red: "#f7768e",
          yellow: "#e0af68",
          cyan: "#7dcfff",
        },
        border: "#292e42",
      },
      fontFamily: {
        mono: ["JetBrains Mono", "monospace"],
        sans: ["Inter", "sans-serif"],
      },
    },
  },
  plugins: [],
};
