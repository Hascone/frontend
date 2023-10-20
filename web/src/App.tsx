import { useState } from "react";

import styles from "./App.module.css";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className={styles.container}>
      <button onClick={() => setCount((c) => c + 1)}>
        Current Count: {count}
      </button>
    </div>
  );
}

export default App;
