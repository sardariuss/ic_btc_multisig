import * as React from 'react';
import Button from '@mui/material/Button';
import Snackbar from '@mui/material/Snackbar';
import IconButton from '@mui/material/IconButton';
import CloseIcon from '@mui/icons-material/Close';
import MuiAlert, { AlertProps } from '@mui/material/Alert';

const Alert = React.forwardRef<HTMLDivElement, AlertProps>(function Alert(
  props,
  ref,
) {
  return <MuiAlert elevation={6} ref={ref} variant="filled" {...props} />;
});

export type Props = {
  success: boolean,
  message: string,
  setMessage: (message: string) => void,
};

const TextSnackbar = ({success, message, setMessage}: Props) => {
  
  const handleClose = (event: React.SyntheticEvent | Event, reason?: string) => {
    if (reason === 'clickaway') {
      return;
    }
    setMessage("");
  };

  const action = (
    <React.Fragment>
      <Button color="secondary" size="small" onClick={handleClose}/>
      <IconButton
        size="small"
        aria-label="close"
        color="inherit"
        onClick={handleClose}
      >
        <CloseIcon fontSize="small" />
      </IconButton>
    </React.Fragment>
  );

  return (
    <Snackbar
      open={message !== ""}
      autoHideDuration={6000}
      onClose={handleClose}
      action={action}
    >
      <Alert onClose={handleClose} severity={success ? "success" : "error"} sx={{ width: '100%' }}>
        { message }
      </Alert>
    </Snackbar>
  );
}

export default TextSnackbar;