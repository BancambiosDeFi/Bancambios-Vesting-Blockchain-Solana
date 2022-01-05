import React from "react"; //{ useEffect, useState }
import { BrowserRouter, Route, Routes } from "react-router-dom";
import Cabinet from "./pages/Cabinet/Cabinet";
import Login from "./pages/Login/Login";
import styles from "./App.module.css";
import Logout from "./pages/Logout/Logout";
import { StylesProvider } from "@mui/styles";

function App() {
  return (
    <StylesProvider injectFirst>
      <div className={styles.App}>
        <div className={styles.main_container}>
          <div className={styles.container}>
            <Routes>
              <Route index={false} path="/" element={<Login />} />
              <Route path="/cabinet" element={<Cabinet />} />
              <Route path="/logout" element={<Logout />} />
            </Routes>
          </div>
        </div>
      </div>
    </StylesProvider>
  );
}

const AppWrapper = () => {
  return (
    <BrowserRouter>
      <App />
    </BrowserRouter>
  );
};

export default AppWrapper;
