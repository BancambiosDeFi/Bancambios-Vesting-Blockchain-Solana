import { Box } from "@mui/system";
import React, { useEffect, useState } from "react";
import { VestingTypeAccount } from "token-vesting-api/dist/models";
import Container from "../../components/Container/Container";
import LinearProgressWithLabel from "../../components/Progress/Progress";
import BlueTitle from "../../components/Title/BlueTitle";
import Caption from "../../components/Title/Caption";
import Heading from "../../components/Title/Heading";
import moment from "moment";
import { Connection, PublicKey } from "@solana/web3.js";
import { TokenVesting } from "token-vesting-api";
import Button from "../../components/Button/Button";
import { VestingStatistic } from "token-vesting-api/dist/query";
import ClaimModal from "../../components/ClaimModal/ClaimModal";
import converterBN from "../../utils";
import CongratulationsModal from "../../components/CongratulationsModal/CongratulationsModal";
import { WithdrawFromVestingInstruction } from "token-vesting-api/dist/schema";
import BigNumber from "bignumber.js";

interface SeedSaleProps {
  name: string;
}

interface IValues {
  total: string;
  released: string;
  available: string;
  claimed: string;
}

const SeedSale: React.FC<SeedSaleProps> = ({ name }) => {
  const [values, setValues] = useState<IValues>({
    total: "0",
    released: "0",
    available: "0",
    claimed: "0",
  });
  const [newWalletKey, setNewWalletKey] = useState<PublicKey>();
  const [connection, setConnection] = useState<Connection>();
  const [token, setToken] = useState<TokenVesting>();

  const network: string = "https://api.devnet.solana.com";
  const pubKey: string = "GFiTCRwUgCynpodh6cCE8GA4RQNSbKHTVHD1sEZPPDJy";
  const mint: string = "7dtmKP7NQ9p2vuHAsPbyeBL1Hws7d4tWszwC8KB9jXxe";
  const creator: string = "67WdZBU8mUC8HWN3cdjVjWftGrmVAfjuFxvjj5tiDorB";

  const [open, setOpen] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isError, setIsError] = useState(false);
  const [data, setData] = useState<VestingStatistic>();
  const [vestingType, setVestingType] = useState<VestingTypeAccount>();

  useEffect(() => {
    const newWalletKey = localStorage.getItem("publicKey");
    newWalletKey && setNewWalletKey(new PublicKey(newWalletKey));
    setConnection(new Connection(network));
    setToken(
      new TokenVesting(
        new Connection(network),
        new PublicKey(pubKey),
        new PublicKey(mint),
        new PublicKey(creator),
        name
      )
    );
    return () => {};
  }, [name]);

  const connectSolana = async () => {
    try {
      if (window.solana) {
        const key = await window.solana.connect({ onlyIfTrusted: true });
        setNewWalletKey(new PublicKey(key.publicKey.toString()));
      }
    } catch (error) {
      console.log("connectSolana error === ", error);
    }
  };

  useEffect(() => {
    const newWalletKey = localStorage.getItem("publicKey");
    if (newWalletKey) {
      setNewWalletKey(new PublicKey(newWalletKey));
    } else {
      connectSolana();
    }
  }, []);

  useEffect(() => {
    if (token) {
      token.getVestingType().then((data) => {
        setVestingType(data);
      });
      newWalletKey &&
        token
          .getVestingStatistic(newWalletKey)
          .then((data) => {
            setData(data);
            setValues({
              total: converterBN(data.allTokens),
              released: converterBN(data.unlockedTokens),
              available: converterBN(data.availableToWithdrawTokens),
              claimed: converterBN(data.withdrawn_tokens),
            });
          })
          .catch((e) => {
            console.log("error", e);
          });
    }
  }, [newWalletKey, token]);

  const handleClaim = () => {
    console.log("window.solana.isConnected", window.solana.isConnected);

    setIsLoading(!isLoading);
    newWalletKey &&
      connection &&
      token &&
      data &&
      data.availableToWithdrawTokens &&
      token
        .withdrawFromVesting(
          newWalletKey,
          new WithdrawFromVestingInstruction(data.availableToWithdrawTokens)
        )
        .then((transaction) => {
          console.log("withdrawFromVesting", transaction);
          connection
            .getRecentBlockhash("confirmed")
            .then(({ blockhash }) => {
              transaction.recentBlockhash = blockhash;
              transaction.feePayer = newWalletKey;

              window.solana
                .signAndSendTransaction(transaction)
                .then((sign: { signature: string }) => {
                  console.log("sign === ", sign);

                  connection
                    .confirmTransaction(sign.signature)
                    .then((signature) => {
                      console.log("signature", signature);
                      handleClose();
                      handleOpen();
                    })
                    .catch((e) => {
                      console.log("signature", e);
                      setIsError(!isError);
                    });
                })
                .catch((e: any) => {
                  console.log("test == ", e);
                  setIsError(!isError);
                });
            })
            .catch((e) => {
              console.log("hash", e);
              setIsError(!isError);
            });
        })
        .catch((e) => {
          console.log("withdrawFromVesting", e);
          setIsError(!isError);
        });
  };

  const handleClickOpen = () => {
    setOpen(true);
  };

  const handleOpen = () => {
    setIsOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
    setIsError(false);
    setIsLoading(false);
  };

  const handleExit = () => {
    setIsOpen(false);
  };

  const { total, available, claimed, released } = values;

  return (
    <>
      <ClaimModal
        available={data ? converterBN(data.availableToWithdrawTokens) : ""}
        {...{ open, isError, isLoading, handleClose, handleClaim }}
      />
      <CongratulationsModal {...{ isOpen }} handleClose={handleExit} />
      <Box
        sx={{
          width: "100%",
          paddingX: { xs: 0, lg: 10 },
          boxSizing: "border-box",
        }}
      >
        <Box
          sx={{
            display: "flex",
            justifyContent: { xs: "center", md: "space-around" },
            alignItems: "center",
            py: "20px",
            flexDirection: { xs: "column", md: "column", lg: "row" },
          }}
        >
          <Box sx={{ paddingY: { xs: 1, md: 0 } }}>
            <BlueTitle
              text={
                vestingType &&
                vestingType.vesting_schedule &&
                vestingType.vesting_schedule.start_time
                  ? moment
                      .unix(
                        +vestingType?.vesting_schedule?.start_time.toString()
                      )
                      .format("L")
                  : ""
              }
            />
            {/*starttime*/}
            <Caption text={"Token Generation Event day"} />
          </Box>
          <Box sx={{ paddingY: { xs: 1, md: 0 } }}>
            <BlueTitle
              text={
                new BigNumber(total)
                  .multipliedBy(
                    vestingType?.vesting_schedule?.initial_unlock as number
                  )
                  .toString() + " ELU"
              }
            />
            <Caption text={"Tokens released on Token Generation Event"} />
          </Box>
          <Box sx={{ paddingY: { xs: 1, md: 0 } }}>
            <BlueTitle
              text={
                vestingType &&
                vestingType.vesting_schedule &&
                vestingType.vesting_schedule.cliff
                  ? moment
                      .unix(+vestingType?.vesting_schedule?.cliff.toString())
                      .format("L")
                  : ""
              }
            />
            {/* <BlueTitle text={"02:19:22"} /> */}
            {/*cliff*/}
            <Caption text={"Cliff period ending in"} />
          </Box>
        </Box>
        <LinearProgressWithLabel
          value={new BigNumber(total).div(new BigNumber(released)).toNumber()}
          topText={released + " ELU"}
          topStickyText={total + " ELU"}
          bottomText={moment(new Date()).format("L")} //current time
          bottomStickyText={
            vestingType &&
            vestingType.vesting_schedule &&
            vestingType.vesting_schedule.end_time
              ? moment
                  .unix(+vestingType?.vesting_schedule?.end_time.toString())
                  .format("L")
              : ""
          }
        />
        <Box
          sx={{
            display: "flex",
            justifyContent: "space-around",
            alignItems: "center",
            py: "20px",
            flexDirection: { xs: "column", md: "column", lg: "row" },
          }}
        >
          <Box
            sx={{
              display: { xs: "flex", md: "flex" },
              flexDirection: { xs: "row" },
              justifyContent: { xs: "space-around" },
              width: { xs: "100%", lg: "50%" },
              paddingY: 1,
            }}
          >
            <Container
              sx={{
                marginX: { xs: "4px", md: 1 },
                width: { xs: 130, md: 160 },
              }}
            >
              <Caption text={"Total Tokens Vested"} />
              <Heading
                sx={{ textAlign: { xs: "center" } }}
                text={total + " ELU"}
              />
            </Container>
            <Container
              sx={{
                marginX: { xs: "4px", md: 1 },
                width: { xs: 130, md: 160 },
              }}
            >
              <Caption text={"Tokens Released"} />
              <Heading
                text={released + " ELU"}
                sx={{ textAlign: { xs: "center" } }}
              />
            </Container>
          </Box>
          <Box
            sx={{
              display: { xs: "flex", md: "flex" },
              flexDirection: { xs: "row" },
              justifyContent: { xs: "space-around" },
              width: { xs: "100%", lg: "50%" },
              paddingY: 1,
            }}
          >
            <Container
              sx={{
                marginX: { xs: "4px", md: 1 },
                width: { xs: 130, md: 160 },
              }}
            >
              <Caption text={"Tokens Claimed"} />
              <Heading
                sx={{ textAlign: { xs: "center" } }}
                text={claimed + " ELU"}
              />
            </Container>
            <Container
              sx={{
                marginX: { xs: "4px", md: 1 },
                width: { xs: 130, md: 160 },
              }}
            >
              <Caption text={"Tokens Available to Claim"} />
              <Heading
                sx={{ textAlign: { xs: "center" } }}
                text={available + " ELU"}
              />
            </Container>
          </Box>
        </Box>
        {data && data.availableToWithdrawTokens?.gtn(0) ? (
          <Box sx={sxStyles.buttonContainer}>
            <Box sx={{ width: { md: "30%", xs: "90%" } }}>
              <Button
                title="Claim!"
                isIconVisible={false}
                onClick={handleClickOpen}
              />
            </Box>
          </Box>
        ) : (
          ""
        )}
      </Box>
      <Box
        sx={{
          backgroundColor: "#F2FBFF",
          width: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: { xs: "space-around", md: "space-between" },
          flexDirection: { xs: "column", lg: "row" },
          p: { xs: 2 },
          paddingX: { md: 10, xs: 0 },
          boxSizing: "border-box",
          mt: { xs: 6, md: 0 },
        }}
      >
        <Box sx={{ display: "flex", alignItems: "center", width: "100%" }}>
          <Caption
            sx={{ width: { xs: "100%", md: "max-content" } }}
            text={"Program address:"}
          />
          <Caption sx={sxStyles.contractAddress} text={pubKey} />
        </Box>
        <Box sx={{ display: "flex", alignItems: "center", width: "100%" }}>
          <Caption
            sx={{ width: { xs: "100%", md: "max-content" } }}
            text={"Tokens are vested to:"}
          />
          <Caption
            sx={sxStyles.contractAddress}
            text={newWalletKey?.toString() ?? ""}
          />
        </Box>
      </Box>
    </>
  );
};

const sxStyles = {
  buttonContainer: {
    display: "flex",
    width: "100%",
    justifyContent: "center",
    alignItems: "center",
    mb: { xs: 0, md: 1 },
  },
  tab: {
    maxHeight: "40px",
    textTransform: "none",
    border: "1px solid #D3D3D3",
    borderBottom: 0,
    boxSizing: "border-box",
    borderRadius: "4px 4px 0px 0px",
    margin: "0 4px",
    padding: 1,
    fontSize: { xs: 14, md: 16 },
    height: 40,
    minHeight: 40,
    minWidth: "auto",
    "&.Mui-selected": {
      backgroundColor: "#1395ff",
      color: "#fff!important",
      fontWeight: "bold",
      height: 40,
      minHeight: 40,
    },
  },
  tabsContainer: {
    borderBottom: 1,
    borderColor: "divider",
    height: "40px",
    minHeight: "40px",
    display: "flex",
    flexDirection: "column",
  },
  contractAddress: {
    color: "rgba(19, 149, 255, 1)",
    paddingX: 1,
    overflow: "hidden",
    whiteSpace: "nowrap",
    textOverflow: "ellipsis",
    // maxWidth: "50%",
  },
} as const;

export default SeedSale;
