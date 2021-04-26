import React from "react";
import SyncLoader from "react-spinners/SyncLoader";
import "./LoadingScreen.css";

function LoadingScreen(props) {
  return (
    <div className = "loader-wrapper">
      <SyncLoader color={"#3f51b5"} loading={true} size={30} />
    </div>
  );
}

export default LoadingScreen;
