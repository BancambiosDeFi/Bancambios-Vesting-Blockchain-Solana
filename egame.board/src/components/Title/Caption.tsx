import { Typography } from "@mui/material";
import { styled } from "@mui/material/styles";

interface TitleProps {
  text: string;
  sx?: any;
}

const Caption: React.FC<TitleProps> = ({ text, sx }) => {
  return <CaptionStyled sx={sx}>{text}</CaptionStyled>;
};

const CaptionStyled = styled(Typography)(() => ({
  fontFamily: "PT Sans",
  fontStyle: "normal",
  fontWeight: "normal",
  fontSize: "14px",
  lineHeight: "21px",
  color: "#000000",
  textAlign: "center",
}));

export default Caption;
