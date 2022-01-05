import React, { FC } from "react";
import styles from "./Button.module.css";
import PhantomIcon from "../../assets/Rectangle.svg";

export interface ButtonProps {
  title: string;
  onClick: React.MouseEventHandler<HTMLInputElement>;
  isIconVisible?: boolean;
}

const Button: FC<ButtonProps> = ({ title, onClick, isIconVisible = true }) => {
  return (
    <div onClick={onClick} className={styles.button_container}>
      {isIconVisible && (
        <img alt={"icon"} src={PhantomIcon} className={styles.icon} />
      )}
      <p className={styles.button_text}>{title.toUpperCase()}</p>
    </div>
  );
};

export default Button;
