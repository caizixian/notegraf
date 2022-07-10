/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/frontend/**/*.{html,js,jsx,ts,tsx}"],
    theme: {
        extend: {},
    },
    variants: {},
    plugins: [
        require('@tailwindcss/forms'),
        require('@tailwindcss/typography'),
    ],
};