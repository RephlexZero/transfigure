/** @type {import('tailwindcss').Config} */

// Colours are CSS custom properties (see input.css) holding space-separated
// RGB channels, so light/dark themes swap in one place and Tailwind's
// opacity modifiers (`bg-src/10`) keep working.
const token = (name) => `rgb(var(--${name}) / <alpha-value>)`;

// One easing for everything that lands: fast out, long settle.
const settle = "cubic-bezier(0.16, 1, 0.3, 1)";

module.exports = {
  content: ["./crates/app/src/**/*.rs", "./crates/app/index.html"],
  darkMode: "media",
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Bricolage Grotesque",
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
        paper: token("paper"),
        "paper-2": token("paper-2"),
        rule: token("rule"),
        ink: token("ink"),
        "ink-2": token("ink-2"),
        // Source ink marks what you have; result ink marks what you get.
        src: token("src"),
        "src-ink": token("src-ink"),
        res: token("res"),
        "res-ink": token("res-ink"),
        "on-src": token("on-src"),
        "on-res": token("on-res"),
      },
      borderRadius: {
        DEFAULT: "3px",
      },
      maxWidth: {
        page: "90rem",
      },
      keyframes: {
        // Rows and specimens arriving.
        settle: {
          "0%": { transform: "translateY(0.375rem)", opacity: "0" },
          "100%": { transform: "none", opacity: "1" },
        },
        // The result ink pulling into register over the source ink.
        register: {
          "0%": { transform: "translate(0.07em, -0.045em)", opacity: "0" },
          "100%": { transform: "none", opacity: "1" },
        },
        // Lands just off register, like the mark, so both inks stay visible.
        misregister: {
          "0%": { transform: "translate(0.12em, -0.08em)", opacity: "0" },
          "100%": { transform: "translate(0.035em, -0.025em)", opacity: "1" },
        },
        // The source ink lifting away once the result has landed.
        lift: {
          "0%": { transform: "none", opacity: "1" },
          "100%": { transform: "translate(-0.04em, 0.03em)", opacity: "0" },
        },
        // While a file converts the result ink drifts in and out of register.
        overprint: {
          "0%, 100%": { transform: "translate(0.07em, -0.045em)", opacity: "0.15" },
          "50%": { transform: "translate(0.015em, -0.01em)", opacity: "1" },
        },
        sweep: {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(250%)" },
        },
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
      },
      animation: {
        settle: `settle 260ms ${settle} both`,
        register: `register 700ms ${settle} both`,
        lift: `lift 700ms ${settle} both`,
        misregister: `misregister 600ms ${settle} both`,
        overprint: "overprint 1.6s ease-in-out infinite",
        sweep: "sweep 1.1s ease-in-out infinite",
        "fade-in": "fade-in 140ms ease-out both",
      },
      transitionTimingFunction: {
        settle,
      },
    },
  },
  plugins: [],
};
