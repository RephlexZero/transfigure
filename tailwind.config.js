/** @type {import('tailwindcss').Config} */

// Colours are CSS custom properties (see input.css) holding space-separated
// RGB channels, so light/dark themes swap in one place and Tailwind's
// opacity modifiers (`bg-accent/10`) keep working.
const token = (name) => `rgb(var(--${name}) / <alpha-value>)`;

module.exports = {
  content: ["./crates/app/src/**/*.rs", "./crates/app/index.html"],
  darkMode: "media",
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "InterVariable",
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
      colors: {
        page: token("page"),
        surface: token("surface"),
        sunken: token("sunken"),
        line: token("line"),
        "line-strong": token("line-strong"),
        fg: token("fg"),
        muted: token("muted"),
        subtle: token("subtle"),
        ink: token("ink"),
        "on-ink": token("on-ink"),
        accent: token("accent"),
        "accent-strong": token("accent-strong"),
        ok: token("ok"),
        bad: token("bad"),
      },
      boxShadow: {
        card: "0 1px 2px rgb(var(--shadow) / 0.06), 0 8px 24px -8px rgb(var(--shadow) / 0.12)",
        pop: "0 2px 4px rgb(var(--shadow) / 0.08), 0 16px 40px -12px rgb(var(--shadow) / 0.28)",
      },
      keyframes: {
        "row-in": {
          "0%": { transform: "translateY(3px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
      },
      animation: {
        "row-in": "row-in 180ms ease-out both",
        "fade-in": "fade-in 120ms ease-out both",
      },
    },
  },
  plugins: [],
};
