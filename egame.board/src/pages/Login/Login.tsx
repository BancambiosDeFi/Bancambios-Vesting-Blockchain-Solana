import React from "react";
import styles from "./Login.module.css";
import Logo from "../../assets/logo.svg";
import Button from "../../components/Button/Button";
import { useNavigate } from "react-router-dom";

declare global {
  interface Window {
    solana: any;
  }
}

const Login = () => {
  const navigate = useNavigate();

  const handleConnectWallet = async () => {
    const isPhantomInstalled: boolean | undefined =
      window?.solana && window?.solana?.isPhantom;

    if (isPhantomInstalled) {
      console.log(isPhantomInstalled, "isPhantomInstalled");

      try {
        const resp = await window.solana.connect();
        const publicKey = resp.publicKey.toString();
        if (publicKey) {
          localStorage.setItem("publicKey", publicKey);
          navigate("cabinet", { state: { publicKey } });
        }
        console.log(window.solana.isConnected);
      } catch (err) {
        handleConnectWallet();
        console.log("isPhantomInstalled === e ===", err);
      }
    } else {
      window.open("https://phantom.app/", "_blank");
    }
  };

  return (
    <>
      <img alt={"logo"} src={Logo} className={styles.logo} />
      <div className={styles.text_container}>
        <p className={styles.text}>Are you an Elumia investor? </p>
        <p className={styles.text_black}>
          Connect a wallet to see your vested tokens
        </p>
        <div className={styles.button_container}>
          <Button
            isIconVisible={true}
            onClick={handleConnectWallet}
            title={"CONNECT PHANTOM WALLET"}
          />
        </div>
      </div>
    </>
  );
};

export default Login;
