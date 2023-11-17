import App                            from './App';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline                    from '@mui/material/CssBaseline';

import React                          from 'react';
import ReactDOM                       from 'react-dom/client';

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
});

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);