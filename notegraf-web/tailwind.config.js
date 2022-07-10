/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/frontend/**/*.{html,js,jsx,ts,tsx}"],
    theme: {
        extend: {
            typography: {
                github: {
                    css: {
                        '--tw-prose-pre-code': '#333',
                        '--tw-prose-pre-bg': '#fff',
                        '--tw-prose-invert-pre-code': '#c9d1d9',
                        '--tw-prose-invert-pre-bg': '#0d1117',
                    },
                },
            }
        },
    },
    variants: {},
    plugins: [
        require('@tailwindcss/forms'),
        require('@tailwindcss/typography'),
    ],
};