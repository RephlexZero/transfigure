/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./crates/app/src/**/*.rs",
    "./crates/app/index.html",
  ],
  theme: {
    extend: {
      animation: {
        'float-slow': 'float 25s ease-in-out infinite',
        'float-slower': 'float 35s ease-in-out infinite reverse',
        'float-slowest': 'float 45s ease-in-out infinite',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translate(0, 0) scale(1)' },
          '25%': { transform: 'translate(60px, 60px) scale(1.1)' },
          '50%': { transform: 'translate(-40px, 80px) scale(0.95)' },
          '75%': { transform: 'translate(80px, -40px) scale(1.05)' },
        },
      },
    },
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: [
      {
        transfigure: {
          "primary": "#a78bfa",
          "secondary": "#818cf8",
          "accent": "#22d3ee",
          "neutral": "#1e1b2e",
          "base-100": "#13111c",
          "base-200": "#0d0b16",
          "base-300": "#080710",
          "info": "#38bdf8",
          "success": "#4ade80",
          "warning": "#fbbf24",
          "error": "#f87171",
        },
      },
    ],
  },
}
