import SearchIcon from "@material-ui/icons/Search";
import IconButton from "@material-ui/core/IconButton";
import InputBase from "@material-ui/core/InputBase";
import Paper from "@material-ui/core/Paper";

function SearchBar() {
  return (
    <Paper component="form" elevation={2} id="paper">
      <InputBase
      id="searchField"
      placeholder="Search"
      inputProps={{ 'aria-label': 'search google maps' }}
      />
      <IconButton type="submit" aria-label="search">
      <SearchIcon />
      </IconButton>
    </Paper>
  );
}

export default SearchBar;