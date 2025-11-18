/** @type {import('tailwindcss').Config} */

// Helper function to generate color utilities
const withOpacity = (variableName) => {
    return ({opacityValue}) => {
        if (opacityValue !== undefined) {
            return `rgba(var(${variableName}), ${opacityValue})`;
        }
        return `rgb(var(${variableName}))`;
    };
};

module.exports = {
    content: [
        "./**/*.rs",
        "./index.html",
    ],
    // Enable dark mode based on the 'class' strategy
    darkMode: 'class',
    theme: {
        extend: {
            // Map our CSS variables to Tailwind's color palette
            colors: {
                primary: withOpacity('--color-primary'),
                'primary-hover': withOpacity('--color-primary-hover'),
                secondary: withOpacity('--color-secondary'),
            },
            textColor: {
                base: withOpacity('--color-text-base'),
                muted: withOpacity('--color-text-muted'),
                inverted: withOpacity('--color-text-inverted'),
                accent: withOpacity('--color-text-accent'),
            },
            backgroundColor: {
                base: withOpacity('--color-bg-base'),
                surface: withOpacity('--color-surface'),
            },
            borderColor: {
                DEFAULT: withOpacity('--color-border'),
            }
        },
    },
    plugins: [],
}