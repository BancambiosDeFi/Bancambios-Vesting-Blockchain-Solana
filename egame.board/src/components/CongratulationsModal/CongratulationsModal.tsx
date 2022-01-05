import { Box } from "@mui/material";
import * as React from "react";
import Button from "../Button/Button";
import ModalWrapper from "../ModalWrapper/ModalWrapper";
import Caption from "../Title/Caption";

interface CongratulationsModalProps {
  isOpen: boolean;
  // title: string;
  handleClose: () => void;
  // children: React.ReactElement;
}

const CongratulationsModal: React.FC<CongratulationsModalProps> = ({
  isOpen,
  handleClose,
}) => {
  return (
    <ModalWrapper
      open={isOpen}
      handleClose={handleClose}
      title={"Claim ELU tokens"}
    >
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-around",
          alignItems: { xs: "stretch", md: "center" },
          flexDirection: "column",
          height: "100%",
          width: "100%",
        }}
      >
        <Box sx={{ p: { xs: 1, md: 3 } }}>
          <Caption text={"Claimed tokens have been sent to you wallet:"} />
          <Caption
            sx={{
              p: 1,
              backgroundColor: "#F2FBFF",
              textOverflow: "ellipsis",
              overflow: "hidden",
            }}
            text={"58rwAow1roHoeFeMCXv5b1xxoKAhkGPqZmrTLAUT84RD"}
          />
        </Box>
        <Box
          sx={{
            width: "100%",
            display: { xs: "flex" },
            justifyContent: "center",
          }}
        >
          <Box sx={{ width: "75%" }}>
            <Button
              isIconVisible={false}
              onClick={handleClose}
              title={"Done"}
            />
          </Box>
        </Box>
      </Box>
    </ModalWrapper>
  );
};

export default CongratulationsModal;
