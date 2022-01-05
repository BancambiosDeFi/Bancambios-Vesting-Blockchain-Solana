import React from "react";
import styles from "./Logout.module.css";
import Logo from "../../assets/logo.svg";
import Button from "../../components/Button/Button";

const Logout = () => {
  const handleDisconnectWallet = () => {
    console.log('click');
    
    // window.solana.on("disconnect", () => console.log(" handleDisconnectWallet disconnected!"));
  };

  return (
    <>
      <img alt={"logo"} src={Logo} className={styles.logo} />
      <div className={styles.text_container}>
        <p className={styles.text_black}>
          Sorry, it looks like you’re not an Elumia investor.
          <br />
          Try to connect another wallet to see your vested tokens
        </p>
        <div className={styles.wallet_container}>
          <div className={styles.wallet_address}>
            <p className={styles.address}>
              58rwAow1roHoeFeMCXv5b1xxoKAhkGPqZmrTLAUT84RD
            </p>
          </div>
        </div>
        <div className={styles.button_container}>
          <Button
            isIconVisible={false}
            onClick={handleDisconnectWallet}
            title={"Disconnect wallet"}
          />
        </div>
      </div>
    </>
  );
};

export default Logout;
