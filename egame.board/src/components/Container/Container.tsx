import { Box } from "@mui/system";
import { styled } from "@mui/material/styles";
import { ReactNode } from "react";

interface ContainerProps {
  children: ReactNode;
  sx: any;
}

const Container: React.FC<ContainerProps> = ({ children, sx }) => {
  return <StyledBox {...{ sx }}>{children}</StyledBox>;
};

const StyledBox = styled(Box)(() => ({
  display: "flex",
  flexDirection: "column",
  justifyContent: "center",
  alignItems: "center",
  padding: "16px",
  border: "0.5px solid #DBDBDB",
  borderRadius: "4px",
  // maxWidth: "160px",
  // width: "160px",
}));

export default Container;
