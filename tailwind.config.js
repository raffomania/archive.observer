/** @type {import('tailwindcss').Config} */
module.exports = {
    // deliberately ignore output here to speed up the process
    content: ["./templates/**/*.liquid"],
    theme: {
        extend: {},
    },
    plugins: [require("@tailwindcss/typography")],
};
