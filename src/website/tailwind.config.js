/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./../**/*.{html, rs}", "./src/index.html", "./../routes/posts.rs", "./src/sign-up.html"],
  theme: {
    extend: {
      colors: {
        accent: '#6E668F',
        primary: '#7b739c',
        secondary: '#282534',
      }
    },
  },
  plugins: [],
}

