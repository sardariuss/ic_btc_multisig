const clampNumber = (
  val: any,
  min: number = -Infinity,
  max: number = Infinity,
  decimalScale: number = 0,
): number => {
  let v = typeof val === "number" ? val : Number(val);
  v = Math.min(max, Math.max(min, isNaN(v) ? 0 : v));
  return Number(v.toFixed(decimalScale));
};

const generateNumberRegex = (
  min: number,
  max: number,
  allowDecimal: boolean,
): RegExp => {
  const floatRegexStr = "(\\.[0-9]*)?";
  const negativeIntRegexStr = "-[0-9]*";
  const positiveIntRegexStr = "[0-9]+";
  const positiveOrNegativeIntRegexStr = "-?[0-9]*";

  let regexStr = "^";
  if (max < 0) regexStr += negativeIntRegexStr;
  else if (min > 0) regexStr += positiveIntRegexStr;
  else regexStr += positiveOrNegativeIntRegexStr;
  if (allowDecimal) regexStr += floatRegexStr;
  regexStr += "$";
  return new RegExp(regexStr);
};

const getFormControlProps = (props: any) => {
  return {
    color: props.color,
    disabled: props.disabled,
    error: props.error,
    fullWidth: props.fullWidth,
    required: props.required,
    variant: props.variant,
  };
};

const frome8s = (e8s: bigint) : number => {
  return Number(e8s) / 1000000000;
}

export { clampNumber, generateNumberRegex, getFormControlProps, frome8s };
