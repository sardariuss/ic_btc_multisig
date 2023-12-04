module.exports = {
	content: ["./src/frontend/index.html", "./src/frontend/*.{vue,js,ts,jsx,tsx}"],
	theme: {
		fontFamily: {
			inter: "'Inter', sans-serif",
			vt323: "'VT323', sans-serif",
		},
		fontSize: {
      '2xl': '1.5rem',
      '3xl': '1.875rem',
      '4xl': '2.25rem',
      '5xl': '3rem',
    },
		lineHeight: {
      '2xl': '2rem',
      '3xl': '2.25rem',
      '4xl': '2.5rem',
      '5xl': '1rem',
    },
		extend: {
			width: {
        '1/24': '4.1666665%;',
      }
		}
	},
	plugins: [],
	darkMode: 'class',
};
