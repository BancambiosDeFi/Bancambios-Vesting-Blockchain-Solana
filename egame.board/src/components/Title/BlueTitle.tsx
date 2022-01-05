import { Typography } from "@mui/material";
import { styled } from "@mui/material/styles";

interface TitleProps {
  text: string;
}

const Title: React.FC<TitleProps> = ({ text }) => {
  return <BlueTitle>{text.toUpperCase()}</BlueTitle>;
};

const BlueTitle = styled(Typography)(() => ({
  color: "#1395FF",
  fontFamily: "Poppins",
  fontSize: "24px",
  fontWeight: "500",
  lineHeight: "35px",
  letterSpacing: "0.03em",
  textAlign: "center",
}));

export default Title;
