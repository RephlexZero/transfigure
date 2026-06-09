/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./crates/app/src/**/*.rs",
    "./crates/app/index.html",
  ],
  theme: {
    extend: {
      fontFamily: {
        mono: [
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "Liberation Mono",
          "monospace",
        ],
      },
      animation: {
        'stamp-in': 'stamp-in 0.25s cubic-bezier(0.2, 1.4, 0.4, 1) both',
        'row-in': 'row-in 0.3s ease-out both',
        'pulse-soft': 'pulse-soft 1.2s ease-in-out infinite',
      },
      keyframes: {
        'stamp-in': {
          '0%': { transform: 'scale(1.6) rotate(-6deg)', opacity: '0' },
          '100%': { transform: 'scale(1) rotate(0deg)', opacity: '1' },
        },
        'row-in': {
          '0%': { transform: 'translateY(4px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        'pulse-soft': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.35' },
        },
      },
    },
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: [
      {
        transfigure: {
          "primary": "#d9a441",
          "primary-content": "#16130a",
          "secondary": "#6fae8f",
          "secondary-content": "#0f1411",
          "accent": "#c96342",
          "accent-content": "#160e0a",
          "neutral": "#1a1712",
          "neutral-content": "#e6dfd0",
          "base-100": "#16140f",
          "base-200": "#100e0a",
          "base-300": "#0b0a07",
          "base-content": "#e6dfd0",
          "info": "#7ba6c9",
          "success": "#6fae8f",
          "warning": "#d9a441",
          "error": "#cf6b57",
        },
      },
    ],
  },
}
