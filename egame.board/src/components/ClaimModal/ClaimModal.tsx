import { Box, CircularProgress } from "@mui/material";
import React from "react";
import Button from "../Button/Button";
import ModalWrapper from "../ModalWrapper/ModalWrapper";
import Caption from "../Title/Caption";
import Heading from "../Title/Heading";
import FmdBadIcon from "@mui/icons-material/FmdBad";

interface ClaimModalProps {
  handleClose: () => void;
  handleClaim: () => void;
  isLoading: boolean;
  open: boolean;
  isError: boolean;
  available: string;
}

const ClaimModal: React.FC<ClaimModalProps> = ({
  open,
  isLoading,
  isError,
  available,
  handleClose,
  handleClaim,
}) => {
  return (
    <ModalWrapper
      open={open}
      handleClose={handleClose}
      title={"Claim ELU tokens"}
    >
      {isError ? (
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
          }}
        >
          <FmdBadIcon sx={{ color: "#f7bab9", fontSize: 68 }} />
          <Caption
            sx={{ p: 2 }}
            text={"Sorry, your tokens were not claimed. Try again later."}
          />
        </Box>
      ) : isLoading ? (
        <Box
          sx={{
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            flexDirection: "column",
          }}
        >
          <CircularProgress
            style={{ color: "#1395FF" }}
            thickness={6}
            size={50}
          />
          <Caption sx={{ p: 2 }} text={"Processing"} />
        </Box>
      ) : (
        <Box
          px={{
            display: "flex",
            justifyContent: "space-between",
            flexDirection: "column",
            alignItems: "center",
            height: "100%",
          }}
        >
          <Caption text={"Tokens Available to Claim"} />
          <Heading
            sx={{ fontSize: "64px", padding: 0 }}
            text={available + " elu"}
          />
          <Box
            sx={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              pb: 4,
              flexDirection: "column",
            }}
          >
            <Caption
              sx={{ padding: 2, color: "#808080" }}
              text={
                "After you claim the tokens, they will be sent to your wallet address"
              }
            />
            <Box sx={{ width: "75%" }}>
              <Button
                title={"claim ALL!"}
                isIconVisible={false}
                onClick={handleClaim}
              />
            </Box>
          </Box>
        </Box>
      )}
    </ModalWrapper>
  );
};

export default ClaimModal;
