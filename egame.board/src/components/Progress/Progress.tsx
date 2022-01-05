import * as React from "react";
import LinearProgress, {
  linearProgressClasses,
} from "@mui/material/LinearProgress";
import Box from "@mui/material/Box";
import { styled } from "@mui/material/styles";
import { Typography } from "@mui/material";

interface LinearProgressWithLabelProps {
  value: number;
  topText: string;
  topStickyText: string;
  bottomText: string;
  bottomStickyText: string;
}

const LinearProgressWithLabel: React.FC<LinearProgressWithLabelProps> = ({
  value,
  topText,
  topStickyText,
  bottomStickyText,
  bottomText,
}) => {
  return (
    <Box sx={{ width: "100%", paddingY: 2 }}>
      <Box sx={{ width: "100%", position: "relative" }}>
        <RelativeTopText
          sx={{
            left: `${value - 5}%`,
          }}
        >
          {topText}
        </RelativeTopText>
        <RelativeTopText sx={{ right: 0 }}>{topStickyText}</RelativeTopText>
        <BorderLinearProgress
          variant="determinate"
          value={value}
          sx={{ height: "15px", borderRadius: "25px" }}
        />
        <RelativeBottomText
          sx={{
            left: `${value - 5}%`,
          }}
        >
          {bottomText}
        </RelativeBottomText>
        <RelativeBottomText sx={{ right: 0 }}>
          {bottomStickyText}
        </RelativeBottomText>
      </Box>
    </Box>
  );
};

export default LinearProgressWithLabel;

const BorderLinearProgress = styled(LinearProgress)(() => ({
  width: "100%",
  height: 15,
  borderRadius: 25,
  [`&.${linearProgressClasses.colorPrimary}`]: {
    background: "#E1F1FF",
    boxShadow: "inset 0px 4px 4px #BAE0FF",
  },
  [`& .${linearProgressClasses.bar}`]: {
    borderRadius: 25,
    background: "linear-gradient(180deg, #49ADFF 47.71%, #1395FF 50.31%)",
  },
}));

const RelativeTopText = styled(Typography)(() => ({
  display: "inline-block",
  margin: 0,
  fontFamily: "PT Sans",
  fontSize: "16px",
  fontStyle: "normal",
  fontWeight: "400",
  lineHeight: "21px",
  letterSpacing: "0em",
  position: "absolute",
  color: "black",
  bottom: 15,
}));

const RelativeBottomText = styled(Typography)(() => ({
  display: "inline-block",
  fontFamily: "PT Sans",
  fontSize: "16px",
  fontStyle: "normal",
  fontWeight: "400",
  lineHeight: "21px",
  letterSpacing: "0em",
  position: "absolute",
  color: "black",
  textAlign: "right",
  top: 15,
}));
