/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./output/**/*.html"],
    theme: {
        extend: {},
    },
    plugins: [require("@tailwindcss/typography")],
};
