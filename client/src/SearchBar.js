import SearchIcon from "@material-ui/icons/Search";
import IconButton from "@material-ui/core/IconButton";
import InputBase from "@material-ui/core/InputBase";
import Paper from "@material-ui/core/Paper";
import { useState } from "react";
import { routeRequest } from "./messages.js";

function SearchBar(props) {

  const [query, setQuery] = useState("");

  const sendRequest = (e) => {
    e.preventDefault();
    if (query) {
      props.wsSend(JSON.stringify(routeRequest(query)));
    } else {
      return;
    }
  };

  return (
    <Paper
      component="form"
      elevation={2}
      id="paper"
      onSubmit={(e) => sendRequest(e)}
    >
      <InputBase
        id="searchField"
        placeholder="Search busline..."
        inputProps={{ "aria-label": "search google maps" }}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <IconButton type="submit" aria-label="search">
        <SearchIcon />
      </IconButton>
    </Paper>
  );
}

export default SearchBar;
