import { Typography } from "@mui/material";
import { styled } from "@mui/material/styles";

interface TitleProps {
  text: string;
  sx?: any;
}

const Heading: React.FC<TitleProps> = ({ text, sx }) => {
  return <HeadingStyled sx={sx}>{text.toUpperCase()}</HeadingStyled>;
};

const HeadingStyled = styled(Typography)(() => ({
  fontFamily: "Poppins",
  fontStyle: "normal",
  fontWeight: "500",
  fontSize: "24px",
  lineHeight: "35px",
  letterSpacing: "0.03em",
  color: "#000000",
}));

export default Heading;
