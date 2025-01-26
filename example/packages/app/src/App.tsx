import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import "./App.css";
import { foo, getBar } from "subpackage";

function App() {
	const [count, setCount] = useState(0);

	const [bar, setBar] = useState(0);

	useEffect(() => {
		getBar().then(setBar);
	}, []);

	return (
		<div className="App">
			<div>
				<a href="https://reactjs.org" target="_blank" rel="noreferrer">
					<img src={reactLogo} className="logo react" alt="React logo" />
				</a>
			</div>
			<h1>Rspack + React + TypeScript</h1>
			<div className="card">
				<button onClick={() => setCount(count => count + 1)}>
					count is {count}
				</button>
				<p>
					Edit <code>src/App.tsx</code> and save to test HMR
				</p>

				<p>foo: {foo}</p>
				<p>bar: {bar}</p>
			</div>
			<p className="read-the-docs">
				Click on the Rspack and React logos to learn more
			</p>
		</div>
	);
}

export default App;
