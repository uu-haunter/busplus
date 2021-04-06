import './App.css';
import Map from "./map.js";
import Menu from "@material-ui/icons/Menu";
import Fab from '@material-ui/core/Fab';
import SearchBar from "./SearchBar.js";


function App() {


  return (
    <div className="App">
      <div className = "App-header">
      <Map/>
        <Fab
        id = "menuButton"
        color = "primary" 
        aria-label = "menuButton">
          <Menu/>
        </Fab>
        <SearchBar/>
      </div>
    </div>
  );
}

export default App;
